#[cfg(target_arch = "wasm32")]
use shared::client::{HttpMethod, fetch};
use shared::dto::PublicDemoResponse;
use shared::models::analytics::AnalyticsEventType;
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
fn analytics_payload_json(
    demo_id: Uuid,
    event_type: AnalyticsEventType,
    step_index: Option<i32>,
) -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({
        "demo_id": demo_id,
        "event_type": event_type,
        "step_index": step_index,
    }))
    .map_err(|e| format!("serialize analytics payload: {e}"))
}

#[cfg(target_arch = "wasm32")]
fn send_analytics_via_beacon(endpoint: &str, payload: &str) -> Result<bool, String> {
    use js_sys::Array;
    use wasm_bindgen::JsValue;

    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let blob_parts = Array::new();
    blob_parts.push(&JsValue::from_str(payload));

    let blob = web_sys::Blob::new_with_str_sequence(&blob_parts)
        .map_err(|e| format!("create analytics beacon blob: {e:?}"))?;

    window
        .navigator()
        .send_beacon_with_opt_blob(endpoint, Some(&blob))
        .map_err(|e| format!("sendBeacon failed: {e:?}"))
}

#[cfg(target_arch = "wasm32")]
async fn send_analytics_via_fetch(endpoint: &str, payload: &str) -> Result<(), String> {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_mode(web_sys::RequestMode::Cors);
    init.set_body(&JsValue::from_str(payload));
    js_sys::Reflect::set(&init, &JsValue::from_str("keepalive"), &JsValue::TRUE)
        .map_err(|e| format!("set analytics keepalive: {e:?}"))?;

    let request = web_sys::Request::new_with_str_and_init(endpoint, &init)
        .map_err(|e| format!("create analytics request: {e:?}"))?;
    request
        .headers()
        .set("content-type", "application/json")
        .map_err(|e| format!("set analytics headers: {e:?}"))?;

    JsFuture::from(window.fetch_with_request(&request))
        .await
        .map(|_| ())
        .map_err(|e| format!("analytics fetch failed: {e:?}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn post_analytics_event(
    endpoint: &str,
    demo_id: Uuid,
    event_type: AnalyticsEventType,
    step_index: Option<i32>,
) -> Result<(), String> {
    let payload = analytics_payload_json(demo_id, event_type, step_index)?;

    if send_analytics_via_beacon(endpoint, &payload).unwrap_or(false) {
        return Ok(());
    }

    send_analytics_via_fetch(endpoint, &payload).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn post_analytics_event(
    _endpoint: &str,
    _demo_id: Uuid,
    _event_type: AnalyticsEventType,
    _step_index: Option<i32>,
) -> Result<(), String> {
    Ok(())
}
