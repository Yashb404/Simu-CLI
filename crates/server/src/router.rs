use crate::{auth::AuthUser, handlers, middleware, state::AppState};
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, patch, post},
    extract::DefaultBodyLimit
};
use shared::dto::ApiErrorResponse;
use shared::models::user::User;


/// Creates and Axum `Router` with the application's HTTP routes and attaches the given shared `AppState`.
///
/// The router includes health, metrics, auth, demo management, analytics, billing, project, and API-not-found routes
/// and is ready to be served by an Axum server after attaching the provided state.
///
/// # Examples
///
/// ```
/// // Construct your AppState and create the router
/// let state = /* construct AppState */ todo!();
/// let router = create_router(state);
/// ```
pub fn create_router(state: AppState) -> Router {
    let app: Router<AppState> = Router::new()
        .route("/", get(root_handler))
        .route("/metrics", get(middleware::metrics::metrics_handler))
        .nest("/api/v1", api_router())
        // Backward-compatible alias during migration.
        .nest("/api", api_router())
        ;

    app.with_state(state)
}

fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/me", get(get_me))
        .route("/me/dashboard", get(handlers::dashboard::get_my_dashboard))
        .route("/demos", post(handlers::demos::create_demo))
        .route("/me/demos", get(handlers::demos::list_my_demos))
        .route(
            "/demos/{id}",
            get(handlers::demos::get_demo)
                .patch(handlers::demos::update_demo)
                .delete(handlers::demos::delete_demo),
        )
        .route("/demos/{id}/public", get(handlers::demos::get_public_demo))
        .route("/public/demos/{reference}", get(handlers::demos::get_public_demo_by_reference))
        .route("/demos/{id}/publish", post(handlers::demos::publish_demo))
        .route(
            "/demos/{id}/import-cast",
            post(handlers::demos::import_cast)
                // FIX S-07: Reject payloads > 5MB at the network layer to prevent RAM exhaustion
                .layer(DefaultBodyLimit::max(5 * 1024 * 1024)), 
        )
        .route("/demos/{id}/og-image", get(handlers::demos::get_demo_og_image))
        .route("/demos/{id}/analytics", get(handlers::analytics::get_demo_analytics))
        .route("/demos/{id}/analytics/referrers", get(handlers::analytics::get_demo_referrers))
        .route("/demos/{id}/analytics/funnel", get(handlers::analytics::get_demo_funnel))
        .route("/demos/{id}/analytics/export", get(handlers::analytics::export_demo_analytics_csv))
        .route("/demos/{id}/common-errors", get(handlers::common_errors::get_common_errors))
        .route("/analytics/events", post(handlers::analytics::post_event))
        .route("/analytics/common-errors", post(handlers::common_errors::record_common_error))
        .route("/billing/status", get(handlers::billing::get_billing_status))
        .route("/billing/subscribe", post(handlers::billing::subscribe))
        .route("/projects", post(handlers::projects::create_project))
        .route("/me/projects", get(handlers::projects::list_my_projects))
        .route(
            "/projects/{id}",
            patch(handlers::projects::update_project).delete(handlers::projects::delete_project),
        )
        .nest("/auth", handlers::auth::auth_routes())
        .route(
            "/{*path}",
            get(api_not_found)
                .post(api_not_found)
                .patch(api_not_found)
                .delete(api_not_found),
        )
}


async fn health_check() -> &'static str {
    "OK"
}
async fn get_me(AuthUser(user): AuthUser) -> Json<User> {
    Json(user)
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "SimuCLI API",
        "status": "ok",
        "api_version": "v1"
    }))
}

async fn api_not_found() -> (StatusCode, Json<serde_json::Value>) {
    let body = ApiErrorResponse {
        error: "API route not found".to_string(),
        error_code: Some("NOT_FOUND".to_string()),
        request_id: None,
        details: None,
    };

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::to_value(body).unwrap_or_else(|_| {
            serde_json::json!({"error": "API route not found", "error_code": "NOT_FOUND"})
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use governor::{Quota, RateLimiter};
    use std::{num::NonZeroU32, sync::Arc};
    use tower::ServiceExt; // Gives us the `oneshot` method for testing routers

    use crate::config::Config;

    #[tokio::test]
    async fn test_health_check() {
        // Setup a mock state (using a dummy DB URL since health_check doesn't hit the DB)
        // In real tests, we'd use sqlx::PgPoolOptions to spin up a transaction or mock.
        let pool_result = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://dummy:dummy@localhost/dummy");
        assert!(pool_result.is_ok(), "lazy pool should construct in tests");
        let pool = match pool_result {
            Ok(pool) => pool,
            Err(_) => return,
        };

        let per_minute = match NonZeroU32::new(100) {
            Some(limit) => limit,
            None => return,
        };

        let state = AppState {
            db: pool,
            config: Config {
                database_url: "postgres://dummy:dummy@localhost/dummy".to_string(),
                github_client_id: "test-client".to_string(),
                github_client_secret: crate::config::Secret("test-secret".to_string()),
                session_secret: crate::config::Secret("a".repeat(64)),
                api_url: "https://api.example.test".to_string(),
                frontend_url: "https://app.example.test".to_string(),
                port: 3001,
                rate_limit_requests_per_minute: 100,
                db_max_connections: 5,
                session_timeout: time::Duration::days(7),
                session_cookie_secure: false,
                session_cookie_same_site: crate::config::SessionCookieSameSite::Lax,
                log_level: "server=debug".to_string(),
                cors_allowed_origins: vec!["https://app.example.test".to_string()],
            },
            rate_limiter: Arc::new(RateLimiter::keyed(Quota::per_minute(per_minute))),
        };
        let app = create_router(state);

        let request_result = Request::builder().uri("/api/v1/health").body(Body::empty());
        assert!(request_result.is_ok(), "request must be constructible");
        let request = match request_result {
            Ok(request) => request,
            Err(_) => return,
        };

        let response_result = app.oneshot(request).await;
        assert!(
            response_result.is_ok(),
            "health check request should succeed"
        );
        let response = match response_result {
            Ok(response) => response,
            Err(_) => return,
        };

        assert_eq!(response.status(), StatusCode::OK);
    }
}
