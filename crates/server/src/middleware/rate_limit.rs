use std::net::IpAddr;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::state::AppState;

fn resolve_client_ip(req: &Request<Body>) -> Option<IpAddr> {
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first) = value.split(',').next() {
                if let Ok(ip) = first.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            if let Ok(ip) = value.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    None
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if req.uri().path() == "/api/health" {
        return next.run(req).await;
    }

    let client_ip = resolve_client_ip(&req).unwrap_or(IpAddr::from([127, 0, 0, 1]));

    if state.rate_limiter.check_key(&client_ip).is_err() {
        let body = Json(json!({
            "error": "Rate limit exceeded",
        }));
        return (StatusCode::TOO_MANY_REQUESTS, body).into_response();
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        http::Request as HttpRequest,
        middleware,
        routing::get,
        Router,
    };
    use governor::{Quota, RateLimiter};
    use std::{num::NonZeroU32, sync::Arc};
    use tower::ServiceExt;

    use crate::config::Config;

    fn test_config() -> Config {
        Config {
            database_url: "postgres://dummy:dummy@localhost/dummy".to_string(),
            github_client_id: "test-client".to_string(),
            github_client_secret: "test-secret".to_string(),
            session_secret: "a".repeat(64),
            api_url: "http://localhost:3001".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
            port: 3001,
            rate_limit_requests_per_minute: 1,
            session_timeout: time::Duration::days(7),
            session_cookie_secure: false,
            log_level: "server=debug".to_string(),
        }
    }

    fn test_state(limit_per_minute: u32) -> Option<AppState> {
        let pool_result = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://dummy:dummy@localhost/dummy");
        let pool = match pool_result {
            Ok(pool) => pool,
            Err(_) => return None,
        };

        let per_minute = NonZeroU32::new(limit_per_minute)?;

        Some(AppState {
            db: pool,
            config: test_config(),
            rate_limiter: Arc::new(RateLimiter::keyed(Quota::per_minute(per_minute))),
        })
    }

    #[test]
    fn resolves_ip_from_forwarded_header() {
        let request_result = HttpRequest::builder()
            .uri("/api/demos")
            .header("x-forwarded-for", "203.0.113.5, 198.51.100.1")
            .body(Body::empty());
        assert!(request_result.is_ok(), "request should build");
        let request = match request_result {
            Ok(request) => request,
            Err(_) => return,
        };

        let ip = resolve_client_ip(&request);
        assert_eq!(ip, Some(IpAddr::from([203, 0, 113, 5])));
    }

    #[tokio::test]
    async fn throttles_after_limit_is_exceeded() {
        let state = match test_state(1) {
            Some(state) => state,
            None => return,
        };

        let app = Router::new()
            .route("/api/ping", get(|| async { "pong" }))
            .with_state(state.clone())
            .layer(middleware::from_fn_with_state(
                state,
                rate_limit_middleware,
            ));

        let request_one_result = HttpRequest::builder()
            .uri("/api/ping")
            .header("x-real-ip", "198.51.100.9")
            .body(Body::empty());
        assert!(request_one_result.is_ok(), "first request should build");
        let request_one = match request_one_result {
            Ok(request) => request,
            Err(_) => return,
        };

        let response_one_result = app.clone().oneshot(request_one).await;
        assert!(response_one_result.is_ok(), "first request should execute");
        let response_one = match response_one_result {
            Ok(response) => response,
            Err(_) => return,
        };
        assert_eq!(response_one.status(), StatusCode::OK);

        let request_two_result = HttpRequest::builder()
            .uri("/api/ping")
            .header("x-real-ip", "198.51.100.9")
            .body(Body::empty());
        assert!(request_two_result.is_ok(), "second request should build");
        let request_two = match request_two_result {
            Ok(request) => request,
            Err(_) => return,
        };

        let response_two_result = app.oneshot(request_two).await;
        assert!(response_two_result.is_ok(), "second request should execute");
        let response_two = match response_two_result {
            Ok(response) => response,
            Err(_) => return,
        };
        assert_eq!(response_two.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
