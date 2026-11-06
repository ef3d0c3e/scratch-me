pub mod auth;
pub mod config;
pub mod cosmetics;
pub mod db;
pub mod envelope;
pub mod error;
pub mod pocket;
pub mod progress;
pub mod ratelimit;
pub mod routes;
pub mod schema;
pub mod tickets;

use std::sync::Arc;

pub use config::Config;

/// Coins a new account starts with
pub const STARTING_COINS: i64 = 500;

pub struct AppState {
    pub pool: db::DbPool,
    pub limits: ratelimit::AuthLimits,
    pub config: Arc<Config>,
}
