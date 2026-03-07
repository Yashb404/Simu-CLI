use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    error::{ApiError, HandlerResult},
    state::AppState,
};
use shared::{
    dto::{CreateProjectRequest, UpdateProjectRequest},
    error::AppError,
    models::project::{Project, ProjectDb},
};

pub async fn create_project(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateProjectRequest>,
) -> HandlerResult<(StatusCode, Json<Project>)> {
    payload.validate()?;

    let row = sqlx::query_as::<_, ProjectDb>(
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

    Ok((StatusCode::CREATED, Json(row.to_domain())))
}

pub async fn list_my_projects(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<Vec<Project>>> {
    let rows = sqlx::query_as::<_, ProjectDb>(
        r#"
        SELECT id, owner_id, name, description, created_at, updated_at
        FROM projects
        WHERE owner_id = $1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows.into_iter().map(ProjectDb::to_domain).collect()))
}

pub async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
    Json(payload): Json<UpdateProjectRequest>,
) -> HandlerResult<Json<Project>> {
    payload.validate()?;

    let existing = sqlx::query_as::<_, ProjectDb>(
        r#"
        SELECT id, owner_id, name, description, created_at, updated_at
        FROM projects
        WHERE id = $1 AND owner_id = $2
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError(AppError::NotFound))?;

    let name = payload.name.unwrap_or(existing.name);
    let description = payload.description.or(existing.description);

    let updated = sqlx::query_as::<_, ProjectDb>(
        r#"
        UPDATE projects
        SET name = $1,
            description = $2,
            updated_at = NOW()
        WHERE id = $3 AND owner_id = $4
        RETURNING id, owner_id, name, description, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(id)
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated.to_domain()))
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
