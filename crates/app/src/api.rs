use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardProject {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDemo {
    pub id: String,
    pub title: String,
    pub published: bool,
}

pub async fn list_projects() -> Result<Vec<DashboardProject>, String> {
    // MVP placeholder: wire to /api/me/projects in next pass.
    Ok(vec![DashboardProject {
        id: "local-project".to_string(),
        name: "My CLI Demos".to_string(),
    }])
}

pub async fn list_demos() -> Result<Vec<DashboardDemo>, String> {
    // MVP placeholder: wire to /api/me/demos in next pass.
    Ok(vec![DashboardDemo {
        id: "local-demo".to_string(),
        title: "Welcome Demo".to_string(),
        published: false,
    }])
}
