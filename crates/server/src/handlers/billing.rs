use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    auth::AuthUser,
    error::{ApiError, HandlerResult},
    services,
    state::AppState,
};
use shared::error::AppError;

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub plan_code: String,
}

#[derive(Debug, Serialize)]
pub struct BillingStatusResponse {
    pub plan_code: String,
    pub max_demos: i32,
    pub max_monthly_views: i64,
    pub status: String,
}

#[derive(Debug, FromRow)]
struct SubscriptionRow {
    plan_code: String,
    status: String,
}

pub async fn get_billing_status(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> HandlerResult<Json<BillingStatusResponse>> {
    let row = sqlx::query_as::<_, SubscriptionRow>(
        r#"
        SELECT p.code AS plan_code, s.status
        FROM subscriptions s
        JOIN plans p ON p.id = s.plan_id
        WHERE s.user_id = $1
        "#,
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;

    let (plan_code, status) = row
        .map(|r| (r.plan_code, r.status))
        .unwrap_or_else(|| ("free".to_string(), "active".to_string()));

    let limits = services::billing::limits_for_plan(&plan_code);

    Ok(Json(BillingStatusResponse {
        plan_code: limits.code,
        max_demos: limits.max_demos,
        max_monthly_views: limits.max_monthly_views,
        status,
    }))
}

pub async fn subscribe(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<SubscribeRequest>,
) -> HandlerResult<StatusCode> {
    let plan_code = payload.plan_code.trim().to_ascii_lowercase();
    if !matches!(plan_code.as_str(), "free" | "pro") {
        return Err(ApiError(AppError::Validation(
            "plan_code must be free or pro".to_string(),
        )));
    }

    let plan_id: Option<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM plans WHERE code = $1")
        .bind(&plan_code)
        .fetch_optional(&state.db)
        .await?;

    let Some(plan_id) = plan_id else {
        return Err(ApiError(AppError::Validation("invalid plan".to_string())));
    };

    sqlx::query(
        r#"
        INSERT INTO subscriptions (user_id, plan_id, status)
        VALUES ($1, $2, 'active')
        ON CONFLICT (user_id)
        DO UPDATE SET plan_id = EXCLUDED.plan_id, status = 'active', updated_at = NOW()
        "#,
    )
    .bind(user.id)
    .bind(plan_id)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
