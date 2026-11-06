//! The shared daily pot: Pot Tickets put coins in it, and one entry of each
//! (UTC) day wins it all after midnight.

use actix_web::{get, web};
use chrono::{DateTime, Days, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

use crate::AppState;
use crate::auth::CurrentUser;
use crate::error::{ApiError, ApiResult};
use crate::schema::{pot_entries, pots, users};
use crate::tickets::pot_ticket::{SEED, TO_POT, winner};

pub fn today() -> NaiveDate {
    Utc::now().date_naive()
}

/// One entry in today's pot, and its share of the price into it
pub async fn enter(conn: &mut AsyncPgConnection, uid: i64) -> ApiResult<()> {
    let day = today();
    diesel::insert_into(pots::table)
        .values((pots::day.eq(day), pots::amount.eq(SEED + TO_POT)))
        .on_conflict(pots::day)
        .do_update()
        .set(pots::amount.eq(pots::amount + TO_POT))
        .execute(conn)
        .await?;
    diesel::insert_into(pot_entries::table)
        .values((pot_entries::day.eq(day), pot_entries::user_id.eq(uid)))
        .execute(conn)
        .await?;
    Ok(())
}

/// Draws every past day's pot not drawn yet: pays the winner. Returns (winner's name, amount) per pot won.
pub async fn draw_due(conn: &mut AsyncPgConnection) -> ApiResult<Vec<(String, i64)>> {
    let due: Vec<(NaiveDate, i64)> = pots::table
        .filter(pots::day.lt(today()))
        .filter(pots::drawn_at.is_null())
        .select((pots::day, pots::amount))
        .load(conn)
        .await?;
    let mut won = vec![];
    for (day, amount) in due {
        let r = conn
            .transaction(async move |conn| {
                // The lock: two servers drawing at once draw once
                let still: Option<NaiveDate> = pots::table
                    .find(day)
                    .filter(pots::drawn_at.is_null())
                    .select(pots::day)
                    .for_update()
                    .first(conn)
                    .await
                    .optional()?;
                if still.is_none() {
                    return Ok::<_, ApiError>(None);
                }
                let entries: Vec<i64> =
                    pot_entries::table.filter(pot_entries::day.eq(day)).select(pot_entries::user_id).load(conn).await?;
                let pick = winner(&entries, &mut rand::rng());
                diesel::update(pots::table.find(day))
                    .set((pots::drawn_at.eq(Utc::now()), pots::winner_id.eq(pick)))
                    .execute(conn)
                    .await?;
                let Some(uid) = pick else { return Ok(None) };
                let name: String = diesel::update(users::table.find(uid))
                    .set(users::coins.eq(users::coins + amount))
                    .returning(users::username)
                    .get_result(conn)
                    .await?;
                Ok(Some((name, amount)))
            })
            .await?;
        won.extend(r);
    }
    Ok(won)
}

#[derive(Debug, Serialize)]
pub struct LastPot {
    pub day: NaiveDate,
    pub winner: Option<String>,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct PotDto {
    pub amount: i64,
    pub entries: i64,
    /// The viewer's entries today
    pub mine: i64,
    pub draws_at: DateTime<Utc>,
    pub last: Option<LastPot>,
}

#[get("/pot")]
pub async fn get_pot(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<PotDto>> {
    let mut conn = state.pool.get().await?;
    let day = today();
    let amount: i64 = pots::table.find(day).select(pots::amount).first(&mut conn).await.optional()?.unwrap_or(SEED);
    let entries: i64 = pot_entries::table.filter(pot_entries::day.eq(day)).count().get_result(&mut conn).await?;
    let mine: i64 = pot_entries::table
        .filter(pot_entries::day.eq(day))
        .filter(pot_entries::user_id.eq(user.id))
        .count()
        .get_result(&mut conn)
        .await?;
    let last: Option<(NaiveDate, i64, Option<String>)> = pots::table
        .left_join(users::table.on(users::id.nullable().eq(pots::winner_id)))
        .filter(pots::drawn_at.is_not_null())
        .order(pots::day.desc())
        .select((pots::day, pots::amount, users::username.nullable()))
        .first(&mut conn)
        .await
        .optional()?;
    let draws_at = (day + Days::new(1)).and_hms_opt(0, 0, 0).expect("midnight").and_utc();
    Ok(web::Json(PotDto {
        amount,
        entries,
        mine,
        draws_at,
        last: last.map(|(day, amount, winner)| LastPot { day, winner, amount }),
    }))
}
