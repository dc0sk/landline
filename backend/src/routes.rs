//! HTTP routes for the ARC-01 server.
//!
//! The Phase 1 walking skeleton exposes only unauthenticated liveness/version
//! endpoints. Feature routes — auth (ARC-02), rig control (ARC-04), spectrum
//! (ARC-06), audio (ARC-05) — are added by later Phase 1/2/3 actions and will
//! sit behind the auth and security middleware.

use axum::{routing::get, Json, Router};
use serde::Serialize;

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
}

async fn healthz() -> Json<Health> {
    Json(Health { status: "ok" })
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
