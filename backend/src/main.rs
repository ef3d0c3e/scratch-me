use std::sync::Arc;

use actix_web::{App, HttpServer, middleware, web};
use scratchme_api::{AppState, Config, db, routes};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .init();

    let config = Config::from_env()?;
    db::run_migrations(&config.database_url).await?;
    let pool = db::pool(&config.database_url)?;

    let bind = config.bind.clone();
    let limits = if std::env::var("SCRATCHME_RELAXED_LIMITS").is_ok_and(|v| v == "1") {
        tracing::warn!("auth rate limits relaxed (SCRATCHME_RELAXED_LIMITS=1): development only");
        scratchme_api::ratelimit::AuthLimits::relaxed()
    } else {
        Default::default()
    };
    let state = web::Data::new(AppState { pool, limits, config: Arc::new(config) });
    tracing::info!("listening on http://{bind}");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Compress::default())
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(routes::configure)
    })
    .bind(bind)?
    .run()
    .await?;
    Ok(())
}
