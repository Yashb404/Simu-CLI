use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Database model for Project
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDb {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl ProjectDb {
    /// Convert database model to domain model
    pub fn to_domain(self) -> Project {
        Project {
            id: self.id,
            owner_id: self.owner_id,
            name: self.name,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}