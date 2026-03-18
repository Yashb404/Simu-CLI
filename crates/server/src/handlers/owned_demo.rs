use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    auth::USER_SESSION_KEY,
    error::ApiError,
    state::AppState,
};
use shared::{
    error::AppError,
    models::demo::Demo,
};

const SELECT_OWNED_DEMO_SQL: &str = r#"
    SELECT id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
           published, version, created_at, updated_at
    FROM demos
    WHERE id = $1 AND owner_id = $2
"#;

pub struct OwnedDemo(pub Demo);

impl FromRequestParts<AppState> for OwnedDemo {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let Path(demo_id) = Path::<Uuid>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError(AppError::NotFound))?;

        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(ApiError(AppError::Internal))?;

        let owner_id: Uuid = session
            .get(USER_SESSION_KEY)
            .await
            .map_err(|_| ApiError(AppError::Internal))?
            .ok_or(ApiError(AppError::Unauthorized))?;

        let demo = sqlx::query_as::<_, Demo>(SELECT_OWNED_DEMO_SQL)
            .bind(demo_id)
            .bind(owner_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or(ApiError(AppError::NotFound))?;

        Ok(Self(demo))
    }
}
