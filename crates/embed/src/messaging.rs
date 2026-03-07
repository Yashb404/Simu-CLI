use serde::Serialize;

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

#[cfg(target_arch = "wasm32")]
pub fn post_event_to_parent(event: &EmbedEvent) -> Result<(), String> {
    use wasm_bindgen::JsValue;

    let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
    let js_value = JsValue::from_serde(event).map_err(|e| format!("serialize event: {e}"))?;

    window
        .post_message(&js_value, "*")
        .map_err(|e| format!("postMessage failed: {e:?}"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn post_event_to_parent(_event: &EmbedEvent) -> Result<(), String> {
    Ok(())
}
