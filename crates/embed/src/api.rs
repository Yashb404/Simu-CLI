use shared::dto::PublicDemoResponse;
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
pub async fn fetch_public_demo(endpoint: &str) -> Result<PublicDemoResponse, String> {
    let response = gloo_net::http::Request::get(endpoint)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !response.ok() {
        return Err(format!("request failed with status {}", response.status()));
    }

    response
        .json::<PublicDemoResponse>()
        .await
        .map_err(|e| format!("invalid demo payload: {e}"))
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

    let response = gloo_net::http::Request::post(endpoint)
        .header("content-type", "application/json")
        .body(payload.to_string())
        .map_err(|e| format!("request build failed: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !response.ok() {
        return Err(format!("analytics request failed with status {}", response.status()));
    }

    Ok(())
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
