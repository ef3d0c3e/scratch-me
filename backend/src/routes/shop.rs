//! The shop: tickets and cosmetics for coins. And putting cosmetics on.

use actix_web::{get, post, web};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use super::tickets::{TicketDto, issue};
use crate::AppState;
use crate::auth::CurrentUser;
use crate::cosmetics::{Cosmetic, Theme, Tool};
use crate::error::{ApiError, ApiResult};
use crate::schema::{cosmetics, users};
use crate::tickets::{Series, Source};

#[derive(Debug, Serialize)]
pub struct TicketOffer {
    pub series: Series,
    pub price: i64,
}

#[derive(Debug, Serialize)]
pub struct CosmeticOffer {
    pub item: Cosmetic,
    pub price: i64,
}

#[derive(Debug, Serialize)]
pub struct ShopDto {
    pub tickets: Vec<TicketOffer>,
    pub cosmetics: Vec<CosmeticOffer>,
}

#[get("/shop")]
pub async fn list(_user: CurrentUser) -> web::Json<ShopDto> {
    let tickets = Series::ALL.into_iter().filter_map(|series| Some(TicketOffer { series, price: series.price()? }));
    let cosmetics = Theme::ALL
        .into_iter()
        .map(Cosmetic::Theme)
        .chain(Tool::ALL.into_iter().map(Cosmetic::Tool))
        .filter_map(|item| Some(CosmeticOffer { item, price: item.price()? }));
    web::Json(ShopDto { tickets: tickets.collect(), cosmetics: cosmetics.collect() })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Buy {
    Ticket { series: Series },
    Cosmetic { item: Cosmetic },
}

#[derive(Debug, Serialize)]
pub struct Bought {
    pub coins: i64,
    pub ticket: Option<TicketDto>,
    pub item: Option<Cosmetic>,
}

#[post("/shop/buy")]
pub async fn buy(state: web::Data<AppState>, user: CurrentUser, body: web::Json<Buy>) -> ApiResult<web::Json<Bought>> {
    let buy = body.into_inner();
    let price = match &buy {
        Buy::Ticket { series } => series.price(),
        Buy::Cosmetic { item } => item.price(),
    }
    .ok_or(ApiError::Invalid("not_for_sale"))?;
    let mut conn = state.pool.get().await?;
    let out = conn
        .transaction(async move |conn| {
            // Pays only if the coins are there: two buys at once cannot both spend them
            let coins: Option<i64> = diesel::update(users::table.find(user.id).filter(users::coins.ge(price)))
                .set(users::coins.eq(users::coins - price))
                .returning(users::coins)
                .get_result(conn)
                .await
                .optional()?;
            let coins = coins.ok_or(ApiError::Conflict("not_enough_coins"))?;
            match buy {
                Buy::Ticket { series } => {
                    let ticket = issue(conn, user.id, series, Source::Shop, 1).await?.pop();
                    Ok(Bought { coins, ticket, item: None })
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
                    Ok(Bought { coins, ticket: None, item: Some(item) })
                }
            }
        })
        .await?;
    Ok(web::Json(out))
}

#[derive(Debug, Deserialize)]
pub struct Equip {
    item: Cosmetic,
}

#[derive(Debug, Serialize)]
pub struct Equipped {
    pub theme: String,
    pub tool: String,
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
    let returned = (users::theme, users::tool);
    let (theme, tool) = match item {
        Cosmetic::Theme(t) => {
            diesel::update(target).set(users::theme.eq(t.key())).returning(returned).get_result(&mut conn).await?
        }
        Cosmetic::Tool(t) => {
            diesel::update(target).set(users::tool.eq(t.key())).returning(returned).get_result(&mut conn).await?
        }
    };
    Ok(web::Json(Equipped { theme, tool }))
}
