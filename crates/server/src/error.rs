use crate::middleware::logging;
use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use shared::error::AppError;
use validator::ValidationErrors;

#[derive(Debug)]
pub struct ApiError(pub AppError);
pub type HandlerResult<T> = Result<T, ApiError>;

fn error_status_and_message(err: &AppError) -> (StatusCode, String) {
    match err {
        AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
        AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
        AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        AppError::RateLimited => (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        ),
        AppError::BadGateway(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
        AppError::Internal => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        ),
    }
}

// Tell Axum how to convert our domain errors into HTTP responses
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = error_status_and_message(&self.0);
        let body = match logging::current_request_id() {
            Some(request_id) => json!({ "error": message, "request_id": request_id }),
            None => json!({ "error": message }),
        };
        (status, Json(body)).into_response()
    }
}

/// Wraps an [`ApiError`] with the correlation ID pulled from request extensions,
/// so the `request_id` field appears in every error JSON body.
pub struct CorrelatedError {
    pub inner: ApiError,
    pub request_id: Option<String>,
}

impl IntoResponse for CorrelatedError {
    fn into_response(self) -> Response {
        let (status, message) = error_status_and_message(&self.inner.0);
        let body = match self.request_id {
            Some(id) => json!({ "error": message, "request_id": id }),
            None => json!({ "error": message }),
        };
        (status, Json(body)).into_response()
    }
}

/// Convenience: create a [`CorrelatedError`] by reading the request ID stored
/// in extensions by the logging middleware.
pub fn with_request_id(err: impl Into<ApiError>, req: &Request) -> CorrelatedError {
    let request_id = req.extensions().get::<String>().cloned();
    CorrelatedError {
        inner: err.into(),
        request_id,
    }
}

// Automatically wrap shared::AppError
impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError(err)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError(AppError::NotFound),
            _ => {
                // Log the real error for our internal telemetry, but return a generic 500 to the user
                tracing::error!("Database error: {:?}", err);
                ApiError(AppError::Internal)
            }
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("HTTP request failed: {:?}", err);

        if err.is_status() {
            if let Some(status) = err.status() {
                if status.is_client_error() {
                    return ApiError(AppError::BadGateway(format!(
                        "External service error: {}",
                        status
                    )));
                }
            }
        }

        ApiError(AppError::BadGateway(
            "External service unavailable".to_string(),
        ))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("JSON serialization error: {:?}", err);
        ApiError(AppError::Internal)
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(err: ValidationErrors) -> Self {
        tracing::warn!("Validation failed: {err}");
        ApiError(AppError::Validation(err.to_string()))
    }
}
