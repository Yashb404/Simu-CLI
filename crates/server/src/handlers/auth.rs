use axum::{
    extract::{Query, State},
    http::HeaderMap,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use oauth2::{AuthorizationCode, CsrfToken, RedirectUrl, Scope, TokenResponse};
use serde::Deserialize;
use tower_sessions::Session;
use crate::{auth::{github_oauth_client, USER_SESSION_KEY}, state::AppState, error::ApiError};
use shared::{models::user::User, error::AppError};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/github", get(github_login))
        .route("/github/callback", get(github_callback))
        .route("/logout", post(logout))
}

fn request_origin(host: Option<&str>, headers: &HeaderMap, fallback_origin: &str) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = host.unwrap_or_else(|| {
        fallback_origin
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
    });
    format!("{scheme}://{host}")
}

async fn github_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    session: Session,
) -> Result<Redirect, Response> {
    let host = headers.get("host").and_then(|value| value.to_str().ok());
    let redirect_uri = format!(
        "{}/api/auth/github/callback",
        request_origin(host, &headers, &state.config.api_url)
    );

    let client = github_oauth_client(&state.config)
        .map_err(|e| {
            tracing::error!("Failed to create GitHub OAuth client: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error").into_response()
        })?
        .set_redirect_uri(RedirectUrl::new(redirect_uri.clone()).map_err(|e| {
            tracing::error!("Invalid OAuth redirect URI {redirect_uri}: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error").into_response()
        })?);

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

    session.insert("oauth_redirect_uri", redirect_uri).await
        .map_err(|e| {
            tracing::error!("Failed to store OAuth redirect URI in session: {:?}", e);
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
    headers: HeaderMap,
    session: Session,
    Query(query): Query<AuthRequest>,
) -> Result<Redirect, Response> {
    let stored_csrf: Option<String> = session.get("csrf_token").await
        .map_err(|e| {
            tracing::error!("Failed to retrieve CSRF token from session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;
    
    if stored_csrf != Some(query.state.clone()) {
        tracing::warn!("CSRF token mismatch during GitHub callback. Session likely changed host/origin between login and callback.");
        return Err((StatusCode::UNAUTHORIZED, "CSRF token mismatch").into_response());
    }

    if let Err(err) = session.remove::<String>("csrf_token").await {
        tracing::warn!("Failed to clear CSRF token from session: {err:?}");
    }

    let redirect_uri: Option<String> = session.get("oauth_redirect_uri").await
        .map_err(|e| {
            tracing::error!("Failed to retrieve OAuth redirect URI from session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;

    if let Err(err) = session.remove::<String>("oauth_redirect_uri").await {
        tracing::warn!("Failed to clear OAuth redirect URI from session: {err:?}");
    }

    let redirect_uri = redirect_uri.unwrap_or_else(|| state.config.github_redirect_uri());

    let client = github_oauth_client(&state.config)
        .map_err(|e| {
            tracing::error!("Failed to create GitHub OAuth client: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error").into_response()
        })?
        .set_redirect_uri(RedirectUrl::new(redirect_uri.clone()).map_err(|e| {
            tracing::error!("Invalid OAuth redirect URI {redirect_uri}: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error").into_response()
        })?);

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
    let user = sqlx::query_as::<_, User>(
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
    )
    .bind(github_user.id)
    .bind(github_user.login)
    .bind(github_user.email)
    .bind(github_user.avatar_url)
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

    let frontend_host = headers.get("host").and_then(|value| value.to_str().ok());
    let frontend_origin = request_origin(frontend_host, &headers, &state.config.frontend_url);
    Ok(Redirect::to(&format!("{}/dashboard", frontend_origin)))
}

async fn logout(session: Session) -> Result<Redirect, StatusCode> {
    session.delete().await
        .map_err(|e| {
            tracing::error!("Failed to delete session: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Redirect::to("/"))
}