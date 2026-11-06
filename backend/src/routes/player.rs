//! The player's own state, and the free drops: pocket and daily envelope

use actix_web::{get, post, web};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Serialize;

use super::tickets::{TicketDto, issue, issue_each};
use crate::AppState;
use crate::auth::{CurrentUser, parse_tz};
use crate::cosmetics::Cosmetic;
use crate::envelope::{self, Contents};
use crate::error::{ApiError, ApiResult};
use crate::pocket::{self, Pocket};
use crate::schema::{cosmetics, streaks, tickets, users};
use crate::tickets::{Series, Source};

#[derive(Debug, Serialize)]
pub struct EnvelopeDto {
    /// Not claimed yet today
    pub available: bool,
    /// Days in a row as of today (0 when broken)
    pub streak: i32,
    pub best: i32,
    /// The streak day the next claim counts as, and what it holds
    pub day: i32,
    pub contents: Contents,
    /// When the player's next day starts
    pub next_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MeDto {
    pub username: String,
    pub coins: i64,
    pub pocket: Pocket,
    pub envelope: EnvelopeDto,
    /// What the player has on
    pub theme: String,
    pub tool: String,
    /// Cosmetics owned besides the defaults
    pub owned: Vec<Cosmetic>,
    /// Unscratched tickets, oldest first
    pub tickets: Vec<TicketDto>,
}

#[get("/me")]
pub async fn get_me(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<MeDto>> {
    let mut conn = state.pool.get().await?;
    let (coins, tz, stored, at, theme, tool): (i64, String, i32, DateTime<Utc>, String, String) = users::table
        .find(user.id)
        .select((users::coins, users::tz, users::pocket_count, users::pocket_at, users::theme, users::tool))
        .first(&mut conn)
        .await?;
    let owned: Vec<(String, String)> = cosmetics::table
        .filter(cosmetics::user_id.eq(user.id))
        .order(cosmetics::obtained_at.asc())
        .select((cosmetics::kind, cosmetics::item))
        .load(&mut conn)
        .await?;
    let owned = owned.iter().filter_map(|(k, i)| Cosmetic::parse(k, i)).collect();
    let streak: Option<(i32, i32, Option<NaiveDate>)> = streaks::table
        .find(user.id)
        .select((streaks::day, streaks::best, streaks::last_claim))
        .first(&mut conn)
        .await
        .optional()?;
    let held = tickets::table
        .filter(tickets::user_id.eq(user.id))
        .filter(tickets::scratched_at.is_null())
        .order(tickets::id.asc())
        .select((tickets::id, tickets::series, tickets::source, tickets::created_at))
        .load(&mut conn)
        .await?;

    let now = Utc::now();
    let tz = parse_tz(Some(&tz));
    let today = envelope::today(tz, now);
    let (day, best, last) = streak.unwrap_or((0, 0, None));
    let next = envelope::claim(day, last, today);
    let envelope = EnvelopeDto {
        available: next.is_some(),
        streak: envelope::current(day, last, today),
        best,
        day: next.unwrap_or(day + 1),
        contents: envelope::contents(next.unwrap_or(day + 1)),
        next_at: envelope::next_day_at(tz, now),
    };
    Ok(web::Json(MeDto {
        username: user.username,
        coins,
        pocket: pocket::level(stored, at, now),
        envelope,
        theme,
        tool,
        owned,
        tickets: held,
    }))
}

#[derive(Debug, Serialize)]
pub struct PocketClaim {
    pub tickets: Vec<TicketDto>,
    pub pocket: Pocket,
}

/// Moves every ticket in the pocket to the player's stack
#[post("/pocket/claim")]
pub async fn claim_pocket(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<PocketClaim>> {
    let mut conn = state.pool.get().await?;
    let out = conn
        .transaction(async move |conn| {
            // The row lock makes concurrent claims take turns
            let (stored, at): (i32, DateTime<Utc>) = users::table
                .find(user.id)
                .select((users::pocket_count, users::pocket_at))
                .for_update()
                .first(conn)
                .await?;
            let now = Utc::now();
            let (n, new_at) = pocket::take(stored, at, now);
            if n == 0 {
                return Err(ApiError::Conflict("pocket_empty"));
            }
            diesel::update(users::table.find(user.id))
                .set((users::pocket_count.eq(0), users::pocket_at.eq(new_at)))
                .execute(conn)
                .await?;
            let tickets = issue(conn, user.id, Series::LuckyThree, Source::Pocket, n).await?;
            Ok::<_, ApiError>(PocketClaim { tickets, pocket: pocket::level(0, new_at, now) })
        })
        .await?;
    Ok(web::Json(out))
}

#[derive(Debug, Serialize)]
pub struct EnvelopeClaim {
    pub day: i32,
    pub contents: Contents,
    pub coins: i64,
    pub tickets: Vec<TicketDto>,
}

/// Opens today's envelope
#[post("/envelope/claim")]
pub async fn claim_envelope(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<EnvelopeClaim>> {
    let mut conn = state.pool.get().await?;
    let out = conn
        .transaction(async move |conn| {
            let tz: String = users::table.find(user.id).select(users::tz).for_update().first(conn).await?;
            let today = envelope::today(parse_tz(Some(&tz)), Utc::now());
            let (day, best, last): (i32, i32, Option<NaiveDate>) = streaks::table
                .find(user.id)
                .select((streaks::day, streaks::best, streaks::last_claim))
                .first(conn)
                .await
                .optional()?
                .unwrap_or((0, 0, None));
            let day = envelope::claim(day, last, today).ok_or(ApiError::Conflict("already_claimed"))?;
            let best = best.max(day);
            diesel::insert_into(streaks::table)
                .values((
                    streaks::user_id.eq(user.id),
                    streaks::day.eq(day),
                    streaks::best.eq(best),
                    streaks::last_claim.eq(today),
                ))
                .on_conflict(streaks::user_id)
                .do_update()
                .set((streaks::day.eq(day), streaks::best.eq(best), streaks::last_claim.eq(today)))
                .execute(conn)
                .await?;
            let contents = envelope::contents(day);
            let coins = diesel::update(users::table.find(user.id))
                .set(users::coins.eq(users::coins + contents.coins))
                .returning(users::coins)
                .get_result(conn)
                .await?;
            let tickets = issue_each(conn, user.id, &contents.tickets, Source::Envelope).await?;
            Ok::<_, ApiError>(EnvelopeClaim { day, contents, coins, tickets })
        })
        .await?;
    Ok(web::Json(out))
}
