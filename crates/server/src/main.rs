use anyhow::Context;
use axum::http::{HeaderValue, Method, header};
use axum::response::Html;
use axum::routing::get_service;
use governor::{Quota, RateLimiter};
use server::config::SessionCookieSameSite;
use server::{config, middleware, router, state};
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    tracing::info!("Starting SimuCLI server...");
    tracing::debug!(
        "Config: API URL = {}, Port = {}",
        config.api_url,
        config.port
    );
    tracing::debug!("Session secret loaded");

    // Create database connection pool
    let db = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to database!");

    let migrations_path = resolve_migrations_path()?;
    sqlx::migrate::Migrator::new(migrations_path.clone())
        .await?
        .run(&db)
        .await?;
    tracing::info!(
        "Database migrations applied from {}",
        migrations_path.display()
    );

    // Set up the session store backed by our Postgres database
    let session_store = PostgresStore::new(db.clone());
    session_store.migrate().await?;

    tracing::info!("Session store initialized");

    let same_site = match config.session_cookie_same_site {
        SessionCookieSameSite::Lax => SameSite::Lax,
        SessionCookieSameSite::Strict => SameSite::Strict,
        SessionCookieSameSite::None => SameSite::None,
    };

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.session_cookie_secure)
        .with_same_site(same_site)
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

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let app_dist_dir = if workspace_root.join("dist").is_dir() {
        workspace_root.join("dist")
    } else {
        workspace_root.join("crates/app/dist")
    };
    let embed_dist_dir = if workspace_root.join("dist-embed").is_dir() {
        workspace_root.join("dist-embed")
    } else {
        workspace_root.join("crates/app/embed")
    };
    let static_dir = workspace_root.join("static");

    let app_index = app_dist_dir.join("index.html");
    let embed_index = embed_dist_dir.join("index.html");
    let has_app_index = app_index.is_file();

    let app_static = get_service(
        // Use fallback (not not_found_service) so SPA routes return index.html with 200.
        ServeDir::new(&app_dist_dir).fallback(ServeFile::new(&app_index)),
    );
    let embed_static =
        get_service(ServeDir::new(&embed_dist_dir).fallback(ServeFile::new(&embed_index)));
    let static_assets = get_service(ServeDir::new(&static_dir));

    if !has_app_index {
        tracing::warn!(
            "Dashboard index not found at {}. Serving API landing page at /.\nIf this is unexpected, verify frontend build artifacts are copied into the runtime image.",
            app_index.display()
        );
    }

    let app = if has_app_index {
        router::create_router(state.clone())
            .route_service("/", ServeFile::new(app_index.clone()))
            .route_service("/embed-runtime", ServeFile::new(&embed_index))
            .nest_service("/embed-runtime/", embed_static)
            .nest_service("/static", static_assets)
            .fallback_service(app_static)
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
            .layer(session_layer)
    } else {
        router::create_router(state.clone())
            .route("/", axum::routing::get(api_landing_page))
            .route_service("/embed-runtime", ServeFile::new(&embed_index))
            .nest_service("/embed-runtime/", embed_static)
            .nest_service("/static", static_assets)
            .fallback_service(app_static)
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
            .layer(session_layer)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("✓ Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

fn resolve_migrations_path() -> anyhow::Result<std::path::PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(from_env) = std::env::var("MIGRATIONS_DIR") {
        let trimmed = from_env.trim();
        if !trimmed.is_empty() {
            candidates.push(std::path::PathBuf::from(trimmed));
        }
    }

    candidates.push(std::path::PathBuf::from("/app/migrations"));
    candidates.push(std::path::PathBuf::from("migrations"));
    candidates.push(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"));

    if let Some(path) = candidates.iter().find(|path| path.is_dir()) {
        return Ok(path.clone());
    }

    let checked = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    anyhow::bail!(
        "No migrations directory found. Checked: {}. Set MIGRATIONS_DIR to a valid path.",
        checked
    )
}

async fn api_landing_page() -> Html<&'static str> {
    Html(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>SimuCLI API</title><style>body{font-family:system-ui,-apple-system,sans-serif;max-width:760px;margin:48px auto;padding:0 16px;line-height:1.5;color:#111}a{color:#0f4c81}code{background:#f4f4f5;padding:2px 6px;border-radius:6px}</style></head><body><h1>SimuCLI API</h1><p>The backend is running.</p><p>Health check: <a href=\"/api/health\">/api/health</a></p><p>If you expected the dashboard at <code>/</code>, ensure frontend build files are available at runtime.</p></body></html>",
    )
}
