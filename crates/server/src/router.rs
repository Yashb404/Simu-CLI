use axum::{routing::get, Router, Json};
use crate::{state::AppState, handlers, auth::AuthUser};
use shared::models::user::User;


pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/me", get(get_me))
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
    use tower::ServiceExt; // Gives us the `oneshot` method for testing routers

    #[tokio::test]
    async fn test_health_check() {
        // Setup a mock state (using a dummy DB URL since health_check doesn't hit the DB)
        // In real tests, we'd use sqlx::PgPoolOptions to spin up a transaction or mock.
        //FIXME: This is a bit hacky, but it allows us to test the router without setting up a real DB connection.
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://dummy:dummy@localhost/dummy")
            .unwrap();
            
        let state = AppState { db: pool };
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}