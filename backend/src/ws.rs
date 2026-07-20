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

use crate::audio::{f32_to_pcm16, split_frame, AudioSink, Codec};
use crate::auth::{Auth, Claims, Role};
use crate::spectrum::{SampleSource, SpectrumAnalyzer};

/// Shared runtime for the WS audio stream (ARC-05), built from config. Audio is
/// delivered as binary frames (8-byte little-endian sequence header + encoded
/// payload); the real capture source and codec plug in behind these seams.
pub struct AudioRuntime {
    /// Sample source (synthetic today; CPAL capture is the Phase-3 HIL adapter).
    pub source: Arc<dyn SampleSource>,
    /// Encoder/decoder (PCM default; libopus is the native adapter).
    pub codec: Arc<dyn Codec>,
    /// Sink for received transmit audio (rig TX; no-op until the Pi adapter).
    pub sink: Arc<dyn AudioSink>,
    /// Samples per audio frame.
    pub frame_samples: usize,
    /// One frame per this period (e.g. 20 ms).
    pub frame_period: Duration,
}

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
    Audio,
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
    Extension(spectrum): Extension<Arc<SpectrumRuntime>>,
    Extension(audio): Extension<Arc<AudioRuntime>>,
) -> Response {
    let max = spectrum.max_frame_bytes;
    ws.max_message_size(max)
        .max_frame_size(max)
        .on_upgrade(move |socket| session(socket, auth, spectrum, audio))
}

async fn session(
    mut socket: WebSocket,
    auth: Arc<Auth>,
    spectrum: Arc<SpectrumRuntime>,
    audio: Arc<AudioRuntime>,
) {
    let Some((claims, token)) = authenticate(&mut socket, &auth, spectrum.auth_timeout).await
    else {
        return;
    };
    // All authenticated roles may view telemetry / hear RX audio (STK-01/02);
    // state-changing control is not exposed here (per-message-type ACL — ADR-02).
    if send(&mut socket, &ServerMessage::Ready { role: claims.role })
        .await
        .is_err()
    {
        return;
    }

    let mut spectrum_on = false;
    let mut audio_on = false;
    let mut spectrum_seq: u64 = 0;
    let mut audio_seq: u64 = 0;
    let rate = spectrum.update_rate_hz.clamp(1.0, 10.0);
    let mut spectrum_ticker = tokio::time::interval(Duration::from_secs_f32(1.0 / rate));
    let mut audio_ticker = tokio::time::interval(audio.frame_period);
    let mut auth_ticker = tokio::time::interval(AUTH_RECHECK_INTERVAL);

    loop {
        tokio::select! {
            // Re-check the credential for the life of the socket (FR-AUTH-02/05).
            // Authenticating only at connect would let a logged-out or expired
            // token keep streaming until the client chose to disconnect.
            _ = auth_ticker.tick() => {
                if auth.verify(&token).is_err() {
                    reject(&mut socket, "session ended").await;
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(text.as_str()) {
                            Ok(ClientMessage::Subscribe { stream }) => match stream {
                                StreamKind::Spectrum => spectrum_on = true,
                                StreamKind::Audio => audio_on = true,
                            },
                            Ok(ClientMessage::Unsubscribe { stream }) => match stream {
                                StreamKind::Spectrum => spectrum_on = false,
                                StreamKind::Audio => audio_on = false,
                            },
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
                    Some(Ok(Message::Binary(bytes))) => {
                        // Transmit audio (FR-AUD-02) requires Operator; any other
                        // role's TX frames are dropped (per-message-type ACL).
                        if claims.role.allows(Role::Operator) {
                            if let Some((_seq, payload)) = split_frame(&bytes) {
                                audio.sink.accept(&audio.codec.decode(payload));
                            }
                        }
                    }
                    Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                    Some(Ok(_)) => {} // ignore ping/pong
                }
            }
            _ = spectrum_ticker.tick(), if spectrum_on => {
                let samples = spectrum.source.next_block(spectrum.fft_size);
                let bins = spectrum.analyzer.analyze(&samples);
                spectrum_seq += 1;
                let frame = ServerMessage::Spectrum {
                    seq: spectrum_seq,
                    sample_rate: spectrum.sample_rate,
                    center_hz: spectrum.center_hz,
                    bins,
                };
                if send(&mut socket, &frame).await.is_err() {
                    break;
                }
            }
            _ = audio_ticker.tick(), if audio_on => {
                audio_seq += 1;
                let payload = audio.codec.encode(&f32_to_pcm16(
                    &audio.source.next_block(audio.frame_samples),
                ));
                // Binary frame: 8-byte LE sequence header + encoded payload.
                let mut frame = Vec::with_capacity(8 + payload.len());
                frame.extend_from_slice(&audio_seq.to_le_bytes());
                frame.extend_from_slice(&payload);
                if socket.send(Message::Binary(frame.into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// How often an open session re-verifies its access token. Verification is a
/// local HMAC check plus a map lookup, so this is cheap; the interval bounds how
/// long a revoked or expired credential can keep streaming.
const AUTH_RECHECK_INTERVAL: Duration = Duration::from_secs(1);

/// Authenticate the first message. Returns the claims **and the token**, which
/// the session keeps so it can re-verify expiry and revocation as it runs.
async fn authenticate(
    socket: &mut WebSocket,
    auth: &Auth,
    timeout: Duration,
) -> Option<(Claims, String)> {
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
        Some((claims, token))
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
