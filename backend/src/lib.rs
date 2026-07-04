//! landline backend library surface.
//!
//! Exposing the application builder as a library keeps the public backend API
//! integration-testable (NFR-MAINT-02) and maps the crate onto the architecture
//! component model:
//!
//! - [`routes`] + [`app`] — ARC-01 (Axum HTTP/WS server + Tower middleware)
//! - [`auth`] — ARC-02 (authentication, session, RBAC)
//! - [`config`] — ARC-09 (single-file TOML config loader)
//! - [`telemetry`] — ARC-01 Tracing initialisation
//!
//! Feature components (ARC-03 security middleware, ARC-04 rig adapter, ARC-07
//! audit, ARC-08 GPIO) land in subsequent Phase 1 actions.

pub mod auth;
pub mod config;
pub mod routes;
pub mod telemetry;

use std::sync::Arc;

use axum::{Extension, Router};
use tower_http::trace::TraceLayer;

use crate::auth::Auth;
use crate::config::Config;

/// Build the top-level Axum application (ARC-01).
///
/// Takes the loaded [`Config`] so the router and middleware stack can be
/// constructed deterministically in `main` and in integration tests. The auth
/// service (ARC-02) is shared with handlers and the [`auth::AuthUser`] extractor
/// via a request extension.
pub fn app(config: &Config) -> Router {
    let auth = Arc::new(Auth::from_config(&config.auth));
    routes::router()
        .merge(auth::router())
        .layer(Extension(auth))
        .layer(TraceLayer::new_for_http())
}
