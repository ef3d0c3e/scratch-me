//! End-to-end tests of the HTTP API against a real Postgres.
//! They need TEST_DATABASE_URL (see .env.example) and are skipped without it.

use std::sync::Arc;

use actix_web::cookie::Cookie;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{App, test, web};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
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
    let config =
        Config { database_url: url.clone(), bind: String::new(), secure_cookies: false, trusted_proxies: vec![] };
    Some(web::Data::new(AppState {
        pool: db::pool(&url).unwrap(),
        live: Default::default(),
        limits: scratchme_api::ratelimit::AuthLimits::relaxed(),
        config: Arc::new(config),
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
    assert_eq!((status, body["error"].as_str()), (409, Some("level_too_low")), "the wheel takes level 5");
    set_xp(&name, 1000).await;
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
    set_xp(&name, 300).await;
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
    set_xp(&name, 95).await;
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
    assert_eq!(q["bonus_done"], true);
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
    let live: Value = serde_json::from_str(&rx.try_recv().unwrap()).unwrap();
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
    let live: Value = serde_json::from_str(&rx.try_recv().unwrap()).unwrap();
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
    let live: Value = serde_json::from_str(&rx.try_recv().unwrap()).unwrap();
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
