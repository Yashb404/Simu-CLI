use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use web_sys::RequestCredentials;

const API_BASE: &str = "http://localhost:3001";

fn api_url(path: &str) -> String {
    format!("{API_BASE}{path}")
}

async fn send_json_builder<T: for<'de> Deserialize<'de>>(request: RequestBuilder) -> Result<T, String> {
    let response = request
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "".to_string());
        return Err(format!("request failed with status {status}: {body}"));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("invalid response payload: {e}"))
}

async fn send_json_request<T: for<'de> Deserialize<'de>>(request: Request) -> Result<T, String> {
    let response = request
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !response.ok() {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| "".to_string());
        return Err(format!("request failed with status {status}: {body}"));
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("invalid response payload: {e}"))
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

#[derive(Debug, Serialize)]
struct UpdateDemoRequest<'a> {
    title: Option<&'a str>,
    slug: Option<&'a str>,
}

pub fn login_url() -> String {
    api_url("/api/auth/github")
}

pub async fn list_projects() -> Result<Vec<DashboardProject>, String> {
    send_json_builder(Request::get(&api_url("/api/me/projects"))).await
}

pub async fn create_project(name: &str, description: Option<&str>) -> Result<DashboardProject, String> {
    let payload = CreateProjectRequest { name, description };
    send_json_request(
        Request::post(&api_url("/api/projects"))
            .credentials(RequestCredentials::Include)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&payload).map_err(|e| format!("serialize body: {e}"))?)
            .map_err(|e| format!("build request: {e}"))?,
    )
    .await
}

pub async fn list_demos() -> Result<Vec<DashboardDemo>, String> {
    send_json_builder(Request::get(&api_url("/api/me/demos"))).await
}

pub async fn get_demo(id: &str) -> Result<DashboardDemo, String> {
    send_json_builder(Request::get(&api_url(&format!("/api/demos/{id}")))).await
}

pub async fn create_demo(title: &str, project_id: Option<&str>) -> Result<DashboardDemo, String> {
    let payload = CreateDemoRequest { title, project_id };
    send_json_request(
        Request::post(&api_url("/api/demos"))
            .credentials(RequestCredentials::Include)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&payload).map_err(|e| format!("serialize body: {e}"))?)
            .map_err(|e| format!("build request: {e}"))?,
    )
    .await
}

pub async fn update_demo(id: &str, title: Option<&str>, slug: Option<&str>) -> Result<DashboardDemo, String> {
    let payload = UpdateDemoRequest { title, slug };
    send_json_request(
        Request::patch(&api_url(&format!("/api/demos/{id}")))
            .credentials(RequestCredentials::Include)
            .header("content-type", "application/json")
            .body(serde_json::to_string(&payload).map_err(|e| format!("serialize body: {e}"))?)
            .map_err(|e| format!("build request: {e}"))?,
    )
    .await
}

pub async fn publish_demo(id: &str) -> Result<PublishResponse, String> {
    send_json_builder(Request::post(&api_url(&format!("/api/demos/{id}/publish")))).await
}

pub async fn get_analytics_series(id: &str) -> Result<Vec<AnalyticsSeriesPoint>, String> {
    send_json_builder(Request::get(&api_url(&format!("/api/demos/{id}/analytics")))).await
}

pub async fn get_analytics_referrers(id: &str) -> Result<Vec<ReferrerCount>, String> {
    send_json_builder(Request::get(&api_url(&format!("/api/demos/{id}/analytics/referrers")))).await
}

pub async fn get_analytics_funnel(id: &str) -> Result<Vec<FunnelPoint>, String> {
    send_json_builder(Request::get(&api_url(&format!("/api/demos/{id}/analytics/funnel")))).await
}
