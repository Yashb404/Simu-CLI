use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, HandlerResult},
    state::AppState,
};
use shared::error::AppError;

#[derive(Debug, Deserialize)]
pub struct RecordCommonErrorRequest {
    pub demo_id: Uuid,
    pub command_text: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CommonErrorRow {
    pub command_text: String,
    pub count: i64,
}

async fn ensure_demo_owner(state: &AppState, demo_id: Uuid, owner_id: Uuid) -> HandlerResult<()> {
    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM demos WHERE id = $1 AND owner_id = $2")
        .bind(demo_id)
        .bind(owner_id)
        .fetch_optional(&state.db)
        .await?;

    if exists.is_none() {
        return Err(ApiError(AppError::NotFound));
    }

    Ok(())
}

pub async fn record_common_error(
    State(state): State<AppState>,
    Json(payload): Json<RecordCommonErrorRequest>,
) -> HandlerResult<StatusCode> {
    let command_text = payload.command_text.trim();
    if command_text.is_empty() {
        return Err(ApiError(AppError::Validation(
            "command_text cannot be empty".to_string(),
        )));
    }

    sqlx::query(
        r#"
        INSERT INTO common_errors (demo_id, command_text, count)
        VALUES ($1, $2, 1)
        ON CONFLICT (demo_id, command_text)
        DO UPDATE SET count = common_errors.count + 1, last_seen_at = NOW()
        "#,
    )
    .bind(payload.demo_id)
    .bind(command_text)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::ACCEPTED)
}

pub async fn get_common_errors(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<Vec<CommonErrorRow>>> {
    ensure_demo_owner(&state, id, user.id).await?;

    let rows = sqlx::query_as::<_, CommonErrorRow>(
        r#"
        SELECT command_text, count
        FROM common_errors
        WHERE demo_id = $1
        ORDER BY count DESC, last_seen_at DESC
        LIMIT 10
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}
