use crate::{
    auth::{USER_SESSION_KEY, github_oauth_client},
    error::ApiError,
    state::AppState,
};
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use oauth2::{AuthorizationCode, CsrfToken, RedirectUrl, Scope, TokenResponse};
use serde::Deserialize;
use shared::{error::AppError, models::user::User};
use tower_sessions::Session;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/github", get(github_login))
        .route("/github/callback", get(github_callback))
        .route("/logout", post(logout))
}

async fn github_login(
    State(state): State<AppState>,
    session: Session,
) -> Result<Response, Response> {
    let redirect_uri = state.config.github_redirect_uri();

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
    session
        .insert("csrf_token", csrf_token.secret().clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to store CSRF token in session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;

    session
        .insert("oauth_redirect_uri", redirect_uri)
        .await
        .map_err(|e| {
            tracing::error!("Failed to store OAuth redirect URI in session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;

    let destination = auth_url.to_string();

    let body = format!(
        r#"<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Redirecting to GitHub...</title>
        <meta http-equiv="refresh" content="0;url={destination}" />
        <style>
            body {{
                margin: 0;
                min-height: 100vh;
                display: grid;
                place-items: center;
                font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
                background: #fff;
                color: #111;
            }}
            main {{
                border: 2px solid #111;
                padding: 20px;
                max-width: 520px;
                width: calc(100% - 32px);
            }}
            a {{ color: inherit; font-weight: 700; }}
        </style>
    </head>
    <body>
        <main>
            <h1>Redirecting to GitHub...</h1>
            <p>If you are not redirected automatically, continue here:</p>
            <p><a href="{destination}">Login with GitHub</a></p>
        </main>
        <script>
            window.location.replace({destination:?});
        </script>
    </body>
</html>"#
    );

    let mut response = Html(body).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    Ok(response)
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
    let stored_csrf: Option<String> = session.get("csrf_token").await.map_err(|e| {
        tracing::error!("Failed to retrieve CSRF token from session: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
    })?;

    if stored_csrf != Some(query.state.clone()) {
        tracing::warn!(
            "CSRF token mismatch during GitHub callback. Session likely changed host/origin between login and callback."
        );
        return Err((StatusCode::UNAUTHORIZED, "CSRF token mismatch").into_response());
    }

    if let Err(err) = session.remove::<String>("csrf_token").await {
        tracing::warn!("Failed to clear CSRF token from session: {err:?}");
    }

    let redirect_uri: Option<String> = session.get("oauth_redirect_uri").await.map_err(|e| {
        tracing::error!(
            "Failed to retrieve OAuth redirect URI from session: {:?}",
            e
        );
        (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
    })?;

    if let Err(err) = session.remove::<String>("oauth_redirect_uri").await {
        tracing::warn!("Failed to clear OAuth redirect URI from session: {err:?}");
    }

    let redirect_uri = redirect_uri.unwrap_or_else(|| state.config.github_redirect_uri());

    // Validate the redirect URI against the configured allow-list
    if redirect_uri != state.config.github_redirect_uri() {
        tracing::warn!("Unexpected OAuth redirect URI in session: {redirect_uri}");
        return Err((StatusCode::BAD_REQUEST, "Invalid redirect URI").into_response());
    }

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
            ApiError(AppError::BadGateway(
                "Failed to authenticate with GitHub".to_string(),
            ))
            .into_response()
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
            ApiError(AppError::BadGateway(
                "Failed to fetch user profile".to_string(),
            ))
            .into_response()
        })?
        .json()
        .await
        .map_err(|e| {
            tracing::error!("Failed to deserialize GitHub user profile: {:?}", e);
            ApiError(AppError::BadGateway(
                "Invalid user profile response".to_string(),
            ))
            .into_response()
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
    session
        .insert(USER_SESSION_KEY, user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to store user ID in session: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Session error").into_response()
        })?;

    tracing::info!("User {} authenticated successfully", user.username);

    // Use relative redirects so callback stays on the same host that initiated login.
    Ok(Redirect::to("/dashboard"))
}

async fn logout(State(_state): State<AppState>, session: Session) -> Result<Redirect, StatusCode> {
    session.delete().await.map_err(|e| {
        tracing::error!("Failed to delete session: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::to("/"))
}
