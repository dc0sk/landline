//! HTTP-level integration tests for the ARC-07 audit log (NFR-MAINT-02).
//!
//! Traces: TC-AUDIT-02 (failed login logged with IP + timestamp, no password),
//! plus Admin-only access to the audit view.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use landline_backend::app;
use landline_backend::auth::{hash_password, Role};
use landline_backend::config::{
    AuditConfig, AuthConfig, Config, SecurityConfig, ServerConfig, UserConfig,
};
use serde_json::{json, Value};
use tower::ServiceExt;

fn config(users: Vec<UserConfig>) -> Config {
    Config {
        server: ServerConfig::default(),
        auth: AuthConfig {
            access_ttl_secs: 900,
            refresh_ttl_secs: 3600,
            users,
        },
        // High rate limit so the multi-request flow is never throttled.
        security: SecurityConfig {
            rate_limit_per_sec: 1000,
            ..SecurityConfig::default()
        },
        audit: AuditConfig::default(),
    }
}

fn user(name: &str, role: Role, password: &str) -> UserConfig {
    UserConfig {
        name: name.to_owned(),
        role,
        password_hash: hash_password(password).unwrap(),
    }
}

fn login_from(name: &str, password: &str, ip: Ipv4Addr) -> Request<Body> {
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
        .insert(ConnectInfo(SocketAddr::new(IpAddr::V4(ip), 55000)));
    request
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn failed_login_is_audited_with_ip_and_no_password() {
    // TC-AUDIT-02
    let app = app(&config(vec![user("admin", Role::Admin, "adminpw")]));
    let attacker_ip = Ipv4Addr::new(203, 0, 113, 7);
    let secret_password = "sup3r-secret-pw";

    // A failed login attempt from the attacker IP.
    let failed = app
        .clone()
        .oneshot(login_from("admin", secret_password, attacker_ip))
        .await
        .unwrap();
    assert_eq!(failed.status(), StatusCode::UNAUTHORIZED);

    // Admin logs in and reads the audit view.
    let ok = app
        .clone()
        .oneshot(login_from("admin", "adminpw", Ipv4Addr::new(10, 0, 0, 1)))
        .await
        .unwrap();
    let admin_token = body_json(ok).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    let view = app
        .oneshot(
            Request::builder()
                .uri("/api/audit")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(view.status(), StatusCode::OK);
    let events = body_json(view).await;
    let events = events.as_array().unwrap();

    // There is a failure event for the attacker IP...
    let failure = events
        .iter()
        .find(|e| e["outcome"] == "failure")
        .expect("a failure event was recorded");
    assert_eq!(failure["client_ip"], "203.0.113.7");
    assert_eq!(failure["action"], "auth.login");
    assert!(failure["timestamp"].as_u64().unwrap() > 0);

    // ...and the password appears nowhere in the audit log.
    let dump = serde_json::to_string(&events).unwrap();
    assert!(
        !dump.contains(secret_password),
        "audit log must not contain the attempted password"
    );
}

#[tokio::test]
async fn audit_view_requires_admin() {
    let app = app(&config(vec![user("op", Role::Operator, "pw")]));
    let ok = app
        .clone()
        .oneshot(login_from("op", "pw", Ipv4Addr::new(10, 0, 0, 1)))
        .await
        .unwrap();
    let token = body_json(ok).await["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    let view = app
        .oneshot(
            Request::builder()
                .uri("/api/audit")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Operator is not Admin.
    assert_eq!(view.status(), StatusCode::FORBIDDEN);
}
