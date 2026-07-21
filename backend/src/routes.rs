//! HTTP routes for the ARC-01 server.
//!
//! The Phase 1 walking skeleton exposes only unauthenticated liveness/version
//! endpoints. Feature routes — auth (ARC-02), rig control (ARC-04), spectrum
//! (ARC-06), audio (ARC-05) — are added by later Phase 1/2/3 actions and will
//! sit behind the auth and security middleware.

use std::sync::Arc;

use axum::{routing::get, Extension, Json, Router};
use serde::Serialize;

use crate::gpio::GpioController;

/// Build the application router.
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/version", get(version))
}

/// Liveness probe response.
#[derive(Serialize)]
struct Health {
    status: &'static str,
    /// Subsystems running degraded. Empty when everything is nominal. The probe
    /// still reports `ok` — a degraded GPIO chip must not fail the health check
    /// and get the whole station restarted — but the fault is visible to anyone
    /// looking, instead of only appearing in a startup log line nobody re-reads.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    degraded: Vec<&'static str>,
}

async fn healthz(gpio: Option<Extension<Arc<GpioController>>>) -> Json<Health> {
    let mut degraded = Vec::new();
    if gpio.is_some_and(|Extension(gpio)| gpio.is_degraded()) {
        degraded.push("gpio");
    }
    Json(Health {
        status: "ok",
        degraded,
    })
}

/// Version/build metadata response.
#[derive(Serialize)]
struct Version {
    name: &'static str,
    version: &'static str,
}

async fn version() -> Json<Version> {
    Json(Version {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    })
}
