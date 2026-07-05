//! WebSocket transport integration tests (NFR-MAINT-02).
//!
//! Traces TC-AUTH-01 (unauthenticated WS rejected) and FR-SPEC-01 (spectrum
//! frames delivered over an authenticated WS). Runs a real server and connects a
//! real WS client so the handshake, auth, and streaming path are exercised
//! end-to-end.

use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use landline_backend::app;
use landline_backend::auth::{hash_password, Role};
use landline_backend::config::{AuthConfig, Config, SecurityConfig, SpectrumConfig, UserConfig};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tower::ServiceExt;

/// Build the app, obtain an operator token via a REST login, then serve the app
/// on an ephemeral port. Returns the address and the token.
async fn start() -> (SocketAddr, String) {
    let config = Config {
        auth: AuthConfig {
            access_ttl_secs: 900,
            refresh_ttl_secs: 3600,
            users: vec![UserConfig {
                name: "op".to_owned(),
                role: Role::Operator,
                password_hash: hash_password("pw").unwrap(),
            }],
        },
        security: SecurityConfig {
            rate_limit_per_sec: 1000,
            ..SecurityConfig::default()
        },
        spectrum: SpectrumConfig {
            fft_size: 256,
            update_rate_hz: 10.0,
            ..SpectrumConfig::default()
        },
        ..Config::default()
    };
    let app = app(&config);
    let token = login(&app).await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (addr, token)
}

async fn login(app: &Router) -> String {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({"name": "op", "password": "pw"})).unwrap(),
        ))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo("10.0.0.1:5000".parse::<SocketAddr>().unwrap()));
    let response = app.clone().oneshot(request).await.unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    value["access_token"].as_str().unwrap().to_owned()
}

fn as_json(message: WsMessage) -> Option<Value> {
    match message {
        WsMessage::Text(text) => serde_json::from_str(text.as_str()).ok(),
        _ => None,
    }
}

async fn send_json(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    value: &Value,
) {
    ws.send(WsMessage::Text(value.to_string())).await.unwrap();
}

#[tokio::test]
async fn unauthenticated_ws_is_rejected() {
    // TC-AUTH-01: a first message that is not a valid auth is rejected.
    let (addr, _token) = start().await;
    let (mut ws, _) = connect_async(format!("ws://{addr}/ws")).await.unwrap();

    send_json(&mut ws, &json!({"type": "subscribe", "stream": "spectrum"})).await;
    let reply = as_json(ws.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(reply["type"], "error");
}

#[tokio::test]
async fn bad_token_is_rejected() {
    let (addr, _token) = start().await;
    let (mut ws, _) = connect_async(format!("ws://{addr}/ws")).await.unwrap();

    send_json(&mut ws, &json!({"type": "auth", "token": "not-a-jwt"})).await;
    let reply = as_json(ws.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(reply["type"], "error");
}

#[tokio::test]
async fn authenticated_client_receives_spectrum_frames() {
    // FR-SPEC-01 / TC-SPEC-01: after auth + subscribe, spectrum frames arrive.
    let (addr, token) = start().await;
    let (mut ws, _) = connect_async(format!("ws://{addr}/ws")).await.unwrap();

    send_json(&mut ws, &json!({"type": "auth", "token": token})).await;
    let ready = as_json(ws.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(ready["type"], "ready");
    assert_eq!(ready["role"], "operator");

    send_json(&mut ws, &json!({"type": "subscribe", "stream": "spectrum"})).await;

    // The next spectrum frame carries a non-empty bin array.
    let frame = loop {
        let value = as_json(ws.next().await.unwrap().unwrap());
        if let Some(value) = value {
            if value["type"] == "spectrum" {
                break value;
            }
        }
    };
    assert_eq!(frame["bins"].as_array().unwrap().len(), 128); // fft_size 256 / 2
    assert!(frame["seq"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn authenticated_client_receives_audio_frames() {
    // FR-AUD-01 (transport): after auth + subscribe audio, binary audio frames
    // arrive — 8-byte LE sequence header + encoded payload.
    let (addr, token) = start().await;
    let (mut ws, _) = connect_async(format!("ws://{addr}/ws")).await.unwrap();

    send_json(&mut ws, &json!({"type": "auth", "token": token})).await;
    let ready = as_json(ws.next().await.unwrap().unwrap()).unwrap();
    assert_eq!(ready["type"], "ready");

    send_json(&mut ws, &json!({"type": "subscribe", "stream": "audio"})).await;

    // The next binary message is an audio frame.
    let payload = loop {
        if let WsMessage::Binary(bytes) = ws.next().await.unwrap().unwrap() {
            break bytes;
        }
    };
    assert!(payload.len() > 8, "frame carries a header + payload");
    let seq = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    assert!(seq >= 1);
}
