use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonError {
    pub id: Uuid,
    pub demo_id: Uuid,
    pub command_text: String,
    pub count: i64,
    pub first_seen_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
}
