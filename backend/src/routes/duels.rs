//! Duels: challenge a player, both stake, a dice-off decides. Each player
//! gets a duel ticket showing the same roll from their side.

use actix_web::{get, post, web};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use super::tickets::issue_drawn;
use crate::AppState;
use crate::auth::CurrentUser;
use crate::error::{ApiError, ApiResult};
use crate::live::Event as Live;
use crate::schema::{duels, users};
use crate::tickets::duel::{self, STAKE};
use crate::tickets::{Series, Source};

/// An open challenge lapses after this long, and its stake comes back
pub const EXPIRES: Duration = Duration::hours(24);

diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

#[derive(Debug, Serialize)]
pub struct DuelDto {
    pub id: i64,
    pub challenger: String,
    pub opponent: String,
    pub stake: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub winner: Option<String>,
}

async fn name_of(conn: &mut AsyncPgConnection, uid: i64) -> ApiResult<String> {
    Ok(users::table.find(uid).select(users::username).first(conn).await?)
}

async fn dto(conn: &mut AsyncPgConnection, id: i64) -> ApiResult<DuelDto> {
    let (c, o, stake, status, created_at, winner): (i64, i64, i64, String, DateTime<Utc>, Option<i64>) = duels::table
        .find(id)
        .select((
            duels::challenger_id,
            duels::opponent_id,
            duels::stake,
            duels::status,
            duels::created_at,
            duels::winner_id,
        ))
        .first(conn)
        .await?;
    let winner = match winner {
        Some(w) => Some(name_of(conn, w).await?),
        None => None,
    };
    Ok(DuelDto {
        id,
        challenger: name_of(conn, c).await?,
        opponent: name_of(conn, o).await?,
        stake,
        status,
        created_at,
        winner,
    })
}

async fn pay(conn: &mut AsyncPgConnection, uid: i64, amount: i64) -> ApiResult<bool> {
    Ok(diesel::update(users::table.find(uid).filter(users::coins.ge(-amount)))
        .set(users::coins.eq(users::coins + amount))
        .execute(conn)
        .await?
        == 1)
}

#[derive(Deserialize)]
pub struct Challenge {
    opponent: String,
}

/// Challenges a player: the stake is taken now, given back if it never happens
#[post("/duels")]
pub async fn challenge(
    state: web::Data<AppState>,
    user: CurrentUser,
    body: web::Json<Challenge>,
) -> ApiResult<web::Json<DuelDto>> {
    user.member()?;
    let wanted = body.into_inner().opponent.to_lowercase();
    let mut conn = state.pool.get().await?;
    let uid = user.id;
    let (id, oid) = conn
        .transaction(async move |conn| {
            let (oid, guest): (i64, bool) = users::table
                .filter(lower(users::username).eq(&wanted))
                .select((users::id, users::is_guest))
                .first(conn)
                .await
                .optional()?
                .ok_or(ApiError::NotFound("player"))?;
            if oid == uid {
                return Err(ApiError::Invalid("duel_self"));
            }
            if guest {
                return Err(ApiError::Conflict("player_is_guest"));
            }
            if !pay(conn, uid, -STAKE).await? {
                return Err(ApiError::Conflict("not_enough_coins"));
            }
            let id: i64 = diesel::insert_into(duels::table)
                .values((duels::challenger_id.eq(uid), duels::opponent_id.eq(oid), duels::stake.eq(STAKE)))
                .returning(duels::id)
                .get_result(conn)
                .await?;
            Ok::<_, ApiError>((id, oid))
        })
        .await?;
    state.live.notify(oid, Live::Duel { id, from: user.username.clone(), status: "open".into() });
    Ok(web::Json(dto(&mut conn, id).await?))
}

#[get("/duels")]
pub async fn list(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<Vec<DuelDto>>> {
    user.member()?;
    let mut conn = state.pool.get().await?;
    let ids: Vec<i64> = duels::table
        .filter(duels::challenger_id.eq(user.id).or(duels::opponent_id.eq(user.id)))
        .filter(duels::status.eq("open").or(duels::created_at.gt(Utc::now() - Duration::days(2))))
        .order(duels::id.desc())
        .limit(20)
        .select(duels::id)
        .load(&mut conn)
        .await?;
    let mut out = vec![];
    for id in ids {
        out.push(dto(&mut conn, id).await?);
    }
    Ok(web::Json(out))
}

#[derive(Deserialize)]
pub struct Answer {
    /// accept, decline, or cancel (the challenger's)
    action: String,
}

/// Accepting pays the stake and plays it out; declining or cancelling refunds the challenger
#[post("/duels/{id}")]
pub async fn answer(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<i64>,
    body: web::Json<Answer>,
) -> ApiResult<web::Json<DuelDto>> {
    user.member()?;
    let id = path.into_inner();
    let action = body.into_inner().action;
    let uid = user.id;
    let mut conn = state.pool.get().await?;
    let (other, status) = conn
        .transaction(async move |conn| {
            let row: Option<(i64, i64, i64)> = duels::table
                .find(id)
                .filter(duels::status.eq("open"))
                .select((duels::challenger_id, duels::opponent_id, duels::stake))
                .for_update()
                .first(conn)
                .await
                .optional()?;
            let (c, o, stake) = row.ok_or(ApiError::NotFound("duel"))?;
            let now = Utc::now();
            match (action.as_str(), uid == c, uid == o) {
                ("accept", _, true) => {
                    if !pay(conn, o, -stake).await? {
                        return Err(ApiError::Conflict("not_enough_coins"));
                    }
                    let (cn, on) = (name_of(conn, c).await?, name_of(conn, o).await?);
                    let (a, b) = duel::roll(&mut rand::rng());
                    let winner = if a.iter().sum::<u8>() > b.iter().sum::<u8>() { c } else { o };
                    issue_drawn(conn, c, Series::Duel, Source::Duel, duel::ticket(&on, &a, &b)).await?;
                    issue_drawn(conn, o, Series::Duel, Source::Duel, duel::ticket(&cn, &b, &a)).await?;
                    diesel::update(duels::table.find(id))
                        .set((duels::status.eq("accepted"), duels::decided_at.eq(now), duels::winner_id.eq(winner)))
                        .execute(conn)
                        .await?;
                    Ok((c, "accepted"))
                }
                ("decline", _, true) | ("cancel", true, _) => {
                    pay(conn, c, stake).await?;
                    let status = if action == "decline" { "declined" } else { "cancelled" };
                    diesel::update(duels::table.find(id))
                        .set((duels::status.eq(status), duels::decided_at.eq(now)))
                        .execute(conn)
                        .await?;
                    Ok((if uid == c { o } else { c }, status))
                }
                _ => Err(ApiError::Invalid("not_your_duel")),
            }
        })
        .await?;
    state.live.notify(other, Live::Duel { id, from: user.username.clone(), status: status.into() });
    Ok(web::Json(dto(&mut conn, id).await?))
}

/// Open challenges past their time: refunded and closed. Returns (challenger, duel id) for each.
pub async fn expire_due(conn: &mut AsyncPgConnection) -> ApiResult<Vec<(i64, i64)>> {
    let due: Vec<(i64, i64, i64)> = duels::table
        .filter(duels::status.eq("open"))
        .filter(duels::created_at.lt(Utc::now() - EXPIRES))
        .select((duels::id, duels::challenger_id, duels::stake))
        .load(conn)
        .await?;
    let mut out = vec![];
    for (id, c, stake) in due {
        let done = conn
            .transaction(async move |conn| {
                let n = diesel::update(duels::table.find(id).filter(duels::status.eq("open")))
                    .set((duels::status.eq("expired"), duels::decided_at.eq(Utc::now())))
                    .execute(conn)
                    .await?;
                if n == 1 {
                    pay(conn, c, stake).await?;
                }
                Ok::<_, ApiError>(n == 1)
            })
            .await?;
        if done {
            out.push((c, id));
        }
    }
    Ok(out)
}
