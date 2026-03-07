use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;
use tower_sessions::Session;
use crate::{auth::{github_oauth_client, USER_SESSION_KEY}, state::AppState, error::ApiError};
use shared::{models::user::User, error::AppError};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/github", get(github_login))
        .route("/github/callback", get(github_callback))
        .route("/logout", get(logout))
}

async fn github_login(State(state): State<AppState>, session: Session) -> Result<Redirect, Response> {
    let client = github_oauth_client(&state.config)
        .map_err(|e| {
            tracing::error!("Failed to create GitHub OAuth client: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error").into_response()
        })?;

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // Store CSRF token
    session.insert("csrf_token", csrf_token.secret().clone()).await
        .map_err(|e| {
            tracing::error!("Failed to store CSRF token in session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;

    Ok(Redirect::to(auth_url.as_ref()))
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
) -> Result<Redirect, Response> {
    let stored_csrf: Option<String> = session.get("csrf_token").await
        .map_err(|e| {
            tracing::error!("Failed to retrieve CSRF token from session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;
    
    if stored_csrf != Some(query.state) {
        tracing::warn!("CSRF token mismatch during GitHub callback");
        return Err((StatusCode::UNAUTHORIZED, "CSRF token mismatch").into_response());
    }

    let client = github_oauth_client(&state.config)
        .map_err(|e| {
            tracing::error!("Failed to create GitHub OAuth client: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error").into_response()
        })?;

    let reqwest_client = reqwest::Client::new();
    
    // Exchange authorization code for access token
    let token = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&reqwest_client)
        .await
        .map_err(|e| {
            tracing::error!("Failed to exchange OAuth code: {:?}", e);
            ApiError(AppError::BadGateway("Failed to authenticate with GitHub".to_string())).into_response()
        })?;

    // Fetch user profile from GitHub
    let github_user: GithubUser = reqwest_client
        .get("https://api.github.com/user")
        .bearer_auth(token.access_token().secret())
        .header("User-Agent", "cli-demo-studio")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch GitHub user profile: {:?}", e);
            ApiError(AppError::BadGateway("Failed to fetch user profile".to_string())).into_response()
        })?
        .json()
        .await
        .map_err(|e| {
            tracing::error!("Failed to deserialize GitHub user profile: {:?}", e);
            ApiError(AppError::BadGateway("Invalid user profile response".to_string())).into_response()
        })?;

    // Upsert user in database
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
    .map_err(|e| {
        tracing::error!("Failed to create/update user in database: {:?}", e);
        ApiError(AppError::Internal).into_response()
    })?;

    // Store user ID in session
    session.insert(USER_SESSION_KEY, user.id).await
        .map_err(|e| {
            tracing::error!("Failed to store user ID in session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;

    tracing::info!("User {} authenticated successfully", user.username);

    let frontend_url = &state.config.frontend_url;
    Ok(Redirect::to(&format!("{}/dashboard", frontend_url)))
}

async fn logout(session: Session) -> Result<Redirect, StatusCode> {
    session.delete().await
        .map_err(|e| {
            tracing::error!("Failed to delete session: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Redirect::to("/"))
}