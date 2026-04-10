use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use validator::Validate;

#[cfg(feature = "backend")]
fn validate_command_text(value: &str) -> Result<(), validator::ValidationError> {
    if !value.trim().is_empty() {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("empty_command_text");
        err.message = Some("command_text cannot be empty".into());
        Err(err)
    }
}

#[cfg_attr(feature = "backend", derive(Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordCommonErrorRequest {
    pub demo_id: Uuid,
    #[cfg_attr(feature = "backend", validate(length(max = 500)))]
    #[cfg_attr(
        feature = "backend",
        validate(custom(function = "validate_command_text"))
    )]
    pub command_text: String,
}

#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonErrorRow {
    pub command_text: String,
    pub count: i64,
}

#[cfg(all(test, feature = "backend"))]
mod tests {
    use super::*;

    #[test]
    fn record_common_error_request_rejects_blank_command_text() {
        let payload = RecordCommonErrorRequest {
            demo_id: Uuid::new_v4(),
            command_text: "   ".to_string(),
        };

        let result = payload.validate();
        assert!(result.is_err(), "blank command_text should fail validation");
    }
}
