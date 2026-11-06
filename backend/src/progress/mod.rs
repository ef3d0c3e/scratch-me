//! Progression: experience and levels, daily quests, achievements.
//!
//! Every action that counts calls `apply` inside its own transaction with
//! what happened, so progress lands with the action or not at all. `apply`
//! answers what it gave, for the client to celebrate.

pub mod achievements;
pub mod events;
pub mod levels;
pub mod quests;

use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

pub use events::{Event, Scratch};

use crate::auth::parse_tz;
use crate::envelope;
use crate::error::ApiResult;
use crate::routes::tickets::{TicketDto, grant};
use crate::schema::{achievements as earned, aquarium, cosmetics, quests as quest_rows, stats, streaks, users};
use crate::tickets::{Prize, Source};
use achievements::{Goal, Metric};
use levels::LevelDto;

#[derive(Debug, Clone, Serialize)]
pub struct Reward {
    /// A level reached, a quest key or an achievement id
    pub what: String,
    pub reward: Prize,
}

/// What an action's progress gave
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub xp_gained: i64,
    pub level: LevelDto,
    pub level_ups: Vec<Reward>,
    pub quests: Vec<Reward>,
    /// The day's quests all done: this bonus
    pub bonus: Option<Prize>,
    pub achievements: Vec<Reward>,
    /// Tickets the rewards gave
    pub tickets: Vec<TicketDto>,
    /// The balance after everything
    pub coins: i64,
}

/// The player's quest day: their own calendar
pub async fn quest_day(conn: &mut AsyncPgConnection, uid: i64) -> ApiResult<NaiveDate> {
    let tz: String = users::table.find(uid).select(users::tz).first(conn).await?;
    Ok(envelope::today(parse_tz(Some(&tz)), Utc::now()))
}

/// Stores the day's quests when they are not yet
pub async fn ensure_quests(conn: &mut AsyncPgConnection, uid: i64, day: NaiveDate) -> ApiResult<()> {
    let have: i64 = quest_rows::table
        .filter(quest_rows::user_id.eq(uid))
        .filter(quest_rows::day.eq(day))
        .count()
        .get_result(conn)
        .await?;
    if have > 0 {
        return Ok(());
    }
    let xp: i64 = users::table.find(uid).select(users::xp).first(conn).await?;
    let rows: Vec<_> = quests::draw(uid, day, levels::level_for(xp))
        .into_iter()
        .map(|k| (quest_rows::user_id.eq(uid), quest_rows::day.eq(day), quest_rows::key.eq(k)))
        .collect();
    diesel::insert_into(quest_rows::table).values(rows).on_conflict_do_nothing().execute(conn).await?;
    Ok(())
}

/// Counts what happened: stats, experience, quests, levels, achievements.
/// Must run inside the action's transaction.
pub async fn apply(conn: &mut AsyncPgConnection, uid: i64, events: &[Event]) -> ApiResult<Progress> {
    // The row lock: two actions at once count one after the other
    let old_xp: i64 = users::table.find(uid).select(users::xp).for_update().first(conn).await?;
    let mut out = Progress {
        xp_gained: 0,
        level: levels::dto(old_xp),
        level_ups: vec![],
        quests: vec![],
        bonus: None,
        achievements: vec![],
        tickets: vec![],
        coins: 0,
    };

    record_stats(conn, uid, events).await?;
    out.xp_gained = events.iter().map(events::xp).sum();
    advance_quests(conn, uid, events, &mut out).await?;

    let xp = old_xp + out.xp_gained;
    diesel::update(users::table.find(uid)).set(users::xp.eq(xp)).execute(conn).await?;
    for level in levels::level_for(old_xp) + 1..=levels::level_for(xp) {
        let reward = levels::reward(level);
        give(conn, uid, &reward, &mut out).await?;
        out.level_ups.push(Reward { what: level.to_string(), reward });
    }
    out.level = levels::dto(xp);

    award_achievements(conn, uid, events, out.level.level, &mut out).await?;
    out.coins = users::table.find(uid).select(users::coins).first(conn).await?;
    Ok(out)
}

async fn give(conn: &mut AsyncPgConnection, uid: i64, reward: &Prize, out: &mut Progress) -> ApiResult<()> {
    let g = grant(conn, uid, reward, Source::Reward).await?;
    out.tickets.extend(g.tickets);
    Ok(())
}

async fn record_stats(conn: &mut AsyncPgConnection, uid: i64, events: &[Event]) -> ApiResult<()> {
    let scratches: Vec<&Scratch> = events
        .iter()
        .filter_map(|e| match e {
            Event::Scratched(s) => Some(s),
            _ => None,
        })
        .collect();
    let scratched = scratches.len() as i32;
    let wins = scratches.iter().filter(|s| s.win).count() as i32;
    let best = scratches.iter().map(|s| s.coins).max().unwrap_or(0);
    diesel::sql_query(
        "INSERT INTO stats (user_id, scratched, wins, best_win) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id) DO UPDATE SET
           scratched = stats.scratched + EXCLUDED.scratched,
           wins = stats.wins + EXCLUDED.wins,
           best_win = GREATEST(stats.best_win, EXCLUDED.best_win)",
    )
    .bind::<BigInt, _>(uid)
    .bind::<Integer, _>(scratched)
    .bind::<Integer, _>(wins)
    .bind::<BigInt, _>(best)
    .execute(conn)
    .await?;
    Ok(())
}

async fn advance_quests(conn: &mut AsyncPgConnection, uid: i64, events: &[Event], out: &mut Progress) -> ApiResult<()> {
    let day = quest_day(conn, uid).await?;
    ensure_quests(conn, uid, day).await?;
    let rows: Vec<(String, i32, Option<DateTime<Utc>>)> = quest_rows::table
        .filter(quest_rows::user_id.eq(uid))
        .filter(quest_rows::day.eq(day))
        .select((quest_rows::key, quest_rows::progress, quest_rows::done_at))
        .load(conn)
        .await?;
    let mut done = 0;
    let mut real = 0;
    for (key, progress, done_at) in rows {
        let Some(q) = quests::quest(&key) else { continue };
        real += 1;
        if done_at.is_some() {
            done += 1;
            continue;
        }
        let gained: i32 = events.iter().map(|e| quests::counts(&key, e)).sum();
        if gained == 0 {
            continue;
        }
        let now = (progress.saturating_add(gained)).min(q.goal);
        let finished = now >= q.goal;
        diesel::update(quest_rows::table.find((uid, day, &key)))
            .set((quest_rows::progress.eq(now), quest_rows::done_at.eq(finished.then(Utc::now))))
            .execute(conn)
            .await?;
        if finished {
            done += 1;
            out.xp_gained += events::QUEST_XP;
            diesel::update(stats::table.find(uid))
                .set(stats::quests_done.eq(stats::quests_done + 1))
                .execute(conn)
                .await?;
            give(conn, uid, &q.reward, out).await?;
            out.quests.push(Reward { what: key, reward: q.reward });
        }
    }
    if real == quests::PER_DAY && done == real && !out.quests.is_empty() {
        // Only the first time the day's three are done
        let paid = diesel::sql_query(
            "INSERT INTO quest_days (user_id, day, bonus_at) VALUES ($1, $2, now())
             ON CONFLICT (user_id, day) DO UPDATE SET bonus_at = now() WHERE quest_days.bonus_at IS NULL",
        )
        .bind::<BigInt, _>(uid)
        .bind::<diesel::sql_types::Date, _>(day)
        .execute(conn)
        .await?;
        if paid == 1 {
            out.xp_gained += events::BONUS_XP;
            give(conn, uid, &quests::BONUS, out).await?;
            out.bonus = Some(quests::BONUS);
        }
    }
    Ok(())
}

/// A player's numbers for the counting achievements
pub async fn metric(conn: &mut AsyncPgConnection, uid: i64, m: Metric, level: i32) -> ApiResult<i64> {
    let stat = |row: Option<(i32, i32, i32)>, i: usize| row.map_or(0, |r| [r.0, r.1, r.2][i] as i64);
    Ok(match m {
        Metric::Scratched | Metric::Wins | Metric::QuestsDone => {
            let row: Option<(i32, i32, i32)> = stats::table
                .find(uid)
                .select((stats::scratched, stats::wins, stats::quests_done))
                .first(conn)
                .await
                .optional()?;
            stat(row, [Metric::Scratched, Metric::Wins, Metric::QuestsDone].iter().position(|x| *x == m).unwrap())
        }
        Metric::BestStreak => {
            streaks::table.find(uid).select(streaks::best).first::<i32>(conn).await.optional()?.unwrap_or(0) as i64
        }
        Metric::Cosmetics => cosmetics::table.filter(cosmetics::user_id.eq(uid)).count().get_result(conn).await?,
        Metric::Species => aquarium::table.filter(aquarium::user_id.eq(uid)).count().get_result(conn).await?,
        Metric::Level => level as i64,
    })
}

async fn award_achievements(
    conn: &mut AsyncPgConnection,
    uid: i64,
    events: &[Event],
    level: i32,
    out: &mut Progress,
) -> ApiResult<()> {
    let have: Vec<String> = earned::table.filter(earned::user_id.eq(uid)).select(earned::id).load(conn).await?;
    for a in achievements::all() {
        if have.iter().any(|h| h == a.id) {
            continue;
        }
        let met = match a.goal {
            Goal::Reach(m, n) => metric(conn, uid, m, level).await? >= n,
            Goal::Moment(moment) => events.iter().any(|e| achievements::caught(moment, e)),
        };
        if !met {
            continue;
        }
        let added = diesel::insert_into(earned::table)
            .values((earned::user_id.eq(uid), earned::id.eq(a.id)))
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;
        if added == 1 {
            give(conn, uid, &a.reward, out).await?;
            out.achievements.push(Reward { what: a.id.to_owned(), reward: a.reward });
        }
    }
    Ok(())
}
