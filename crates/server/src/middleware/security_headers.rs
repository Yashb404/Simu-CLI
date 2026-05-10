use axum::{
    body::Body,
    http::{HeaderValue, Request, header},
    middleware::Next,
    response::Response,
};

pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let is_embed_route = path.starts_with("/d/")
        || path.starts_with("/embed/")
        || path == "/embed"
        || path == "/embed-runtime"
        || path.starts_with("/embed-runtime/");
    let mut response = next.run(req).await;
    let connect_src = if cfg!(debug_assertions) {
        "connect-src 'self' ws: wss:;"
    } else {
        "connect-src 'self';"
    };

    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    if !is_embed_route {
        headers.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
    }
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    if is_embed_route {
        let csp = format!(
            "default-src 'self'; frame-ancestors *; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' data: https://fonts.gstatic.com; img-src 'self' https://avatars.githubusercontent.com; {connect_src}"
        );
        headers.insert(
            "content-security-policy",
            HeaderValue::from_str(&csp)
                .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
        );
    } else {
        let csp = format!(
            "default-src 'self'; frame-ancestors 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' data: https://fonts.gstatic.com; img-src 'self' https://avatars.githubusercontent.com; {connect_src}"
        );
        headers.insert(
            "content-security-policy",
            HeaderValue::from_str(&csp)
                .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
        );
    }
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response
}
