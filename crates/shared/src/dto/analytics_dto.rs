use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::analytics::AnalyticsEventType;

#[cfg(feature = "backend")]
use validator::Validate;

#[cfg(feature = "backend")]
fn validate_step_index(value: i32) -> Result<(), validator::ValidationError> {
    if value >= 0 {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_step_index");
        err.message = Some("step_index must be greater than or equal to 0".into());
        Err(err)
    }
}

#[cfg_attr(feature = "backend", derive(Validate))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEventRequest {
    pub demo_id: Uuid,
    pub event_type: AnalyticsEventType,
    #[cfg_attr(feature = "backend", validate(custom(function = "validate_step_index")))]
    pub step_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsWindowQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsExportQuery {
    pub days: Option<i64>,
    pub limit: Option<i64>,
}

#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSeriesPoint {
    pub bucket: OffsetDateTime,
    pub event_type: AnalyticsEventType,
    pub total: i64,
}

#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerCount {
    pub referrer: String,
    pub total: i64,
}

#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelPoint {
    pub step_index: i32,
    pub total: i64,
}

#[cfg(all(test, feature = "backend"))]
mod tests {
    use super::*;

    #[test]
    fn analytics_event_request_rejects_negative_step_index() {
        let payload = AnalyticsEventRequest {
            demo_id: Uuid::new_v4(),
            event_type: AnalyticsEventType::Interaction,
            step_index: Some(-1),
        };

        let result = payload.validate();
        assert!(result.is_err(), "negative step index should fail validation");
    }

    #[test]
    fn analytics_event_request_accepts_none_or_non_negative_step_index() {
        let payload_with_none = AnalyticsEventRequest {
            demo_id: Uuid::new_v4(),
            event_type: AnalyticsEventType::View,
            step_index: None,
        };

        let payload_with_value = AnalyticsEventRequest {
            demo_id: Uuid::new_v4(),
            event_type: AnalyticsEventType::Completion,
            step_index: Some(3),
        };

        assert!(payload_with_none.validate().is_ok());
        assert!(payload_with_value.validate().is_ok());
    }
}
