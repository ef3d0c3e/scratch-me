use actix_web::{HttpResponse, get, web};

use crate::auth;

pub mod player;
pub mod progression;
pub mod shop;
pub mod social;
pub mod tickets;

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "ok": true }))
}

/// Every route lives under /api
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .app_data(web::JsonConfig::default().limit(16 * 1024).error_handler(|err, _| {
                tracing::debug!("bad json: {err}");
                crate::error::ApiError::Invalid("invalid_body").into()
            }))
            .service(health)
            .service(auth::register)
            .service(auth::login)
            .service(auth::logout)
            .service(player::get_me)
            .service(player::claim_pocket)
            .service(player::claim_envelope)
            .service(tickets::scratch)
            .service(tickets::get_dig)
            .service(tickets::dig)
            .service(tickets::look)
            .service(tickets::play_move)
            .service(shop::list)
            .service(shop::buy)
            .service(shop::equip)
            .service(progression::get_quests)
            .service(progression::reroll)
            .service(progression::get_achievements)
            .service(social::live)
            .service(social::get_shoutbox)
            .service(social::post_shout)
            .service(social::list_mutes)
            .service(social::mute)
            .service(social::unmute)
            .service(social::get_announcements)
            .service(social::post_announcement)
            .service(social::delete_announcement)
            .service(social::profile)
            .service(social::set_title),
    );
}
