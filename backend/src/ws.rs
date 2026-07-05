//! ARC-01 WebSocket transport (ADR-02, Phase 2).
//!
//! One authenticated WebSocket carries telemetry (spectrum today; audio/live
//! S-meter later) with per-message-type handling. Security properties:
//!
//! - **Authentication (FR-AUTH-01, TC-AUTH-01):** the client must send an `auth`
//!   message with a valid JWT as its *first* frame; anything else is rejected and
//!   the socket closed. The token travels in the message body, never the URL
//!   (NFR-SEC-12), and carries its own expiry (addresses replayed handshakes,
//!   TC-SEC-11).
//! - **Frame limits (NFR-SEC-05, TC-SEC-05):** the upgrade caps message/frame
//!   size; oversized frames close the connection.
//! - **Read-only telemetry:** the WS surface only *subscribes* to streams; all
//!   state-changing control stays on the authenticated REST API, so a replayed
//!   WS frame cannot mutate rig state.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::Extension;
use serde::{Deserialize, Serialize};

use crate::auth::{Auth, Claims, Role};
use crate::spectrum::{SampleSource, SpectrumAnalyzer};

/// Shared runtime for the WS spectrum stream (ARC-06), built from config.
pub struct SpectrumRuntime {
    /// FFT analyser.
    pub analyzer: Arc<SpectrumAnalyzer>,
    /// Sample source feeding the analyser.
    pub source: Arc<dyn SampleSource>,
    /// Frame rate in Hz (clamped to 1–10 at send time).
    pub update_rate_hz: f32,
    /// Samples per FFT block.
    pub fft_size: usize,
    /// Source sample rate in Hz (reported to clients).
    pub sample_rate: u32,
    /// Nominal centre frequency in Hz (reported to clients).
    pub center_hz: u64,
    /// Maximum accepted WS message/frame size in bytes (NFR-SEC-05).
    pub max_frame_bytes: usize,
    /// How long to wait for the client's `auth` handshake frame.
    pub auth_timeout: Duration,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    Auth { token: String },
    Subscribe { stream: StreamKind },
    Unsubscribe { stream: StreamKind },
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum StreamKind {
    Spectrum,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Ready {
        role: Role,
    },
    Error {
        message: String,
    },
    Spectrum {
        seq: u64,
        sample_rate: u32,
        center_hz: u64,
        bins: Vec<f32>,
    },
}

/// Axum handler for `GET /ws`: upgrade with size caps, then run the session.
pub async fn handler(
    ws: WebSocketUpgrade,
    Extension(auth): Extension<Arc<Auth>>,
    Extension(runtime): Extension<Arc<SpectrumRuntime>>,
) -> Response {
    let max = runtime.max_frame_bytes;
    ws.max_message_size(max)
        .max_frame_size(max)
        .on_upgrade(move |socket| session(socket, auth, runtime))
}

async fn session(mut socket: WebSocket, auth: Arc<Auth>, runtime: Arc<SpectrumRuntime>) {
    let Some(claims) = authenticate(&mut socket, &auth, runtime.auth_timeout).await else {
        return;
    };
    // All authenticated roles may view telemetry (STK-02); state-changing control
    // is not exposed here (per-message-type ACL — ADR-02).
    if send(&mut socket, &ServerMessage::Ready { role: claims.role })
        .await
        .is_err()
    {
        return;
    }

    let mut subscribed = false;
    let mut seq: u64 = 0;
    let rate = runtime.update_rate_hz.clamp(1.0, 10.0);
    let mut ticker = tokio::time::interval(Duration::from_secs_f32(1.0 / rate));

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(text.as_str()) {
                            Ok(ClientMessage::Subscribe { stream: StreamKind::Spectrum }) => {
                                subscribed = true;
                            }
                            Ok(ClientMessage::Unsubscribe { stream: StreamKind::Spectrum }) => {
                                subscribed = false;
                            }
                            Ok(ClientMessage::Auth { .. }) => {} // already authenticated
                            Err(_) => {
                                let _ = send(
                                    &mut socket,
                                    &ServerMessage::Error { message: "bad message".to_owned() },
                                )
                                .await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                    Some(Ok(_)) => {} // ignore binary/ping/pong
                }
            }
            _ = ticker.tick(), if subscribed => {
                let samples = runtime.source.next_block(runtime.fft_size);
                let bins = runtime.analyzer.analyze(&samples);
                seq += 1;
                let frame = ServerMessage::Spectrum {
                    seq,
                    sample_rate: runtime.sample_rate,
                    center_hz: runtime.center_hz,
                    bins,
                };
                if send(&mut socket, &frame).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn authenticate(socket: &mut WebSocket, auth: &Auth, timeout: Duration) -> Option<Claims> {
    let first = tokio::time::timeout(timeout, socket.recv()).await;
    let Ok(Some(Ok(Message::Text(text)))) = first else {
        reject(socket, "authentication required").await;
        return None;
    };
    let Ok(ClientMessage::Auth { token }) = serde_json::from_str::<ClientMessage>(text.as_str())
    else {
        reject(socket, "authentication required").await;
        return None;
    };
    if let Ok(claims) = auth.verify(&token) {
        Some(claims)
    } else {
        reject(socket, "invalid token").await;
        None
    }
}

async fn reject(socket: &mut WebSocket, message: &str) {
    let _ = send(
        socket,
        &ServerMessage::Error {
            message: message.to_owned(),
        },
    )
    .await;
    let _ = socket.send(Message::Close(None)).await;
}

async fn send(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), axum::Error> {
    let text = serde_json::to_string(message)
        .unwrap_or_else(|_| String::from(r#"{"type":"error","message":"serialize"}"#));
    socket.send(Message::Text(text.into())).await
}
