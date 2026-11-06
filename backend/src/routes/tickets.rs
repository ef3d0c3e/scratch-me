//! Issuing and scratching tickets

use actix_web::{post, web};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

use crate::AppState;
use crate::auth::CurrentUser;
use crate::cosmetics::DUPLICATE_COINS;
use crate::error::{ApiError, ApiResult};
use crate::pocket;
use crate::schema::{cosmetics, tickets, users};
use crate::tickets::{Prize, Series, Source};

/// An unscratched ticket, as the player sees it: no layout, no prize
#[derive(Debug, Serialize, Queryable)]
pub struct TicketDto {
    pub id: i64,
    pub series: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Draws `n` tickets of `series` and gives them to the player
pub(crate) async fn issue(
    conn: &mut AsyncPgConnection,
    user_id: i64,
    series: Series,
    source: Source,
    n: i32,
) -> ApiResult<Vec<TicketDto>> {
    if n <= 0 {
        return Ok(Vec::new());
    }
    let rows: Vec<_> = {
        let mut rng = rand::rng();
        (0..n)
            .map(|_| {
                let drawn = series.draw(&mut rng);
                (
                    tickets::user_id.eq(user_id),
                    tickets::series.eq(series.key()),
                    tickets::layout.eq(drawn.layout),
                    tickets::prize.eq(serde_json::to_value(drawn.prize).expect("prize serializes")),
                    tickets::source.eq(source.key()),
                )
            })
            .collect()
    };
    let issued = diesel::insert_into(tickets::table)
        .values(&rows)
        .returning((tickets::id, tickets::series, tickets::source, tickets::created_at))
        .get_results(conn)
        .await?;
    Ok(issued)
}

/// Draws one ticket of each series listed, in order
pub(crate) async fn issue_each(
    conn: &mut AsyncPgConnection,
    user_id: i64,
    series: &[Series],
    source: Source,
) -> ApiResult<Vec<TicketDto>> {
    let mut out = Vec::with_capacity(series.len());
    for s in series {
        out.extend(issue(conn, user_id, *s, source, 1).await?);
    }
    Ok(out)
}

/// What a prize did to the player's account
pub(crate) struct Granted {
    /// The balance after it
    pub coins: i64,
    pub tickets: Vec<TicketDto>,
    /// A cosmetic already owned, paid in coins instead
    pub duplicate: bool,
}

/// Gives the player a prize, inside the caller's transaction
pub(crate) async fn grant(conn: &mut AsyncPgConnection, user_id: i64, prize: &Prize) -> ApiResult<Granted> {
    let add = async |conn: &mut AsyncPgConnection, amount: i64| -> ApiResult<i64> {
        Ok(diesel::update(users::table.find(user_id))
            .set(users::coins.eq(users::coins + amount))
            .returning(users::coins)
            .get_result(conn)
            .await?)
    };
    let mut tickets = Vec::new();
    let mut duplicate = false;
    let coins = match prize {
        Prize::Nothing => add(conn, 0).await?,
        Prize::Coins { amount } => add(conn, *amount).await?,
        Prize::Ticket { series } => {
            tickets = issue(conn, user_id, *series, Source::Prize, 1).await?;
            add(conn, 0).await?
        }
        Prize::Tickets { series } => {
            tickets = issue_each(conn, user_id, series, Source::Prize).await?;
            add(conn, 0).await?
        }
        Prize::PocketRefill => {
            diesel::update(users::table.find(user_id))
                .set((users::pocket_count.eq(pocket::SIZE), users::pocket_at.eq(Utc::now())))
                .returning(users::coins)
                .get_result(conn)
                .await?
        }
        Prize::Cosmetic { item } => {
            let added = !item.is_default()
                && diesel::insert_into(cosmetics::table)
                    .values((
                        cosmetics::user_id.eq(user_id),
                        cosmetics::kind.eq(item.kind()),
                        cosmetics::item.eq(item.item()),
                    ))
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .await?
                    == 1;
            duplicate = !added;
            add(conn, if added { 0 } else { DUPLICATE_COINS }).await?
        }
    };
    Ok(Granted { coins, tickets, duplicate })
}

#[derive(Debug, Serialize)]
pub struct Scratched {
    pub id: i64,
    pub series: String,
    pub layout: serde_json::Value,
    pub prize: Prize,
    /// The balance after the prize
    pub coins: i64,
    /// Tickets won
    pub tickets: Vec<TicketDto>,
    /// The cosmetic won was already owned: coins instead
    pub duplicate: bool,
}

/// Uncovers a ticket: its layout, and the prize credited, once
#[post("/tickets/{id}/scratch")]
pub async fn scratch(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<i64>,
) -> ApiResult<web::Json<Scratched>> {
    let id = path.into_inner();
    let mut conn = state.pool.get().await?;
    let out = conn
        .transaction(async move |conn| {
            // Only one request can flip scratched_at: a double tap pays once
            let row: Option<(String, serde_json::Value, serde_json::Value)> = diesel::update(
                tickets::table
                    .filter(tickets::id.eq(id))
                    .filter(tickets::user_id.eq(user.id))
                    .filter(tickets::scratched_at.is_null()),
            )
            .set(tickets::scratched_at.eq(Utc::now()))
            .returning((tickets::series, tickets::layout, tickets::prize))
            .get_result(conn)
            .await
            .optional()?;
            let Some((series, layout, prize)) = row else {
                let exists: bool = diesel::select(diesel::dsl::exists(
                    tickets::table.filter(tickets::id.eq(id)).filter(tickets::user_id.eq(user.id)),
                ))
                .get_result(conn)
                .await?;
                return Err(if exists {
                    ApiError::Conflict("already_scratched")
                } else {
                    ApiError::NotFound("ticket")
                });
            };
            let prize: Prize = serde_json::from_value(prize).map_err(|e| ApiError::Internal(e.into()))?;
            let g = grant(conn, user.id, &prize).await?;
            Ok::<_, ApiError>(Scratched {
                id,
                series,
                layout,
                prize,
                coins: g.coins,
                tickets: g.tickets,
                duplicate: g.duplicate,
            })
        })
        .await?;
    Ok(web::Json(out))
}
