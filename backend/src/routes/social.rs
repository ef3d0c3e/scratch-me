//! The live socket (push only), the shoutbox, mutes, announcements, profiles.
//! Everything a client sends goes through plain REST calls.

use std::time::Duration;

use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use actix_ws::Message;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;

use crate::AppState;
use crate::auth::CurrentUser;
use crate::error::{ApiError, ApiResult};
use crate::live::{AnnouncementDto, ChatEvent};
use crate::progress::{self, Event, levels};
use crate::schema::{achievements, announcements, mutes, shouts, stats, streaks, users};

diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

const HISTORY: i64 = 50;
pub const MAX_BODY: usize = 300;
/// At most one shout every 2 s, and 10 a minute
const MIN_GAP: chrono::Duration = chrono::Duration::seconds(2);
const PER_MINUTE: usize = 10;
const MAX_ANNOUNCEMENT: usize = 500;
/// Announcements shown at once
const LIVE_ANNOUNCEMENTS: i64 = 3;
const PING_EVERY: Duration = Duration::from_secs(30);

/// Ids of players the viewer muted
async fn muted_by(conn: &mut AsyncPgConnection, viewer: i64) -> ApiResult<Vec<i64>> {
    Ok(mutes::table.filter(mutes::user_id.eq(viewer)).select(mutes::muted_id).load(conn).await?)
}

/// id, author, author's xp, author's title, body, time
type ShoutRow = (i64, String, i64, Option<String>, String, DateTime<Utc>);

/// The last shouts, oldest first, without the muted
async fn history(conn: &mut AsyncPgConnection, viewer: i64) -> ApiResult<Vec<ChatEvent>> {
    let muted = muted_by(conn, viewer).await?;
    let rows: Vec<ShoutRow> = shouts::table
        .inner_join(users::table)
        .filter(diesel::dsl::not(shouts::user_id.eq_any(&muted)))
        .order(shouts::id.desc())
        .limit(HISTORY)
        .select((shouts::id, users::username, users::xp, users::title, shouts::body, shouts::created_at))
        .load(conn)
        .await?;
    Ok(rows
        .into_iter()
        .rev()
        .map(|(id, user, xp, title, body, at)| ChatEvent { id, user, level: levels::level_for(xp), title, body, at })
        .collect())
}

async fn live_announcements(conn: &mut AsyncPgConnection) -> ApiResult<Vec<AnnouncementDto>> {
    let rows: Vec<(i64, String, DateTime<Utc>)> = announcements::table
        .filter(announcements::removed_at.is_null())
        .order(announcements::id.desc())
        .limit(LIVE_ANNOUNCEMENTS)
        .select((announcements::id, announcements::body, announcements::created_at))
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(|(id, body, at)| AnnouncementDto { id, body, at }).collect())
}

/// Push-only WebSocket: a "hello" snapshot, then live events
#[get("/live")]
pub async fn live(
    state: web::Data<AppState>,
    user: CurrentUser,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    // Guests have no chat: no shouts in the hello, none live
    let guest = user.is_guest;
    let (messages, news) = {
        let mut conn = state.pool.get().await.map_err(ApiError::from)?;
        let messages = if guest { vec![] } else { history(&mut conn, user.id).await? };
        (messages, live_announcements(&mut conn).await?)
    };
    let hello = serde_json::json!({
        "type": "hello",
        "messages": messages,
        "feed": state.live.feed(),
        "announcements": news,
    });
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;
    let mut rx = state.live.subscribe();
    let uid = user.id;
    let hub = state.clone();
    actix_web::rt::spawn(async move {
        hub.live.connected(uid);
        if session.text(hello.to_string()).await.is_err() {
            hub.live.disconnected(uid);
            return;
        }
        let mut ping = tokio::time::interval(PING_EVERY);
        ping.tick().await;
        loop {
            tokio::select! {
                event = rx.recv() => match event {
                    Ok(env) if env.is_for(uid) && !(guest && env.chat) => {
                        if session.text(env.json.clone()).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    // A slow client missed some events: it keeps going with the next ones
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => break,
                },
                msg = stream.recv() => match msg {
                    Some(Ok(Message::Ping(bytes))) => {
                        if session.pong(&bytes).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                },
                _ = ping.tick() => {
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }
            }
        }
        hub.live.disconnected(uid);
        let _ = session.close(None).await;
    });
    Ok(response)
}

#[get("/shoutbox")]
pub async fn get_shoutbox(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<Vec<ChatEvent>>> {
    user.member()?;
    let mut conn = state.pool.get().await?;
    Ok(web::Json(history(&mut conn, user.id).await?))
}

#[derive(Deserialize)]
pub struct ShoutBody {
    body: String,
}

#[derive(Serialize)]
pub struct Shouted {
    pub shout: ChatEvent,
    pub progress: progress::Progress,
}

#[post("/shoutbox")]
pub async fn post_shout(
    state: web::Data<AppState>,
    user: CurrentUser,
    body: web::Json<ShoutBody>,
) -> ApiResult<HttpResponse> {
    user.member()?;
    let text = body.into_inner().body.trim().to_owned();
    if text.is_empty() {
        return Err(ApiError::Invalid("empty_message"));
    }
    if text.chars().count() > MAX_BODY {
        return Err(ApiError::Invalid("message_too_long"));
    }
    let mut conn = state.pool.get().await?;
    let uid = user.id;
    let name = user.username.clone();
    let out = conn
        .transaction(async move |conn| {
            // The row lock makes a burst of shouts from one player take turns
            let (xp, title): (i64, Option<String>) =
                users::table.find(uid).select((users::xp, users::title)).for_update().first(conn).await?;
            let now = Utc::now();
            let recent: Vec<DateTime<Utc>> = shouts::table
                .filter(shouts::user_id.eq(uid))
                .filter(shouts::created_at.gt(now - chrono::Duration::minutes(1)))
                .select(shouts::created_at)
                .load(conn)
                .await?;
            if recent.iter().any(|t| now - *t < MIN_GAP) || recent.len() >= PER_MINUTE {
                return Err(ApiError::TooMany("slow_down"));
            }
            let (id, at): (i64, DateTime<Utc>) = diesel::insert_into(shouts::table)
                .values((shouts::user_id.eq(uid), shouts::body.eq(&text)))
                .returning((shouts::id, shouts::created_at))
                .get_result(conn)
                .await?;
            let progress = progress::apply(conn, uid, &[Event::Shouted]).await?;
            let level = progress.level.level.max(levels::level_for(xp));
            Ok(Shouted { shout: ChatEvent { id, user: name, level, title, body: text, at }, progress })
        })
        .await?;
    state.live.chat(out.shout.clone());
    crate::live::broadcast(&state.live, &user.username, None, &out.progress.achievements);
    Ok(HttpResponse::Created().json(out))
}

async fn user_id(conn: &mut AsyncPgConnection, name: &str) -> ApiResult<i64> {
    users::table
        .filter(lower(users::username).eq(name.to_lowercase()))
        .select(users::id)
        .first(conn)
        .await
        .optional()?
        .ok_or(ApiError::NotFound("player"))
}

#[get("/mutes")]
pub async fn list_mutes(state: web::Data<AppState>, user: CurrentUser) -> ApiResult<web::Json<Vec<String>>> {
    user.member()?;
    let mut conn = state.pool.get().await?;
    let names = mutes::table
        .inner_join(users::table.on(users::id.eq(mutes::muted_id)))
        .filter(mutes::user_id.eq(user.id))
        .select(users::username)
        .load(&mut conn)
        .await?;
    Ok(web::Json(names))
}

#[post("/mutes/{name}")]
pub async fn mute(state: web::Data<AppState>, user: CurrentUser, path: web::Path<String>) -> ApiResult<HttpResponse> {
    user.member()?;
    let mut conn = state.pool.get().await?;
    let other = user_id(&mut conn, &path).await?;
    if other == user.id {
        return Err(ApiError::Invalid("cannot_mute_self"));
    }
    diesel::insert_into(mutes::table)
        .values((mutes::user_id.eq(user.id), mutes::muted_id.eq(other)))
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete("/mutes/{name}")]
pub async fn unmute(state: web::Data<AppState>, user: CurrentUser, path: web::Path<String>) -> ApiResult<HttpResponse> {
    user.member()?;
    let mut conn = state.pool.get().await?;
    let other = user_id(&mut conn, &path).await?;
    diesel::delete(mutes::table.find((user.id, other))).execute(&mut conn).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/announcements")]
pub async fn get_announcements(
    state: web::Data<AppState>,
    _user: CurrentUser,
) -> ApiResult<web::Json<Vec<AnnouncementDto>>> {
    let mut conn = state.pool.get().await?;
    Ok(web::Json(live_announcements(&mut conn).await?))
}

async fn require_admin(conn: &mut AsyncPgConnection, uid: i64) -> ApiResult<()> {
    let admin: bool = users::table.find(uid).select(users::is_admin).first(conn).await?;
    if admin { Ok(()) } else { Err(ApiError::Forbidden) }
}

#[derive(Deserialize)]
pub struct AnnouncementBody {
    body: String,
}

#[post("/announcements")]
pub async fn post_announcement(
    state: web::Data<AppState>,
    user: CurrentUser,
    body: web::Json<AnnouncementBody>,
) -> ApiResult<HttpResponse> {
    let text = body.into_inner().body.trim().to_owned();
    if text.is_empty() || text.chars().count() > MAX_ANNOUNCEMENT {
        return Err(ApiError::Invalid("invalid_announcement"));
    }
    let mut conn = state.pool.get().await?;
    require_admin(&mut conn, user.id).await?;
    let (id, at): (i64, DateTime<Utc>) = diesel::insert_into(announcements::table)
        .values((announcements::author_id.eq(user.id), announcements::body.eq(&text)))
        .returning((announcements::id, announcements::created_at))
        .get_result(&mut conn)
        .await?;
    let a = AnnouncementDto { id, body: text, at };
    state.live.announce(a.clone());
    Ok(HttpResponse::Created().json(a))
}

#[delete("/announcements/{id}")]
pub async fn delete_announcement(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<i64>,
) -> ApiResult<HttpResponse> {
    let id = path.into_inner();
    let mut conn = state.pool.get().await?;
    require_admin(&mut conn, user.id).await?;
    let n = diesel::update(announcements::table.find(id).filter(announcements::removed_at.is_null()))
        .set(announcements::removed_at.eq(Utc::now()))
        .execute(&mut conn)
        .await?;
    if n == 0 {
        return Err(ApiError::NotFound("announcement"));
    }
    state.live.unannounce(id);
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize)]
pub struct ProfileDto {
    pub username: String,
    pub level: levels::LevelDto,
    pub title: Option<String>,
    pub theme: String,
    pub joined: DateTime<Utc>,
    pub online: bool,
    pub scratched: i32,
    pub wins: i32,
    pub best_win: i64,
    pub best_streak: i32,
    /// Achievements earned, newest first
    pub achievements: Vec<String>,
    /// The viewer muted them
    pub muted: bool,
    /// No gifts, duels or chat with a guest
    pub guest: bool,
}

#[get("/players/{name}")]
pub async fn profile(
    state: web::Data<AppState>,
    user: CurrentUser,
    path: web::Path<String>,
) -> ApiResult<web::Json<ProfileDto>> {
    let mut conn = state.pool.get().await?;
    let id = user_id(&mut conn, &path).await?;
    type Row = (String, i64, Option<String>, String, DateTime<Utc>, bool);
    let (username, xp, title, theme, joined, guest): Row = users::table
        .find(id)
        .select((users::username, users::xp, users::title, users::theme, users::created_at, users::is_guest))
        .first(&mut conn)
        .await?;
    let (scratched, wins, best_win): (i32, i32, i64) = stats::table
        .find(id)
        .select((stats::scratched, stats::wins, stats::best_win))
        .first(&mut conn)
        .await
        .optional()?
        .unwrap_or((0, 0, 0));
    let best_streak: i32 =
        streaks::table.find(id).select(streaks::best).first(&mut conn).await.optional()?.unwrap_or(0);
    let earned = achievements::table
        .filter(achievements::user_id.eq(id))
        .order(achievements::earned_at.desc())
        .select(achievements::id)
        .load(&mut conn)
        .await?;
    let muted = diesel::select(diesel::dsl::exists(mutes::table.find((user.id, id)))).get_result(&mut conn).await?;
    Ok(web::Json(ProfileDto {
        username,
        level: levels::dto(xp),
        title,
        theme,
        joined,
        online: state.live.is_online(id),
        scratched,
        wins,
        best_win,
        best_streak,
        achievements: earned,
        muted,
        guest,
    }))
}

#[derive(Deserialize)]
pub struct TitleBody {
    /// An achievement id the player earned, or null for none
    title: Option<String>,
}

#[post("/me/title")]
pub async fn set_title(
    state: web::Data<AppState>,
    user: CurrentUser,
    body: web::Json<TitleBody>,
) -> ApiResult<web::Json<Option<String>>> {
    let title = body.into_inner().title;
    let mut conn = state.pool.get().await?;
    if let Some(t) = &title {
        let earned: bool =
            diesel::select(diesel::dsl::exists(achievements::table.find((user.id, t)))).get_result(&mut conn).await?;
        if !earned {
            return Err(ApiError::Conflict("not_earned"));
        }
    }
    diesel::update(users::table.find(user.id)).set(users::title.eq(&title)).execute(&mut conn).await?;
    Ok(web::Json(title))
}
