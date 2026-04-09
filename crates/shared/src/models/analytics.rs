use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[cfg_attr(feature = "backend", derive(sqlx::Type))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "backend", sqlx(rename_all = "snake_case", type_name = "text"))]
pub enum AnalyticsEventType {
    View,
    Interaction,
    Completion,
}

impl AnalyticsEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Interaction => "interaction",
            Self::Completion => "completion",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: Uuid,
    pub demo_id: Uuid,
    pub event_type: AnalyticsEventType,
    pub step_index: Option<i32>,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}
