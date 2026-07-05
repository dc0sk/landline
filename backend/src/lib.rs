//! landline backend library surface.
//!
//! Exposing the application builder as a library keeps the public backend API
//! integration-testable (NFR-MAINT-02) and maps the crate onto the architecture
//! component model:
//!
//! - [`routes`] + [`app`] — ARC-01 (Axum HTTP/WS server + Tower middleware)
//! - [`auth`] — ARC-02 (authentication, session, RBAC)
//! - [`security`] — ARC-03 (rate limiting, body-size limit, CORS)
//! - [`rig`] — ARC-04 (hamlib/rigctld adapter + command validation)
//! - [`gpio`] — ARC-08 (allowlisted GPIO control)
//! - [`spectrum`] — ARC-06 (FFT pipeline + sample source)
//! - [`ws`] — ARC-01 (authenticated WebSocket telemetry transport)
//! - [`audit`] — ARC-07 (tamper-evident audit log)
//! - [`config`] — ARC-09 (single-file TOML config loader)
//! - [`telemetry`] — ARC-01 Tracing initialisation

pub mod audio;
pub mod audit;
pub mod auth;
pub mod config;
pub mod control;
pub mod gpio;
pub mod rig;
pub mod routes;
pub mod security;
pub mod spectrum;
pub mod telemetry;
pub mod ws;

use std::sync::Arc;
use std::time::Duration;

use axum::routing::get;
use axum::{middleware, Extension, Router};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::audio::{Codec, NoopSink, PcmCodec};
use crate::audit::AuditLog;
use crate::auth::Auth;
use crate::config::Config;
use crate::gpio::GpioController;
use crate::rig::{PttGuard, RigAdapter};
use crate::security::RateLimiter;
use crate::spectrum::{SampleSource, SpectrumAnalyzer, SyntheticSource};
use crate::ws::{AudioRuntime, SpectrumRuntime};

/// Build the top-level Axum application (ARC-01).
///
/// Takes the loaded [`Config`] so the router and middleware stack can be
/// constructed deterministically in `main` and in integration tests. Shared
/// state (auth service ARC-02, rate limiter ARC-03) reaches handlers and
/// extractors via request extensions.
///
/// Serve this with `into_make_service_with_connect_info::<SocketAddr>()` so the
/// rate limiter can key on the peer IP.
pub fn app(config: &Config) -> Router {
    let auth = Arc::new(Auth::from_config(&config.auth));
    let audit = Arc::new(AuditLog::from_config(&config.audit));
    let rig = Arc::new(RigAdapter::from_config(&config.rig));
    let ptt = Arc::new(PttGuard::new(
        Arc::clone(&rig),
        Duration::from_secs(config.rig.ptt_timeout_secs),
    ));
    let gpio = Arc::new(GpioController::from_config(&config.gpio));
    // Synthetic tone at 1/8 of the sample rate (small integer → exact in f32).
    #[allow(clippy::cast_precision_loss)]
    let tone_hz = config.spectrum.sample_rate_hz as f32 / 8.0;
    let source: Arc<dyn SampleSource> = Arc::new(SyntheticSource::new(
        config.spectrum.sample_rate_hz,
        tone_hz,
    ));
    let spectrum = Arc::new(SpectrumRuntime {
        analyzer: Arc::new(SpectrumAnalyzer::new(config.spectrum.fft_size)),
        source,
        update_rate_hz: config.spectrum.update_rate_hz,
        fft_size: config.spectrum.fft_size,
        sample_rate: config.spectrum.sample_rate_hz,
        center_hz: config.spectrum.center_hz,
        max_frame_bytes: config.security.max_body_bytes,
        auth_timeout: Duration::from_secs(5),
    });
    let audio_source: Arc<dyn SampleSource> =
        Arc::new(SyntheticSource::new(config.audio.sample_rate_hz, tone_hz));
    let frame_samples =
        (config.audio.sample_rate_hz as usize * config.audio.frame_ms as usize) / 1000;
    // Opus when built with --features opus (Pi/native), PCM otherwise (default,
    // C-free). A failed Opus init falls back to PCM rather than aborting startup.
    #[cfg(feature = "opus")]
    let codec: Arc<dyn Codec> =
        match crate::audio::OpusCodec::new(config.audio.sample_rate_hz, config.audio.bitrate_bps) {
            Ok(codec) => Arc::new(codec),
            Err(err) => {
                tracing::warn!(error = %err, "opus init failed; falling back to PCM");
                Arc::new(PcmCodec)
            }
        };
    #[cfg(not(feature = "opus"))]
    let codec: Arc<dyn Codec> = Arc::new(PcmCodec);

    let audio_runtime = Arc::new(AudioRuntime {
        source: audio_source,
        codec,
        sink: Arc::new(NoopSink),
        frame_samples: frame_samples.max(1),
        frame_period: Duration::from_millis(u64::from(config.audio.frame_ms.max(1))),
    });
    let limiter = Arc::new(RateLimiter::new(config.security.rate_limit_per_sec));

    // Rate limiting guards the auth + protected API surface (not liveness).
    let protected = auth::router()
        .merge(audit::router())
        .merge(control::router())
        .merge(gpio::router())
        .route("/ws", get(ws::handler))
        .layer(middleware::from_fn_with_state(
            limiter,
            security::rate_limit,
        ));

    routes::router()
        .merge(protected)
        .layer(Extension(auth))
        .layer(Extension(audit))
        .layer(Extension(rig))
        .layer(Extension(ptt))
        .layer(Extension(gpio))
        .layer(Extension(spectrum))
        .layer(Extension(audio_runtime))
        .layer(security::cors_layer(&config.security.allowed_origins))
        .layer(RequestBodyLimitLayer::new(config.security.max_body_bytes))
        .layer(TraceLayer::new_for_http())
        // Outermost: catch any handler panic and return a sanitised 500 (NFR-SEC-09).
        .layer(security::catch_panic_layer())
}
