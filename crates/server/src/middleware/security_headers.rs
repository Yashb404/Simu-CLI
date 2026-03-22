use axum::{
    body::Body,
    http::{header, HeaderValue, Request},
    middleware::Next,
    response::Response,
};

pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let is_embed_route = path.starts_with("/d/") || path == "/embed-runtime" || path.starts_with("/embed-runtime/");
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    if !is_embed_route {
        headers.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
    }
    headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    if is_embed_route {
        headers.insert(
            "content-security-policy",
            HeaderValue::from_static("default-src 'self'; frame-ancestors *; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' data: https://fonts.gstatic.com; img-src 'self' https://avatars.githubusercontent.com"),
        );
    } else {
        headers.insert(
            "content-security-policy",
            HeaderValue::from_static("default-src 'self'; frame-ancestors 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' data: https://fonts.gstatic.com; img-src 'self' https://avatars.githubusercontent.com"),
        );
    }
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}
