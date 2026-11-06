use std::sync::Arc;

use actix_web::{App, HttpServer, middleware, web};
use anyhow::Context;
use scratchme_api::{AppState, Config, db, routes};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    // `scratchme-api make-admin <username>`: lets that player post announcements
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("make-admin") {
        let name = args.get(2).context("usage: scratchme-api make-admin <username>")?;
        let config = Config::from_env()?;
        db::run_migrations(&config.database_url).await?;
        let made = scratchme_api::make_admin(&config.database_url, name).await?;
        anyhow::ensure!(made, "no player named {name}");
        println!("{name} is now an admin");
        return Ok(());
    }
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
    let state = web::Data::new(AppState { pool, live: Default::default(), limits, config: Arc::new(config) });
    // Pots to draw and duels to expire, once a minute
    let chores = state.clone();
    actix_web::rt::spawn(async move {
        let mut every = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            every.tick().await;
            if let Err(e) = scratchme_api::chores(&chores).await {
                tracing::error!("chores: {e:#}");
            }
        }
    });
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
