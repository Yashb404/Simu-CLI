use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use tower_sessions::Session;
use shared::models::user::User;
use crate::{state::AppState, config::Config};
use oauth2::{
    EndpointSet, EndpointNotSet
};

pub const USER_SESSION_KEY: &str = "user_id";

pub fn github_oauth_client(config: &Config) -> Result<BasicClient<
    EndpointSet,    // HasAuthUrl
    EndpointNotSet, // HasDeviceAuthUrl
    EndpointNotSet, // HasIntrospectionUrl
    EndpointNotSet, // HasRevocationUrl
    EndpointSet     // HasTokenUrl
>, anyhow::Error> {
    Ok(BasicClient::new(ClientId::new(config.github_client_id.clone()))
        .set_client_secret(ClientSecret::new(config.github_client_secret.clone()))
        .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?)
        .set_token_uri(TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?)
        .set_redirect_uri(RedirectUrl::new(config.github_redirect_uri())?))
}

// The Extractor we will use to protect routes
pub struct AuthUser(pub User);

// Axum 0.8 natively supports async trait methods!
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing session extension"))?;

        let user_id: uuid::Uuid = session
            .get(USER_SESSION_KEY)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Session error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Not logged in"))?;

        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, github_id, username, email, avatar_url, created_at, updated_at FROM users WHERE id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found"))?;

        Ok(AuthUser(user))
    }
}