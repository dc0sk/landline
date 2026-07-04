//! Integration tests for the public backend API surface (NFR-MAINT-02, TC-MAINT-02).
//!
//! These exercise the router built by `landline_backend::app` end-to-end through
//! the Tower service interface, without binding a socket.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use landline_backend::app;
use landline_backend::config::Config;
use tower::ServiceExt; // brings `oneshot` into scope

#[tokio::test]
async fn healthz_returns_200() {
    let response = app(&Config::default())
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn version_returns_200() {
    let response = app(&Config::default())
        .oneshot(
            Request::builder()
                .uri("/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let response = app(&Config::default())
        .oneshot(
            Request::builder()
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
