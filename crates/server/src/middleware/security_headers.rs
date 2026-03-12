use axum::{
    body::Body,
    http::{header, HeaderValue, Request},
    middleware::Next,
    response::Response,
};

pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
    headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'self'; frame-ancestors 'self' *; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"),
    );
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}
