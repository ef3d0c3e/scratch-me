//! The shop: tickets and cosmetics for coins. And putting cosmetics on.

use actix_web::{get, post, web};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use super::pot;
use super::tickets::{TicketDto, issue};
use crate::AppState;
use crate::auth::CurrentUser;
use crate::cosmetics::{Cosmetic, Origin};
use crate::error::{ApiError, ApiResult};
use crate::progress::{self, Event, Progress, levels};
use crate::schema::{cosmetics, tickets, users};
use crate::tickets::{Series, Source};

#[derive(Debug, Serialize)]
pub struct TicketOffer {
    pub series: Series,
    pub price: i64,
    /// The level it takes to buy it
    pub level: i32,
    /// A seasonal series: its last day on sale, as (month, day)
    pub until: Option<(u32, u32)>,
}

/// Every cosmetic and where it comes from: a price, a Golden Ticket, a trophy
#[derive(Debug, Serialize)]
pub struct CosmeticOffer {
    pub item: Cosmetic,
    pub origin: Origin,
}

#[derive(Debug, Serialize)]
pub struct ShopDto {
    pub tickets: Vec<TicketOffer>,
    pub cosmetics: Vec<CosmeticOffer>,
}

#[get("/shop")]
pub async fn list(_user: CurrentUser) -> web::Json<ShopDto> {
    let today = Utc::now().date_naive();
    let tickets = Series::ALL.into_iter().filter(|s| s.in_season(today)).filter_map(|series| {
        Some(TicketOffer {
            series,
            price: series.price()?,
            level: levels::shop_level(series),
            until: series.season().map(|(_, to)| to),
        })
    });
    let cosmetics = Cosmetic::all().into_iter().map(|item| CosmeticOffer { item, origin: item.origin() });
    web::Json(ShopDto { tickets: tickets.collect(), cosmetics: cosmetics.collect() })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Buy {
    Ticket {
        series: Series,
        /// A gift: the player it is for
        #[serde(default)]
        to: Option<String>,
    },
    Cosmetic {
        item: Cosmetic,
    },
}

#[derive(Debug, Serialize)]
pub struct Bought {
    pub coins: i64,
    pub ticket: Option<TicketDto>,
    pub item: Option<Cosmetic>,
    /// A gift went to this player
    pub gift_to: Option<String>,
    pub progress: Progress,
}

#[post("/shop/buy")]
pub async fn buy(state: web::Data<AppState>, user: CurrentUser, body: web::Json<Buy>) -> ApiResult<web::Json<Bought>> {
    let buy = body.into_inner();
    let price = match &buy {
        Buy::Ticket { series, .. } => series.price(),
        Buy::Cosmetic { item } => item.price(),
    }
    .ok_or(ApiError::Invalid("not_for_sale"))?;
    let mut conn = state.pool.get().await?;
    let name = user.username.clone();
    let out = conn
        .transaction(async move |conn| {
            if let Buy::Ticket { series, .. } = &buy {
                if !series.in_season(Utc::now().date_naive()) {
                    return Err(ApiError::Conflict("out_of_season"));
                }
                let xp: i64 = users::table.find(user.id).select(users::xp).first(conn).await?;
                if levels::level_for(xp) < levels::shop_level(*series) {
                    return Err(ApiError::Conflict("level_too_low"));
                }
            }
            // Pays only if the coins are there: two buys at once cannot both spend them
            let coins: Option<i64> = diesel::update(users::table.find(user.id).filter(users::coins.ge(price)))
                .set(users::coins.eq(users::coins - price))
                .returning(users::coins)
                .get_result(conn)
                .await
                .optional()?;
            coins.ok_or(ApiError::Conflict("not_enough_coins"))?;
            match buy {
                Buy::Ticket { series, to: Some(to) } => {
                    // A gift: drawn for the other player, marked with the sender. Members only, both sides.
                    user.member()?;
                    let (rid, rname, guest): (i64, String, bool) = users::table
                        .filter(lower(users::username).eq(to.to_lowercase()))
                        .select((users::id, users::username, users::is_guest))
                        .first(conn)
                        .await
                        .optional()?
                        .ok_or(ApiError::NotFound("player"))?;
                    if rid == user.id {
                        return Err(ApiError::Invalid("gift_to_self"));
                    }
                    if guest {
                        return Err(ApiError::Conflict("player_is_guest"));
                    }
                    let mut ticket = issue(conn, rid, series, Source::Gift, 1).await?.pop().expect("one ticket");
                    diesel::update(tickets::table.find(ticket.id))
                        .set((tickets::from_user_id.eq(user.id), tickets::from_name.eq(&user.username)))
                        .execute(conn)
                        .await?;
                    ticket.from = Some(user.username.clone());
                    if series == Series::PotTicket {
                        pot::enter(conn, rid).await?;
                    }
                    let progress = progress::apply(conn, user.id, &[Event::Bought(series), Event::Gifted]).await?;
                    Ok(Bought { coins: progress.coins, ticket: None, item: None, gift_to: Some(rname), progress }
                        .with_gift(rid, ticket))
                }
                Buy::Ticket { series, to: None } => {
                    let ticket = issue(conn, user.id, series, Source::Shop, 1).await?.pop();
                    if series == Series::PotTicket {
                        pot::enter(conn, user.id).await?;
                    }
                    let progress = progress::apply(conn, user.id, &[Event::Bought(series)]).await?;
                    Ok((Bought { coins: progress.coins, ticket, item: None, gift_to: None, progress }, None))
                }
                Buy::Cosmetic { item } => {
                    let added = diesel::insert_into(cosmetics::table)
                        .values((
                            cosmetics::user_id.eq(user.id),
                            cosmetics::kind.eq(item.kind()),
                            cosmetics::item.eq(item.item()),
                        ))
                        .on_conflict_do_nothing()
                        .execute(conn)
                        .await?;
                    // Rolls the payment back too
                    if added == 0 {
                        return Err(ApiError::Conflict("already_owned"));
                    }
                    // Owning more may earn the collector achievement
                    let progress = progress::apply(conn, user.id, &[]).await?;
                    Ok((
                        Bought { coins: progress.coins, ticket: None, item: Some(item), gift_to: None, progress },
                        None,
                    ))
                }
            }
        })
        .await?;
    let (out, gift) = out;
    crate::live::broadcast(&state.live, &name, None, &out.progress.achievements);
    // Committed: the receiver hears of it
    if let Some((rid, ticket)) = gift {
        let series = Series::parse(&ticket.series).expect("a series");
        state.live.notify(rid, crate::live::Event::Gift { from: name.clone(), series });
    }
    Ok(web::Json(out))
}

impl Bought {
    fn with_gift(self, rid: i64, ticket: TicketDto) -> (Bought, Option<(i64, TicketDto)>) {
        (self, Some((rid, ticket)))
    }
}

diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

#[derive(Debug, Deserialize)]
pub struct Equip {
    item: Cosmetic,
}

#[derive(Debug, Serialize)]
pub struct Equipped {
    pub theme: String,
    pub tool: String,
    pub foil: String,
}

#[post("/cosmetics/equip")]
pub async fn equip(
    state: web::Data<AppState>,
    user: CurrentUser,
    body: web::Json<Equip>,
) -> ApiResult<web::Json<Equipped>> {
    let item = body.into_inner().item;
    let mut conn = state.pool.get().await?;
    if !item.is_default() {
        let owned: bool = diesel::select(diesel::dsl::exists(
            cosmetics::table
                .filter(cosmetics::user_id.eq(user.id))
                .filter(cosmetics::kind.eq(item.kind()))
                .filter(cosmetics::item.eq(item.item())),
        ))
        .get_result(&mut conn)
        .await?;
        if !owned {
            return Err(ApiError::Conflict("not_owned"));
        }
    }
    let target = users::table.find(user.id);
    let returned = (users::theme, users::tool, users::foil);
    let (theme, tool, foil) = match item {
        Cosmetic::Theme(t) => {
            diesel::update(target).set(users::theme.eq(t.key())).returning(returned).get_result(&mut conn).await?
        }
        Cosmetic::Tool(t) => {
            diesel::update(target).set(users::tool.eq(t.key())).returning(returned).get_result(&mut conn).await?
        }
        Cosmetic::Foil(t) => {
            diesel::update(target).set(users::foil.eq(t.key())).returning(returned).get_result(&mut conn).await?
        }
    };
    Ok(web::Json(Equipped { theme, tool, foil }))
}
