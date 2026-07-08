//! HTTP-level integration tests for the ARC-08 GPIO endpoints (NFR-MAINT-02).
//!
//! Traces TC-SEC-15 (non-allowlisted pins denied; safe startup states) and the
//! GPIO control path (FR-GPIO-01) at the HTTP layer.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::Router;
use landline_backend::app;
use landline_backend::auth::{hash_password, Role};
use landline_backend::config::{
    AuthConfig, Config, GpioConfig, GpioPinConfig, SecurityConfig, UserConfig,
};
use landline_backend::gpio::{Direction, Level};
use serde_json::{json, Value};
use tower::ServiceExt;

fn app_with_gpio(users: Vec<UserConfig>) -> Router {
    app(&Config {
        auth: AuthConfig {
            access_ttl_secs: 900,
            refresh_ttl_secs: 3600,
            users,
        },
        security: SecurityConfig {
            rate_limit_per_sec: 1000,
            ..SecurityConfig::default()
        },
        gpio: GpioConfig {
            enabled: true,
            pins: vec![
                GpioPinConfig {
                    pin: 17,
                    direction: Direction::Out,
                    safe_state: Level::Low,
                },
                GpioPinConfig {
                    pin: 27,
                    direction: Direction::In,
                    safe_state: Level::High,
                },
            ],
            chip: None,
        },
        ..Config::default()
    })
}

fn user(name: &str, role: Role, password: &str) -> UserConfig {
    UserConfig {
        name: name.to_owned(),
        role,
        password_hash: hash_password(password).unwrap(),
    }
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn login(app: &Router, name: &str, password: &str) -> String {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({"name": name, "password": password})).unwrap(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo("10.0.0.1:5000".parse::<SocketAddr>().unwrap()));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

fn post(uri: &str, token: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn operator_sets_and_reads_allowlisted_pin() {
    let app = app_with_gpio(vec![user("op", Role::Operator, "pw")]);
    let token = login(&app, "op", "pw").await;

    // Safe startup state is Low.
    let read = app
        .clone()
        .oneshot(get("/api/gpio/17", &token))
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(body_json(read).await["level"], "low");

    let set = app
        .clone()
        .oneshot(post("/api/gpio/17", &token, &json!({"level": "high"})))
        .await
        .unwrap();
    assert_eq!(set.status(), StatusCode::NO_CONTENT);

    let read2 = app.oneshot(get("/api/gpio/17", &token)).await.unwrap();
    assert_eq!(body_json(read2).await["level"], "high");
}

#[tokio::test]
async fn non_allowlisted_pin_is_forbidden() {
    // TC-SEC-15 / NFR-SEC-16
    let app = app_with_gpio(vec![user("op", Role::Operator, "pw")]);
    let token = login(&app, "op", "pw").await;

    let read = app
        .clone()
        .oneshot(get("/api/gpio/5", &token))
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::FORBIDDEN);

    let set = app
        .oneshot(post("/api/gpio/5", &token, &json!({"level": "high"})))
        .await
        .unwrap();
    assert_eq!(set.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn input_pin_cannot_be_driven() {
    let app = app_with_gpio(vec![user("op", Role::Operator, "pw")]);
    let token = login(&app, "op", "pw").await;

    let set = app
        .oneshot(post("/api/gpio/27", &token, &json!({"level": "low"})))
        .await
        .unwrap();
    assert_eq!(set.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn observer_denied_gpio() {
    let app = app_with_gpio(vec![user("obs", Role::Observer, "pw")]);
    let token = login(&app, "obs", "pw").await;

    let set = app
        .oneshot(post("/api/gpio/17", &token, &json!({"level": "high"})))
        .await
        .unwrap();
    assert_eq!(set.status(), StatusCode::FORBIDDEN);
}
