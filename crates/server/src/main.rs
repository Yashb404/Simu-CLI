mod router;
mod state;
mod auth;
mod handlers;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_sessions=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    tracing::info!("Connected to database!");

    // Set up the session store backed by our Postgres database
    let session_store = PostgresStore::new(db.clone());
    session_store.migrate().await?; // Auto-creates the tower-sessions table if needed

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in prod (requires HTTPS)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let state = state::AppState { db };
    let app = router::create_router(state).layer(session_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}