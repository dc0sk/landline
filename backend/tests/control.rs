//! HTTP-level integration tests for the rig control handlers (NFR-MAINT-02).
//!
//! Traces: TC-RIG-01/02 (frequency), TC-RIG-03 (mode + reject unsupported),
//! TC-RIG-04 (PTT as Operator), TC-RIG-05 (Observer denied PTT + audited),
//! TC-AUDIT-01 (rig command produces an audit entry).

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::Router;
use landline_backend::app;
use landline_backend::auth::{hash_password, Role};
use landline_backend::config::{AuthConfig, Config, RigConfig, SecurityConfig, UserConfig};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tower::ServiceExt;

async fn spawn_mock_rigctld() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                    let command = line.trim_end();
                    let response: &[u8] = if command == "f" {
                        b"14074000\n"
                    } else if command == "m" {
                        b"USB\n2400\n"
                    } else if command.starts_with("F ")
                        || command.starts_with("M ")
                        || command.starts_with("T ")
                    {
                        b"RPRT 0\n"
                    } else {
                        b"RPRT -1\n"
                    };
                    if reader.get_mut().write_all(response).await.is_err() {
                        break;
                    }
                    let _ = reader.get_mut().flush().await;
                }
            });
        }
    });
    addr
}

fn app_with_rig(users: Vec<UserConfig>, rig_addr: SocketAddr) -> Router {
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
        rig: RigConfig {
            host: rig_addr.ip().to_string(),
            port: rig_addr.port(),
            timeout_ms: 2000,
            ..RigConfig::default()
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
    let bytes = axum::body::to_bytes(response.into_body(), 256 * 1024)
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
async fn operator_reads_and_sets_frequency() {
    // TC-RIG-01 / TC-RIG-02
    let rig = spawn_mock_rigctld().await;
    let app = app_with_rig(vec![user("op", Role::Operator, "pw")], rig);
    let token = login(&app, "op", "pw").await;

    let read = app
        .clone()
        .oneshot(get("/api/rig/frequency", &token))
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(body_json(read).await["hz"], 14_074_000);

    let set = app
        .clone()
        .oneshot(post(
            "/api/rig/frequency",
            &token,
            &json!({"hz": 14_100_000}),
        ))
        .await
        .unwrap();
    assert_eq!(set.status(), StatusCode::NO_CONTENT);

    // Out-of-range rejected with 400 (FR-RIG-09).
    let bad = app
        .oneshot(post("/api/rig/frequency", &token, &json!({"hz": -1})))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn operator_reads_and_sets_mode_rejects_unsupported() {
    // TC-RIG-03
    let rig = spawn_mock_rigctld().await;
    let app = app_with_rig(vec![user("op", Role::Operator, "pw")], rig);
    let token = login(&app, "op", "pw").await;

    let read = app
        .clone()
        .oneshot(get("/api/rig/mode", &token))
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(body_json(read).await["mode"], "USB");

    let set = app
        .clone()
        .oneshot(post(
            "/api/rig/mode",
            &token,
            &json!({"mode": "LSB", "passband_hz": 2400}),
        ))
        .await
        .unwrap();
    assert_eq!(set.status(), StatusCode::NO_CONTENT);

    // An unsupported / injection-y mode is a clean 400 (NFR-SEC-08).
    let bad = app
        .oneshot(post(
            "/api/rig/mode",
            &token,
            &json!({"mode": "USB; rm -rf /"}),
        ))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn operator_toggles_ptt() {
    // TC-RIG-04
    let rig = spawn_mock_rigctld().await;
    let app = app_with_rig(vec![user("op", Role::Operator, "pw")], rig);
    let token = login(&app, "op", "pw").await;

    let on = app
        .clone()
        .oneshot(post("/api/rig/ptt", &token, &json!({"transmit": true})))
        .await
        .unwrap();
    assert_eq!(on.status(), StatusCode::NO_CONTENT);

    let off = app
        .oneshot(post("/api/rig/ptt", &token, &json!({"transmit": false})))
        .await
        .unwrap();
    assert_eq!(off.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn observer_denied_ptt_and_audited() {
    // TC-RIG-05: Observer PTT attempt is rejected (403) and audited.
    let rig = spawn_mock_rigctld().await;
    let app = app_with_rig(
        vec![
            user("obs", Role::Observer, "pw"),
            user("admin", Role::Admin, "adminpw"),
        ],
        rig,
    );

    let obs_token = login(&app, "obs", "pw").await;
    let denied = app
        .clone()
        .oneshot(post("/api/rig/ptt", &obs_token, &json!({"transmit": true})))
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);

    // The denied attempt appears in the audit log as a failure.
    let admin_token = login(&app, "admin", "adminpw").await;
    let view = app.oneshot(get("/api/audit", &admin_token)).await.unwrap();
    let events = body_json(view).await;
    let denied_ptt = events
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["action"] == "rig.ptt" && e["outcome"] == "failure");
    assert!(denied_ptt.is_some(), "denied PTT must be audited");
}

#[tokio::test]
async fn set_frequency_is_audited() {
    // TC-AUDIT-01: a rig state-changing command produces an audit entry with
    // user, action, and params.
    let rig = spawn_mock_rigctld().await;
    let app = app_with_rig(
        vec![
            user("op", Role::Operator, "pw"),
            user("admin", Role::Admin, "adminpw"),
        ],
        rig,
    );

    let op_token = login(&app, "op", "pw").await;
    let set = app
        .clone()
        .oneshot(post(
            "/api/rig/frequency",
            &op_token,
            &json!({"hz": 14_074_000}),
        ))
        .await
        .unwrap();
    assert_eq!(set.status(), StatusCode::NO_CONTENT);

    let admin_token = login(&app, "admin", "adminpw").await;
    let view = app.oneshot(get("/api/audit", &admin_token)).await.unwrap();
    let events = body_json(view).await;
    let entry = events
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["action"] == "rig.set_freq")
        .expect("set_freq must be audited");
    assert_eq!(entry["user"], "op");
    assert_eq!(entry["params"], "14074000");
    assert_eq!(entry["outcome"], "success");
}
