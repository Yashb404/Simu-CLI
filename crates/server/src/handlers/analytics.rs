use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
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
    dto::{
        AnalyticsEventRequest, AnalyticsExportQuery, AnalyticsSeriesPoint,
        AnalyticsWindowQuery, FunnelPoint, ReferrerCount,
    },
    error::AppError,
    models::analytics::AnalyticsEventType,
};

const DEFAULT_EXPORT_DAYS: i64 = 30;
const MAX_EXPORT_DAYS: i64 = 365;
const DEFAULT_EXPORT_LIMIT: i64 = 2000;
const MAX_EXPORT_LIMIT: i64 = 5000;

fn sanitize_export_bounds(days: Option<i64>, limit: Option<i64>) -> (i64, i64) {
    let days = days.unwrap_or(DEFAULT_EXPORT_DAYS).clamp(1, MAX_EXPORT_DAYS);
    let limit = limit
        .unwrap_or(DEFAULT_EXPORT_LIMIT)
        .clamp(1, MAX_EXPORT_LIMIT);
    (days, limit)
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
    payload.validate()?;

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
                    AND event_type = $2
        GROUP BY step_index
        ORDER BY step_index ASC
        "#,
    )
    .bind(id)
        .bind(AnalyticsEventType::Interaction)
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
        csv.push_str(&format!(
            "{},{},{}\n",
            row.bucket.date(),
            row.event_type.as_str(),
            row.total
        ));
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
}
