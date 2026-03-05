use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::demo::{Theme, DemoSettings, Step};

// What the API accepts for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDemoRequest {
    pub title: String,
    pub project_id: Option<Uuid>,
}

// What the API accepts for updates (partial updates allowed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDemoRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub theme: Option<Theme>,
    pub settings: Option<DemoSettings>,
    pub steps: Option<Vec<Step>>,
}

// What the CDN-cached public endpoint returns (no internal IDs/owner info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicDemoResponse {
    pub id: Uuid,
    pub slug: Option<String>,
    pub version: i32,
    pub theme: Theme,
    pub settings: DemoSettings,
    pub steps: Vec<Step>,
}