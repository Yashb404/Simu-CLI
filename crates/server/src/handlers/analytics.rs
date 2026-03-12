use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
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

const DEFAULT_EXPORT_DAYS: i64 = 30;
const MAX_EXPORT_DAYS: i64 = 365;
const DEFAULT_EXPORT_LIMIT: i64 = 2000;
const MAX_EXPORT_LIMIT: i64 = 5000;
const DEFAULT_REFERRER_LIMIT: i64 = 10;
const MAX_REFERRER_LIMIT: i64 = 100;
const DEFAULT_FUNNEL_LIMIT: i64 = 100;
const MAX_FUNNEL_LIMIT: i64 = 500;

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

#[derive(Debug, Deserialize)]
pub struct AnalyticsExportQuery {
    pub days: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsReferrerQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsFunnelQuery {
    pub limit: Option<i64>,
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

fn sanitize_export_bounds(days: Option<i64>, limit: Option<i64>) -> (i64, i64) {
    let days = days.unwrap_or(DEFAULT_EXPORT_DAYS).clamp(1, MAX_EXPORT_DAYS);
    let limit = limit
        .unwrap_or(DEFAULT_EXPORT_LIMIT)
        .clamp(1, MAX_EXPORT_LIMIT);
    (days, limit)
}

fn sanitize_referrer_limit(limit: Option<i64>) -> i64 {
    limit
        .unwrap_or(DEFAULT_REFERRER_LIMIT)
        .clamp(1, MAX_REFERRER_LIMIT)
}

fn sanitize_funnel_limit(limit: Option<i64>) -> i64 {
    limit
        .unwrap_or(DEFAULT_FUNNEL_LIMIT)
        .clamp(1, MAX_FUNNEL_LIMIT)
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
    Query(query): Query<AnalyticsReferrerQuery>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<Vec<ReferrerCount>>> {
    ensure_demo_owner(&state, id, user.id).await?;

    let limit = sanitize_referrer_limit(query.limit);

    let rows = sqlx::query_as::<_, ReferrerCount>(
        r#"
        SELECT COALESCE(referrer, 'direct') AS referrer, COUNT(*)::bigint AS total
        FROM analytics_events
        WHERE demo_id = $1
        GROUP BY referrer
        ORDER BY total DESC
        LIMIT $2
        "#,
    )
    .bind(id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

pub async fn get_demo_funnel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<AnalyticsFunnelQuery>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<Vec<FunnelPoint>>> {
    ensure_demo_owner(&state, id, user.id).await?;

    let limit = sanitize_funnel_limit(query.limit);

    let rows = sqlx::query_as::<_, FunnelPoint>(
        r#"
        SELECT COALESCE(step_index, -1) AS step_index, COUNT(*)::bigint AS total
        FROM analytics_events
        WHERE demo_id = $1
          AND event_type = 'interaction'
        GROUP BY step_index
        ORDER BY step_index ASC
        LIMIT $2
        "#,
    )
    .bind(id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

pub async fn export_demo_analytics_csv(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<AnalyticsExportQuery>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Response> {
    ensure_demo_owner(&state, id, user.id).await?;

    let (days, limit) = sanitize_export_bounds(query.days, query.limit);

    let rows = sqlx::query_as::<_, AnalyticsSeriesPoint>(
        r#"
        SELECT date_trunc('day', created_at) AS bucket, event_type, COUNT(*)::bigint AS total
        FROM analytics_events
        WHERE demo_id = $1
          AND created_at >= NOW() - ($2 * INTERVAL '1 day')
        GROUP BY bucket, event_type
        ORDER BY bucket ASC
        LIMIT $3
        "#,
    )
    .bind(id)
    .bind(days)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let mut csv = String::from("bucket,event_type,total\n");
    for row in rows {
        csv.push_str(&format!("{},{},{}\n", row.bucket.date(), row.event_type, row.total));
    }

    let mut response = (StatusCode::OK, csv).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=analytics.csv"),
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_export_bounds_applies_defaults_and_limits() {
        assert_eq!(sanitize_export_bounds(None, None), (30, 2000));
        assert_eq!(sanitize_export_bounds(Some(7), Some(500)), (7, 500));
        assert_eq!(sanitize_export_bounds(Some(0), Some(0)), (1, 1));
        assert_eq!(sanitize_export_bounds(Some(900), Some(999999)), (365, 5000));
    }

    #[test]
    fn sanitize_referrer_limit_applies_bounds() {
        assert_eq!(sanitize_referrer_limit(None), 10);
        assert_eq!(sanitize_referrer_limit(Some(1)), 1);
        assert_eq!(sanitize_referrer_limit(Some(999)), 100);
    }

    #[test]
    fn sanitize_funnel_limit_applies_bounds() {
        assert_eq!(sanitize_funnel_limit(None), 100);
        assert_eq!(sanitize_funnel_limit(Some(1)), 1);
        assert_eq!(sanitize_funnel_limit(Some(9999)), 500);
    }
}
