//! HTTP-level integration tests for the ARC-03 security middleware (NFR-MAINT-02).
//!
//! Traces: TC-SEC-04 (rate limiting), TC-SEC-06 (CORS allowlist), and the
//! request body-size limit (NFR-SEC-05, HTTP analogue).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use landline_backend::app;
use landline_backend::config::{Config, SecurityConfig};
use tower::ServiceExt;

fn config(security: SecurityConfig) -> Config {
    Config {
        security,
        ..Config::default()
    }
}

fn get_from(uri: &str, ip: Ipv4Addr) -> Request<Body> {
    let mut request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(IpAddr::V4(ip), 40000)));
    request
}

#[tokio::test]
async fn rate_limit_triggers_after_configured_rate() {
    // TC-SEC-04: with a 5/s limit, the 6th request from one client is blocked.
    let app = app(&config(SecurityConfig {
        rate_limit_per_sec: 5,
        ..SecurityConfig::default()
    }));
    let client = Ipv4Addr::new(10, 0, 0, 1);

    // Unauthenticated /api/me returns 401, but the rate limiter runs first and
    // still counts each request.
    for _ in 0..5 {
        let response = app
            .clone()
            .oneshot(get_from("/api/me", client))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    let blocked = app
        .clone()
        .oneshot(get_from("/api/me", client))
        .await
        .unwrap();
    assert_eq!(blocked.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn rate_limit_is_per_client() {
    let app = app(&config(SecurityConfig {
        rate_limit_per_sec: 2,
        ..SecurityConfig::default()
    }));
    let a = Ipv4Addr::new(10, 0, 0, 1);
    let b = Ipv4Addr::new(10, 0, 0, 2);

    for _ in 0..2 {
        let _ = app.clone().oneshot(get_from("/api/me", a)).await.unwrap();
    }
    // client A is now exhausted...
    let a_blocked = app.clone().oneshot(get_from("/api/me", a)).await.unwrap();
    assert_eq!(a_blocked.status(), StatusCode::TOO_MANY_REQUESTS);
    // ...but client B still has its own budget.
    let b_ok = app.clone().oneshot(get_from("/api/me", b)).await.unwrap();
    assert_eq!(b_ok.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn cors_allows_configured_origin_only() {
    // TC-SEC-06: a configured origin is echoed; a disallowed one is not.
    let app = app(&config(SecurityConfig {
        allowed_origins: vec!["https://good.example".to_owned()],
        ..SecurityConfig::default()
    }));

    let allowed = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/version")
                .header("origin", "https://good.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        allowed
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap().to_owned()),
        Some("https://good.example".to_owned())
    );

    let denied = app
        .oneshot(
            Request::builder()
                .uri("/version")
                .header("origin", "https://evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(denied
        .headers()
        .get("access-control-allow-origin")
        .is_none());
}

#[tokio::test]
async fn panics_return_a_sanitised_500() {
    // NFR-SEC-09: a handler panic must not leak its message; the catch-panic
    // layer returns a generic 500.
    async fn boom() -> &'static str {
        panic!("internal secret detail /etc/shadow");
    }
    let app = Router::new()
        .route("/boom", get(boom))
        .layer(landline_backend::security::catch_panic_layer());

    let response = app
        .oneshot(Request::builder().uri("/boom").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert!(
        !body.contains("secret detail"),
        "panic message leaked: {body}"
    );
}

#[tokio::test]
async fn oversized_request_body_is_rejected() {
    // NFR-SEC-05 (HTTP analogue): a body over the limit is rejected with 413.
    let app = app(&config(SecurityConfig {
        max_body_bytes: 1024,
        ..SecurityConfig::default()
    }));
    let big = vec![b'x'; 2048];
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(big))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
