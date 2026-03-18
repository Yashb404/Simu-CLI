use shared::dto::PublicDemoResponse;
#[cfg(target_arch = "wasm32")]
use shared::client::{fetch, send, HttpMethod};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
pub async fn fetch_public_demo(endpoint: &str) -> Result<PublicDemoResponse, String> {
    fetch(HttpMethod::Get, endpoint, None, false).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_public_demo(_endpoint: &str) -> Result<PublicDemoResponse, String> {
    Err("fetch_public_demo is only available on wasm32 targets".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn post_analytics_event(
    endpoint: &str,
    demo_id: Uuid,
    event_type: &str,
    step_index: Option<i32>,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "demo_id": demo_id,
        "event_type": event_type,
        "step_index": step_index,
    });

    send(HttpMethod::Post, endpoint, Some(&payload.to_string()), false).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn post_analytics_event(
    _endpoint: &str,
    _demo_id: Uuid,
    _event_type: &str,
    _step_index: Option<i32>,
) -> Result<(), String> {
    Ok(())
}
