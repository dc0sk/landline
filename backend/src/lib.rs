//! landline backend library surface.
//!
//! Exposing the application builder as a library keeps the public backend API
//! integration-testable (NFR-MAINT-02) and maps the crate onto the architecture
//! component model:
//!
//! - [`routes`] + [`app`] — ARC-01 (Axum HTTP/WS server + Tower middleware)
//! - [`auth`] — ARC-02 (authentication, session, RBAC)
//! - [`security`] — ARC-03 (rate limiting, body-size limit, CORS)
//! - [`config`] — ARC-09 (single-file TOML config loader)
//! - [`telemetry`] — ARC-01 Tracing initialisation
//!
//! Feature components (ARC-04 rig adapter, ARC-07 audit, ARC-08 GPIO) land in
//! subsequent Phase 1 actions.

pub mod auth;
pub mod config;
pub mod routes;
pub mod security;
pub mod telemetry;

use std::sync::Arc;

use axum::{middleware, Extension, Router};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::auth::Auth;
use crate::config::Config;
use crate::security::RateLimiter;

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
    let limiter = Arc::new(RateLimiter::new(config.security.rate_limit_per_sec));

    // Rate limiting guards the auth + protected API surface (not liveness).
    let protected = auth::router().layer(middleware::from_fn_with_state(
        limiter,
        security::rate_limit,
    ));

    routes::router()
        .merge(protected)
        .layer(Extension(auth))
        .layer(security::cors_layer(&config.security.allowed_origins))
        .layer(RequestBodyLimitLayer::new(config.security.max_body_bytes))
        .layer(TraceLayer::new_for_http())
}
