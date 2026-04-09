use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    error::{ApiError, HandlerResult},
    handlers::sanitize_pagination,
    state::AppState,
};
use shared::{
    dto::{CreateProjectRequest, UpdateProjectRequest},
    error::AppError,
    models::project::Project,
};

#[derive(Debug, Deserialize)]
pub struct ListMyProjectsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_project(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateProjectRequest>,
) -> HandlerResult<(StatusCode, Json<Project>)> {
    payload.validate()?;

    let row = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (owner_id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id, owner_id, name, description, created_at, updated_at
        "#,
    )
    .bind(user.id)
    .bind(payload.name)
    .bind(payload.description)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn list_my_projects(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<ListMyProjectsQuery>,
) -> HandlerResult<Json<Vec<Project>>> {
    let (limit, offset) = sanitize_pagination(query.limit, query.offset);

    let rows = sqlx::query_as::<_, Project>(
        r#"
        SELECT id, owner_id, name, description, created_at, updated_at
        FROM projects
        WHERE owner_id = $1
        ORDER BY updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user.id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

pub async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
    Json(payload): Json<UpdateProjectRequest>,
) -> HandlerResult<Json<Project>> {
    payload.validate()?;

    let name = payload.name;
    let description = payload.description;

    let updated = sqlx::query_as::<_, Project>(
        r#"
        UPDATE projects
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            updated_at = NOW()
        WHERE id = $3 AND owner_id = $4
        RETURNING id, owner_id, name, description, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;

    let updated = updated.ok_or(ApiError(AppError::NotFound))?;

    Ok(Json(updated))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> HandlerResult<StatusCode> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError(AppError::NotFound));
    }

    Ok(StatusCode::NO_CONTENT)
}
