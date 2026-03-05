use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;
use tower_sessions::Session;
use crate::{auth::{github_oauth_client, USER_SESSION_KEY}, state::AppState};
use shared::models::user::User;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/github", get(github_login))
        .route("/github/callback", get(github_callback))
        .route("/logout", get(logout))
}

async fn github_login(session: Session) -> impl IntoResponse {
    let client = github_oauth_client();
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // Store CSRF token as a String to satisfy strict type inference
    session.insert("csrf_token", csrf_token.secret().clone()).await.unwrap();

    Redirect::to(auth_url.as_ref())
}

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
    email: Option<String>,
    avatar_url: Option<String>,
}

async fn github_callback(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<AuthRequest>,
) -> impl IntoResponse {
    let stored_csrf: Option<String> = session.get("csrf_token").await.unwrap();
    
    if stored_csrf != Some(query.state) {
        return (axum::http::StatusCode::UNAUTHORIZED, "CSRF token mismatch").into_response();
    }

    let client = github_oauth_client();
    let reqwest_client = reqwest::Client::new(); // Use standard reqwest client in v5
    
    let token = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&reqwest_client)
        .await
        .expect("Failed to exchange code");

    let github_user: GithubUser = reqwest_client
        .get("https://api.github.com/user")
        .bearer_auth(token.access_token().secret())
        .header("User-Agent", "cli-demo-studio")
        .send()
        .await
        .expect("Failed to fetch user")
        .json()
        .await
        .expect("Failed to parse user JSON");

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (github_id, username, email, avatar_url)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (github_id) DO UPDATE
        SET username = EXCLUDED.username,
            email = EXCLUDED.email,
            avatar_url = EXCLUDED.avatar_url,
            updated_at = NOW()
        RETURNING id, github_id, username, email, avatar_url, created_at, updated_at
        "#,
        github_user.id,
        github_user.login,
        github_user.email,
        github_user.avatar_url
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    session.insert(USER_SESSION_KEY, user.id).await.unwrap();

    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    Redirect::to(&format!("{}/dashboard", frontend_url)).into_response()
}

async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.unwrap();
    Redirect::to("/")
}