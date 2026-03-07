use shared::dto::PublicDemoResponse;

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
