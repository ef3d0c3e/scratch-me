//! The shop: tickets and cosmetics for coins. And putting cosmetics on.

use actix_web::{get, post, web};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use super::charms::{self as worn, WornCharm};
use super::pot;
use super::tickets::{TicketDto, issue, issue_with};
use crate::AppState;
use crate::auth::CurrentUser;
use crate::bundles;
use crate::charms::Charm;
use crate::cosmetics::{Cosmetic, Origin};
use crate::error::{ApiError, ApiResult};
use crate::progress::{self, Event, Progress, levels};
use crate::schema::{cosmetics, tickets, users};
use crate::tickets::{Series, Source};

#[derive(Debug, Serialize)]
pub struct TicketOffer {
    pub series: Series,
    pub price: i64,
    /// 1 to 3
    pub tier: u8,
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

/// A charm for sale: what it does, for how long, and on which tickets
#[derive(Debug, Serialize)]
pub struct CharmOffer {
    #[serde(flatten)]
    pub charm: Charm,
    pub price: i64,
    pub level: i32,
    /// Percent: off, chance of a second draw, or coins added
    pub strength: i64,
    pub charges: i32,
    /// How long it lasts, in minutes
    pub minutes: i64,
    /// The dearest ticket it works on; None for any
    pub reach: Option<i64>,
}

/// A bundle for sale: its tickets with their tiers, its seal in full
#[derive(Debug, Serialize)]
pub struct BundleOffer {
    pub key: &'static str,
    pub price: i64,
    pub level: i32,
    pub seal: crate::tickets::seals::Seal,
    pub tickets: Vec<BundleTicket>,
    /// What its tickets cost one by one
    pub separately: i64,
}

#[derive(Debug, Serialize)]
pub struct BundleTicket {
    pub series: Series,
    pub tier: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct ShopDto {
    pub bundles: Vec<BundleOffer>,
    pub tickets: Vec<TicketOffer>,
    pub cosmetics: Vec<CosmeticOffer>,
    pub charms: Vec<CharmOffer>,
}

#[get("/shop")]
pub async fn list(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<ShopDto>> {
    // Some charms cost by the player's level
    let mut conn = state.pool.get().await?;
    let xp: i64 = users::table.find(user.id).select(users::xp).first(&mut conn).await?;
    let level = levels::level_for(xp);
    let today = Utc::now().date_naive();
    let bundles_today = |b: &bundles::Bundle| b.in_season(today);
    let mut tickets: Vec<TicketOffer> = Series::ALL
        .into_iter()
        .filter(|s| s.in_shop() && s.in_season(today))
        .filter_map(|series| {
            Some(TicketOffer {
                series,
                price: series.price()?,
                tier: series.tier()?,
                level: levels::shop_level(series),
                until: series.season().map(|(_, to)| to),
            })
        })
        .collect();
    // Tier by tier, cheapest first
    tickets.sort_by_key(|t| (t.tier, t.price));
    let cosmetics = Cosmetic::all().into_iter().map(|item| CosmeticOffer { item, origin: item.origin() });
    let charms = Charm::all().into_iter().map(|charm| CharmOffer {
        charm,
        price: charm.price(level),
        level: charm.tier.level(),
        strength: charm.strength(),
        charges: charm.charges(),
        minutes: charm.lasts().num_minutes(),
        reach: charm.tier.reach(),
    });
    let bundles = bundles::all()
        .into_iter()
        .filter(bundles_today)
        .map(|b| BundleOffer {
            key: b.key,
            price: b.price,
            level: b.level(),
            seal: b.seal,
            tickets: b.tickets.iter().map(|s| BundleTicket { series: *s, tier: s.tier() }).collect(),
            separately: b.separately(),
        })
        .collect();
    Ok(web::Json(ShopDto { bundles, tickets, cosmetics: cosmetics.collect(), charms: charms.collect() }))
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
    Charm {
        charm: Charm,
    },
    Bundle {
        bundle: String,
    },
}

#[derive(Debug, Serialize)]
pub struct Bought {
    pub coins: i64,
    pub ticket: Option<TicketDto>,
    /// A bundle's tickets, in the order they landed
    pub tickets: Vec<TicketDto>,
    pub item: Option<Cosmetic>,
    /// A gift went to this player
    pub gift_to: Option<String>,
    /// What was paid
    pub price: i64,
    /// Coins a discount charm took off the price
    pub saved: i64,
    /// Charms worn after this purchase
    pub charms: Vec<WornCharm>,
    pub progress: Progress,
}

#[post("/shop/buy")]
pub async fn buy(state: web::Data<AppState>, user: CurrentUser, body: web::Json<Buy>) -> ApiResult<web::Json<Bought>> {
    let buy = body.into_inner();
    let mut conn = state.pool.get().await?;
    let xp: i64 = users::table.find(user.id).select(users::xp).first(&mut conn).await?;
    let level = levels::level_for(xp);
    let list_price = match &buy {
        Buy::Ticket { series, .. } => series.price(),
        Buy::Cosmetic { item } => item.price(),
        Buy::Charm { charm } => Some(charm.price(level)),
        Buy::Bundle { bundle } => bundles::bundle(bundle).map(|b| b.price),
    }
    .ok_or(ApiError::Invalid("not_for_sale"))?;
    let bought = if let Buy::Charm { charm } = &buy { Some(*charm) } else { None };
    let bundle = if let Buy::Bundle { bundle } = &buy { Some(bundle.clone()) } else { None };
    let name = user.username.clone();
    let out = conn
        .transaction(async move |conn| {
            let mut price = list_price;
            let mut saved = 0;
            match &buy {
                Buy::Ticket { series, .. } => {
                    if !series.in_season(Utc::now().date_naive()) {
                        return Err(ApiError::Conflict("out_of_season"));
                    }
                    let xp: i64 = users::table.find(user.id).select(users::xp).first(conn).await?;
                    if levels::level_for(xp) < levels::shop_level(*series) {
                        return Err(ApiError::Conflict("level_too_low"));
                    }
                    // Spent here, given back with the rest if the purchase fails
                    (price, saved) = worn::discount(conn, user.id, *series, list_price).await?;
                }
                Buy::Charm { charm } => {
                    let xp: i64 = users::table.find(user.id).select(users::xp).first(conn).await?;
                    if levels::level_for(xp) < charm.tier.level() {
                        return Err(ApiError::Conflict("level_too_low"));
                    }
                }
                Buy::Bundle { bundle } => {
                    let b = bundles::bundle(bundle).expect("priced above");
                    if !b.in_season(Utc::now().date_naive()) {
                        return Err(ApiError::Conflict("out_of_season"));
                    }
                    let xp: i64 = users::table.find(user.id).select(users::xp).first(conn).await?;
                    if levels::level_for(xp) < b.level() {
                        return Err(ApiError::Conflict("level_too_low"));
                    }
                }
                Buy::Cosmetic { .. } => {}
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
                    Ok(Bought {
                        coins: progress.coins,
                        ticket: None,
                        tickets: vec![],
                        item: None,
                        gift_to: Some(rname),
                        price,
                        saved,
                        charms: worn::worn(conn, user.id).await?,
                        progress,
                    }
                    .with_gift(rid, ticket))
                }
                Buy::Ticket { series, to: None } => {
                    let ticket = issue(conn, user.id, series, Source::Shop, 1).await?.pop();
                    if series == Series::PotTicket {
                        pot::enter(conn, user.id).await?;
                    }
                    let progress = progress::apply(conn, user.id, &[Event::Bought(series)]).await?;
                    let charms = worn::worn(conn, user.id).await?;
                    Ok((
                        Bought {
                            coins: progress.coins,
                            ticket,
                            tickets: vec![],
                            item: None,
                            gift_to: None,
                            price,
                            saved,
                            charms,
                            progress,
                        },
                        None,
                    ))
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
                    let charms = worn::worn(conn, user.id).await?;
                    Ok((
                        Bought {
                            coins: progress.coins,
                            ticket: None,
                            tickets: vec![],
                            item: Some(item),
                            gift_to: None,
                            price,
                            saved,
                            charms,
                            progress,
                        },
                        None,
                    ))
                }
                Buy::Charm { charm } => {
                    // Refused while one of its kind still goes: rolls the payment back
                    worn::wear(conn, user.id, charm).await?;
                    let progress = progress::apply(conn, user.id, &[]).await?;
                    let charms = worn::worn(conn, user.id).await?;
                    Ok((
                        Bought {
                            coins: progress.coins,
                            ticket: None,
                            tickets: vec![],
                            item: None,
                            gift_to: None,
                            price,
                            saved,
                            charms,
                            progress,
                        },
                        None,
                    ))
                }
                Buy::Bundle { bundle } => {
                    let b = bundles::bundle(&bundle).expect("checked before the transaction");
                    let mut issued = Vec::with_capacity(b.tickets.len());
                    for series in b.tickets {
                        issued.extend(issue_with(conn, user.id, *series, Source::Bundle, 1, Some(b.seal)).await?);
                    }
                    // Counts as one purchase for the quests
                    let events = [Event::Bought(b.tickets[0]), Event::BundleBought];
                    let progress = progress::apply(conn, user.id, &events).await?;
                    let charms = worn::worn(conn, user.id).await?;
                    Ok((
                        Bought {
                            coins: progress.coins,
                            ticket: None,
                            tickets: issued,
                            item: None,
                            gift_to: None,
                            price,
                            saved,
                            charms,
                            progress,
                        },
                        None,
                    ))
                }
            }
        })
        .await?;
    let (out, gift) = out;
    // Committed: counted
    let m = &state.metrics;
    let price = out.price;
    if out.saved > 0 {
        m.charm_used(crate::charms::Kind::Discount, out.saved);
    }
    match (&bought, &out.item, &gift) {
        _ if bundle.is_some() => {
            m.bundle(bundle.as_deref().unwrap_or_default(), price);
            m.issued(&out.tickets);
        }
        (Some(charm), _, _) => m.charm(*charm, price),
        (None, Some(item), _) => m.cosmetic(item.kind(), price),
        (None, None, Some((_, ticket))) => {
            m.spent("gift", price);
            m.gift();
            m.issued(std::slice::from_ref(ticket));
        }
        (None, None, None) => {
            m.spent("ticket", price);
            m.issued(out.ticket.as_slice());
        }
    }
    let ticket = gift.as_ref().map(|(_, t)| t).or(out.ticket.as_ref());
    if ticket.is_some_and(|t| t.series == Series::PotTicket.key()) {
        m.pot_entry();
    }
    m.progress(&out.progress);
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
