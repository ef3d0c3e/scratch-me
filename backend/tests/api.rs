//! End-to-end tests of the HTTP API against a real Postgres.
//! They need TEST_DATABASE_URL (see .env.example) and are skipped without it.

use std::sync::Arc;

use actix_web::cookie::Cookie;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{App, test, web};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use scratchme_api::progress::levels;
use scratchme_api::{AppState, Config, STARTING_COINS, db, pocket, routes, schema};
use serde_json::{Value, json};

async fn state() -> Option<web::Data<AppState>> {
    let _ = dotenvy::dotenv();
    let _ = tracing_subscriber::fmt().with_test_writer().with_env_filter("scratchme_api=error").try_init();
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("TEST_DATABASE_URL not set, skipping");
        return None;
    };
    // Tests run in parallel: migrate once per process
    static MIGRATED: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
    MIGRATED.get_or_init(|| async { db::run_migrations(&url).await.unwrap() }).await;
    let config = Config {
        database_url: url.clone(),
        bind: String::new(),
        secure_cookies: false,
        trusted_proxies: vec![],
        metrics_bind: None,
        metrics_token: None,
    };
    Some(web::Data::new(AppState {
        pool: db::pool(&url).unwrap(),
        live: Default::default(),
        limits: scratchme_api::ratelimit::AuthLimits::relaxed(),
        config: Arc::new(config),
        metrics: Default::default(),
    }))
}

async fn conn() -> AsyncPgConnection {
    AsyncPgConnection::establish(&std::env::var("TEST_DATABASE_URL").unwrap()).await.unwrap()
}

macro_rules! app {
    ($state:expr) => {
        test::init_service(App::new().app_data($state.clone()).configure(routes::configure)).await
    };
}

fn unique(prefix: &str) -> String {
    format!("{prefix}{}", rand::random::<u32>())
}

async fn body(res: ServiceResponse) -> Value {
    serde_json::from_slice(&test::read_body(res).await).unwrap_or(Value::Null)
}

/// A player whose quests and achievements stay out of the way: no quest
/// today (a placeholder row stands for them), every achievement earned. For
/// tests that count coins.
async fn quiet(name: &str) {
    diesel::sql_query(
        "INSERT INTO quests (user_id, day, key)
         SELECT id, (now() AT TIME ZONE tz)::date, 'none' FROM users WHERE username = $1",
    )
    .bind::<diesel::sql_types::Text, _>(name)
    .execute(&mut conn().await)
    .await
    .unwrap();
    for a in scratchme_api::progress::achievements::all() {
        diesel::sql_query("INSERT INTO achievements (user_id, id) SELECT id, $2 FROM users WHERE username = $1")
            .bind::<diesel::sql_types::Text, _>(name)
            .bind::<diesel::sql_types::Text, _>(a.id)
            .execute(&mut conn().await)
            .await
            .unwrap();
    }
}

async fn register<S>(app: &S, name: &str, tz: &str) -> Cookie<'static>
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    let cookie = register_fresh(app, name, tz).await;
    quiet(name).await;
    cookie
}

async fn register_fresh<S>(app: &S, name: &str, tz: &str) -> Cookie<'static>
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(
            json!({ "username": name, "email": format!("{name}@example.com"), "password": "123456789", "tz": tz }),
        )
        .to_request();
    let res = test::call_service(app, req).await;
    assert_eq!(res.status(), 201);
    res.response().cookies().find(|c| c.name() == "session").expect("session cookie").into_owned()
}

async fn get<S>(app: &S, uri: &str, cookie: &Cookie<'static>) -> (u16, Value)
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    let res = test::call_service(app, test::TestRequest::get().uri(uri).cookie(cookie.clone()).to_request()).await;
    (res.status().as_u16(), body(res).await)
}

async fn post<S>(app: &S, uri: &str, cookie: &Cookie<'static>) -> (u16, Value)
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    let req = test::TestRequest::post().uri(uri).cookie(cookie.clone()).set_json(json!({})).to_request();
    let res = test::call_service(app, req).await;
    (res.status().as_u16(), body(res).await)
}

async fn user_id(name: &str) -> i64 {
    use schema::users;
    users::table.filter(users::username.eq(name)).select(users::id).first(&mut conn().await).await.unwrap()
}

#[actix_web::test]
async fn new_player_starts_with_coins_and_a_full_pocket() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("new");
    let me = register(&app, &name, "Europe/Paris").await;
    let (status, body) = get(&app, "/api/me", &me).await;
    assert_eq!(status, 200);
    assert_eq!(body["username"], name);
    assert_eq!(body["coins"], STARTING_COINS);
    assert_eq!(body["pocket"]["count"], pocket::SIZE);
    assert_eq!(body["pocket"]["next_at"], Value::Null);
    assert_eq!(body["envelope"]["available"], true);
    assert_eq!(body["envelope"]["day"], 1);
    assert_eq!(body["tickets"], json!([]));

    let anon = test::call_service(&app, test::TestRequest::get().uri("/api/me").to_request()).await;
    assert_eq!(anon.status(), 401);
}

#[actix_web::test]
async fn pocket_claim_gives_tickets_once() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let me = register(&app, &unique("pocket"), "UTC").await;
    // Two claims at once: one gets the tickets, the other finds the pocket empty
    let (a, b) = futures_join(post(&app, "/api/pocket/claim", &me), post(&app, "/api/pocket/claim", &me)).await;
    let mut statuses = [a.0, b.0];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    let won = if a.0 == 200 { a.1 } else { b.1 };
    assert_eq!(won["tickets"].as_array().unwrap().len(), pocket::SIZE as usize);
    assert_eq!(won["pocket"]["count"], 0);
    assert!(won["pocket"]["next_at"].is_string());
    // Layouts never leave the server before scratching
    assert!(won["tickets"][0].get("layout").is_none());
    assert!(won["tickets"][0].get("prize").is_none());

    let (_, body) = get(&app, "/api/me", &me).await;
    assert_eq!(body["tickets"].as_array().unwrap().len(), pocket::SIZE as usize);
    assert_eq!(body["tickets"][0]["source"], "pocket");
}

/// Runs two requests concurrently on the same test service
async fn futures_join<A: Future, B: Future>(a: A, b: B) -> (A::Output, B::Output) {
    tokio::join!(a, b)
}

#[actix_web::test]
async fn scratching_pays_once() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("scratch");
    let me = register(&app, &name, "UTC").await;
    // A ticket with a known prize, straight in the database
    use schema::tickets;
    let id: i64 = diesel::insert_into(tickets::table)
        .values((
            tickets::user_id.eq(user_id(&name).await),
            tickets::series.eq("lucky_three"),
            tickets::layout.eq(json!({ "cells": vec!["coin"; 9], "win": "coin" })),
            tickets::prize.eq(json!({ "kind": "coins", "amount": 200 })),
            tickets::source.eq("prize"),
        ))
        .returning(tickets::id)
        .get_result(&mut conn().await)
        .await
        .unwrap();
    let uri = format!("/api/tickets/{id}/scratch");
    let (a, b) = futures_join(post(&app, &uri, &me), post(&app, &uri, &me)).await;
    let mut statuses = [a.0, b.0];
    statuses.sort();
    assert_eq!(statuses, [200, 409], "a double tap pays once");
    let ok = if a.0 == 200 { a.1 } else { b.1 };
    assert_eq!(ok["prize"], json!({ "kind": "coins", "amount": 200 }));
    assert_eq!(ok["layout"]["win"], "coin");
    assert_eq!(ok["coins"], STARTING_COINS + 200);
    let (_, body) = get(&app, "/api/me", &me).await;
    assert_eq!(body["coins"], STARTING_COINS + 200);
    assert_eq!(body["tickets"], json!([]));

    // Someone else's ticket does not exist for you
    let other = register(&app, &unique("other"), "UTC").await;
    assert_eq!(post(&app, &uri, &other).await.0, 404);
}

#[actix_web::test]
async fn ticket_prize_issues_a_ticket() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("chain");
    let me = register(&app, &name, "UTC").await;
    use schema::tickets;
    let id: i64 = diesel::insert_into(tickets::table)
        .values((
            tickets::user_id.eq(user_id(&name).await),
            tickets::series.eq("lucky_three"),
            tickets::layout.eq(json!({ "cells": vec!["ticket"; 9], "win": "ticket" })),
            tickets::prize.eq(json!({ "kind": "ticket", "series": "lucky_three" })),
            tickets::source.eq("prize"),
        ))
        .returning(tickets::id)
        .get_result(&mut conn().await)
        .await
        .unwrap();
    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!(status, 200);
    let won = body["tickets"][0]["id"].as_i64().expect("a ticket was won");
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["tickets"][0]["id"], won);
    assert_eq!(me_body["tickets"][0]["source"], "prize");
}

#[actix_web::test]
async fn envelope_once_a_day_and_streak() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("env");
    let me = register(&app, &name, "Asia/Tokyo").await;
    let (status, body) = post(&app, "/api/envelope/claim", &me).await;
    assert_eq!(status, 200);
    assert_eq!(body["day"], 1);
    assert_eq!(body["coins"], STARTING_COINS + 100);
    assert_eq!(post(&app, "/api/envelope/claim", &me).await.0, 409);

    // Pretend the last claim was yesterday (player's day): the streak goes on
    use schema::streaks;
    let id = user_id(&name).await;
    let today = chrono::Utc::now().with_timezone(&chrono_tz::Asia::Tokyo).date_naive();
    diesel::update(streaks::table.find(id))
        .set(streaks::last_claim.eq(today - chrono::Days::new(1)))
        .execute(&mut conn().await)
        .await
        .unwrap();
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["envelope"]["streak"], 1);
    assert_eq!(me_body["envelope"]["day"], 2);
    let (status, body) = post(&app, "/api/envelope/claim", &me).await;
    assert_eq!(status, 200);
    assert_eq!(body["day"], 2);
    assert_eq!(body["tickets"].as_array().unwrap().len(), 2);
}

async fn post_json<S>(app: &S, uri: &str, cookie: &Cookie<'static>, json: Value) -> (u16, Value)
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    let req = test::TestRequest::post().uri(uri).cookie(cookie.clone()).set_json(json).to_request();
    let res = test::call_service(app, req).await;
    (res.status().as_u16(), body(res).await)
}

/// A ticket with a known prize, straight in the database
async fn known_ticket(name: &str, series: &str, layout: Value, prize: Value) -> i64 {
    use schema::tickets;
    diesel::insert_into(tickets::table)
        .values((
            tickets::user_id.eq(user_id(name).await),
            tickets::series.eq(series),
            tickets::layout.eq(layout),
            tickets::prize.eq(prize),
            tickets::source.eq("prize"),
        ))
        .returning(tickets::id)
        .get_result(&mut conn().await)
        .await
        .unwrap()
}

async fn set_coins(name: &str, coins: i64) {
    use schema::users;
    diesel::update(users::table.filter(users::username.eq(name)))
        .set(users::coins.eq(coins))
        .execute(&mut conn().await)
        .await
        .unwrap();
}

#[actix_web::test]
async fn shop_sells_tickets_for_coins() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("shop");
    let me = register(&app, &name, "UTC").await;
    let (status, body) =
        post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "cash_wheel" })).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("level_too_low")), "the wheel is tier 3");
    set_xp(&name, levels::xp_for(6)).await;
    let (status, shop) = get(&app, "/api/shop", &me).await;
    assert_eq!(status, 200);
    let wheel = shop["tickets"].as_array().unwrap().iter().find(|t| t["series"] == "cash_wheel").unwrap();
    assert_eq!(wheel["price"], 400);
    assert!(shop["tickets"].as_array().unwrap().iter().all(|t| t["series"] != "golden"), "Golden is not sold");

    let (status, body) =
        post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "cash_wheel" })).await;
    assert_eq!(status, 200);
    assert_eq!(body["coins"], STARTING_COINS - 400);
    assert_eq!(body["ticket"]["series"], "cash_wheel");
    assert_eq!(body["ticket"]["source"], "shop");

    let (status, body) =
        post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "cash_wheel" })).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("not_enough_coins")));
    let (status, _) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "golden" })).await;
    assert_eq!(status, 422);
}

#[actix_web::test]
async fn buying_at_once_never_overspends() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("race");
    let me = register(&app, &name, "UTC").await;
    set_coins(&name, 160).await;
    set_xp(&name, levels::xp_for(3)).await;
    let buy = || post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "number_match" }));
    let (a, b) = tokio::join!(buy(), buy());
    let mut statuses = [a.0, b.0];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["coins"], 10);
}

#[actix_web::test]
async fn cosmetics_are_bought_once_and_equipped_when_owned() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("style");
    let me = register(&app, &name, "UTC").await;
    set_coins(&name, 5000).await;
    let sunset = json!({ "kind": "theme", "id": "sunset" });

    // Not owned yet: cannot wear it
    let (status, _) = post_json(&app, "/api/cosmetics/equip", &me, json!({ "item": sunset })).await;
    assert_eq!(status, 409);

    let (status, body) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "cosmetic", "item": sunset })).await;
    assert_eq!(status, 200);
    assert_eq!(body["coins"], 3500);
    // A second time: refused, and not charged
    let (status, body) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "cosmetic", "item": sunset })).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("already_owned")));
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["coins"], 3500);
    assert_eq!(me_body["owned"], json!([sunset]));

    let (status, body) = post_json(&app, "/api/cosmetics/equip", &me, json!({ "item": sunset })).await;
    assert_eq!(status, 200);
    assert_eq!(body["theme"], "sunset");
    // Defaults are always yours
    let (status, body) =
        post_json(&app, "/api/cosmetics/equip", &me, json!({ "item": { "kind": "tool", "id": "coin" } })).await;
    assert_eq!((status, body["tool"].as_str()), (200, Some("coin")));
    // Golden-only cosmetics are not sold
    let (status, _) =
        post_json(&app, "/api/shop/buy", &me, json!({ "kind": "cosmetic", "item": { "kind": "tool", "id": "laser" } }))
            .await;
    assert_eq!(status, 422);
}

#[actix_web::test]
async fn golden_cosmetic_then_its_duplicate() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("gold");
    let me = register(&app, &name, "UTC").await;
    let prize = json!({ "kind": "cosmetic", "item": { "kind": "tool", "id": "laser" } });
    let first = known_ticket(&name, "golden", json!({ "prize": prize }), prize.clone()).await;
    let second = known_ticket(&name, "golden", json!({ "prize": prize }), prize.clone()).await;

    let (status, body) = post(&app, &format!("/api/tickets/{first}/scratch"), &me).await;
    assert_eq!(status, 200);
    assert_eq!(body["duplicate"], false);
    assert_eq!(body["coins"], STARTING_COINS);
    let (status, body) = post(&app, &format!("/api/tickets/{second}/scratch"), &me).await;
    assert_eq!(status, 200);
    assert_eq!(body["duplicate"], true);
    assert_eq!(body["coins"], STARTING_COINS + scratchme_api::cosmetics::DUPLICATE_COINS);
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["owned"], json!([{ "kind": "tool", "id": "laser" }]));
}

#[actix_web::test]
async fn chain_prize_issues_every_ticket_found() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("chain");
    let me = register(&app, &name, "UTC").await;
    let series = json!(["number_match", "golden"]);
    let id = known_ticket(
        &name,
        "ticket_chain",
        json!({ "panels": ["number_match", null, null, "golden", null, null] }),
        json!({ "kind": "tickets", "series": series }),
    )
    .await;
    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!(status, 200);
    let won: Vec<&str> = body["tickets"].as_array().unwrap().iter().map(|t| t["series"].as_str().unwrap()).collect();
    assert_eq!(won, ["number_match", "golden"]);
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["tickets"].as_array().unwrap().len(), 2);
}

async fn set_xp(name: &str, xp: i64) {
    use schema::users;
    diesel::update(users::table.filter(users::username.eq(name)))
        .set(users::xp.eq(xp))
        .execute(&mut conn().await)
        .await
        .unwrap();
}

/// Sets today's quests to these, with this much progress
async fn set_quests(name: &str, quests: &[(&str, i32)]) {
    let c = &mut conn().await;
    diesel::sql_query("DELETE FROM quests WHERE user_id = (SELECT id FROM users WHERE username = $1)")
        .bind::<diesel::sql_types::Text, _>(name)
        .execute(c)
        .await
        .unwrap();
    for (key, progress) in quests {
        diesel::sql_query(
            "INSERT INTO quests (user_id, day, key, progress)
             SELECT id, (now() AT TIME ZONE tz)::date, $2, $3 FROM users WHERE username = $1",
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(*key)
        .bind::<diesel::sql_types::Integer, _>(*progress)
        .execute(c)
        .await
        .unwrap();
    }
}

#[actix_web::test]
async fn scratching_gives_xp_achievements_and_levels() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("xp");
    let me = register_fresh(&app, &name, "UTC").await;
    set_quests(&name, &[]).await;
    set_xp(&name, levels::xp_for(2) - 5).await;
    let prize = json!({ "kind": "coins", "amount": 50 });
    let id = known_ticket(&name, "lucky_three", json!({ "cells": vec!["clover"; 9], "win": "clover" }), prize).await;
    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!(status, 200);
    let p = &body["progress"];
    // 10 for scratching, 5 for winning, 50/20 = 2 for the coins
    assert_eq!(p["xp_gained"], 17);
    assert_eq!(p["level"]["level"], 2, "95 + 17 passes 100");
    assert_eq!(p["level_ups"], json!([{ "what": "2", "reward": { "kind": "ticket", "series": "number_match" } }]));
    let got: Vec<&str> = p["achievements"].as_array().unwrap().iter().map(|a| a["what"].as_str().unwrap()).collect();
    assert_eq!(got, ["first_scratch", "first_win"]);
    // 50 won, 50 + 50 for the two achievements
    assert_eq!(body["coins"], STARTING_COINS + 150);
    assert_eq!(p["tickets"][0]["series"], "number_match");
    assert_eq!(p["tickets"][0]["source"], "reward");

    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["level"], json!({ "level": 2, "xp": 112, "from": 100, "to": 300 }));
    let (_, list) = get(&app, "/api/achievements", &me).await;
    let first = list.as_array().unwrap().iter().find(|a| a["id"] == "scratch_100").unwrap();
    assert_eq!(first["progress"], json!([1, 100]));
    assert!(first["earned_at"].is_null());
}

#[actix_web::test]
async fn quests_pay_once_and_the_bonus_once() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("quest");
    let me = register(&app, &name, "UTC").await;
    set_quests(&name, &[("scratch_5", 4), ("pocket", 1), ("win_3", 2)]).await;
    // pocket done already: mark it
    diesel::sql_query("UPDATE quests SET done_at = now() WHERE key = 'pocket' AND user_id = (SELECT id FROM users WHERE username = $1)")
        .bind::<diesel::sql_types::Text, _>(&name)
        .execute(&mut conn().await)
        .await
        .unwrap();
    let (_, q) = get(&app, "/api/quests", &me).await;
    assert_eq!(q["items"].as_array().unwrap().len(), 3);
    assert_eq!(q["bonus_done"], false);

    let prize = json!({ "kind": "coins", "amount": 50 });
    let id =
        known_ticket(&name, "lucky_three", json!({ "cells": vec!["clover"; 9], "win": "clover" }), prize.clone()).await;
    let (_, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    let p = &body["progress"];
    let done: Vec<&str> = p["quests"].as_array().unwrap().iter().map(|q| q["what"].as_str().unwrap()).collect();
    assert_eq!(done, ["scratch_5", "win_3"]);
    assert_eq!(p["bonus"], json!({ "kind": "ticket", "series": "ticket_chain" }));
    assert_eq!(p["next_round"], 2, "three harder quests open");
    // 50 won, 100 for scratch_5; win_3 gives a Lucky Three, the bonus a chain
    assert_eq!(body["coins"], STARTING_COINS + 150);
    let series: Vec<&str> = p["tickets"].as_array().unwrap().iter().map(|t| t["series"].as_str().unwrap()).collect();
    // The quests' and bonus' xp (25 + 25 + 50) and the scratch's reach level 2: a Number Match
    assert_eq!(series, ["lucky_three", "ticket_chain", "number_match"]);

    // Done is done: another scratch pays neither again
    let id = known_ticket(&name, "lucky_three", json!({ "cells": vec!["clover"; 9], "win": "clover" }), prize).await;
    let (_, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!(body["progress"]["quests"], json!([]));
    assert_eq!(body["progress"]["bonus"], Value::Null);
    let (_, q) = get(&app, "/api/quests", &me).await;
    assert_eq!((q["round"].as_i64(), q["bonus_done"].as_bool()), (Some(2), Some(false)), "on to round 2");
}

#[actix_web::test]
async fn quest_rounds_ask_more_and_stop_after_the_fourth() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("rounds");
    let me = register(&app, &name, "UTC").await;
    set_quests(&name, &[("scratch_5", 4), ("pocket", 1), ("win_3", 2)]).await;
    diesel::sql_query("UPDATE quests SET done_at = now() WHERE key = 'pocket' AND user_id = (SELECT id FROM users WHERE username = $1)")
        .bind::<diesel::sql_types::Text, _>(&name)
        .execute(&mut conn().await)
        .await
        .unwrap();
    let win = || {
        known_ticket(
            &name,
            "lucky_three",
            json!({ "cells": vec!["clover"; 9], "win": "clover" }),
            json!({ "kind": "coins", "amount": 10 }),
        )
    };
    let (_, body) = post(&app, &format!("/api/tickets/{}/scratch", win().await), &me).await;
    assert_eq!(body["progress"]["next_round"], 2);
    let (_, q) = get(&app, "/api/quests", &me).await;
    assert_eq!((q["round"].as_i64(), q["rounds"].as_i64()), (Some(2), Some(4)));
    let items = q["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    for item in items {
        assert_ne!(item["key"], "envelope", "opens once a day");
        assert_eq!((item["progress"].as_i64(), item["done"].as_bool()), (Some(0), Some(false)));
        let base = match item["key"].as_str().unwrap() {
            "scratch_5" => 5,
            "win_3" => 3,
            "coins_300" => 300,
            _ => 1,
        };
        assert_eq!(item["goal"].as_i64().unwrap(), (base * 3 + 1) / 2, "{item}");
    }

    // The fourth round: its bonus is the day's last, and no fifth opens
    let c = &mut conn().await;
    let uid = user_id(&name).await;
    diesel::sql_query("DELETE FROM quests WHERE user_id = $1")
        .bind::<diesel::sql_types::BigInt, _>(uid)
        .execute(c)
        .await
        .unwrap();
    diesel::sql_query(
        "INSERT INTO quests (user_id, day, round, key, progress, done_at)
         SELECT id, (now() AT TIME ZONE tz)::date, 4, k, p, d FROM users,
           (VALUES ('scratch_5', 12, NULL::timestamptz), ('pocket', 3, now()), ('buy', 3, now())) v(k, p, d)
         WHERE id = $1",
    )
    .bind::<diesel::sql_types::BigInt, _>(uid)
    .execute(c)
    .await
    .unwrap();
    diesel::sql_query("UPDATE quest_days SET bonuses = 3 WHERE user_id = $1")
        .bind::<diesel::sql_types::BigInt, _>(uid)
        .execute(c)
        .await
        .unwrap();
    let (_, body) = post(&app, &format!("/api/tickets/{}/scratch", win().await), &me).await;
    let p = &body["progress"];
    assert_eq!(p["quests"][0]["what"], "scratch_5");
    assert_eq!(p["quests"][0]["goal"], 13);
    assert_eq!(p["quests"][0]["reward"], json!({ "kind": "coins", "amount": 250 }));
    assert_eq!(p["bonus"]["kind"], "ticket");
    assert_eq!(p["next_round"], Value::Null);
    let (_, q) = get(&app, "/api/quests", &me).await;
    assert_eq!((q["round"].as_i64(), q["bonus_done"].as_bool()), (Some(4), Some(true)));
    let (_, body) = post(&app, &format!("/api/tickets/{}/scratch", win().await), &me).await;
    assert_eq!(body["progress"]["bonus"], Value::Null, "four rounds a day");
}

#[actix_web::test]
async fn one_reroll_a_day() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("reroll");
    let me = register(&app, &name, "UTC").await;
    set_quests(&name, &[("scratch_5", 0), ("pocket", 0), ("win_3", 0)]).await;
    let (status, q) = post_json(&app, "/api/quests/reroll", &me, json!({ "key": "pocket" })).await;
    assert_eq!(status, 200);
    let keys: Vec<&str> = q["items"].as_array().unwrap().iter().map(|i| i["key"].as_str().unwrap()).collect();
    assert_eq!(keys.len(), 3);
    assert!(!keys.contains(&"pocket"));
    assert_eq!(q["rerolled"], true);
    let (status, body) = post_json(&app, "/api/quests/reroll", &me, json!({ "key": "win_3" })).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("already_rerolled")));
}

async fn delete<S>(app: &S, uri: &str, cookie: &Cookie<'static>) -> u16
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    test::call_service(app, test::TestRequest::delete().uri(uri).cookie(cookie.clone()).to_request())
        .await
        .status()
        .as_u16()
}

#[actix_web::test]
async fn shouts_are_shared_paced_and_mutable() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let a_name = unique("loud");
    let a = register(&app, &a_name, "UTC").await;
    let b = register(&app, &unique("ears"), "UTC").await;
    let mut rx = state.live.subscribe();

    let (status, body) = post_json(&app, "/api/shoutbox", &a, json!({ "body": "  hello all  " })).await;
    assert_eq!(status, 201);
    assert_eq!(body["shout"]["body"], "hello all");
    assert_eq!(body["shout"]["level"], 1);
    let live: Value = serde_json::from_str(&rx.try_recv().unwrap().json).unwrap();
    assert_eq!((live["type"].as_str(), live["body"].as_str()), (Some("chat"), Some("hello all")));

    let (status, _) = post_json(&app, "/api/shoutbox", &a, json!({ "body": "again" })).await;
    assert_eq!(status, 429, "one shout every 2 seconds");
    let (status, _) = post_json(&app, "/api/shoutbox", &a, json!({ "body": "x".repeat(301) })).await;
    assert_eq!(status, 422);
    let (status, _) = post_json(&app, "/api/shoutbox", &a, json!({ "body": "   " })).await;
    assert_eq!(status, 422);

    let (_, seen) = get(&app, "/api/shoutbox", &b).await;
    assert!(seen.as_array().unwrap().iter().any(|m| m["user"] == a_name.as_str()));
    assert_eq!(post(&app, &format!("/api/mutes/{a_name}"), &b).await.0, 204);
    let (_, seen) = get(&app, "/api/shoutbox", &b).await;
    assert!(!seen.as_array().unwrap().iter().any(|m| m["user"] == a_name.as_str()), "muted");
    let (_, list) = get(&app, "/api/mutes", &b).await;
    assert_eq!(list, json!([a_name]));
    assert_eq!(delete(&app, &format!("/api/mutes/{a_name}"), &b).await, 204);
    let (_, seen) = get(&app, "/api/shoutbox", &b).await;
    assert!(seen.as_array().unwrap().iter().any(|m| m["user"] == a_name.as_str()));
}

#[actix_web::test]
async fn shouting_moves_its_quest() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("hi");
    let me = register(&app, &name, "UTC").await;
    set_quests(&name, &[("shout", 0), ("pocket", 0), ("win_3", 0)]).await;
    let (_, body) = post_json(&app, "/api/shoutbox", &me, json!({ "body": "hi" })).await;
    assert_eq!(body["progress"]["quests"][0]["what"], "shout");
    assert_eq!(body["progress"]["coins"], STARTING_COINS + 50);
}

#[actix_web::test]
async fn only_admins_announce() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("boss");
    let admin = register(&app, &name, "UTC").await;
    let player = register(&app, &unique("pleb"), "UTC").await;
    let (status, _) = post_json(&app, "/api/announcements", &player, json!({ "body": "free coins!" })).await;
    assert_eq!(status, 403);

    assert!(scratchme_api::make_admin(&std::env::var("TEST_DATABASE_URL").unwrap(), &name).await.unwrap());
    let mut rx = state.live.subscribe();
    let (status, a) = post_json(&app, "/api/announcements", &admin, json!({ "body": "Happy hour tonight" })).await;
    assert_eq!(status, 201);
    let live: Value = serde_json::from_str(&rx.try_recv().unwrap().json).unwrap();
    assert_eq!(live["type"], "announcement");
    let (_, list) = get(&app, "/api/announcements", &player).await;
    assert_eq!(list[0]["body"], "Happy hour tonight");

    let id = a["id"].as_i64().unwrap();
    assert_eq!(delete(&app, &format!("/api/announcements/{id}"), &player).await, 403);
    assert_eq!(delete(&app, &format!("/api/announcements/{id}"), &admin).await, 204);
    let (_, list) = get(&app, "/api/announcements", &player).await;
    assert!(list.as_array().unwrap().iter().all(|x| x["id"] != id));
    assert_eq!(delete(&app, &format!("/api/announcements/{id}"), &admin).await, 404);
}

#[actix_web::test]
async fn big_wins_reach_everyone_after_the_scratch() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("lucky");
    let me = register(&app, &name, "UTC").await;
    let mut rx = state.live.subscribe();
    let small = known_ticket(
        &name,
        "lucky_three",
        json!({ "cells": vec!["clover"; 9], "win": "clover" }),
        json!({ "kind": "coins", "amount": 50 }),
    )
    .await;
    let big = known_ticket(
        &name,
        "lucky_three",
        json!({ "cells": vec!["gem"; 9], "win": "gem" }),
        json!({ "kind": "coins", "amount": 1000 }),
    )
    .await;
    post(&app, &format!("/api/tickets/{small}/scratch"), &me).await;
    assert!(rx.try_recv().is_err(), "50 coins is not news");
    post(&app, &format!("/api/tickets/{big}/scratch"), &me).await;
    let live: Value = serde_json::from_str(&rx.try_recv().unwrap().json).unwrap();
    assert_eq!(live["type"], "win");
    assert_eq!(live["user"], name.as_str());
    assert_eq!(live["prize"], json!({ "kind": "coins", "amount": 1000 }));
    assert!(state.live.feed().iter().any(|f| matches!(f, scratchme_api::live::FeedItem::Win(w) if w.user == name)));
}

#[actix_web::test]
async fn profiles_and_titles() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("pro");
    let me = register_fresh(&app, &name, "UTC").await;
    set_quests(&name, &[]).await;
    let other = register(&app, &unique("look"), "UTC").await;
    let id = known_ticket(
        &name,
        "lucky_three",
        json!({ "cells": vec!["coin"; 9], "win": "coin" }),
        json!({ "kind": "coins", "amount": 200 }),
    )
    .await;
    post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;

    let (status, p) = get(&app, &format!("/api/players/{}", name.to_uppercase()), &other).await;
    assert_eq!(status, 200, "names are found whatever their case");
    assert_eq!(p["username"], name.as_str());
    assert_eq!((p["scratched"].as_i64(), p["wins"].as_i64(), p["best_win"].as_i64()), (Some(1), Some(1), Some(200)));
    assert_eq!(p["achievements"].as_array().unwrap().len(), 3);
    assert_eq!(get(&app, "/api/players/nobody-at-all", &other).await.0, 404);

    let (status, _) = post_json(&app, "/api/me/title", &me, json!({ "title": "jackpot" })).await;
    assert_eq!(status, 409, "not earned");
    let (status, _) = post_json(&app, "/api/me/title", &me, json!({ "title": "big_win" })).await;
    assert_eq!(status, 200);
    let (_, body) = post_json(&app, "/api/shoutbox", &me, json!({ "body": "look at my title" })).await;
    assert_eq!(body["shout"]["title"], "big_win");
    let (_, p) = get(&app, &format!("/api/players/{name}"), &other).await;
    assert_eq!(p["title"], "big_win");
}

fn beach() -> Value {
    let mut cells = vec![json!({ "kind": "sand" }); 16];
    cells[0] = json!({ "kind": "chest", "coins": 400 });
    cells[1] = json!({ "kind": "map" });
    cells[2] = json!({ "kind": "chest", "coins": 100 });
    json!({ "cells": cells })
}

#[actix_web::test]
async fn treasure_dig_pays_what_the_digs_found() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("dig");
    let me = register(&app, &name, "UTC").await;
    let id = known_ticket(&name, "treasure_dig", beach(), json!({ "kind": "nothing" })).await;
    let uri = format!("/api/tickets/{id}/dig");
    let dig = |cell: usize| post_json(&app, &uri, &me, json!({ "cell": cell }));

    // It is played by digging, not by scratching
    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!((status, body["error"].as_str()), (422, Some("dig_ticket")));

    let (status, s) = dig(1).await;
    assert_eq!(status, 200);
    assert_eq!(s["dug"], json!([{ "cell": 1, "spot": { "kind": "map" } }]));
    assert_eq!(s["digs_left"], 4, "the map gives one more");
    assert_eq!(s["done"], Value::Null);
    // Nothing of the rest of the beach leaks
    assert!(s.get("layout").is_none());
    let (status, body) = dig(1).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("dug_already")));
    let (status, _) = dig(16).await;
    assert_eq!(status, 409);

    // Leave and come back: the digs are kept
    let (_, again) = get(&app, &format!("/api/tickets/{id}/dig"), &me).await;
    assert_eq!(again["digs_left"], 4);
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert!(me_body["tickets"].as_array().unwrap().iter().any(|t| t["id"] == id), "still in the stack");

    dig(0).await;
    dig(5).await;
    dig(6).await;
    let (status, last) = dig(7).await;
    assert_eq!(status, 200);
    assert_eq!(last["digs_left"], 0);
    let done = &last["done"];
    assert_eq!(done["prize"], json!({ "kind": "coins", "amount": 400 }));
    assert_eq!(done["coins"], STARTING_COINS + 400);
    assert_eq!(done["layout"], beach(), "the whole beach, to see what was missed");
    let (status, _) = dig(8).await;
    assert_eq!(status, 404, "over");
}

#[actix_web::test]
async fn two_digs_at_once_take_turns() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("race");
    let me = register(&app, &name, "UTC").await;
    let id = known_ticket(&name, "treasure_dig", beach(), json!({ "kind": "nothing" })).await;
    let uri = format!("/api/tickets/{id}/dig");
    let (a, b) = tokio::join!(
        post_json(&app, &uri, &me, json!({ "cell": 0 })),
        post_json(&app, &uri, &me, json!({ "cell": 0 }))
    );
    let mut statuses = [a.0, b.0];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
}

#[actix_web::test]
async fn shop_tiers_open_with_levels_and_list_cheapest_first() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("lvl");
    let me = register(&app, &name, "UTC").await;
    set_coins(&name, 5000).await;
    let buy =
        |series: &'static str| post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": series }));
    assert_eq!(buy("high_card").await.0, 200, "tier 1 is open from the start");
    assert_eq!(buy("bingo").await.0, 409, "tier 2 opens at level 3");
    set_xp(&name, levels::xp_for(3)).await;
    let (status, body) = buy("bingo").await;
    assert_eq!(
        (status, body["ticket"]["series"].as_str(), body["ticket"]["tier"].as_i64()),
        (200, Some("bingo"), Some(2))
    );
    assert_eq!(buy("treasure_dig").await.0, 409, "tier 3 opens at level 6");
    set_xp(&name, levels::xp_for(6)).await;
    let (status, body) = buy("treasure_dig").await;
    assert_eq!((status, body["ticket"]["series"].as_str()), (200, Some("treasure_dig")));

    let (_, shop) = get(&app, "/api/shop", &me).await;
    let offers: Vec<(String, i64, i64, i64)> = shop["tickets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            let n = |k: &str| t[k].as_i64().unwrap();
            (t["series"].as_str().unwrap().to_owned(), n("tier"), n("price"), n("level"))
        })
        .collect();
    assert!(offers.iter().all(|(s, ..)| s != "pot_ticket"), "the pot is entered from the lobby");
    assert!(offers.windows(2).all(|w| (w[0].1, w[0].2) <= (w[1].1, w[1].2)), "tier by tier, cheapest first");
    assert!(offers.iter().all(|(_, tier, _, level)| *level == [1, 3, 6][*tier as usize - 1]));
    // Still sold, from the pot card
    assert_eq!(buy("pot_ticket").await.0, 200);
}

#[actix_web::test]
async fn foils_are_worn_and_trophy_items_are_earned_not_bought() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("foil");
    let me = register_fresh(&app, &name, "UTC").await;
    set_quests(&name, &[]).await;
    set_coins(&name, 5000).await;
    let (_, shop) = get(&app, "/api/shop", &me).await;
    let origin = |kind: &str, id: &str| {
        shop["cosmetics"].as_array().unwrap().iter().find(|c| c["item"] == json!({ "kind": kind, "id": id })).unwrap()["origin"].clone()
    };
    assert_eq!(origin("foil", "holo"), json!({ "kind": "shop", "price": 2500 }));
    assert_eq!(origin("tool", "shovel"), json!({ "kind": "trophy", "achievement": "deep_digger" }));
    assert_eq!(origin("foil", "galaxy"), json!({ "kind": "golden" }));

    let holo = json!({ "kind": "foil", "id": "holo" });
    let (status, _) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "cosmetic", "item": holo })).await;
    assert_eq!(status, 200);
    let (status, body) = post_json(&app, "/api/cosmetics/equip", &me, json!({ "item": holo })).await;
    assert_eq!((status, body["foil"].as_str()), (200, Some("holo")));
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["foil"], "holo");
    let shovel = json!({ "kind": "tool", "id": "shovel" });
    let (status, _) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "cosmetic", "item": shovel })).await;
    assert_eq!(status, 422, "trophy items are not sold");

    // A big dig earns the trophy, and the trophy gives the shovel
    let mut cells = vec![json!({ "kind": "sand" }); 16];
    cells[0] = json!({ "kind": "chest", "coins": 3000 });
    let id = known_ticket(&name, "treasure_dig", json!({ "cells": cells }), json!({ "kind": "nothing" })).await;
    let uri = format!("/api/tickets/{id}/dig");
    let mut last = Value::Null;
    for cell in [0, 1, 2, 3] {
        last = post_json(&app, &uri, &me, json!({ "cell": cell })).await.1;
    }
    let got: Vec<&str> = last["done"]["progress"]["achievements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["what"].as_str().unwrap())
        .collect();
    assert!(got.contains(&"deep_digger"), "{got:?}");
    let (status, _) = post_json(&app, "/api/cosmetics/equip", &me, json!({ "item": shovel })).await;
    assert_eq!(status, 200, "the shovel is theirs");
}

#[actix_web::test]
async fn played_tickets_go_move_by_move() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("play");
    let me = register(&app, &name, "UTC").await;
    let layout = json!({ "wires": ["dud", "boom", "dud", "defuse", "dud", "dud"], "coins": 400 });
    let id = known_ticket(&name, "defuse_bomb", layout.clone(), json!({ "kind": "nothing" })).await;
    let uri = format!("/api/tickets/{id}/play");

    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!((status, body["error"].as_str()), (422, Some("played_ticket")));

    let (status, look) = get(&app, &uri, &me).await;
    assert_eq!(status, 200);
    assert_eq!(look["view"]["coins"], 400);
    assert!(look["view"]["wires"].is_null(), "hidden while it is going");
    assert!(look["view"]["ms_left"].as_i64().unwrap() > 19_000);

    let (status, body) = post_json(&app, &uri, &me, json!({ "move": { "cut": 0 } })).await;
    assert_eq!(status, 200);
    assert_eq!(body["view"]["cut"], json!([{ "wire": 0, "is": "dud" }]));
    assert_eq!(body["done"], Value::Null);
    let (status, body) = post_json(&app, &uri, &me, json!({ "move": { "cut": 0 } })).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("cut_already")));
    let (status, body) = post_json(&app, &uri, &me, json!({ "move": { "nonsense": true } })).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("bad_move")));

    let (status, body) = post_json(&app, &uri, &me, json!({ "move": { "cut": 3 } })).await;
    assert_eq!(status, 200);
    assert_eq!(body["done"]["prize"], json!({ "kind": "coins", "amount": 400 }));
    assert_eq!(body["done"]["coins"], STARTING_COINS + 400);
    assert_eq!(body["view"]["wires"], layout["wires"], "everything shows at the end");
    assert_eq!(get(&app, &uri, &me).await.0, 404, "over");
}

#[actix_web::test]
async fn coin_flip_roulette_and_battleship_are_played_by_choice() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("choice");
    let me = register(&app, &name, "UTC").await;
    let nothing = json!({ "kind": "nothing" });

    // Coin Flip: the call is the scratched cell; the bet under it is what it pays
    let id = known_ticket(&name, "coin_flip", json!({ "bets": [50, 250], "flip": "tails" }), nothing.clone()).await;
    let uri = format!("/api/tickets/{id}/play");
    let (_, look) = get(&app, &uri, &me).await;
    assert_eq!(look["view"], json!({}), "the bets stay hidden");
    let (_, body) = post_json(&app, &uri, &me, json!({ "move": { "side": "tails" } })).await;
    assert_eq!(body["done"]["prize"], json!({ "kind": "coins", "amount": 250 }));
    assert_eq!(body["view"]["bets"], json!([50, 250]));

    // Roulette: 0 is green
    let id = known_ticket(&name, "roulette", json!({ "bets": [140, 170, 2500], "pocket": 0 }), nothing.clone()).await;
    let uri = format!("/api/tickets/{id}/play");
    let (_, body) = post_json(&app, &uri, &me, json!({ "move": { "colour": "red" } })).await;
    assert_eq!(body["done"]["prize"], nothing);
    assert_eq!((body["view"]["colour"].as_str(), body["view"]["pocket"].as_i64()), (Some("green"), Some(0)));

    // Battleship: shot by shot, ships hidden until sunk
    let sea = json!({ "ships": [
        { "cells": [0, 1, 2], "coins": 15 }, { "cells": [10, 15], "coins": 5 }, { "cells": [23, 24], "coins": 5 }
    ] });
    let id = known_ticket(&name, "battleship", sea, nothing).await;
    let uri = format!("/api/tickets/{id}/play");
    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!((status, body["error"].as_str()), (422, Some("played_ticket")));
    let (_, body) = post_json(&app, &uri, &me, json!({ "move": { "cell": 0 } })).await;
    assert_eq!(body["view"]["shots"], json!([{ "cell": 0, "hit": true }]));
    assert_eq!(body["view"]["ships"], json!([]));
    for cell in [1, 2] {
        post_json(&app, &uri, &me, json!({ "move": { "cell": cell } })).await;
    }
    let (_, body) = post_json(&app, &uri, &me, json!({ "move": { "cell": 5 } })).await;
    assert_eq!(body["view"]["sunk"], 1);
    assert_eq!(body["view"]["coins"], 3 * 15 + 15);
    let mut done = Value::Null;
    for cell in [6, 7, 8, 9, 11, 12] {
        let (_, body) = post_json(&app, &uri, &me, json!({ "move": { "cell": cell } })).await;
        done = body["done"].clone();
    }
    assert_eq!(done["prize"], json!({ "kind": "coins", "amount": 60 }), "ten shots: three hits and a sunk ship");
    assert_eq!(done["layout"]["ships"].as_array().unwrap().len(), 3);
}

#[actix_web::test]
async fn rocket_pays_what_the_server_measured() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("rocket");
    let me = register(&app, &name, "UTC").await;
    let id = known_ticket(&name, "rocket", json!({ "crash": 40.0 }), json!({ "kind": "nothing" })).await;
    let uri = format!("/api/tickets/{id}/play");
    let (_, body) = post_json(&app, &uri, &me, json!({ "move": { "kind": "cash" } })).await;
    assert_eq!(body["error"], "launch_first");
    post_json(&app, &uri, &me, json!({ "move": { "kind": "launch" } })).await;
    let (_, body) = post_json(&app, &uri, &me, json!({ "move": { "kind": "cash" } })).await;
    // Cashed at once: about the stake back
    let won = body["done"]["prize"]["amount"].as_i64().unwrap();
    assert!((100..=110).contains(&won), "{won}");
}

#[actix_web::test]
async fn a_bonus_bundle_pays_coins_and_a_ticket() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("bonus");
    let me = register(&app, &name, "UTC").await;
    let prize = json!({ "kind": "bundle", "items": [
        { "kind": "coins", "amount": 50 },
        { "kind": "ticket", "series": "lucky_three" }
    ] });
    let layout =
        json!({ "cells": vec!["clover"; 9], "win": "clover", "bonus": { "kind": "ticket", "series": "lucky_three" } });
    let id = known_ticket(&name, "lucky_three", layout, prize.clone()).await;
    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!(status, 200);
    assert_eq!(body["prize"], prize);
    assert_eq!(body["coins"], STARTING_COINS + 50);
    assert_eq!(body["tickets"][0]["series"], "lucky_three");
}

#[actix_web::test]
async fn fish_go_into_the_aquarium() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("fish");
    let me = register(&app, &name, "UTC").await;
    let casts = json!({ "casts": [
        { "fish": "tuna", "coins": 20 }, { "fish": null, "coins": 0 },
        { "fish": "tuna", "coins": 20 }, { "fish": "golden_koi", "coins": 600 }
    ] });
    let id = known_ticket(&name, "fishing", casts, json!({ "kind": "coins", "amount": 640 })).await;
    let (status, body) = post(&app, &format!("/api/tickets/{id}/scratch"), &me).await;
    assert_eq!((status, body["coins"].as_i64()), (200, Some(STARTING_COINS + 640)));
    let (_, aq) = get(&app, "/api/aquarium", &me).await;
    assert_eq!(aq["species"].as_array().unwrap().len(), 12);
    let mut caught: Vec<&str> = aq["caught"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    caught.sort();
    assert_eq!(caught, ["golden_koi", "tuna"]);
}

#[actix_web::test]
async fn seasonal_tickets_are_sold_in_season_only() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("season");
    let me = register(&app, &name, "UTC").await;
    set_xp(&name, levels::xp_for(3)).await;
    let (_, shop) = get(&app, "/api/shop", &me).await;
    let sold: Vec<&str> = shop["tickets"].as_array().unwrap().iter().map(|t| t["series"].as_str().unwrap()).collect();
    let today = chrono::Utc::now().date_naive();
    for (series, key) in [
        (scratchme_api::tickets::Series::PumpkinPatch, "pumpkin_patch"),
        (scratchme_api::tickets::Series::Snowball, "snowball"),
    ] {
        assert_eq!(sold.contains(&key), series.in_season(today), "{key}");
        let (status, body) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": key })).await;
        if series.in_season(today) {
            assert_eq!(status, 200, "{key}");
        } else {
            assert_eq!((status, body["error"].as_str()), (409, Some("out_of_season")), "{key}");
        }
    }
}

#[actix_web::test]
async fn gifts_go_to_the_other_player_marked_with_the_sender() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let (a, b) = (unique("giver"), unique("taker"));
    let me = register(&app, &a, "UTC").await;
    let them = register(&app, &b, "UTC").await;
    let bid = user_id(&b).await;
    let mut rx = state.live.subscribe();
    let gift = |to: &str| json!({ "kind": "ticket", "series": "lucky_three", "to": to });
    let (status, body) = post_json(&app, "/api/shop/buy", &me, gift(&a)).await;
    assert_eq!((status, body["error"].as_str()), (422, Some("gift_to_self")));
    let (status, _) = post_json(&app, "/api/shop/buy", &me, gift("nobody-at-all")).await;
    assert_eq!(status, 404);

    let (status, body) = post_json(&app, "/api/shop/buy", &me, gift(&b.to_uppercase())).await;
    assert_eq!(status, 200);
    assert_eq!(body["coins"], STARTING_COINS - 35);
    assert_eq!(body["gift_to"], b.as_str());
    assert!(body["ticket"].is_null(), "the sender gets no ticket");
    let env = rx.try_recv().unwrap();
    assert!(env.is_for(bid) && !env.is_for(bid + 1), "only the receiver hears of it");
    let live: Value = serde_json::from_str(&env.json).unwrap();
    assert_eq!((live["type"].as_str(), live["from"].as_str()), (Some("gift"), Some(a.as_str())));

    let (_, mine) = get(&app, "/api/me", &them).await;
    let t = mine["tickets"].as_array().unwrap().iter().find(|t| t["source"] == "gift").expect("the gift");
    assert_eq!(t["from"], a.as_str());
    assert_eq!(t["series"], "lucky_three");
}

#[actix_web::test]
async fn pot_tickets_fill_the_pot_and_one_entry_wins_it() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("pot");
    let me = register(&app, &name, "UTC").await;
    let (_, before) = get(&app, "/api/pot", &me).await;
    assert_eq!(before["mine"], 0);
    let (status, body) =
        post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "pot_ticket" })).await;
    assert_eq!(status, 200);
    assert_eq!(body["ticket"]["series"], "pot_ticket");
    let (_, after) = get(&app, "/api/pot", &me).await;
    assert_eq!(after["mine"], 1);
    assert!(after["entries"].as_i64() > before["entries"].as_i64());
    assert!(after["amount"].as_i64().unwrap() >= before["amount"].as_i64().unwrap() + 20);

    // A past day with this one entry: the chores draw it, once. (Other tests run
    // chores too and may draw it first, on their own hub: the hub's own test
    // checks the news goes out.)
    let uid = user_id(&name).await;
    let mut c = conn().await;
    // A day no pot has: before the earliest one (earlier runs leave theirs)
    let earliest: Option<chrono::NaiveDate> =
        schema::pots::table.select(diesel::dsl::min(schema::pots::day)).first(&mut c).await.unwrap();
    let day = earliest.unwrap_or_else(|| chrono::Utc::now().date_naive()) - chrono::Days::new(1);
    // The entry first: other tests' chores may draw the pot the moment it exists
    diesel::insert_into(schema::pot_entries::table)
        .values((schema::pot_entries::day.eq(day), schema::pot_entries::user_id.eq(uid)))
        .execute(&mut c)
        .await
        .unwrap();
    diesel::insert_into(schema::pots::table)
        .values((schema::pots::day.eq(day), schema::pots::amount.eq(777i64)))
        .execute(&mut c)
        .await
        .unwrap();
    let (_, me_before) = get(&app, "/api/me", &me).await;
    let (a, b) = tokio::join!(scratchme_api::chores(&state), scratchme_api::chores(&state));
    a.unwrap();
    b.unwrap();
    let (_, me_after) = get(&app, "/api/me", &me).await;
    assert_eq!(me_after["coins"].as_i64().unwrap(), me_before["coins"].as_i64().unwrap() + 777, "paid once");
    let winner: Option<i64> =
        schema::pots::table.find(day).select(schema::pots::winner_id).first(&mut c).await.unwrap();
    assert_eq!(winner, Some(uid));
}

#[actix_web::test]
async fn duels_stake_play_out_and_refund() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let (a, b) = (unique("duela"), unique("duelb"));
    let ca = register(&app, &a, "UTC").await;
    let cb = register(&app, &b, "UTC").await;
    let (status, body) = post_json(&app, "/api/duels", &ca, json!({ "opponent": a })).await;
    assert_eq!((status, body["error"].as_str()), (422, Some("duel_self")));

    // Declined: the stake comes back
    let (status, d) = post_json(&app, "/api/duels", &ca, json!({ "opponent": b })).await;
    assert_eq!(status, 200);
    assert_eq!(get(&app, "/api/me", &ca).await.1["coins"], STARTING_COINS - 100);
    let id = d["id"].as_i64().unwrap();
    let (status, _) = post_json(&app, &format!("/api/duels/{id}"), &ca, json!({ "action": "accept" })).await;
    assert_eq!(status, 422, "the challenger cannot accept");
    let (status, d) = post_json(&app, &format!("/api/duels/{id}"), &cb, json!({ "action": "decline" })).await;
    assert_eq!((status, d["status"].as_str()), (200, Some("declined")));
    assert_eq!(get(&app, "/api/me", &ca).await.1["coins"], STARTING_COINS);

    // Accepted twice at once: played once
    let (_, d) = post_json(&app, "/api/duels", &ca, json!({ "opponent": b })).await;
    let uri = format!("/api/duels/{}", d["id"]);
    let accept = || post_json(&app, &uri, &cb, json!({ "action": "accept" }));
    let (x, y) = tokio::join!(accept(), accept());
    let mut statuses = [x.0, y.0];
    statuses.sort();
    assert_eq!(statuses, [200, 404]);
    let (_, list) = get(&app, "/api/duels", &cb).await;
    let played = &list.as_array().unwrap()[0];
    assert_eq!(played["status"], "accepted");
    let winner = played["winner"].as_str().unwrap().to_owned();

    // Each has a duel ticket; scratched, the winner's pays the pot
    let mut total = 0;
    for (name, cookie) in [(&a, &ca), (&b, &cb)] {
        let (_, me) = get(&app, "/api/me", cookie).await;
        let t =
            me["tickets"].as_array().unwrap().iter().find(|t| t["series"] == "duel").expect("a duel ticket").clone();
        let (status, s) = post(&app, &format!("/api/tickets/{}/scratch", t["id"]), cookie).await;
        assert_eq!(status, 200);
        assert_eq!(s["layout"]["won"], *name == winner);
        total += s["coins"].as_i64().unwrap();
    }
    assert_eq!(total, 2 * STARTING_COINS - 200 + 180, "the house keeps 20");

    // Expired: refunded by the chores
    let (_, d) = post_json(&app, "/api/duels", &ca, json!({ "opponent": b })).await;
    let id = d["id"].as_i64().unwrap();
    let mut c = conn().await;
    diesel::update(schema::duels::table.find(id))
        .set(schema::duels::created_at.eq(chrono::Utc::now() - chrono::Duration::hours(25)))
        .execute(&mut c)
        .await
        .unwrap();
    let before = get(&app, "/api/me", &ca).await.1["coins"].as_i64().unwrap();
    scratchme_api::chores(&state).await.unwrap();
    assert_eq!(get(&app, "/api/me", &ca).await.1["coins"].as_i64().unwrap(), before + 100);
    let (_, list) = get(&app, "/api/duels", &ca).await;
    assert_eq!(list[0]["status"], "expired");
}

async fn guest<S>(app: &S) -> Cookie<'static>
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    let req = test::TestRequest::post().uri("/api/auth/guest").set_json(json!({ "tz": "UTC" })).to_request();
    let res = test::call_service(app, req).await;
    assert_eq!(res.status(), 201);
    res.response().cookies().find(|c| c.name() == "session").expect("session cookie").into_owned()
}

#[actix_web::test]
async fn a_guest_plays_at_once_then_keeps_the_account() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let g = guest(&app).await;
    let (status, me) = get(&app, "/api/me", &g).await;
    assert_eq!(status, 200);
    assert_eq!(me["guest"], true);
    assert!(me["username"].as_str().unwrap().starts_with("Guest-"));
    assert_eq!(me["coins"], STARTING_COINS);
    // Plays like anyone: the pocket, a ticket
    let (status, _) = post(&app, "/api/pocket/claim", &g).await;
    assert_eq!(status, 200);
    let (_, me) = get(&app, "/api/me", &g).await;
    let held = me["tickets"].as_array().unwrap().len();
    assert!(held > 0);

    // Keeping it: the name rules hold, taken names and guest names are refused
    let taken = unique("taken");
    register(&app, &taken, "UTC").await;
    let name = unique("kept");
    let claim = |username: &str, email: &str| json!({ "username": username, "email": email, "password": "123456789" });
    let (status, body) = post_json(&app, "/api/auth/claim", &g, claim("Guest-mine", "a@example.com")).await;
    assert_eq!((status, body["error"].as_str()), (422, Some("username_reserved")));
    let (status, body) = post_json(&app, "/api/auth/claim", &g, claim(&taken, "b@example.com")).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("username_taken")));
    let email = format!("{name}@example.com");
    let (status, _) = post_json(&app, "/api/auth/claim", &g, claim(&name, &email)).await;
    assert_eq!(status, 200);

    // Same account, same session: everything is still there
    let (_, me) = get(&app, "/api/me", &g).await;
    assert_eq!((me["guest"].as_bool(), me["username"].as_str()), (Some(false), Some(name.as_str())));
    assert_eq!(me["tickets"].as_array().unwrap().len(), held);
    let (status, body) = post_json(&app, "/api/auth/claim", &g, claim(&unique("again"), "c@example.com")).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("not_a_guest")));
    // And it logs in like any account
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "login": email, "password": "123456789" }))
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 200);
}

#[actix_web::test]
async fn guests_have_no_chat() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let g = guest(&app).await;
    let (status, body) = get(&app, "/api/shoutbox", &g).await;
    assert_eq!((status, body["error"].as_str()), (403, Some("guest")));
    let (status, _) = post_json(&app, "/api/shoutbox", &g, json!({ "body": "hi" })).await;
    assert_eq!(status, 403);
    let (status, _) = get(&app, "/api/mutes", &g).await;
    assert_eq!(status, 403);
    let other = unique("muteme");
    register(&app, &other, "UTC").await;
    let (status, _) = post(&app, &format!("/api/mutes/{other}"), &g).await;
    assert_eq!(status, 403);
    // The rest of the social side is theirs: announcements, profiles
    assert_eq!(get(&app, "/api/announcements", &g).await.0, 200);
    assert_eq!(get(&app, &format!("/api/players/{other}"), &g).await.0, 200);
    let (_, q) = get(&app, "/api/quests", &g).await;
    assert!(q["items"].as_array().unwrap().iter().all(|x| x["key"] != "shout"));
}

#[actix_web::test]
async fn guests_nobody_can_reach_are_forgotten() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let g = guest(&app).await;
    let (_, me) = get(&app, "/api/me", &g).await;
    let lost = me["username"].as_str().unwrap().to_owned();
    let member = unique("stays");
    register(&app, &member, "UTC").await;
    let mut c = conn().await;
    for name in [&lost, &member] {
        diesel::update(schema::sessions::table)
            .filter(
                schema::sessions::user_id
                    .eq_any(schema::users::table.filter(schema::users::username.eq(name)).select(schema::users::id)),
            )
            .set(schema::sessions::expires_at.eq(chrono::Utc::now() - chrono::Duration::days(1)))
            .execute(&mut c)
            .await
            .unwrap();
    }
    // Another guest still playing stays
    let kept = guest(&app).await;
    scratchme_api::chores(&state).await.unwrap();
    let left: Vec<String> = schema::users::table
        .filter(schema::users::username.eq_any([&lost, &member]))
        .select(schema::users::username)
        .load(&mut c)
        .await
        .unwrap();
    assert_eq!(left, vec![member], "the lost guest is gone, the member stays");
    assert_eq!(get(&app, "/api/me", &kept).await.0, 200);
}

#[actix_web::test]
async fn guests_neither_give_nor_duel_nor_are_given_or_challenged() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let g = guest(&app).await;
    let gname = get(&app, "/api/me", &g).await.1["username"].as_str().unwrap().to_owned();
    let member = unique("member");
    let m = register(&app, &member, "UTC").await;

    // From a guest: refused before any coin moves
    let gift = |to: &str| json!({ "kind": "ticket", "series": "lucky_three", "to": to });
    let (status, body) = post_json(&app, "/api/shop/buy", &g, gift(&member)).await;
    assert_eq!((status, body["error"].as_str()), (403, Some("guest")));
    let (status, _) = post_json(&app, "/api/duels", &g, json!({ "opponent": member })).await;
    assert_eq!(status, 403);
    assert_eq!(get(&app, "/api/duels", &g).await.0, 403);
    assert_eq!(get(&app, "/api/me", &g).await.1["coins"], STARTING_COINS);

    // To a guest: refused too, and the member keeps their coins
    let (status, body) = post_json(&app, "/api/shop/buy", &m, gift(&gname)).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("player_is_guest")));
    let (status, body) = post_json(&app, "/api/duels", &m, json!({ "opponent": gname })).await;
    assert_eq!((status, body["error"].as_str()), (409, Some("player_is_guest")));
    assert_eq!(get(&app, "/api/me", &m).await.1["coins"], STARTING_COINS);

    // Profiles say who is a guest, so the app offers nothing
    assert_eq!(get(&app, &format!("/api/players/{gname}"), &m).await.1["guest"], true);
    assert_eq!(get(&app, &format!("/api/players/{member}"), &g).await.1["guest"], false);
}

#[actix_web::test]
async fn pocket_tickets_grow_with_the_level() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("pocketlvl");
    let me = register(&app, &name, "UTC").await;
    assert_eq!(get(&app, "/api/me", &me).await.1["pocket_worth"], 35);
    // Level 10: tickets worth up to 125, none worth less than half of it
    set_xp(&name, scratchme_api::progress::levels::xp_for(10)).await;
    let (_, me_body) = get(&app, "/api/me", &me).await;
    assert_eq!(me_body["pocket_worth"], 125);
    let (status, body) = post(&app, "/api/pocket/claim", &me).await;
    assert_eq!(status, 200);
    let shop = get(&app, "/api/shop", &me).await.1;
    let price = |series: &Value| {
        shop["tickets"].as_array().unwrap().iter().find(|t| t["series"] == *series).unwrap()["price"].as_i64().unwrap()
    };
    let tickets = body["tickets"].as_array().unwrap();
    assert_eq!(tickets.len(), 3);
    for t in tickets {
        let p = price(&t["series"]);
        assert!((63..=125).contains(&p), "{} costs {p}", t["series"]);
        assert_eq!(t["source"], "pocket");
    }
}

#[actix_web::test]
async fn leaderboard_ranks_members_by_coins_and_level() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    // Above anything other tests leave behind, and above earlier runs
    let base = 1_000_000_000_000_000 + chrono::Utc::now().timestamp_millis() * 10;
    let (a, b, c) = (unique("top"), unique("second"), unique("tied"));
    let ca = register(&app, &a, "UTC").await;
    register(&app, &b, "UTC").await;
    register(&app, &c, "UTC").await;
    set_coins(&a, base + 3).await;
    set_coins(&b, base + 2).await;
    set_coins(&c, base + 2).await;
    // A guest richer than all of them is not ranked
    let g = guest(&app).await;
    let gname = get(&app, "/api/me", &g).await.1["username"].as_str().unwrap().to_owned();
    set_coins(&gname, base + 9).await;

    let (status, board) = get(&app, "/api/leaderboard?by=coins", &ca).await;
    assert_eq!(status, 200);
    let rows = board["rows"].as_array().unwrap();
    assert!(rows.len() <= 50);
    assert!(rows.iter().all(|r| r["username"] != gname.as_str()), "no guests");
    let at = |name: &str| rows.iter().find(|r| r["username"] == name).expect("ranked");
    assert_eq!(at(&a)["value"], base + 3);
    let top = at(&a)["rank"].as_i64().unwrap();
    // A tie shares its rank; the earlier sign-up is listed first
    assert_eq!((at(&b)["rank"].as_i64(), at(&c)["rank"].as_i64()), (Some(top + 1), Some(top + 1)));
    let pos = |name: &str| rows.iter().position(|r| r["username"] == name).unwrap();
    assert!(pos(&a) < pos(&b) && pos(&b) < pos(&c));
    assert_eq!(board["me"], json!({ "rank": top, "value": base + 3 }));

    // By level: experience decides, the level comes with it
    set_xp(&b, 10_000_000 + chrono::Utc::now().timestamp_millis() % 1_000_000).await;
    let (_, board) = get(&app, "/api/leaderboard?by=level", &ca).await;
    let row = board["rows"].as_array().unwrap().iter().find(|r| r["username"] == b.as_str()).expect("ranked");
    assert!(row["level"].as_i64().unwrap() > 400);
    assert!(board["me"]["rank"].as_i64().unwrap() > row["rank"].as_i64().unwrap());

    // A guest looks, unranked
    let (status, board) = get(&app, "/api/leaderboard?by=level", &g).await;
    assert_eq!((status, &board["me"]), (200, &Value::Null));
    assert_eq!(get(&app, "/api/leaderboard?by=gold", &g).await.0, 400);
}

#[actix_web::test]
async fn metrics_count_what_players_do_behind_a_token() {
    let Some(state) = state().await else { return };
    // The same state, with a token on the scrape
    let config = Config { metrics_token: Some("s3cret".into()), ..(*state.config).clone() };
    let state = web::Data::new(AppState {
        pool: db::pool(&config.database_url).unwrap(),
        live: Default::default(),
        limits: scratchme_api::ratelimit::AuthLimits::relaxed(),
        config: Arc::new(config),
        metrics: Default::default(),
    });
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .wrap(actix_web::middleware::from_fn(scratchme_api::metrics::track))
            .configure(routes::configure),
    )
    .await;
    let scraper = test::init_service(App::new().app_data(state.clone()).service(scratchme_api::metrics::scrape)).await;
    let scrape = |token: Option<&'static str>| {
        let mut req = test::TestRequest::get().uri("/metrics");
        if let Some(t) = token {
            req = req.insert_header(("authorization", format!("Bearer {t}")));
        }
        test::call_service(&scraper, req.to_request())
    };

    let name = unique("metered");
    let me = register(&app, &name, "UTC").await;
    post(&app, "/api/pocket/claim", &me).await;
    set_xp(&name, levels::xp_for(6)).await;
    let charm = |kind: &str| json!({ "kind": "charm", "charm": { "kind": kind, "tier": "bronze" } });
    post_json(&app, "/api/shop/buy", &me, charm("payout")).await;
    let won = known_ticket(
        &name,
        "lucky_three",
        json!({ "cells": vec!["coin"; 9], "win": "coin" }),
        json!({ "kind": "coins", "amount": 75 }),
    )
    .await;
    post(&app, &format!("/api/tickets/{won}/scratch"), &me).await;
    post_json(&app, "/api/shop/buy", &me, charm("discount")).await;
    post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "lucky_three" })).await;
    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(json!({ "login": name, "password": "wrong-password" }))
        .to_request();
    test::call_service(&app, req).await;

    assert_eq!(scrape(None).await.status(), 401);
    assert_eq!(scrape(Some("nope")).await.status(), 401);
    let res = scrape(Some("s3cret")).await;
    assert_eq!(res.status(), 200);
    let text = String::from_utf8(test::read_body(res).await.to_vec()).unwrap();
    for line in [
        r#"scratchme_auth_attempts_total{action="register",result="ok"} 1"#,
        r#"scratchme_auth_attempts_total{action="login",result="failed"} 1"#,
        r#"scratchme_coins_earned_total{reason="start"} 500"#,
        "scratchme_pocket_claims_total 1",
        r#"scratchme_tickets_issued_total{series="lucky_three",source="pocket"} 3"#,
        r#"scratchme_tickets_issued_total{series="lucky_three",source="shop"} 1"#,
        r#"scratchme_tickets_finished_total{series="lucky_three",result="win"} 1"#,
        r#"scratchme_ticket_prize_coins_total{series="lucky_three"} 75"#,
        r#"scratchme_coins_spent_total{reason="ticket"} 30"#,
        r#"scratchme_coins_spent_total{reason="charm"} 90"#,
        r#"scratchme_charms_bought_total{kind="payout",tier="bronze"} 1"#,
        r#"scratchme_charm_uses_total{kind="payout"} 1"#,
        r#"scratchme_charm_coins_total{kind="payout"} 15"#,
        r#"scratchme_charm_coins_total{kind="discount"} 5"#,
        r#"scratchme_coins_earned_total{reason="charm"} 15"#,
        r#"scratchme_http_requests_total{method="POST",route="/api/tickets/{id}/scratch",status="200"} 1"#,
    ] {
        assert!(text.contains(line), "missing {line}\n{text}");
    }
    assert!(text.contains(r#"scratchme_players{kind="member"}"#));
    assert!(text.contains(r#"scratchme_players{kind="guest"}"#));
    assert!(text.contains("scratchme_db_pool_connections"));
}

/// The charm of `kind` the player wears, from /me
async fn charm<S>(app: &S, me: &Cookie<'static>, kind: &str) -> Value
where
    S: Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>,
{
    let (_, body) = get(app, "/api/me", me).await;
    body["charms"].as_array().unwrap().iter().find(|c| c["kind"] == kind).cloned().unwrap_or(Value::Null)
}

#[actix_web::test]
async fn charms_discount_redraw_and_add_while_they_last() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("charm");
    let me = register(&app, &name, "UTC").await;
    set_coins(&name, 10_000).await;
    let buy = |kind: &str, tier: &str| {
        post_json(&app, "/api/shop/buy", &me, json!({ "kind": "charm", "charm": { "kind": kind, "tier": tier } }))
    };
    let (status, body) = buy("discount", "bronze").await;
    assert_eq!((status, body["error"].as_str()), (409, Some("level_too_low")));
    set_xp(&name, levels::xp_for(6)).await;
    let (_, shop) = get(&app, "/api/shop", &me).await;
    assert_eq!(shop["charms"].as_array().unwrap().len(), 9);

    // Discount: a charge per ticket bought within its reach
    let (status, body) = buy("discount", "bronze").await;
    assert_eq!(status, 200);
    assert_eq!(body["coins"], 10_000 - 40);
    assert_eq!(body["charms"][0]["charges"], 5);
    let (status, body) = buy("discount", "gold").await;
    assert_eq!((status, body["error"].as_str()), (409, Some("charm_active")), "one of a kind at a time");
    let (_, body) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "lucky_three" })).await;
    assert_eq!((body["price"].as_i64(), body["saved"].as_i64()), (Some(30), Some(5)));
    assert_eq!(body["coins"], 10_000 - 40 - 30);
    let (_, body) = post_json(&app, "/api/shop/buy", &me, json!({ "kind": "ticket", "series": "cash_wheel" })).await;
    assert_eq!((body["price"].as_i64(), body["saved"].as_i64()), (Some(400), Some(0)), "beyond bronze's reach");
    assert_eq!(charm(&app, &me, "discount").await["charges"], 4);

    // Expired: gone, and another of the kind may be bought
    diesel::sql_query("UPDATE charms SET expires_at = now() - interval '1 second' WHERE user_id = $1")
        .bind::<diesel::sql_types::BigInt, _>(user_id(&name).await)
        .execute(&mut conn().await)
        .await
        .unwrap();
    assert_eq!(charm(&app, &me, "discount").await, Value::Null);
    assert_eq!(buy("discount", "gold").await.0, 200);

    // Payout: a charge per ticket finished, the bonus on its coins
    assert_eq!(buy("payout", "gold").await.0, 200);
    let layout = json!({ "cells": vec!["coin"; 9], "win": "coin" });
    let win = known_ticket(&name, "lucky_three", layout.clone(), json!({ "kind": "coins", "amount": 100 })).await;
    let (_, before) = get(&app, "/api/me", &me).await;
    let (status, body) = post(&app, &format!("/api/tickets/{win}/scratch"), &me).await;
    assert_eq!(status, 200);
    assert_eq!(body["charm_bonus"], 30);
    assert_eq!(body["coins"].as_i64().unwrap(), before["coins"].as_i64().unwrap() + 130);
    let payout = body["charms"].as_array().unwrap().iter().find(|c| c["kind"] == "payout").unwrap();
    assert_eq!(payout["charges"], 11);

    // Luck: a charge per losing ticket only; a second draw replaces the stored one
    assert_eq!(buy("luck", "gold").await.0, 200);
    let win = known_ticket(&name, "lucky_three", layout.clone(), json!({ "kind": "coins", "amount": 10 })).await;
    post(&app, &format!("/api/tickets/{win}/scratch"), &me).await;
    assert_eq!(charm(&app, &me, "luck").await["charges"], 12, "a win needs no luck");
    let mut redrawn = 0;
    for left in (0..12).rev() {
        let lose = known_ticket(&name, "lucky_three", layout.clone(), json!({ "kind": "nothing" })).await;
        let (_, body) = post(&app, &format!("/api/tickets/{lose}/scratch"), &me).await;
        let luck = body["charms"].as_array().unwrap().iter().find(|c| c["kind"] == "luck").cloned();
        assert_eq!(luck.map(|c| c["charges"].clone()).unwrap_or(json!(0)), left);
        use schema::tickets;
        let stored: Value = tickets::table.find(lose).select(tickets::prize).first(&mut conn().await).await.unwrap();
        assert_eq!(stored, body["prize"], "what was paid is what is stored");
        if body["second_chance"] == true {
            redrawn += 1;
            assert_ne!(body["layout"], layout);
        } else {
            assert_eq!(body["prize"]["kind"], "nothing");
        }
    }
    eprintln!("{redrawn} of 12 drawn again");
    assert_eq!(charm(&app, &me, "luck").await, Value::Null, "spent");
}

#[actix_web::test]
async fn one_charm_of_a_kind_even_bought_at_once() {
    let Some(state) = state().await else { return };
    let app = app!(state);
    let name = unique("charm2");
    let me = register(&app, &name, "UTC").await;
    set_coins(&name, 1_000).await;
    set_xp(&name, levels::xp_for(6)).await;
    let buy = || {
        post_json(&app, "/api/shop/buy", &me, json!({ "kind": "charm", "charm": { "kind": "luck", "tier": "silver" } }))
    };
    let (a, b) = futures_join(buy(), buy()).await;
    let mut statuses = [a.0, b.0];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    let (_, body) = get(&app, "/api/me", &me).await;
    assert_eq!(body["coins"], 1_000 - 260, "the refused one paid nothing");
}
