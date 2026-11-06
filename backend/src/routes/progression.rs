//! Quests and achievements, as the player sees them

use actix_web::{get, post, web};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::{CurrentUser, parse_tz};
use crate::envelope;
use crate::error::{ApiError, ApiResult};
use crate::progress::{self, achievements, levels, quests};
use crate::schema::{achievements as earned, quest_days, quests as rows, users};
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
    pub items: Vec<QuestDto>,
    pub bonus: Prize,
    /// All three done and the bonus paid
    pub bonus_done: bool,
    /// The day's one reroll is spent
    pub rerolled: bool,
    pub resets_at: DateTime<Utc>,
}

async fn quests_dto(conn: &mut diesel_async::AsyncPgConnection, uid: i64) -> ApiResult<QuestsDto> {
    let day = progress::quest_day(conn, uid).await?;
    progress::ensure_quests(conn, uid, day).await?;
    let found: Vec<(String, i32, Option<DateTime<Utc>>)> = rows::table
        .filter(rows::user_id.eq(uid))
        .filter(rows::day.eq(day))
        .order(rows::key)
        .select((rows::key, rows::progress, rows::done_at))
        .load(conn)
        .await?;
    let items = found
        .into_iter()
        .filter_map(|(key, progress, done)| {
            let q = quests::quest(&key)?;
            Some(QuestDto { key, goal: q.goal, progress, done: done.is_some(), reward: q.reward })
        })
        .collect();
    let flags: Option<(bool, Option<DateTime<Utc>>)> = quest_days::table
        .find((uid, day))
        .select((quest_days::rerolled, quest_days::bonus_at))
        .first(conn)
        .await
        .optional()?;
    let (rerolled, bonus_at) = flags.unwrap_or((false, None));
    let tz: String = users::table.find(uid).select(users::tz).first(conn).await?;
    Ok(QuestsDto {
        items,
        bonus: quests::BONUS,
        bonus_done: bonus_at.is_some(),
        rerolled,
        resets_at: envelope::next_day_at(parse_tz(Some(&tz)), Utc::now()),
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

/// Swaps one unfinished quest for another, once a day
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
            let day = progress::quest_day(conn, user.id).await?;
            progress::ensure_quests(conn, user.id, day).await?;
            let rerolled: bool = quest_days::table
                .find((user.id, day))
                .select(quest_days::rerolled)
                .first(conn)
                .await
                .optional()?
                .unwrap_or(false);
            if rerolled {
                return Err(ApiError::Conflict("already_rerolled"));
            }
            let done: Option<Option<DateTime<Utc>>> =
                rows::table.find((user.id, day, &key)).select(rows::done_at).first(conn).await.optional()?;
            match done {
                None => return Err(ApiError::NotFound("quest")),
                Some(Some(_)) => return Err(ApiError::Conflict("quest_done")),
                Some(None) => {}
            }
            let have: Vec<String> = rows::table
                .filter(rows::user_id.eq(user.id))
                .filter(rows::day.eq(day))
                .select(rows::key)
                .load(conn)
                .await?;
            let new = quests::reroll(user.id, day, levels::level_for(xp), user.is_guest, &have)
                .ok_or(ApiError::Conflict("no_other_quest"))?;
            diesel::delete(rows::table.find((user.id, day, &key))).execute(conn).await?;
            diesel::insert_into(rows::table)
                .values((rows::user_id.eq(user.id), rows::day.eq(day), rows::key.eq(new)))
                .execute(conn)
                .await?;
            diesel::insert_into(quest_days::table)
                .values((quest_days::user_id.eq(user.id), quest_days::day.eq(day), quest_days::rerolled.eq(true)))
                .on_conflict((quest_days::user_id, quest_days::day))
                .do_update()
                .set(quest_days::rerolled.eq(true))
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
