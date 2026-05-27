use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standardized API error response shared between frontend and backend.
///
/// Keep this small and stable — frontends should rely on this shape when
/// rendering error messages or surfacing request IDs for support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// Human-friendly message suitable for displaying to end users.
    pub error: String,

    /// Optional machine-friendly error code, e.g. `NOT_FOUND`, `VALIDATION_ERROR`.
    pub error_code: Option<String>,

    /// Optional correlation id for debugging/support. Often provided via `X-Request-Id` header.
    pub request_id: Option<String>,

    /// Optional structured details; commonly used for validation field errors.
    pub details: Option<Value>,
}
