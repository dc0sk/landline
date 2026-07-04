//! Rig control HTTP handlers (actions A11–A13).
//!
//! The authenticated, RBAC-gated (Operator) surface over the ARC-04 rig adapter:
//! frequency (FR-RIG-01/02), mode (FR-RIG-03/04), and PTT (FR-RIG-05). Every
//! state-changing action is written to the ARC-07 audit log (FR-AUDIT-01), and
//! PTT is protected by the server-side safety timeout (NFR-SEC-07) via
//! [`PttGuard`]. Denied PTT attempts are also audited (TC-RIG-05).

use std::sync::Arc;

use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Router};
use serde::{Deserialize, Serialize};

use crate::audit::{AuditLog, ClientIp};
use crate::auth::{AuthUser, Role};
use crate::rig::{Mode, PttGuard, RigAdapter};

/// Router for the `/api/rig/*` control endpoints.
pub fn router() -> Router {
    Router::new()
        .route("/api/rig/frequency", get(get_frequency).post(set_frequency))
        .route("/api/rig/mode", get(get_mode).post(set_mode))
        .route("/api/rig/ptt", post(set_ptt))
}

#[derive(Serialize)]
struct FrequencyResponse {
    hz: u64,
}

#[derive(Deserialize)]
struct SetFrequencyRequest {
    hz: i64,
}

#[derive(Serialize)]
struct ModeResponse {
    mode: Mode,
}

#[derive(Deserialize)]
struct SetModeRequest {
    mode: String,
    #[serde(default)]
    passband_hz: u32,
}

#[derive(Deserialize)]
struct SetPttRequest {
    transmit: bool,
}

async fn get_frequency(user: AuthUser, Extension(rig): Extension<Arc<RigAdapter>>) -> Response {
    if let Err(err) = user.require(Role::Operator) {
        return err.into_response();
    }
    match rig.get_frequency().await {
        Ok(hz) => Json(FrequencyResponse { hz }).into_response(),
        Err(err) => err.into_response(),
    }
}

async fn set_frequency(
    user: AuthUser,
    Extension(rig): Extension<Arc<RigAdapter>>,
    Extension(audit): Extension<Arc<AuditLog>>,
    ClientIp(ip): ClientIp,
    Json(req): Json<SetFrequencyRequest>,
) -> Response {
    if let Err(err) = user.require(Role::Operator) {
        return err.into_response();
    }
    match rig.set_frequency(req.hz).await {
        Ok(()) => {
            audit.record_action(
                ip.as_deref(),
                &user.claims.sub,
                "rig.set_freq",
                &req.hz.to_string(),
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}

async fn get_mode(user: AuthUser, Extension(rig): Extension<Arc<RigAdapter>>) -> Response {
    if let Err(err) = user.require(Role::Operator) {
        return err.into_response();
    }
    match rig.get_mode().await {
        Ok(mode) => Json(ModeResponse { mode }).into_response(),
        Err(err) => err.into_response(),
    }
}

async fn set_mode(
    user: AuthUser,
    Extension(rig): Extension<Arc<RigAdapter>>,
    Extension(audit): Extension<Arc<AuditLog>>,
    ClientIp(ip): ClientIp,
    Json(req): Json<SetModeRequest>,
) -> Response {
    if let Err(err) = user.require(Role::Operator) {
        return err.into_response();
    }
    // Parse against the allowlist here so an invalid token is a clean 400
    // (NFR-SEC-08) rather than a JSON deserialisation error.
    let mode = match Mode::parse(&req.mode) {
        Ok(mode) => mode,
        Err(err) => return err.into_response(),
    };
    match rig.set_mode(mode, req.passband_hz).await {
        Ok(()) => {
            audit.record_action(
                ip.as_deref(),
                &user.claims.sub,
                "rig.set_mode",
                &format!("{} {}", mode.as_str(), req.passband_hz),
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}

async fn set_ptt(
    user: AuthUser,
    Extension(ptt): Extension<Arc<PttGuard>>,
    Extension(audit): Extension<Arc<AuditLog>>,
    ClientIp(ip): ClientIp,
    Json(req): Json<SetPttRequest>,
) -> Response {
    // PTT requires Operator (NFR-SEC-07); a denied attempt is itself audited
    // (TC-RIG-05).
    if let Err(err) = user.require(Role::Operator) {
        audit.record_denied(ip.as_deref(), &user.claims.sub, "rig.ptt");
        return err.into_response();
    }
    let result = if req.transmit {
        ptt.activate().await
    } else {
        ptt.deactivate().await
    };
    match result {
        Ok(()) => {
            let state = if req.transmit { "on" } else { "off" };
            audit.record_action(ip.as_deref(), &user.claims.sub, "rig.ptt", state);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}
