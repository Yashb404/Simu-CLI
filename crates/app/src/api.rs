use serde::{Deserialize, Serialize};
use shared::{
    client::{ClientError, HttpMethod, fetch, fetch_typed, send},
    dto::PublicDemoResponse,
    dto::UpdateDemoRequest,
    models::demo::{Demo, DemoSettings, Step, Theme},
};
use time::OffsetDateTime;
use uuid::Uuid;
use web_sys::js_sys;

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

    if port.is_empty() {
        Some(format!("{scheme}://{hostname}"))
    } else {
        Some(format!("{scheme}://{hostname}:{port}"))
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

fn encode_query_value(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect()
}

/// Appends URL-encoded query parameters to an API path.
///
/// If `params` is empty this returns the same value as `api_url(path)`.
/// Only parameter values are percent-encoded; keys are used as provided.
/// The resulting string is the API URL for `path` with the encoded query string appended (e.g. `"/api/x?k=v&k2=v2"`).
///
/// # Examples
///
/// ```
/// let p = vec![("q", "hello world".to_string()), ("flag", "1".to_string())];
/// let out = build_query_path("/api/search", p);
/// assert!(out.contains("/api/search?"));
/// assert!(out.contains("q=hello%20world"));
/// assert!(out.contains("flag=1"));
/// ```
fn build_query_path(path: &str, params: Vec<(&str, String)>) -> String {
    if params.is_empty() {
        return api_url(path);
    }

    let query = params
        .into_iter()
        .map(|(key, value)| {
            // Percent-encode the value portion only
            let encoded = encode_query_value(&value);
            format!("{key}={encoded}")
        })
        .collect::<Vec<_>>()
        .join("&");

    api_url(&format!("{path}?{query}"))
}

/// Converts a string into a URL-friendly path segment (slug).
///
/// The returned slug is trimmed, lowercased, has every non-ASCII-alphanumeric
/// character replaced with `-`, collapses consecutive dashes into a single
/// dash, and has leading/trailing dashes removed.
///
/// # Examples
///
/// ```
/// let s = slugify_segment(" Hello, World! ");
/// assert_eq!(s, "hello-world");
/// ```
pub fn slugify_segment(value: &str) -> String {
    let mut slug = value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();

    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    slug.trim_matches('-').to_string()
}

/// Builds a namespaced demo path for a user, optionally within a project and with an optional trailing suffix.
///
/// The `project_slug` when provided produces a path under `/user/projects/{project_slug}/demos/{demo_id}`; when omitted the path is `/user/demos/{demo_id}`. Any `suffix` provided will be appended once, with leading/trailing slashes trimmed.
///
/// # Examples
///
/// ```
/// let p1 = namespaced_demo_path("Alice", "123", None, None);
/// assert_eq!(p1, "/alice/demos/123");
///
/// let p2 = namespaced_demo_path("Bob", "abc", Some("My Project"), Some("/view/"));
/// assert_eq!(p2, "/bob/projects/my-project/demos/abc/view");
/// ```
pub fn namespaced_demo_path(
    username: &str,
    demo_id: &str,
    project_slug: Option<&str>,
    suffix: Option<&str>,
) -> String {
    let user = slugify_segment(username);
    let mut base = if let Some(slug) = project_slug {
        format!("/{user}/projects/{}/demos/{demo_id}", slugify_segment(slug))
    } else {
        format!("/{user}/demos/{demo_id}")
    };

    if let Some(suffix) = suffix {
        let trimmed = suffix.trim_matches('/');
        if !trimmed.is_empty() {
            base.push('/');
            base.push_str(trimmed);
        }
    }

    base
}

/// Builds a namespaced project URL path for a user's project.
///
/// The returned string has the form `/username/projects/project-slug` where `username` and `project_slug` are slugified (lowercased, non-alphanumeric characters replaced by `-`, repeated `-` collapsed, and leading/trailing `-` removed).
///
/// # Examples
///
/// ```
/// let p = namespaced_project_path("Alice Smith", "My Project!");
/// assert_eq!(p, "/alice-smith/projects/my-project");
/// ```
pub fn namespaced_project_path(username: &str, project_slug: &str) -> String {
    format!(
        "/{}/projects/{}",
        slugify_segment(username),
        slugify_segment(project_slug)
    )
}

/// Path for the dashboard home.
///
/// # Examples
///
/// ```
/// let p = dashboard_home_path();
/// assert_eq!(p, "/dashboard");
/// ```
pub fn dashboard_home_path() -> &'static str {
    "/dashboard"
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
    pub project_id: Option<String>,
    pub title: String,
    pub slug: Option<String>,
    pub published: bool,
    pub version: i32,
    pub theme: Theme,
    pub settings: DemoSettings,
    pub steps: Vec<Step>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub projects: Vec<DashboardProject>,
    pub demos: Vec<DashboardDemo>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

pub fn logout_url() -> String {
    api_url("/api/auth/logout")
}

pub async fn get_current_user() -> Result<CurrentUser, String> {
    let ts = js_sys::Date::now() as u64;
    let url = api_url(&format!("/api/me?ts={ts}"));
    fetch(HttpMethod::Get, &url, None, true).await
}

pub async fn logout() -> Result<(), String> {
    send(HttpMethod::Post, &logout_url(), None, true).await
}

/// Fetches the current user's dashboard projects.
///
/// Returns `Ok` with a `Vec<DashboardProject>` on success, `Err` with an error message otherwise.
///
/// # Examples
///
/// ```no_run
/// # async fn run() {
/// let res = list_projects().await;
/// match res {
///     Ok(projects) => println!("found {} projects", projects.len()),
///     Err(err) => eprintln!("list_projects error: {}", err),
/// }
/// # }
/// ```
pub async fn list_projects() -> Result<Vec<DashboardProject>, String> {
    list_projects_with_paging(None, None).await
}

/// Fetches the authenticated user's dashboard snapshot containing projects and demos.
///
/// # Returns
///
/// `Ok(DashboardSnapshot)` with projects and demos on success, `Err(ClientError)` on failure.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), crate::api::ClientError> {
/// let snapshot = crate::api::get_dashboard_snapshot().await?;
/// // snapshot.projects and snapshot.demos are available
/// assert!(snapshot.projects.len() >= 0);
/// # Ok(()) }
/// ```
pub async fn get_dashboard_snapshot() -> Result<DashboardSnapshot, ClientError> {
    fetch_typed(HttpMethod::Get, &api_url("/api/me/dashboard"), None, true).await
}

/// Lists dashboard projects, optionally limited and offset for paging.
///
/// If `limit` is provided, the result set will contain at most that many projects.
/// If `offset` is provided, the result set will start at that zero-based offset.
///
/// # Returns
///
/// `Ok(Vec<DashboardProject>)` with the retrieved projects on success, `Err(String)` with an error message on failure.
///
/// # Examples
///
/// ```
/// use futures::executor::block_on;
///
/// let projects = block_on(list_projects_with_paging(Some(10), Some(0))).unwrap();
/// assert!(projects.len() <= 10);
/// ```
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

pub async fn create_project(
    name: &str,
    description: Option<&str>,
) -> Result<DashboardProject, String> {
    let payload = CreateProjectRequest { name, description };
    let body = serde_json::to_string(&payload).map_err(|e| format!("serialize body: {e}"))?;
    fetch(
        HttpMethod::Post,
        &api_url("/api/projects"),
        Some(&body),
        true,
    )
    .await
}

pub async fn delete_project(id: &str) -> Result<(), String> {
    send(
        HttpMethod::Delete,
        &api_url(&format!("/api/projects/{id}")),
        None,
        true,
    )
    .await
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

pub async fn list_demos_with_filters_typed(
    limit: Option<u32>,
    offset: Option<u32>,
    project_id: Option<&str>,
    published: Option<bool>,
) -> Result<Vec<DashboardDemo>, ClientError> {
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
    fetch_typed(HttpMethod::Get, &url, None, true).await
}
pub async fn get_demo(id: &str) -> Result<DashboardDemo, String> {
    fetch(
        HttpMethod::Get,
        &api_url(&format!("/api/demos/{id}")),
        None,
        true,
    )
    .await
}

pub async fn get_demo_detail(id: &str) -> Result<Demo, String> {
    fetch(
        HttpMethod::Get,
        &api_url(&format!("/api/demos/{id}")),
        None,
        true,
    )
    .await
}

pub async fn get_public_demo(reference: &str) -> Result<PublicDemoResponse, String> {
    fetch(
        HttpMethod::Get,
        &api_url(&format!("/api/public/demos/{reference}")),
        None,
        false,
    )
    .await
}

pub async fn create_demo(title: &str, project_id: Option<&str>) -> Result<DashboardDemo, String> {
    let payload = CreateDemoRequest { title, project_id };
    let body = serde_json::to_string(&payload).map_err(|e| format!("serialize body: {e}"))?;
    fetch(HttpMethod::Post, &api_url("/api/demos"), Some(&body), true).await
}
pub async fn update_demo(
    id: &str,
    title: Option<&str>,
    slug: Option<&str>,
) -> Result<DashboardDemo, String> {
    update_demo_payload(
        id,
        &UpdateDemoRequest {
            title: title.map(ToString::to_string),
            project_id: None,
            slug: slug.map(ToString::to_string),
            theme: None,
            settings: None,
            steps: None,
        },
    )
    .await
}
pub async fn update_demo_payload(
    id: &str,
    payload: &UpdateDemoRequest,
) -> Result<DashboardDemo, String> {
    let body = serde_json::to_string(payload).map_err(|e| format!("serialize body: {e}"))?;
    fetch(
        HttpMethod::Patch,
        &api_url(&format!("/api/demos/{id}")),
        Some(&body),
        true,
    )
    .await
}

pub async fn update_demo_project(
    id: &str,
    project_id: Option<&str>,
) -> Result<DashboardDemo, String> {
    let parsed_project_id = match project_id {
        Some(value) if !value.trim().is_empty() => {
            Some(Uuid::parse_str(value.trim()).map_err(|e| format!("invalid project id: {e}"))?)
        }
        _ => None,
    };

    update_demo_payload(
        id,
        &UpdateDemoRequest {
            title: None,
            project_id: Some(parsed_project_id),
            slug: None,
            theme: None,
            settings: None,
            steps: None,
        },
    )
    .await
}

pub async fn delete_demo(id: &str) -> Result<(), String> {
    send(
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

#[cfg(test)]
mod tests {
    use super::encode_query_value;

    #[test]
    fn query_path_encodes_special_chars() {
        let encoded = encode_query_value("hello world & more");
        assert_eq!(encoded, "hello%20world%20%26%20more");
        assert!(!encoded.contains(' '));
    }
}
