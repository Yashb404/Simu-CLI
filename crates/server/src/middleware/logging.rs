use std::time::Instant;

use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};

pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;
    let elapsed_ms = start.elapsed().as_millis();

    tracing::info!(
        method = %method,
        path = %path,
        status = %response.status().as_u16(),
        elapsed_ms = %elapsed_ms,
        "request completed"
    );

    response
}
