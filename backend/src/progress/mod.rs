//! Progression: experience and levels, daily quests, achievements.
//!
//! Every action that counts calls `apply` inside its own transaction with
//! what happened, so progress lands with the action or not at all. `apply`
//! answers what it gave, for the client to celebrate.

pub mod achievements;
pub mod events;
pub mod levels;
mod pace;
pub mod quests;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Integer};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

pub use events::{Event, Scratch};

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
    /// A quest's goal, which grows with the round
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<i32>,
}

/// What an action's progress gave
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub xp_gained: i64,
    pub level: LevelDto,
    pub level_ups: Vec<Reward>,
    pub quests: Vec<Reward>,
    /// A round of quests all done: this bonus
    pub bonus: Option<Prize>,
    /// ...and the next round it opened
    pub next_round: Option<i32>,
    pub achievements: Vec<Reward>,
    /// Tickets the rewards gave
    pub tickets: Vec<TicketDto>,
    /// The balance after everything
    pub coins: i64,
}

/// The quests' hour now: its start. Every player's quests turn over on the hour.
pub fn quest_slot(now: DateTime<Utc>) -> DateTime<Utc> {
    quests::slot_of(now)
}

/// Stores the hour's first round of quests when it is not yet. Answers the round the player is on.
pub async fn ensure_quests(conn: &mut AsyncPgConnection, uid: i64, slot: DateTime<Utc>) -> ApiResult<i32> {
    let round: Option<i32> = quest_rows::table
        .filter(quest_rows::user_id.eq(uid))
        .filter(quest_rows::slot.eq(slot))
        .select(diesel::dsl::max(quest_rows::round))
        .first(conn)
        .await?;
    if let Some(round) = round {
        return Ok(round);
    }
    open_round(conn, uid, slot, 1).await?;
    Ok(1)
}

/// Draws and stores a round of quests
async fn open_round(conn: &mut AsyncPgConnection, uid: i64, slot: DateTime<Utc>, round: i32) -> ApiResult<()> {
    let (xp, guest): (i64, bool) = users::table.find(uid).select((users::xp, users::is_guest)).first(conn).await?;
    let rows: Vec<_> = quests::draw(uid, slot, round, levels::level_for(xp), guest)
        .into_iter()
        .map(|k| {
            (quest_rows::user_id.eq(uid), quest_rows::slot.eq(slot), quest_rows::round.eq(round), quest_rows::key.eq(k))
        })
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
        next_round: None,
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
        out.level_ups.push(Reward { what: level.to_string(), reward, goal: None });
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
    let won: i64 = scratches.iter().map(|s| s.coins.max(0)).sum();
    let bundles = events.iter().filter(|e| matches!(e, Event::BundleBought)).count() as i32;
    diesel::sql_query(
        "INSERT INTO stats (user_id, scratched, wins, best_win, coins_won, bundles) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id) DO UPDATE SET
           scratched = stats.scratched + EXCLUDED.scratched,
           wins = stats.wins + EXCLUDED.wins,
           best_win = GREATEST(stats.best_win, EXCLUDED.best_win),
           coins_won = stats.coins_won + EXCLUDED.coins_won,
           bundles = stats.bundles + EXCLUDED.bundles",
    )
    .bind::<BigInt, _>(uid)
    .bind::<Integer, _>(scratched)
    .bind::<Integer, _>(wins)
    .bind::<BigInt, _>(best)
    .bind::<BigInt, _>(won)
    .bind::<Integer, _>(bundles)
    .execute(conn)
    .await?;
    Ok(())
}

async fn advance_quests(conn: &mut AsyncPgConnection, uid: i64, events: &[Event], out: &mut Progress) -> ApiResult<()> {
    let slot = quest_slot(Utc::now());
    let round = ensure_quests(conn, uid, slot).await?;
    let rows: Vec<(String, i32, Option<DateTime<Utc>>)> = quest_rows::table
        .filter(quest_rows::user_id.eq(uid))
        .filter(quest_rows::slot.eq(slot))
        .filter(quest_rows::round.eq(round))
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
        let goal = q.goal_in(round);
        let now = (progress.saturating_add(gained)).min(goal);
        let finished = now >= goal;
        diesel::update(quest_rows::table.find((uid, slot, round, &key)))
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
            let reward = q.reward_in(round);
            give(conn, uid, &reward, out).await?;
            out.quests.push(Reward { what: key, reward, goal: Some(goal) });
        }
    }
    if real == quests::PER_DAY && done == real && !out.quests.is_empty() {
        // Once per round: the round's bonus, and the next round
        let paid = diesel::sql_query(
            "INSERT INTO quest_slots (user_id, slot, bonus_at, bonuses) VALUES ($1, $2, now(), $3)
             ON CONFLICT (user_id, slot) DO UPDATE SET bonus_at = now(), bonuses = EXCLUDED.bonuses
             WHERE quest_slots.bonuses < EXCLUDED.bonuses",
        )
        .bind::<BigInt, _>(uid)
        .bind::<diesel::sql_types::Timestamptz, _>(slot)
        .bind::<Integer, _>(round)
        .execute(conn)
        .await?;
        if paid == 1 {
            out.xp_gained += events::BONUS_XP;
            let bonus = quests::bonus(round);
            give(conn, uid, &bonus, out).await?;
            out.bonus = Some(bonus);
            if round < quests::ROUNDS {
                open_round(conn, uid, slot, round + 1).await?;
                out.next_round = Some(round + 1);
            }
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
        Metric::CoinsWon | Metric::Bundles => {
            let row: Option<(i64, i32)> =
                stats::table.find(uid).select((stats::coins_won, stats::bundles)).first(conn).await.optional()?;
            let (won, bundles) = row.unwrap_or((0, 0));
            if m == Metric::CoinsWon { won } else { bundles as i64 }
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
            out.achievements.push(Reward { what: a.id.to_owned(), reward: a.reward, goal: None });
        }
    }
    Ok(())
}
