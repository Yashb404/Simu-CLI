use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use url::Url;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedEventType {
    View,
    Interaction,
    Completion,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmbedEvent {
    pub event_type: EmbedEventType,
    pub demo_id: String,
    pub payload: serde_json::Value,
}

impl EmbedEvent {
    pub fn view(demo_id: impl Into<String>) -> Self {
        Self {
            event_type: EmbedEventType::View,
            demo_id: demo_id.into(),
            payload: serde_json::json!({}),
        }
    }

    pub fn interaction(demo_id: impl Into<String>, command: &str) -> Self {
        Self {
            event_type: EmbedEventType::Interaction,
            demo_id: demo_id.into(),
            payload: serde_json::json!({ "command": command }),
        }
    }

    pub fn completion(demo_id: impl Into<String>) -> Self {
        Self {
            event_type: EmbedEventType::Completion,
            demo_id: demo_id.into(),
            payload: serde_json::json!({}),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn validated_target_origin(api_base: &str) -> Result<String, String> {
    let parsed = Url::parse(api_base).map_err(|e| format!("invalid api_base origin: {e}"))?;
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => return Err(format!("unsupported target origin scheme: {scheme}")),
    }

    let origin = parsed.origin().ascii_serialization();
    if origin == "null" {
        return Err("api_base did not resolve to a concrete origin".to_string());
    }

    Ok(origin)
}

#[cfg(target_arch = "wasm32")]
pub fn post_event_to_parent(event: &EmbedEvent, api_base: &str) -> Result<(), String> {
    use wasm_bindgen::JsValue;

    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let target_origin = validated_target_origin(api_base)?;
    // TODO: Switch to structured JsValue payload once we standardize postMessage contracts.
    let payload = serde_json::to_string(event).map_err(|e| format!("serialize event: {e}"))?;
    let js_value = JsValue::from_str(&payload);

    window
        .post_message(&js_value, &target_origin)
        .map_err(|e| format!("postMessage failed: {e:?}"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn post_event_to_parent(_event: &EmbedEvent, _api_base: &str) -> Result<(), String> {
    Ok(())
}
