use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
static ERROR_COUNT: AtomicU64 = AtomicU64::new(0);

pub async fn metrics_middleware(req: Request<Body>, next: Next) -> Response {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);

    let response = next.run(req).await;
    if response.status().is_server_error() {
        ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    response
}

pub async fn metrics_handler() -> impl IntoResponse {
    let requests = REQUEST_COUNT.load(Ordering::Relaxed);
    let errors = ERROR_COUNT.load(Ordering::Relaxed);

    format!(
        "# HELP app_requests_total Total HTTP requests\n# TYPE app_requests_total counter\napp_requests_total {}\n# HELP app_errors_total Total HTTP 5xx responses\n# TYPE app_errors_total counter\napp_errors_total {}\n",
        requests, errors
    )
}
