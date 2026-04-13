use axum::{Json, extract::State};
use serde::Serialize;

use crate::{auth::AuthUser, error::HandlerResult, state::AppState};
use shared::models::{demo::Demo, project::Project};

#[derive(Debug, Serialize)]
pub struct DashboardPayload {
    pub projects: Vec<Project>,
    pub demos: Vec<Demo>,
}

/// Return the authenticated user's dashboard containing their projects and demos.
///
/// Queries the database for projects and demos where `owner_id` matches the authenticated user,
/// ordering each list by `updated_at` descending, and returns them as a JSON `DashboardPayload`.
///
/// # Examples
///
/// ```ignore
/// // Async context required (e.g., inside a tokio test or handler).
/// # async fn _example() -> anyhow::Result<()> {
/// let state = /* AppState value */; // provide application state with a DB connection
/// let auth_user = /* AuthUser value */; // authenticated user extractor
///
/// let response = get_my_dashboard(State(state), AuthUser(auth_user)).await?;
/// let dashboard = response.0; // DashboardPayload { projects, demos }
/// # Ok(())
/// # }
/// ```
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
