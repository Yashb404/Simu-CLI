use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
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
pub struct AnalyticsEventRequest {
    pub demo_id: Uuid,
    pub event_type: String,
    pub step_index: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsWindowQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AnalyticsSeriesPoint {
    pub bucket: time::OffsetDateTime,
    pub event_type: String,
    pub total: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReferrerCount {
    pub referrer: String,
    pub total: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FunnelPoint {
    pub step_index: i32,
    pub total: i64,
}

fn is_valid_event_type(value: &str) -> bool {
    matches!(value, "view" | "interaction" | "completion")
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

pub async fn post_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AnalyticsEventRequest>,
) -> HandlerResult<StatusCode> {
    if !is_valid_event_type(&payload.event_type) {
        return Err(ApiError(AppError::Validation(
            "event_type must be one of: view, interaction, completion".to_string(),
        )));
    }

    let demo_exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM demos WHERE id = $1")
        .bind(payload.demo_id)
        .fetch_optional(&state.db)
        .await?;

    if demo_exists.is_none() {
        return Err(ApiError(AppError::NotFound));
    }

    let referrer = headers
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    sqlx::query(
        r#"
        INSERT INTO analytics_events (demo_id, event_type, step_index, referrer, user_agent)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(payload.demo_id)
    .bind(payload.event_type)
    .bind(payload.step_index)
    .bind(referrer)
    .bind(user_agent)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::ACCEPTED)
}

pub async fn get_demo_analytics(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<AnalyticsWindowQuery>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<Vec<AnalyticsSeriesPoint>>> {
    ensure_demo_owner(&state, id, user.id).await?;

    let days = query.days.unwrap_or(30).clamp(1, 365);

    let rows = sqlx::query_as::<_, AnalyticsSeriesPoint>(
        r#"
        SELECT date_trunc('day', created_at) AS bucket, event_type, COUNT(*)::bigint AS total
        FROM analytics_events
        WHERE demo_id = $1
          AND created_at >= NOW() - ($2 * INTERVAL '1 day')
        GROUP BY bucket, event_type
        ORDER BY bucket ASC
        "#,
    )
    .bind(id)
    .bind(days)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

pub async fn get_demo_referrers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<Vec<ReferrerCount>>> {
    ensure_demo_owner(&state, id, user.id).await?;

    let rows = sqlx::query_as::<_, ReferrerCount>(
        r#"
        SELECT COALESCE(referrer, 'direct') AS referrer, COUNT(*)::bigint AS total
        FROM analytics_events
        WHERE demo_id = $1
        GROUP BY referrer
        ORDER BY total DESC
        LIMIT 10
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

pub async fn get_demo_funnel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<Vec<FunnelPoint>>> {
    ensure_demo_owner(&state, id, user.id).await?;

    let rows = sqlx::query_as::<_, FunnelPoint>(
        r#"
        SELECT COALESCE(step_index, -1) AS step_index, COUNT(*)::bigint AS total
        FROM analytics_events
        WHERE demo_id = $1
          AND event_type = 'interaction'
        GROUP BY step_index
        ORDER BY step_index ASC
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

pub async fn export_demo_analytics_csv(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> HandlerResult<(StatusCode, String)> {
    ensure_demo_owner(&state, id, user.id).await?;

    let rows = sqlx::query_as::<_, AnalyticsSeriesPoint>(
        r#"
        SELECT date_trunc('day', created_at) AS bucket, event_type, COUNT(*)::bigint AS total
        FROM analytics_events
        WHERE demo_id = $1
        GROUP BY bucket, event_type
        ORDER BY bucket ASC
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let mut csv = String::from("bucket,event_type,total\n");
    for row in rows {
        csv.push_str(&format!("{},{},{}\n", row.bucket.date(), row.event_type, row.total));
    }

    Ok((StatusCode::OK, csv))
}
