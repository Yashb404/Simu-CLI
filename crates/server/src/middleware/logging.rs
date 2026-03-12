use std::time::Instant;

use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
const MAX_REQUEST_ID_LEN: usize = 128;

fn resolve_request_id(req: &Request<Body>) -> String {
    let incoming = req
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter(|v| v.len() <= MAX_REQUEST_ID_LEN)
        .map(ToOwned::to_owned);

    incoming.unwrap_or_else(|| Uuid::new_v4().to_string())
}

pub async fn logging_middleware(mut req: Request<Body>, next: Next) -> Response {
    let request_id = resolve_request_id(&req);
    req.extensions_mut().insert(request_id.clone());

    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let mut response = next.run(req).await;
    let elapsed_ms = start.elapsed().as_millis();

    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, header_value);
    }

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status = %response.status().as_u16(),
        elapsed_ms = %elapsed_ms,
        "request completed"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_request_id_uses_incoming_header_when_valid() {
        let req = Request::builder()
            .uri("/")
            .header("x-request-id", "req-123")
            .body(Body::empty())
            .expect("request build should succeed");

        let request_id = resolve_request_id(&req);
        assert_eq!(request_id, "req-123");
    }

    #[test]
    fn resolve_request_id_generates_when_missing() {
        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .expect("request build should succeed");

        let request_id = resolve_request_id(&req);
        assert!(!request_id.is_empty());
    }

    #[test]
    fn resolve_request_id_rejects_oversized_value() {
        let oversized = "x".repeat(MAX_REQUEST_ID_LEN + 1);
        let req = Request::builder()
            .uri("/")
            .header("x-request-id", oversized)
            .body(Body::empty())
            .expect("request build should succeed");

        let request_id = resolve_request_id(&req);
        assert!(request_id.len() <= 36);
    }
}
