use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use validator::Validate;

use crate::{
    error::HandlerResult,
    handlers::owned_demo::OwnedDemo,
    state::AppState,
};
use shared::{
    dto::{CommonErrorRow, RecordCommonErrorRequest},
};

pub async fn record_common_error(
    State(state): State<AppState>,
    Json(payload): Json<RecordCommonErrorRequest>,
) -> HandlerResult<StatusCode> {
    payload.validate()?;
    let command_text = payload.command_text.trim().to_string();

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
    OwnedDemo(demo): OwnedDemo,
) -> HandlerResult<Json<Vec<CommonErrorRow>>> {
    let rows = sqlx::query_as::<_, CommonErrorRow>(
        r#"
        SELECT command_text, count
        FROM common_errors
        WHERE demo_id = $1
        ORDER BY count DESC, last_seen_at DESC
        LIMIT 10
        "#,
    )
    .bind(demo.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}
