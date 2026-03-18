use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

#[cfg(target_arch = "wasm32")]
fn method_builder(method: HttpMethod, url: &str) -> gloo_net::http::RequestBuilder {
    match method {
        HttpMethod::Get => gloo_net::http::Request::get(url),
        HttpMethod::Post => gloo_net::http::Request::post(url),
        HttpMethod::Patch => gloo_net::http::Request::patch(url),
        HttpMethod::Put => gloo_net::http::Request::put(url),
        HttpMethod::Delete => gloo_net::http::Request::delete(url),
    }
}

#[cfg(target_arch = "wasm32")]
async fn execute(
    method: HttpMethod,
    url: &str,
    body: Option<&str>,
    include_credentials: bool,
) -> Result<gloo_net::http::Response, String> {
    let mut builder = method_builder(method, url);
    if include_credentials {
        builder = builder.credentials(web_sys::RequestCredentials::Include);
    }

    let response = match body {
        Some(payload) => builder
            .header("content-type", "application/json")
            .body(payload)
            .map_err(|e| format!("build request: {e}"))?
            .send()
            .await,
        None => builder.send().await,
    }
    .map_err(|e| format!("request failed: {e}"))?;

    if !response.ok() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        if status == 401 {
            return Err("Not logged in. Click Login with GitHub and retry.".to_string());
        }
        if body_text.is_empty() {
            return Err(format!("request failed with status {status}"));
        }
        return Err(format!("request failed with status {status}: {body_text}"));
    }

    Ok(response)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch<T: DeserializeOwned>(
    method: HttpMethod,
    url: &str,
    body: Option<&str>,
    include_credentials: bool,
) -> Result<T, String> {
    let response = execute(method, url, body, include_credentials).await?;
    response
        .json::<T>()
        .await
        .map_err(|e| format!("invalid response payload: {e}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn send(
    method: HttpMethod,
    url: &str,
    body: Option<&str>,
    include_credentials: bool,
) -> Result<(), String> {
    execute(method, url, body, include_credentials).await?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch<T: DeserializeOwned>(
    _method: HttpMethod,
    _url: &str,
    _body: Option<&str>,
    _include_credentials: bool,
) -> Result<T, String> {
    Err("shared::client::fetch is only available on wasm32 targets".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn send(
    _method: HttpMethod,
    _url: &str,
    _body: Option<&str>,
    _include_credentials: bool,
) -> Result<(), String> {
    Err("shared::client::send is only available on wasm32 targets".to_string())
}
