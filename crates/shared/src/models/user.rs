use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User{
    pub id: Uuid,
    pub github_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}