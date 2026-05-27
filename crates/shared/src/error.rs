#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Bad gateway: {0}")]
    BadGateway(String),
    #[error("Internal error")]
    Internal,
}

impl AppError {
    /// Create a validation error from a message
    pub fn validation(msg: impl Into<String>) -> Self {
        AppError::Validation(msg.into())
    }

    /// Create a bad gateway error from a message
    pub fn bad_gateway(msg: impl Into<String>) -> Self {
        AppError::BadGateway(msg.into())
    }

    /// Stable machine-readable code for API responses.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound => "NOT_FOUND",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::RateLimited => "RATE_LIMITED",
            AppError::BadGateway(_) => "BAD_GATEWAY",
            AppError::Internal => "INTERNAL_ERROR",
        }
    }
}
