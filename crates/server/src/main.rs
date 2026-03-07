mod router;
mod state;
mod auth;
mod handlers;
mod error;
mod config;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Load and validate configuration first
    let config = config::Config::from_env()?;

    // Set up logging with configured level
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.parse().expect("Invalid RUST_LOG format")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting CLI Demo Studio server...");
    tracing::debug!("Config: API URL = {}, Port = {}", config.api_url, config.port);

    // Create database connection pool
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to database!");

    // Set up the session store backed by our Postgres database
    let session_store = PostgresStore::new(db.clone());
    session_store.migrate().await?;

    tracing::info!("Session store initialized");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // TODO: Set to true in prod (requires HTTPS)
        .with_expiry(Expiry::OnInactivity(config.session_timeout));

    let state = state::AppState { db, config: config.clone() };
    let app = router::create_router(state).layer(session_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("✓ Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}