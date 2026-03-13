/// Shared test helpers for integration tests.
use std::{num::NonZeroU32, sync::Arc};

use axum::Router;
use governor::{Quota, RateLimiter};
use server::{
    config::Config,
    middleware,
    router::create_router,
    state::AppState,
};
use sqlx::PgPool;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

/// Build a [`Config`] suitable for tests.
pub fn test_config() -> Config {
    Config {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/cli_demo_studio".to_string()),
        github_client_id: "test-client-id".to_string(),
        github_client_secret: "test-client-secret".to_string(),
        session_secret: "a".repeat(64),
        api_url: "http://localhost:3001".to_string(),
        frontend_url: "http://localhost:8080".to_string(),
        port: 3001,
        rate_limit_requests_per_minute: 100,
        session_timeout: time::Duration::days(7),
        session_cookie_secure: false,
        log_level: "error".to_string(),
        cors_allowed_origins: vec!["http://localhost:8080".to_string()],
    }
}

/// Build a complete Axum [`Router`] with the full middleware stack (logging,
/// metrics, security headers, rate-limiting, and an in-memory session store).
/// Uses a lazy pool for tests that do not actually hit Postgres.
pub fn test_router(pool: PgPool) -> Router {
    test_router_with_rate_limit(pool, 100)
}

/// Like [`test_router`] but with a configurable rate limit so callers can test
/// 429 behaviour by passing a low value like `1`.
pub fn test_router_with_rate_limit(pool: PgPool, requests_per_minute: u32) -> Router {
    let per_minute = NonZeroU32::new(requests_per_minute).expect("rpm must be > 0");
    let state = AppState {
        db: pool,
        config: test_config(),
        rate_limiter: Arc::new(RateLimiter::keyed(Quota::per_minute(per_minute))),
    };

    // In-memory session store — no DB required for non-DB tests
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));

    create_router(state.clone())
        .layer(axum::middleware::from_fn(
            middleware::logging::logging_middleware,
        ))
        .layer(axum::middleware::from_fn(
            middleware::security_headers::security_headers_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state,
            middleware::rate_limit::rate_limit_middleware,
        ))
        .layer(session_layer)
}

/// Lazy (not-yet-connected) pool for tests that do not touch Postgres.
pub fn dummy_pool() -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://dummy:dummy@localhost/dummy")
        .expect("lazy pool must always construct")
}

/// Parse the body of a `Response` as JSON.
pub async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    use axum::body::to_bytes;
    let bytes = to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("failed to read response body");
    serde_json::from_slice(&bytes).expect("body must be valid JSON")
}
