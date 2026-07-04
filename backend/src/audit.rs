//! ARC-07 tamper-evident audit log (action A9).
//!
//! Realises:
//! - **FR-AUDIT-01** — a tamper-evident log of rig state-changing actions. Each
//!   event carries a SHA-256 hash chained over the previous event, so altering
//!   or deleting any entry breaks the chain ([`verify_chain`]).
//! - **FR-AUDIT-02** — every event records timestamp, client IP, user identity,
//!   action, and parameter values.
//! - **FR-AUDIT-03** — retention ≥ 30 days: the app appends to a durable file;
//!   rotation/retention is enforced by the deployment (config `retention_days`).
//! - **FR-AUDIT-04** — authentication failures are logged with client IP and
//!   timestamp; passwords are never passed to the log (NFR-SEC-12).
//!
//! The recent-events window is exposed (Admin only) at `GET /api/audit`.

use std::collections::VecDeque;
use std::convert::Infallible;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::auth::{AuthError, AuthUser, Role};

/// Genesis hash seeding the chain (no predecessor).
const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/// How many recent events to keep queryable in memory (the file is durable).
const RECENT_CAP: usize = 1000;

/// Outcome of an audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// The action succeeded.
    Success,
    /// The action was rejected or failed.
    Failure,
}

impl Outcome {
    fn as_str(self) -> &'static str {
        match self {
            Outcome::Success => "success",
            Outcome::Failure => "failure",
        }
    }
}

/// A single audit event (FR-AUDIT-02) with tamper-evidence fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Monotonic sequence number.
    pub seq: u64,
    /// Event time (unix seconds).
    pub timestamp: u64,
    /// Client IP, if known.
    pub client_ip: Option<String>,
    /// Acting user identity, if known.
    pub user: Option<String>,
    /// Action name (e.g. `rig.set_freq`, `auth.login`).
    pub action: String,
    /// Parameter values, rendered as a string. Never contains secrets.
    pub params: String,
    /// Outcome.
    pub outcome: Outcome,
    /// Hash of the previous event (chain link).
    pub prev_hash: String,
    /// SHA-256 over this event's fields + `prev_hash`.
    pub hash: String,
}

struct Inner {
    seq: u64,
    prev_hash: String,
    file: Option<BufWriter<std::fs::File>>,
    recent: VecDeque<AuditEvent>,
}

/// The audit log service (ARC-07).
pub struct AuditLog {
    inner: Mutex<Inner>,
}

impl AuditLog {
    /// An in-memory-only audit log (development / tests).
    #[must_use]
    pub fn in_memory() -> Self {
        Self::with_file(None)
    }

    /// Build from configuration: a durable append file when `path` is set,
    /// otherwise in-memory. A file that cannot be opened degrades to in-memory
    /// with a warning rather than failing startup.
    #[must_use]
    pub fn from_config(config: &crate::config::AuditConfig) -> Self {
        let Some(path) = config.path.as_ref() else {
            return Self::in_memory();
        };
        match Self::open_file(Path::new(path)) {
            Ok(file) => Self::with_file(Some(file)),
            Err(err) => {
                tracing::warn!(error = %err, path, "audit: falling back to in-memory log");
                Self::in_memory()
            }
        }
    }

    fn open_file(path: &Path) -> std::io::Result<BufWriter<std::fs::File>> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(BufWriter::new(file))
    }

    fn with_file(file: Option<BufWriter<std::fs::File>>) -> Self {
        Self {
            inner: Mutex::new(Inner {
                seq: 0,
                prev_hash: GENESIS.to_owned(),
                file,
                recent: VecDeque::new(),
            }),
        }
    }

    /// Record a rig state-changing action (FR-AUDIT-01/02).
    pub fn record_action(
        &self,
        client_ip: Option<&str>,
        user: &str,
        action: &str,
        params: &str,
    ) -> AuditEvent {
        self.record(client_ip, Some(user), action, params, Outcome::Success)
    }

    /// Record a successful login (FR-AUDIT-04 companion).
    pub fn record_login(&self, client_ip: Option<&str>, user: &str) -> AuditEvent {
        self.record(client_ip, Some(user), "auth.login", "", Outcome::Success)
    }

    /// Record an authentication failure (FR-AUDIT-04). The attempted user name
    /// is recorded; the password is never provided to this method (NFR-SEC-12).
    pub fn record_auth_failure(&self, client_ip: Option<&str>, attempted_user: &str) -> AuditEvent {
        self.record(
            client_ip,
            Some(attempted_user),
            "auth.login",
            "",
            Outcome::Failure,
        )
    }

    /// A snapshot of the recent-events window (newest last).
    #[must_use]
    pub fn recent(&self) -> Vec<AuditEvent> {
        lock(&self.inner).recent.iter().cloned().collect()
    }

    fn record(
        &self,
        client_ip: Option<&str>,
        user: Option<&str>,
        action: &str,
        params: &str,
        outcome: Outcome,
    ) -> AuditEvent {
        let mut inner = lock(&self.inner);
        let seq = inner.seq;
        let timestamp = now_unix();
        let client_ip = client_ip.map(ToOwned::to_owned);
        let user = user.map(ToOwned::to_owned);
        let prev_hash = inner.prev_hash.clone();
        let hash = event_hash(
            seq,
            timestamp,
            client_ip.as_deref(),
            user.as_deref(),
            action,
            params,
            outcome,
            &prev_hash,
        );
        let event = AuditEvent {
            seq,
            timestamp,
            client_ip,
            user,
            action: action.to_owned(),
            params: params.to_owned(),
            outcome,
            prev_hash,
            hash: hash.clone(),
        };

        inner.seq += 1;
        inner.prev_hash = hash;
        if let Some(file) = inner.file.as_mut() {
            if let Ok(line) = serde_json::to_string(&event) {
                let _ = writeln!(file, "{line}");
                let _ = file.flush();
            }
        }
        inner.recent.push_back(event.clone());
        while inner.recent.len() > RECENT_CAP {
            inner.recent.pop_front();
        }
        tracing::info!(
            seq = event.seq,
            action = %event.action,
            outcome = event.outcome.as_str(),
            "audit"
        );
        event
    }
}

/// Verify the tamper-evidence of a slice of consecutive events: every event's
/// hash must recompute, and each must chain to its predecessor. Returns `false`
/// if any field was altered or an event was inserted/removed.
#[must_use]
pub fn verify_chain(events: &[AuditEvent]) -> bool {
    let mut prev: Option<&AuditEvent> = None;
    for event in events {
        let recomputed = event_hash(
            event.seq,
            event.timestamp,
            event.client_ip.as_deref(),
            event.user.as_deref(),
            &event.action,
            &event.params,
            event.outcome,
            &event.prev_hash,
        );
        if recomputed != event.hash {
            return false;
        }
        if let Some(prev) = prev {
            if event.prev_hash != prev.hash || event.seq != prev.seq + 1 {
                return false;
            }
        }
        prev = Some(event);
    }
    true
}

/// Extractor for the client IP, read from the [`ConnectInfo`] extension. Never
/// fails: yields `None` when connection info is unavailable (e.g. in tests).
pub struct ClientIp(pub Option<String>);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|info| info.0.ip().to_string()),
        ))
    }
}

/// Router exposing the Admin-only audit view.
pub fn router() -> Router {
    Router::new().route("/api/audit", get(recent_events))
}

async fn recent_events(
    user: AuthUser,
    Extension(audit): Extension<Arc<AuditLog>>,
) -> Result<Json<Vec<AuditEvent>>, AuthError> {
    user.require(Role::Admin)?;
    Ok(Json(audit.recent()))
}

#[allow(clippy::too_many_arguments)]
fn event_hash(
    seq: u64,
    timestamp: u64,
    client_ip: Option<&str>,
    user: Option<&str>,
    action: &str,
    params: &str,
    outcome: Outcome,
    prev_hash: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seq.to_le_bytes());
    hasher.update(timestamp.to_le_bytes());
    for field in [
        client_ip.unwrap_or("-"),
        user.unwrap_or("-"),
        action,
        params,
        outcome.as_str(),
        prev_hash,
    ] {
        hasher.update(field.as_bytes());
        hasher.update([0x1f]); // unit separator to disambiguate boundaries
    }
    hex(hasher.finalize().as_slice())
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{verify_chain, AuditLog, Outcome};

    #[test]
    fn action_event_has_all_fields() {
        // FR-AUDIT-02
        let log = AuditLog::in_memory();
        let event = log.record_action(Some("10.0.0.5"), "op", "rig.set_freq", "14074000");
        assert_eq!(event.client_ip.as_deref(), Some("10.0.0.5"));
        assert_eq!(event.user.as_deref(), Some("op"));
        assert_eq!(event.action, "rig.set_freq");
        assert_eq!(event.params, "14074000");
        assert_eq!(event.outcome, Outcome::Success);
        assert!(event.timestamp > 0);
    }

    #[test]
    fn auth_failure_records_ip_and_no_password() {
        // FR-AUDIT-04 / NFR-SEC-12: password is never given to the audit API.
        let log = AuditLog::in_memory();
        let event = log.record_auth_failure(Some("203.0.113.9"), "op");
        assert_eq!(event.outcome, Outcome::Failure);
        assert_eq!(event.client_ip.as_deref(), Some("203.0.113.9"));
        assert_eq!(event.action, "auth.login");
        assert!(event.params.is_empty());
    }

    #[test]
    fn chain_verifies_and_detects_tampering() {
        // FR-AUDIT-01
        let log = AuditLog::in_memory();
        log.record_action(Some("10.0.0.1"), "op", "rig.set_mode", "USB");
        log.record_action(Some("10.0.0.1"), "op", "rig.ptt", "on");
        log.record_auth_failure(Some("10.0.0.2"), "mallory");
        let mut events = log.recent();
        assert_eq!(events.len(), 3);
        assert!(verify_chain(&events));

        // Tamper: change a recorded parameter without recomputing the hash.
        events[1].params = "off".to_owned();
        assert!(!verify_chain(&events));
    }

    #[test]
    fn removing_an_event_breaks_the_chain() {
        let log = AuditLog::in_memory();
        log.record_action(Some("10.0.0.1"), "op", "a", "1");
        log.record_action(Some("10.0.0.1"), "op", "b", "2");
        log.record_action(Some("10.0.0.1"), "op", "c", "3");
        let mut events = log.recent();
        events.remove(1); // drop the middle event
        assert!(!verify_chain(&events));
    }
}
