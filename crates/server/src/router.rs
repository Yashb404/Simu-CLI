use axum::{
    routing::{get, patch, post},
    Json, Router,
};
use crate::{state::AppState, handlers, auth::AuthUser};
use shared::models::user::User;


pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/me", get(get_me))
        .route("/api/demos", post(handlers::demos::create_demo))
        .route("/api/me/demos", get(handlers::demos::list_my_demos))
        .route(
            "/api/demos/{id}",
            get(handlers::demos::get_demo)
                .patch(handlers::demos::update_demo)
                .delete(handlers::demos::delete_demo),
        )
        .route(
            "/api/demos/{id}/public",
            get(handlers::demos::get_public_demo),
        )
        .route("/api/projects", post(handlers::projects::create_project))
        .route("/api/me/projects", get(handlers::projects::list_my_projects))
        .route(
            "/api/projects/{id}",
            patch(handlers::projects::update_project)
                .delete(handlers::projects::delete_project),
        )
        .nest("/api/auth", handlers::auth::auth_routes())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
async fn get_me(AuthUser(user): AuthUser) -> Json<User> {
    Json(user)
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
        //FIXME: This is a bit hacky, but it allows us to test the router without setting up a real DB connection.
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
                github_client_secret: "test-secret".to_string(),
                session_secret: "a".repeat(64),
                api_url: "http://localhost:3001".to_string(),
                frontend_url: "http://localhost:3000".to_string(),
                port: 3001,
                rate_limit_requests_per_minute: 100,
                session_timeout: time::Duration::days(7),
                log_level: "server=debug".to_string(),
            },
            rate_limiter: Arc::new(RateLimiter::keyed(Quota::per_minute(per_minute))),
        };
        let app = create_router(state);

        let request_result = Request::builder().uri("/api/health").body(Body::empty());
        assert!(request_result.is_ok(), "request must be constructible");
        let request = match request_result {
            Ok(request) => request,
            Err(_) => return,
        };

        let response_result = app.oneshot(request).await;
        assert!(response_result.is_ok(), "health check request should succeed");
        let response = match response_result {
            Ok(response) => response,
            Err(_) => return,
        };

        assert_eq!(response.status(), StatusCode::OK);
    }
}