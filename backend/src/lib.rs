pub mod auth;
pub mod charms;
pub mod config;
pub mod cosmetics;
pub mod db;
pub mod envelope;
pub mod error;
pub mod live;
pub mod metrics;
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
    pub live: live::Hub,
    pub limits: ratelimit::AuthLimits,
    pub config: Arc<Config>,
    pub metrics: metrics::Metrics,
}

/// Makes a player an admin; false when there is no such player
pub async fn make_admin(database_url: &str, username: &str) -> anyhow::Result<bool> {
    use diesel::prelude::*;
    use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
    let mut conn = AsyncPgConnection::establish(database_url).await?;
    let n = diesel::update(schema::users::table.filter(schema::users::username.eq(username)))
        .set(schema::users::is_admin.eq(true))
        .execute(&mut conn)
        .await?;
    Ok(n == 1)
}

/// Work done on a clock: drawing past days' pots, expiring stale duels.
/// Runs once a minute in the server; tests call it directly.
pub async fn chores(state: &AppState) -> anyhow::Result<()> {
    let mut conn = state.pool.get().await?;
    for (user, amount) in routes::pot::draw_due(&mut conn).await.map_err(|e| anyhow::anyhow!("{e}"))? {
        tracing::info!(%user, amount, "pot won");
        state.metrics.pot_won(amount);
        state.live.pot(&user, amount);
    }
    let gone = auth::forget_lost_guests(&mut conn).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    if gone > 0 {
        tracing::info!(gone, "lost guests forgotten");
        state.metrics.guests_forgotten(gone);
    }
    for (challenger, id) in routes::duels::expire_due(&mut conn).await.map_err(|e| anyhow::anyhow!("{e}"))? {
        state.metrics.duel("expired");
        state.metrics.earned("refund", routes::duels::STAKE_REFUND);
        state.live.notify(challenger, live::Event::Duel { id, from: String::new(), status: "expired".into() });
    }
    Ok(())
}
