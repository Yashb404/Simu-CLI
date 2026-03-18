use serde::{Deserialize, Serialize};
use shared::{
    client::{fetch, HttpMethod},
    dto::UpdateDemoRequest,
    models::demo::{Demo, DemoSettings, Step, Theme},
};

fn normalize_base(base: &str) -> String {
    let trimmed = base.trim().trim_end_matches('/');
    let Ok(parsed) = web_sys::Url::new(trimmed) else {
        return trimmed.to_string();
    };

    let protocol = parsed.protocol();
    let host = parsed.hostname();
    let host = host.trim_end_matches('.');
    if host.is_empty() {
        return trimmed.to_string();
    }

    let port = parsed.port();
    if port.is_empty() {
        format!("{}//{}", protocol, host)
    } else {
        format!("{}//{}:{}", protocol, host, port)
    }
}

pub fn browser_origin() -> String {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_default()
}

fn derived_api_base_from_location() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();

    let protocol = location.protocol().ok()?;
    let hostname = location.hostname().ok()?;
    let hostname = hostname.trim_end_matches('.').to_string();
    let port = location.port().ok().unwrap_or_default();

    if hostname.is_empty() {
        return None;
    }

    let scheme = protocol.trim_end_matches(':');
    let target_port = if port == "8080" { "3001" } else { &port };

    if target_port.is_empty() {
        Some(format!("{scheme}://{hostname}"))
    } else {
        Some(format!("{scheme}://{hostname}:{target_port}"))
    }
}

pub fn api_base() -> String {
    if let Some(base) = option_env!("APP_API_BASE_URL") {
        return normalize_base(base);
    }
    derived_api_base_from_location()
        .or_else(|| {
            let origin = browser_origin();
            if origin.is_empty() {
                None
            } else {
                Some(origin)
            }
        })
        .unwrap_or_default()
}

fn api_url(path: &str) -> String {
    let base = api_base();
    if base.is_empty() {
        return path.to_string();
    }
    format!("{base}{path}")
}

fn build_query_path(path: &str, params: Vec<(&str, String)>) -> String {
    if params.is_empty() {
        return api_url(path);
    }

    let query = params
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");

    api_url(&format!("{path}?{query}"))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardProject {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDemo {
    pub id: String,
    pub title: String,
    pub slug: Option<String>,
    pub published: bool,
    pub version: i32,
    pub theme: Theme,
    pub settings: DemoSettings,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDemoDetail {
    pub id: String,
    pub title: String,
    pub slug: Option<String>,
    pub published: bool,
    pub version: i32,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub id: String,
    pub slug: String,
    pub version: i32,
    pub public_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSeriesPoint {
    pub bucket: String,
    pub event_type: String,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerCount {
    pub referrer: String,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelPoint {
    pub step_index: i32,
    pub total: i64,
}

#[derive(Debug, Serialize)]
struct CreateProjectRequest<'a> {
    name: &'a str,
    description: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct CreateDemoRequest<'a> {
    title: &'a str,
    project_id: Option<&'a str>,
}
pub fn login_url() -> String {
    api_url("/api/auth/github")
}

pub async fn list_projects() -> Result<Vec<DashboardProject>, String> {
    list_projects_with_paging(None, None).await
}

pub async fn list_projects_with_paging(
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<DashboardProject>, String> {
    let mut params = Vec::new();
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(offset) = offset {
        params.push(("offset", offset.to_string()));
    }

    let url = build_query_path("/api/me/projects", params);
    fetch(HttpMethod::Get, &url, None, true).await
}

pub async fn create_project(name: &str, description: Option<&str>) -> Result<DashboardProject, String> {
    let payload = CreateProjectRequest { name, description };
    let body = serde_json::to_string(&payload).map_err(|e| format!("serialize body: {e}"))?;
    fetch(HttpMethod::Post, &api_url("/api/projects"), Some(&body), true).await
}

pub async fn list_demos() -> Result<Vec<DashboardDemo>, String> {
    list_demos_with_filters(None, None, None, None).await
}

pub async fn list_demos_with_filters(
    limit: Option<u32>,
    offset: Option<u32>,
    project_id: Option<&str>,
    published: Option<bool>,
) -> Result<Vec<DashboardDemo>, String> {
    let mut params = Vec::new();
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(offset) = offset {
        params.push(("offset", offset.to_string()));
    }
    if let Some(project_id) = project_id {
        params.push(("project_id", project_id.to_string()));
    }
    if let Some(published) = published {
        params.push(("published", published.to_string()));
    }

    let url = build_query_path("/api/me/demos", params);
    fetch(HttpMethod::Get, &url, None, true).await
}
pub async fn get_demo(id: &str) -> Result<DashboardDemo, String> {
    fetch(HttpMethod::Get, &api_url(&format!("/api/demos/{id}")), None, true).await
}

pub async fn get_demo_detail(id: &str) -> Result<Demo, String> {
    fetch(HttpMethod::Get, &api_url(&format!("/api/demos/{id}")), None, true).await
}

pub async fn create_demo(title: &str, project_id: Option<&str>) -> Result<DashboardDemo, String> {
    let payload = CreateDemoRequest { title, project_id };
    let body = serde_json::to_string(&payload).map_err(|e| format!("serialize body: {e}"))?;
    fetch(HttpMethod::Post, &api_url("/api/demos"), Some(&body), true).await
}
pub async fn update_demo(id: &str, title: Option<&str>, slug: Option<&str>) -> Result<DashboardDemo, String> {
    update_demo_payload(
        id,
        &UpdateDemoRequest {
            title: title.map(ToString::to_string),
            slug: slug.map(ToString::to_string),
            theme: None,
            settings: None,
            steps: None,
        },
    )
    .await
}
pub async fn update_demo_payload(id: &str, payload: &UpdateDemoRequest) -> Result<DashboardDemo, String> {
    let body = serde_json::to_string(payload).map_err(|e| format!("serialize body: {e}"))?;
    fetch(HttpMethod::Patch, &api_url(&format!("/api/demos/{id}")), Some(&body), true).await
}

pub async fn delete_demo(id: &str) -> Result<(), String> {
    shared::client::send(
        HttpMethod::Delete,
        &api_url(&format!("/api/demos/{id}")),
        None,
        true,
    )
    .await
}

pub async fn publish_demo(id: &str) -> Result<PublishResponse, String> {
    fetch(
        HttpMethod::Post,
        &api_url(&format!("/api/demos/{id}/publish")),
        None,
        true,
    )
    .await
}

pub async fn get_analytics_series(id: &str) -> Result<Vec<AnalyticsSeriesPoint>, String> {
    get_analytics_series_with_days(id, None).await
}

pub async fn get_analytics_series_with_days(
    id: &str,
    days: Option<u32>,
) -> Result<Vec<AnalyticsSeriesPoint>, String> {
    let mut params = Vec::new();
    if let Some(days) = days {
        params.push(("days", days.to_string()));
    }

    let url = build_query_path(&format!("/api/demos/{id}/analytics"), params);
    fetch(HttpMethod::Get, &url, None, true).await
}

pub async fn get_analytics_referrers(id: &str) -> Result<Vec<ReferrerCount>, String> {
    get_analytics_referrers_with_limit(id, None).await
}

pub async fn get_analytics_referrers_with_limit(
    id: &str,
    limit: Option<u32>,
) -> Result<Vec<ReferrerCount>, String> {
    let mut params = Vec::new();
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }

    let url = build_query_path(&format!("/api/demos/{id}/analytics/referrers"), params);
    fetch(HttpMethod::Get, &url, None, true).await
}

pub async fn get_analytics_funnel(id: &str) -> Result<Vec<FunnelPoint>, String> {
    get_analytics_funnel_with_limit(id, None).await
}

pub async fn get_analytics_funnel_with_limit(
    id: &str,
    limit: Option<u32>,
) -> Result<Vec<FunnelPoint>, String> {
    let mut params = Vec::new();
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }

    let url = build_query_path(&format!("/api/demos/{id}/analytics/funnel"), params);
    fetch(HttpMethod::Get, &url, None, true).await
}
