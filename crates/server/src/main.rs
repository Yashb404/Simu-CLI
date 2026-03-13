use anyhow::Context;
use server::{config, middleware, router, state};
use axum::http::{header, HeaderValue, Method};
use governor::{Quota, RateLimiter};
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Load and validate configuration first
    let config = config::Config::from_env()?;

    // Set up logging with configured level
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| config.log_level.parse())
        .context("Invalid RUST_LOG format")?;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting CLI Demo Studio server...");
    tracing::debug!("Config: API URL = {}, Port = {}", config.api_url, config.port);
    tracing::debug!("Session secret loaded ({} hex chars)", config.session_secret.len());

    // Create database connection pool
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to database!");

    let migrations_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../migrations");
    sqlx::migrate::Migrator::new(migrations_path.clone())
        .await?
        .run(&db)
        .await?;
    tracing::info!("Database migrations applied from {}", migrations_path.display());

    // Set up the session store backed by our Postgres database
    let session_store = PostgresStore::new(db.clone());
    session_store.migrate().await?;

    tracing::info!("Session store initialized");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.session_cookie_secure)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(config.session_timeout));

    let rate_limit_quota = NonZeroU32::new(config.rate_limit_requests_per_minute)
        .context("RATE_LIMIT_REQUESTS_PER_MINUTE must be greater than 0")?;
    let rate_limiter = Arc::new(RateLimiter::keyed(Quota::per_minute(rate_limit_quota)));

    let state = state::AppState {
        db,
        config: config.clone(),
        rate_limiter,
    };

    let cors_origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin))
        .collect::<Result<_, _>>()
        .context("Invalid CORS origin header value")?;

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(cors_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION])
        .allow_credentials(true);

    let app = router::create_router(state.clone())
        .layer(axum::middleware::from_fn(
            middleware::logging::logging_middleware,
        ))
        .layer(axum::middleware::from_fn(
            middleware::metrics::metrics_middleware,
        ))
        .layer(axum::middleware::from_fn(
            middleware::security_headers::security_headers_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit::rate_limit_middleware,
        ))
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(session_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("✓ Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}