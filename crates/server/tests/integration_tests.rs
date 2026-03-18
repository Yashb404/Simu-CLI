/// Integration tests for the CLI Demo Studio API server.
///
/// Tests marked `#[ignore]` require a live Postgres connection via the
/// `DATABASE_URL` environment variable.  Run them with:
///
///   cargo test -p server -- --include-ignored
///
/// The CI pipeline provides `DATABASE_URL` automatically; unit tests (no DB
/// required) run without the flag.
mod fixtures;
use fixtures::{dummy_pool, json_body, test_router, test_router_with_rate_limit, try_db_fixture};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    Json,
};
use server::{
    auth::AuthUser,
    handlers::{
        demos::{self, ListMyDemosQuery},
        owned_demo::OwnedDemo,
    },
};
use shared::dto::{CreateDemoRequest, UpdateDemoRequest};
use tower::ServiceExt; // `oneshot`

// ─── Health check ────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_check_returns_200() {
    let app = test_router(dummy_pool());
    let response = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ─── Authentication guard ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_me_without_session_returns_401() {
    let app = test_router(dummy_pool());
    let response = app
        .oneshot(Request::builder().uri("/api/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_my_demos_without_session_returns_401() {
    let app = test_router(dummy_pool());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/me/demos")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_my_projects_without_session_returns_401() {
    let app = test_router(dummy_pool());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/me/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_demo_without_session_returns_401() {
    let app = test_router(dummy_pool());
    let body = serde_json::json!({ "title": "My Demo" }).to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/demos")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_project_without_session_returns_401() {
    let app = test_router(dummy_pool());
    let body = serde_json::json!({ "name": "My Project" }).to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ─── Public demo access control ───────────────────────────────────────────────

/// `GET /api/demos/:id` queries the DB before checking auth — it needs a real
/// Postgres connection.  The expected behavior is 404 (no such published demo).
#[tokio::test]
async fn get_demo_unknown_id_returns_404() {
    let Some(fixture) = try_db_fixture().await else {
        return;
    };
    let app = test_router(fixture.pool);
    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/demos/{fake_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Unauthenticated + unpublished = 404 (we treat not-found and forbidden the same for public)
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// `GET /api/demos/:id/public` also queries the DB; needs a real connection.
#[tokio::test]
async fn get_public_demo_unknown_id_returns_404() {
    let Some(fixture) = try_db_fixture().await else {
        return;
    };
    let app = test_router(fixture.pool);
    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/demos/{fake_id}/public"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ─── Analytics event ingestion ────────────────────────────────────────────────

#[tokio::test]
async fn post_analytics_event_with_invalid_type_returns_400() {
    let app = test_router(dummy_pool());
    let fake_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "demo_id": fake_id,
        "event_type": "not_a_real_event",
    })
    .to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/analytics/events")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    // Invalid event type must be rejected — 400 or 422, not 200/500
    let status = response.status().as_u16();
    assert!(
        status == 400 || status == 422,
        "invalid event type should be rejected, got {status}"
    );
}

// ─── API error body contract ──────────────────────────────────────────────────

#[tokio::test]
async fn error_responses_include_error_field() {
    let app = test_router(dummy_pool());
    let response = app
        .oneshot(Request::builder().uri("/api/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;
    assert!(
        body.get("error").is_some(),
        "error response body must contain an 'error' field, got: {body}"
    );
}

#[tokio::test]
async fn api_error_response_includes_request_id_when_header_present() {
    let app = test_router(dummy_pool());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header("x-request-id", "rid-apierror-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;
    assert_eq!(
        body.get("request_id").and_then(|v| v.as_str()),
        Some("rid-apierror-1"),
        "ApiError responses should carry request_id in JSON body"
    );
}

// ─── Correlation ID header ────────────────────────────────────────────────────

#[tokio::test]
async fn requests_echo_x_request_id_header() {
    let app = test_router(dummy_pool());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("x-request-id", "test-correlation-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("x-request-id").and_then(|v| v.to_str().ok()),
        Some("test-correlation-123"),
        "server must echo x-request-id from the request"
    );
}

#[tokio::test]
async fn requests_without_request_id_get_one_generated() {
    let app = test_router(dummy_pool());
    let response = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let id_header = response.headers().get("x-request-id");
    assert!(
        id_header.is_some(),
        "server must generate and return x-request-id when none is provided"
    );
    let id_str = id_header.unwrap().to_str().unwrap();
    assert!(!id_str.is_empty(), "generated request id must not be empty");
}

// ─── Rate limiting ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn rate_limiter_blocks_after_limit_exceeded() {
    // /api/health is exempt; use /api/me which IS rate-limited.
    let app = test_router_with_rate_limit(dummy_pool(), 1);

    // First request — exhausts the 1 req/min token bucket
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header("x-forwarded-for", "1.2.3.4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Second request from the same IP must be rate-limited (429) before auth runs
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header("x-forwarded-for", "1.2.3.4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "second request from same IP must be rate-limited"
    );
}

// ─── DB-dependent tests (require DATABASE_URL) ───────────────────────────────

/// These tests need a real Postgres instance.  They are skipped unless
/// `--include-ignored` is passed or `DATABASE_URL` is set (handled externally
/// by the developer or CI).
#[tokio::test]
async fn demo_crud_happy_path() {
    let Some(fixture) = try_db_fixture().await else {
        return;
    };

    let state = fixture.state;
    let user = fixture.user;

    let (created_status, created_demo) = demos::create_demo(
        State(state.clone()),
        AuthUser(user.clone()),
        Json(CreateDemoRequest {
            title: "CRUD Demo".to_string(),
            project_id: None,
        }),
    )
    .await
    .expect("create_demo should succeed");

    assert_eq!(created_status, StatusCode::CREATED);

    let updated_demo = demos::update_demo(
        State(state.clone()),
        OwnedDemo(created_demo.0.clone()),
        Json(UpdateDemoRequest {
            title: Some("CRUD Demo Updated".to_string()),
            slug: Some("crud-demo-updated".to_string()),
            theme: None,
            settings: None,
            steps: None,
        }),
    )
    .await
    .expect("update_demo should succeed")
    .0;

    assert_eq!(updated_demo.title, "CRUD Demo Updated");
    assert_eq!(updated_demo.slug.as_deref(), Some("crud-demo-updated"));

    let listed = demos::list_my_demos(
        State(state.clone()),
        AuthUser(user.clone()),
        Query(ListMyDemosQuery {
            limit: Some(10),
            offset: Some(0),
            project_id: None,
            published: None,
        }),
    )
    .await
    .expect("list_my_demos should succeed")
    .0;

    assert!(
        listed.iter().any(|d| d.id == created_demo.0.id),
        "created demo should appear in owner listing"
    );

    let delete_status = demos::delete_demo(
        State(state.clone()),
        OwnedDemo(updated_demo.clone()),
    )
    .await
    .expect("delete_demo should succeed");

    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let deleted_exists: Option<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM demos WHERE id = $1")
        .bind(created_demo.0.id)
        .fetch_optional(&state.db)
        .await
        .expect("delete verification query should succeed");

    assert!(deleted_exists.is_none(), "deleted demo should be removed from DB");
}

#[tokio::test]
async fn publish_makes_demo_publicly_accessible() {
    let Some(fixture) = try_db_fixture().await else {
        return;
    };

    let state = fixture.state;
    let user = fixture.user;

    let (_, created_demo) = demos::create_demo(
        State(state.clone()),
        AuthUser(user.clone()),
        Json(CreateDemoRequest {
            title: "Publishable Demo".to_string(),
            project_id: None,
        }),
    )
    .await
    .expect("create_demo should succeed");

    let publish = demos::publish_demo(
        State(state.clone()),
        OwnedDemo(created_demo.0.clone()),
    )
    .await
    .expect("publish_demo should succeed")
    .0;

    assert_eq!(publish.id, created_demo.0.id);
    assert!(publish.slug.starts_with("publishable-demo"));
    assert!(publish.version >= 2);

    let response = demos::get_public_demo(State(state), Path(created_demo.0.id))
        .await
        .expect("public demo should be available after publish");

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("cache-control").is_some());
    assert!(response.headers().get("etag").is_some());
}
