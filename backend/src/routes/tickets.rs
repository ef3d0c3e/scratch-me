//! Issuing and scratching tickets

use actix_web::{get, post, web};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

use crate::AppState;
use crate::auth::CurrentUser;
use crate::cosmetics::DUPLICATE_COINS;
use crate::error::{ApiError, ApiResult};
use crate::pocket;
use crate::progress::{self, Event, Progress, Scratch};
use crate::routes::charms::{self, Boost, WornCharm};
use crate::schema::{aquarium, cosmetics, tickets, users};
use crate::tickets::play;
use crate::tickets::treasure_dig::{self, Layout as DigLayout, Spot};
use crate::tickets::{Prize, Series, Source};

/// An unscratched ticket, as the player sees it: no layout, no prize
#[derive(Debug, Clone, Serialize, Queryable)]
pub struct TicketDto {
    pub id: i64,
    pub series: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    /// Who sent it, for a gift
    pub from: Option<String>,
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
        .returning((tickets::id, tickets::series, tickets::source, tickets::created_at, tickets::from_name))
        .get_results(conn)
        .await?;
    Ok(issued)
}

/// Gives a player a ticket drawn already (a duel's), rather than drawing one
pub(crate) async fn issue_drawn(
    conn: &mut AsyncPgConnection,
    user_id: i64,
    series: Series,
    source: Source,
    drawn: crate::tickets::Drawn,
) -> ApiResult<TicketDto> {
    Ok(diesel::insert_into(tickets::table)
        .values((
            tickets::user_id.eq(user_id),
            tickets::series.eq(series.key()),
            tickets::layout.eq(drawn.layout),
            tickets::prize.eq(serde_json::to_value(drawn.prize).expect("prize serializes")),
            tickets::source.eq(source.key()),
        ))
        .returning((tickets::id, tickets::series, tickets::source, tickets::created_at, tickets::from_name))
        .get_result(conn)
        .await?)
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
    pub tickets: Vec<TicketDto>,
    /// A cosmetic already owned, paid in coins instead
    pub duplicate: bool,
}

/// Gives the player a prize, inside the caller's transaction
pub(crate) async fn grant(
    conn: &mut AsyncPgConnection,
    user_id: i64,
    prize: &Prize,
    source: Source,
) -> ApiResult<Granted> {
    let Prize::Bundle { items } = prize else { return grant_one(conn, user_id, prize, source).await };
    let mut out = Granted { tickets: vec![], duplicate: false };
    for item in items {
        let g = grant_one(conn, user_id, item, source).await?;
        out.tickets.extend(g.tickets);
        out.duplicate |= g.duplicate;
    }
    Ok(out)
}

async fn grant_one(conn: &mut AsyncPgConnection, user_id: i64, prize: &Prize, source: Source) -> ApiResult<Granted> {
    let add = async |conn: &mut AsyncPgConnection, amount: i64| -> ApiResult<i64> {
        Ok(diesel::update(users::table.find(user_id))
            .set(users::coins.eq(users::coins + amount))
            .returning(users::coins)
            .get_result(conn)
            .await?)
    };
    let mut tickets = Vec::new();
    let mut duplicate = false;
    match prize {
        Prize::Nothing => add(conn, 0).await?,
        Prize::Coins { amount } => add(conn, *amount).await?,
        Prize::Ticket { series } => {
            tickets = issue(conn, user_id, *series, source, 1).await?;
            add(conn, 0).await?
        }
        Prize::Tickets { series } => {
            tickets = issue_each(conn, user_id, series, source).await?;
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
        // Flattened by `grant`
        Prize::Bundle { .. } => add(conn, 0).await?,
    };
    Ok(Granted { tickets, duplicate })
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
    /// A luck charm drew this losing ticket again: the layout and prize are the new draw
    pub second_chance: bool,
    /// Coins a payout charm added to the prize
    pub charm_bonus: i64,
    /// Charms worn after this ticket
    pub charms: Vec<WornCharm>,
    pub progress: Progress,
    #[serde(skip)]
    pub(crate) boost: Boost,
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
    let uid = user.id;
    let out = conn
        .transaction(async move |conn| {
            // Only one request can flip scratched_at: a double tap pays once
            let row: Option<(String, serde_json::Value, serde_json::Value)> = diesel::update(
                tickets::table
                    .filter(tickets::id.eq(id))
                    .filter(tickets::user_id.eq(uid))
                    .filter(tickets::scratched_at.is_null())
                    .filter(tickets::series.ne(Series::TreasureDig.key()))
                    .filter(diesel::dsl::not(tickets::series.eq_any(play::GAMES.iter().map(|s| s.key())))),
            )
            .set(tickets::scratched_at.eq(Utc::now()))
            .returning((tickets::series, tickets::layout, tickets::prize))
            .get_result(conn)
            .await
            .optional()?;
            let Some((series, mut layout, prize)) = row else {
                let exists: bool = diesel::select(diesel::dsl::exists(
                    tickets::table.filter(tickets::id.eq(id)).filter(tickets::user_id.eq(uid)),
                ))
                .get_result(conn)
                .await?;
                if !exists {
                    return Err(ApiError::NotFound("ticket"));
                }
                let series: String = tickets::table.find(id).select(tickets::series).first(conn).await?;
                let played = Series::parse(&series).is_some_and(|s| s.is_played());
                return Err(if series == Series::TreasureDig.key() {
                    ApiError::Invalid("dig_ticket")
                } else if played {
                    ApiError::Invalid("played_ticket")
                } else {
                    ApiError::Conflict("already_scratched")
                });
            };
            let mut prize: Prize = serde_json::from_value(prize).map_err(|e| ApiError::Internal(e.into()))?;
            let kind = Series::parse(&series).ok_or_else(|| ApiError::Internal(anyhow::anyhow!("series {series}")))?;
            let mut boost = Boost::default();
            charms::luck(conn, uid, id, kind, &mut layout, &mut prize, &mut boost).await?;
            let g = grant(conn, uid, &prize, Source::Prize).await?;
            charms::payout(conn, uid, kind, &prize, &mut boost).await?;
            if kind == Series::Fishing {
                record_catch(conn, uid, &layout).await?;
            }
            let extra = if g.duplicate { DUPLICATE_COINS } else { 0 };
            let event = Event::Scratched(Scratch::of(kind, &layout, &prize, extra + boost.bonus));
            let progress = progress::apply(conn, uid, &[event]).await?;
            Ok::<_, ApiError>(Scratched {
                id,
                series,
                layout,
                prize,
                coins: progress.coins,
                tickets: g.tickets,
                duplicate: g.duplicate,
                second_chance: boost.second_chance,
                charm_bonus: boost.bonus,
                charms: charms::worn(conn, uid).await?,
                progress,
                boost,
            })
        })
        .await?;
    // Committed: now everyone may hear of it
    let extra = if out.duplicate { DUPLICATE_COINS } else { 0 };
    state.metrics.finished(&out.series, &out.prize, extra);
    state.metrics.issued(&out.tickets);
    state.metrics.progress(&out.progress);
    out.boost.count(&state.metrics);
    if let Some(series) = Series::parse(&out.series) {
        let s = Scratch::of(series, &out.layout, &out.prize, extra + out.charm_bonus);
        crate::live::broadcast(&state.live, &user.username, Some((series, &s, &out.prize)), &out.progress.achievements);
    }
    Ok(web::Json(out))
}

#[derive(Debug, Serialize)]
pub struct Dug {
    pub cell: usize,
    pub spot: Spot,
}

#[derive(Debug, Serialize)]
pub struct DigState {
    pub dug: Vec<Dug>,
    pub digs_left: usize,
    /// The last dig: the whole beach and the prize, as a scratch answers
    pub done: Option<Scratched>,
}

fn dig_state(layout: &DigLayout, picks: &[usize]) -> DigState {
    DigState {
        dug: picks.iter().map(|&cell| Dug { cell, spot: layout.cells[cell] }).collect(),
        digs_left: treasure_dig::digs_allowed(layout, picks).saturating_sub(picks.len()),
        done: None,
    }
}

fn parse_picks(picks: Option<serde_json::Value>) -> ApiResult<Vec<usize>> {
    Ok(picks.map(serde_json::from_value).transpose().map_err(|e| ApiError::Internal(e.into()))?.unwrap_or_default())
}

/// A Treasure Dig in play: what was dug so far
#[get("/tickets/{id}/dig")]
pub async fn get_dig(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<i64>,
) -> ApiResult<web::Json<DigState>> {
    let id = path.into_inner();
    let mut conn = state.pool.get().await?;
    let row: Option<(serde_json::Value, Option<serde_json::Value>)> = tickets::table
        .filter(tickets::id.eq(id))
        .filter(tickets::user_id.eq(user.id))
        .filter(tickets::scratched_at.is_null())
        .filter(tickets::series.eq(Series::TreasureDig.key()))
        .select((tickets::layout, tickets::picks))
        .first(&mut conn)
        .await
        .optional()?;
    let (layout, picks) = row.ok_or(ApiError::NotFound("ticket"))?;
    let layout: DigLayout = serde_json::from_value(layout).map_err(|e| ApiError::Internal(e.into()))?;
    Ok(web::Json(dig_state(&layout, &parse_picks(picks)?)))
}

#[derive(Debug, serde::Deserialize)]
pub struct DigBody {
    cell: usize,
}

/// Digs one spot. The last dig pays what the digs found.
#[post("/tickets/{id}/dig")]
pub async fn dig(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<i64>,
    body: web::Json<DigBody>,
) -> ApiResult<web::Json<DigState>> {
    let id = path.into_inner();
    let cell = body.into_inner().cell;
    let uid = user.id;
    let mut conn = state.pool.get().await?;
    let out = conn
        .transaction(async move |conn| {
            // The row lock: two digs at once take turns
            let row: Option<(serde_json::Value, Option<serde_json::Value>)> = tickets::table
                .filter(tickets::id.eq(id))
                .filter(tickets::user_id.eq(uid))
                .filter(tickets::scratched_at.is_null())
                .filter(tickets::series.eq(Series::TreasureDig.key()))
                .select((tickets::layout, tickets::picks))
                .for_update()
                .first(conn)
                .await
                .optional()?;
            let (raw, picks) = row.ok_or(ApiError::NotFound("ticket"))?;
            let layout: DigLayout = serde_json::from_value(raw.clone()).map_err(|e| ApiError::Internal(e.into()))?;
            let mut picks = parse_picks(picks)?;
            treasure_dig::can_dig(&layout, &picks, cell).map_err(ApiError::Conflict)?;
            picks.push(cell);
            let mut out = dig_state(&layout, &picks);
            let picks_json = serde_json::to_value(&picks).expect("picks serialize");
            if !treasure_dig::finished(&layout, &picks) {
                diesel::update(tickets::table.find(id)).set(tickets::picks.eq(picks_json)).execute(conn).await?;
                return Ok::<_, ApiError>(out);
            }
            let prize = treasure_dig::prize(&layout, &picks);
            out.done = Some(finish(conn, uid, id, Series::TreasureDig, raw, picks_json, prize).await?);
            Ok(out)
        })
        .await?;
    if let Some(done) = &out.done {
        announce(&state, &user.username, Series::TreasureDig, done);
    }
    Ok(web::Json(out))
}

/// Ends a ticket played move by move: its prize is now known and paid
async fn finish(
    conn: &mut AsyncPgConnection,
    uid: i64,
    id: i64,
    series: Series,
    layout: serde_json::Value,
    state: serde_json::Value,
    prize: Prize,
) -> ApiResult<Scratched> {
    diesel::update(tickets::table.find(id))
        .set((
            tickets::picks.eq(state),
            tickets::prize.eq(serde_json::to_value(&prize).expect("prize serializes")),
            tickets::scratched_at.eq(Utc::now()),
        ))
        .execute(conn)
        .await?;
    let g = grant(conn, uid, &prize, Source::Prize).await?;
    let mut boost = Boost::default();
    charms::payout(conn, uid, series, &prize, &mut boost).await?;
    let event = Event::Scratched(Scratch::of(series, &layout, &prize, boost.bonus));
    let progress = progress::apply(conn, uid, &[event]).await?;
    Ok(Scratched {
        id,
        series: series.key().to_owned(),
        layout,
        prize,
        coins: progress.coins,
        tickets: g.tickets,
        duplicate: false,
        second_chance: false,
        charm_bonus: boost.bonus,
        charms: charms::worn(conn, uid).await?,
        progress,
        boost,
    })
}

/// Tells everyone about a finished ticket worth it, once committed
fn announce(state: &AppState, user: &str, series: Series, done: &Scratched) {
    state.metrics.finished(series.key(), &done.prize, 0);
    state.metrics.issued(&done.tickets);
    state.metrics.progress(&done.progress);
    done.boost.count(&state.metrics);
    let s = Scratch::of(series, &done.layout, &done.prize, done.charm_bonus);
    crate::live::broadcast(&state.live, user, Some((series, &s, &done.prize)), &done.progress.achievements);
}

#[derive(Debug, Serialize)]
pub struct PlayState {
    /// What the player may see now
    pub view: serde_json::Value,
    /// The game is over: the whole layout and the prize, as a scratch answers
    pub done: Option<Scratched>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PlayBody {
    #[serde(rename = "move")]
    mv: serde_json::Value,
}

/// A look (no move) or a move at a ticket played move by move
async fn play_ticket(
    state: &AppState,
    user: &CurrentUser,
    id: i64,
    mv: Option<serde_json::Value>,
) -> ApiResult<(Series, PlayState)> {
    let uid = user.id;
    let mut conn = state.pool.get().await?;
    let out = conn
        .transaction(async move |conn| {
            // The row lock: two moves at once take turns
            let row: Option<(String, serde_json::Value, Option<serde_json::Value>)> = tickets::table
                .filter(tickets::id.eq(id))
                .filter(tickets::user_id.eq(uid))
                .filter(tickets::scratched_at.is_null())
                .select((tickets::series, tickets::layout, tickets::picks))
                .for_update()
                .first(conn)
                .await
                .optional()?;
            let (series, layout, saved) = row.ok_or(ApiError::NotFound("ticket"))?;
            let series = Series::parse(&series).filter(|s| s.is_played()).ok_or(ApiError::Invalid("not_played"))?;
            let played = play::dispatch(series, &layout, saved.as_ref(), mv.as_ref(), Utc::now())
                .expect("a played series")
                .map_err(ApiError::Conflict)?;
            let done = match played.outcome {
                play::Outcome::Going => {
                    diesel::update(tickets::table.find(id)).set(tickets::picks.eq(&played.state)).execute(conn).await?;
                    None
                }
                play::Outcome::Over(prize) => Some(finish(conn, uid, id, series, layout, played.state, prize).await?),
            };
            Ok::<_, ApiError>((series, PlayState { view: played.view, done }))
        })
        .await?;
    if let Some(done) = &out.1.done {
        announce(state, &user.username, out.0, done);
    }
    Ok(out)
}

/// Where a ticket played move by move stands (a timed one may end on its own)
#[get("/tickets/{id}/play")]
pub async fn look(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<i64>,
) -> ApiResult<web::Json<PlayState>> {
    Ok(web::Json(play_ticket(&state, &user, path.into_inner(), None).await?.1))
}

#[post("/tickets/{id}/play")]
pub async fn play_move(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<i64>,
    body: web::Json<PlayBody>,
) -> ApiResult<web::Json<PlayState>> {
    Ok(web::Json(play_ticket(&state, &user, path.into_inner(), Some(body.into_inner().mv)).await?.1))
}

/// Puts the fish a Fishing ticket brought up into the player's aquarium
async fn record_catch(conn: &mut AsyncPgConnection, uid: i64, layout: &serde_json::Value) -> ApiResult<()> {
    let rows: Vec<_> = crate::tickets::fishing::caught(layout)
        .into_iter()
        .map(|s| (aquarium::user_id.eq(uid), aquarium::species.eq(s)))
        .collect();
    if !rows.is_empty() {
        diesel::insert_into(aquarium::table).values(rows).on_conflict_do_nothing().execute(conn).await?;
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Aquarium {
    /// Every species, in order
    pub species: Vec<&'static str>,
    pub caught: Vec<String>,
}

#[get("/aquarium")]
pub async fn get_aquarium(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<Aquarium>> {
    let mut conn = state.pool.get().await?;
    let caught = aquarium::table
        .filter(aquarium::user_id.eq(user.id))
        .order(aquarium::caught_at.asc())
        .select(aquarium::species)
        .load(&mut conn)
        .await?;
    let species = crate::tickets::fishing::SPECIES.iter().map(|(n, _, _)| *n).collect();
    Ok(web::Json(Aquarium { species, caught }))
}
