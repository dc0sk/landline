//! landline backend library surface.
//!
//! Exposing the application builder as a library keeps the public backend API
//! integration-testable (NFR-MAINT-02) and maps the crate onto the architecture
//! component model:
//!
//! - [`routes`] + [`app`] — ARC-01 (Axum HTTP/WS server + Tower middleware)
//! - [`config`] — ARC-09 (single-file TOML config loader)
//! - [`telemetry`] — ARC-01 Tracing initialisation
//!
//! Feature components (ARC-02 auth, ARC-03 security middleware, ARC-04 rig
//! adapter, ARC-07 audit, ARC-08 GPIO) land in subsequent Phase 1 actions.

pub mod config;
pub mod routes;
pub mod telemetry;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::config::Config;

/// Build the top-level Axum application (ARC-01).
///
/// Takes the loaded [`Config`] so the router and middleware stack can be
/// constructed deterministically in `main` and in integration tests. The
/// config is currently the seam for feature routes/middleware added by later
/// Phase 1 actions.
pub fn app(config: &Config) -> Router {
    let _ = config;
    routes::router().layer(TraceLayer::new_for_http())
}
