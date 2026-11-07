//! Quests and achievements, as the player sees them

use actix_web::{get, post, web};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::CurrentUser;
use crate::error::{ApiError, ApiResult};
use crate::progress::{self, achievements, levels, quests};
use crate::schema::{achievements as earned, quest_slots, quests as rows, users};
use crate::tickets::Prize;

#[derive(Debug, Serialize)]
pub struct QuestDto {
    pub key: String,
    pub goal: i32,
    pub progress: i32,
    pub done: bool,
    pub reward: Prize,
}

#[derive(Debug, Serialize)]
pub struct QuestsDto {
    /// The round on now: its three quests
    pub items: Vec<QuestDto>,
    /// 1 to `rounds`: finishing one opens the next, harder
    pub round: i32,
    pub rounds: i32,
    pub bonus: Prize,
    /// This round's three done and its bonus paid (the last round: the hour is done)
    pub bonus_done: bool,
    /// The hour's one swap is spent
    pub rerolled: bool,
    pub resets_at: DateTime<Utc>,
}

async fn quests_dto(conn: &mut diesel_async::AsyncPgConnection, uid: i64) -> ApiResult<QuestsDto> {
    let slot = progress::quest_slot(Utc::now());
    let round = progress::ensure_quests(conn, uid, slot).await?;
    let found: Vec<(String, i32, Option<DateTime<Utc>>)> = rows::table
        .filter(rows::user_id.eq(uid))
        .filter(rows::slot.eq(slot))
        .filter(rows::round.eq(round))
        .order(rows::key)
        .select((rows::key, rows::progress, rows::done_at))
        .load(conn)
        .await?;
    let items = found
        .into_iter()
        .filter_map(|(key, progress, done)| {
            let q = quests::quest(&key)?;
            Some(QuestDto { key, goal: q.goal_in(round), progress, done: done.is_some(), reward: q.reward_in(round) })
        })
        .collect();
    let flags: Option<(bool, i32)> = quest_slots::table
        .find((uid, slot))
        .select((quest_slots::rerolled, quest_slots::bonuses))
        .first(conn)
        .await
        .optional()?;
    let (rerolled, bonuses) = flags.unwrap_or((false, 0));
    Ok(QuestsDto {
        items,
        round,
        rounds: quests::ROUNDS,
        bonus: quests::bonus(round),
        bonus_done: bonuses >= round,
        rerolled,
        resets_at: slot + chrono::Duration::hours(1),
    })
}

#[get("/quests")]
pub async fn get_quests(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<QuestsDto>> {
    let mut conn = state.pool.get().await?;
    Ok(web::Json(quests_dto(&mut conn, user.id).await?))
}

#[derive(Debug, Deserialize)]
pub struct Reroll {
    key: String,
}

/// Swaps one unfinished quest for another, once an hour
#[post("/quests/reroll")]
pub async fn reroll(
    state: web::Data<AppState>,
    user: CurrentUser,
    body: web::Json<Reroll>,
) -> ApiResult<web::Json<QuestsDto>> {
    let key = body.into_inner().key;
    let mut conn = state.pool.get().await?;
    let out = conn
        .transaction(async move |conn| {
            let xp: i64 = users::table.find(user.id).select(users::xp).for_update().first(conn).await?;
            let slot = progress::quest_slot(Utc::now());
            let round = progress::ensure_quests(conn, user.id, slot).await?;
            let rerolled: bool = quest_slots::table
                .find((user.id, slot))
                .select(quest_slots::rerolled)
                .first(conn)
                .await
                .optional()?
                .unwrap_or(false);
            if rerolled {
                return Err(ApiError::Conflict("already_rerolled"));
            }
            let done: Option<Option<DateTime<Utc>>> =
                rows::table.find((user.id, slot, round, &key)).select(rows::done_at).first(conn).await.optional()?;
            match done {
                None => return Err(ApiError::NotFound("quest")),
                Some(Some(_)) => return Err(ApiError::Conflict("quest_done")),
                Some(None) => {}
            }
            let have: Vec<String> = rows::table
                .filter(rows::user_id.eq(user.id))
                .filter(rows::slot.eq(slot))
                .filter(rows::round.eq(round))
                .select(rows::key)
                .load(conn)
                .await?;
            let new = quests::reroll(user.id, slot, round, levels::level_for(xp), user.is_guest, &have)
                .ok_or(ApiError::Conflict("no_other_quest"))?;
            diesel::delete(rows::table.find((user.id, slot, round, &key))).execute(conn).await?;
            diesel::insert_into(rows::table)
                .values((rows::user_id.eq(user.id), rows::slot.eq(slot), rows::round.eq(round), rows::key.eq(new)))
                .execute(conn)
                .await?;
            diesel::insert_into(quest_slots::table)
                .values((quest_slots::user_id.eq(user.id), quest_slots::slot.eq(slot), quest_slots::rerolled.eq(true)))
                .on_conflict((quest_slots::user_id, quest_slots::slot))
                .do_update()
                .set(quest_slots::rerolled.eq(true))
                .execute(conn)
                .await?;
            quests_dto(conn, user.id).await
        })
        .await?;
    Ok(web::Json(out))
}

#[derive(Debug, Serialize)]
pub struct AchievementDto {
    pub id: &'static str,
    pub reward: Prize,
    pub earned_at: Option<DateTime<Utc>>,
    /// For counting ones: where the player stands
    pub progress: Option<(i64, i64)>,
}

#[get("/achievements")]
pub async fn get_achievements(
    state: web::Data<AppState>,
    user: CurrentUser,
) -> ApiResult<web::Json<Vec<AchievementDto>>> {
    let mut conn = state.pool.get().await?;
    let xp: i64 = users::table.find(user.id).select(users::xp).first(&mut conn).await?;
    let level = levels::level_for(xp);
    let have: Vec<(String, DateTime<Utc>)> = earned::table
        .filter(earned::user_id.eq(user.id))
        .select((earned::id, earned::earned_at))
        .load(&mut conn)
        .await?;
    let mut out = Vec::new();
    for a in achievements::all() {
        let earned_at = have.iter().find(|(id, _)| id == a.id).map(|(_, at)| *at);
        let progress = match a.goal {
            achievements::Goal::Reach(m, n) => Some((progress::metric(&mut conn, user.id, m, level).await?.min(n), n)),
            achievements::Goal::Moment(_) => None,
        };
        out.push(AchievementDto { id: a.id, reward: a.reward, earned_at, progress });
    }
    Ok(web::Json(out))
}
