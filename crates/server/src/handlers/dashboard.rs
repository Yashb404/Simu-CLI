use axum::{Json, extract::State};
use serde::Serialize;

use crate::{auth::AuthUser, error::HandlerResult, state::AppState};
use shared::models::{demo::Demo, project::Project};

#[derive(Debug, Serialize)]
pub struct DashboardPayload {
    pub projects: Vec<Project>,
    pub demos: Vec<Demo>,
}

pub async fn get_my_dashboard(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<DashboardPayload>> {
    let projects = sqlx::query_as::<_, Project>(
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

    let demos = sqlx::query_as::<_, Demo>(
        r#"
        SELECT id, owner_id, project_id, slug, title, engine_mode, theme, settings, steps,
               published, version, created_at, updated_at
        FROM demos
        WHERE owner_id = $1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(DashboardPayload { projects, demos }))
}
