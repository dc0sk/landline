//! ARC-02 authentication, session, and RBAC (action A6).
//!
//! Realises:
//! - **FR-AUTH-01** — authentication required before protected endpoints (the
//!   [`AuthUser`] extractor).
//! - **FR-AUTH-02** — short-lived access tokens with expiry (HS256 JWT, `exp`).
//! - **FR-AUTH-03** — token refresh without full re-authentication, with
//!   refresh-token rotation.
//! - **FR-AUTH-04** — role-based access control ([`Role`], [`AuthUser::require`]).
//! - **FR-AUTH-05** — session invalidation on logout or expiry (refresh-session
//!   store + short-lived access-token revocation set).
//! - **NFR-SEC-02** — cryptographically random tokens (256-bit refresh tokens
//!   and signing secret via the OS CSPRNG).
//! - **NFR-SEC-12** — credentials never logged; only argon2 hashes are stored.
//!
//! Transport confidentiality (NFR-SEC-01) is provided by TLS/WSS at the reverse
//! proxy and by WireGuard/Tailscale for split-host (ADR-05, ADR-08); this module
//! is the app-layer authn/authz.
//!
//! The JWT is signed with HS256 using pure-Rust primitives (`hmac`/`sha2`) rather
//! than a `ring`-based library, keeping the aarch64 cross-build free of a C
//! toolchain (see ADR-08 consequences).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tokio::sync::{Semaphore, SemaphorePermit};

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::audit::{AuditLog, ClientIp};
use crate::config::AuthConfig;

type HmacSha256 = Hmac<Sha256>;

/// The exact JWT header this service emits and accepts. Pinning it rejects
/// algorithm-confusion attempts (e.g. `alg: none`).
const HEADER_JSON: &[u8] = br#"{"alg":"HS256","typ":"JWT"}"#;

/// A role in the RBAC model (FR-AUTH-04). Privilege increases Observer <
/// Operator < Admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Full administrative control.
    Admin,
    /// May control the rig (tune, mode, PTT).
    Operator,
    /// Read-only: may monitor audio/spectrum/meters but not transmit or control.
    Observer,
}

impl Role {
    /// Whether this role satisfies a `required` minimum role.
    #[must_use]
    pub fn allows(self, required: Role) -> bool {
        self.rank() >= required.rank()
    }

    fn rank(self) -> u8 {
        match self {
            Role::Admin => 3,
            Role::Operator => 2,
            Role::Observer => 1,
        }
    }
}

/// JWT claims carried by an access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the login name.
    pub sub: String,
    /// Granted role.
    pub role: Role,
    /// Expiry (unix seconds).
    pub exp: u64,
    /// Issued-at (unix seconds).
    pub iat: u64,
    /// Unique token id, used for revocation on logout.
    pub jti: String,
}

/// A freshly issued access + refresh token pair.
#[derive(Debug, Clone)]
pub struct TokenPair {
    /// Short-lived HS256 access token (JWT).
    pub access_token: String,
    /// Opaque refresh token (256-bit random).
    pub refresh_token: String,
    /// Access-token lifetime in seconds.
    pub expires_in: u64,
    /// Role granted to the session.
    pub role: Role,
}

/// Errors from authentication and authorisation.
///
/// Rendered to clients as sanitised status codes only (NFR-SEC-09): all
/// authentication failures collapse to `401 unauthorized` so they cannot be used
/// to distinguish "no such user" from "wrong password" from "expired token".
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// No/!malformed `Authorization` header.
    #[error("missing or malformed authorization")]
    Missing,
    /// Signature invalid, malformed, or revoked token.
    #[error("invalid token")]
    Invalid,
    /// Token past its expiry.
    #[error("token expired")]
    Expired,
    /// Login name/password did not match.
    #[error("invalid credentials")]
    InvalidCredentials,
    /// Authenticated but the role is insufficient (RBAC denial).
    #[error("insufficient role")]
    Forbidden,
    /// Server-side fault (e.g. auth state not wired up).
    #[error("internal error")]
    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AuthError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            AuthError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
            // All authentication failures render identically (no enumeration).
            AuthError::Missing
            | AuthError::Invalid
            | AuthError::Expired
            | AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "unauthorized"),
        };
        (status, body).into_response()
    }
}

struct UserRecord {
    role: Role,
    password_hash: String,
}

struct Session {
    sub: String,
    role: Role,
    expires: u64,
}

/// The authentication service (ARC-02): user store, signing secret, and the
/// in-memory session/revocation stores.
pub struct Auth {
    secret: [u8; 32],
    access_ttl: u64,
    refresh_ttl: u64,
    users: HashMap<String, UserRecord>,
    /// refresh token -> session
    sessions: Mutex<HashMap<String, Session>>,
    /// revoked access-token `jti` -> its `exp` (purged lazily)
    revoked: Mutex<HashMap<String, u64>>,
    /// PHC hash of a fixed throwaway password, verified against on the
    /// unknown-user path so it costs the same as a real verification.
    dummy_hash: String,
    /// Bounds concurrent password hashing. Argon2's default parameters allocate
    /// ~19 MiB per verification, so an unbounded number of in-flight logins is
    /// a memory-exhaustion lever for an unauthenticated caller.
    login_slots: Semaphore,
}

/// Fixed password behind the dummy hash. Never a valid credential: it is only
/// verified against on the unknown-user path, which always refuses.
const DUMMY_PASSWORD: &str = "landline-dummy-verification-password";

/// Concurrent Argon2 verifications allowed at once (~19 MiB each).
const MAX_CONCURRENT_LOGINS: usize = 4;

impl Auth {
    /// Build the auth service from configuration.
    ///
    /// The HS256 signing secret is 256 bits of OS CSPRNG output (NFR-SEC-02),
    /// generated fresh per process start.
    ///
    /// # Panics
    /// Panics if the operating-system CSPRNG is unavailable.
    #[must_use]
    pub fn from_config(config: &AuthConfig) -> Self {
        let users = config
            .users
            .iter()
            .map(|u| {
                (
                    u.name.clone(),
                    UserRecord {
                        role: u.role,
                        password_hash: u.password_hash.clone(),
                    },
                )
            })
            .collect();
        Self {
            secret: random_bytes::<32>(),
            access_ttl: config.access_ttl_secs,
            refresh_ttl: config.refresh_ttl_secs,
            users,
            sessions: Mutex::new(HashMap::new()),
            revoked: Mutex::new(HashMap::new()),
            // Computed once at startup; the password is arbitrary and never
            // matches, since it is only ever verified against on a path that
            // returns InvalidCredentials regardless.
            dummy_hash: hash_password(DUMMY_PASSWORD).unwrap_or_default(),
            login_slots: Semaphore::new(MAX_CONCURRENT_LOGINS),
        }
    }

    /// Acquire a password-hashing slot, bounding concurrent Argon2 work.
    ///
    /// # Errors
    /// Returns [`AuthError::Internal`] if the semaphore has been closed.
    pub async fn login_permit(&self) -> Result<SemaphorePermit<'_>, AuthError> {
        self.login_slots
            .acquire()
            .await
            .map_err(|_| AuthError::Internal)
    }

    /// Authenticate a user and issue a token pair (FR-AUTH-01/02).
    ///
    /// # Errors
    /// Returns [`AuthError::InvalidCredentials`] if the name is unknown or the
    /// password does not verify, or [`AuthError::Internal`] if a stored hash is
    /// malformed.
    pub fn login(&self, name: &str, password: &str) -> Result<TokenPair, AuthError> {
        let Some(user) = self.users.get(name) else {
            // Verify against a dummy hash before refusing. Returning early on an
            // unknown name made the response time a user-enumeration oracle:
            // measured 25 ns for an unknown user against 19.3 ms for a known one
            // with the wrong password — a ratio of ~773,000, readable over any
            // network. Both paths must do the same work.
            self.verify_dummy(password);
            return Err(AuthError::InvalidCredentials);
        };
        let parsed = PasswordHash::new(&user.password_hash).map_err(|_| AuthError::Internal)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AuthError::InvalidCredentials)?;
        Ok(self.issue(name, user.role))
    }

    /// Burn the same work a real verification costs, then discard the result.
    fn verify_dummy(&self, password: &str) {
        if let Ok(parsed) = PasswordHash::new(&self.dummy_hash) {
            let _ = Argon2::default().verify_password(password.as_bytes(), &parsed);
        }
    }

    /// Exchange a valid refresh token for a new token pair, rotating the refresh
    /// token so the presented one is invalidated (FR-AUTH-03).
    ///
    /// # Errors
    /// Returns [`AuthError::Invalid`] if the refresh token is unknown/already
    /// used, or [`AuthError::Expired`] if the refresh session has expired.
    pub fn refresh(&self, refresh_token: &str) -> Result<TokenPair, AuthError> {
        let session = {
            let mut sessions = lock(&self.sessions);
            sessions.remove(refresh_token).ok_or(AuthError::Invalid)?
        };
        if session.expires <= now_unix() {
            return Err(AuthError::Expired);
        }
        Ok(self.issue(&session.sub, session.role))
    }

    /// Invalidate a session on logout (FR-AUTH-05): drop the refresh session and
    /// revoke the presented access token's `jti` until it would expire anyway.
    pub fn logout(&self, claims: &Claims, refresh_token: &str) {
        lock(&self.sessions).remove(refresh_token);
        lock(&self.revoked).insert(claims.jti.clone(), claims.exp);
    }

    /// Verify an access token: signature, expiry, and revocation (FR-AUTH-01/05).
    ///
    /// # Errors
    /// Returns [`AuthError::Invalid`] if the signature/format is bad or the token
    /// was revoked, or [`AuthError::Expired`] if past `exp`.
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = decode_jwt(&self.secret, token)?;
        if self.is_revoked(&claims.jti) {
            return Err(AuthError::Invalid);
        }
        Ok(claims)
    }

    fn issue(&self, sub: &str, role: Role) -> TokenPair {
        let now = now_unix();
        let claims = Claims {
            sub: sub.to_owned(),
            role,
            iat: now,
            exp: now + self.access_ttl,
            jti: URL_SAFE_NO_PAD.encode(random_bytes::<16>()),
        };
        let access_token = encode_jwt(&self.secret, &claims);
        let refresh_token = URL_SAFE_NO_PAD.encode(random_bytes::<32>());
        lock(&self.sessions).insert(
            refresh_token.clone(),
            Session {
                sub: sub.to_owned(),
                role,
                expires: now + self.refresh_ttl,
            },
        );
        TokenPair {
            access_token,
            refresh_token,
            expires_in: self.access_ttl,
            role,
        }
    }

    fn is_revoked(&self, jti: &str) -> bool {
        let mut revoked = lock(&self.revoked);
        let now = now_unix();
        revoked.retain(|_, &mut exp| exp > now);
        revoked.contains_key(jti)
    }
}

/// Generate an argon2 password hash (PHC string) for use in config `users`.
///
/// # Errors
/// Returns [`AuthError::Internal`] if the OS CSPRNG or the hasher fails.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).map_err(|_| AuthError::Internal)?;
    let salt = SaltString::encode_b64(&salt).map_err(|_| AuthError::Internal)?;
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthError::Internal)?;
    Ok(hash.to_string())
}

/// Extractor for an authenticated request (FR-AUTH-01).
///
/// Reads a `Bearer` token from the `Authorization` header and verifies it against
/// the [`Auth`] service placed in request extensions. Rejects with
/// [`AuthError`] (401) when absent/invalid.
pub struct AuthUser {
    /// Verified claims for the request.
    pub claims: Claims,
}

impl AuthUser {
    /// Enforce a minimum role (FR-AUTH-04).
    ///
    /// # Errors
    /// Returns [`AuthError::Forbidden`] if the user's role does not satisfy
    /// `required`.
    pub fn require(&self, required: Role) -> Result<(), AuthError> {
        if self.claims.role.allows(required) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts
            .extensions
            .get::<Arc<Auth>>()
            .cloned()
            .ok_or(AuthError::Internal)?;
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AuthError::Missing)?;
        let claims = auth.verify(token)?;
        Ok(AuthUser { claims })
    }
}

/// Router for the auth endpoints and the (placeholder) protected API surface.
///
/// `/api/*` routes here are the seam for real control routes (rig, spectrum,
/// audio) landing in later actions; they demonstrate the auth + RBAC guards.
pub fn router() -> Router {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/operator-ping", get(operator_ping))
}

#[derive(Deserialize)]
struct LoginRequest {
    name: String,
    password: String,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    role: Role,
}

impl From<TokenPair> for TokenResponse {
    fn from(pair: TokenPair) -> Self {
        Self {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
            expires_in: pair.expires_in,
            role: pair.role,
        }
    }
}

async fn login(
    Extension(auth): Extension<Arc<Auth>>,
    Extension(audit): Extension<Arc<AuditLog>>,
    ClientIp(ip): ClientIp,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AuthError> {
    // Argon2 is deliberately expensive (~19 MiB, ~20 ms). Running it directly on
    // a Tokio worker blocks that thread for the whole verification, stalling
    // unrelated requests — measured ~400x scheduling-latency growth under four
    // concurrent logins on a 4-worker runtime. Hash on the blocking pool, and
    // bound how many run at once so an unauthenticated caller cannot pin
    // gigabytes by opening enough connections.
    let _permit = auth.login_permit().await?;
    let hashing = {
        let auth = Arc::clone(&auth);
        let name = req.name.clone();
        let password = req.password.clone();
        tokio::task::spawn_blocking(move || auth.login(&name, &password))
    };
    let outcome = hashing.await.map_err(|_| AuthError::Internal)?;

    // NFR-SEC-12: never log the password; audit records only IP, user, outcome.
    match outcome {
        Ok(pair) => {
            audit.record_login(ip.as_deref(), &req.name);
            Ok(Json(pair.into()))
        }
        Err(err) => {
            audit.record_auth_failure(ip.as_deref(), &req.name); // FR-AUDIT-04
            Err(err)
        }
    }
}

async fn refresh(
    Extension(auth): Extension<Arc<Auth>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AuthError> {
    let pair = auth.refresh(&req.refresh_token)?;
    Ok(Json(pair.into()))
}

#[derive(Serialize)]
struct MeResponse {
    sub: String,
    role: Role,
}

async fn logout(
    user: AuthUser,
    Extension(auth): Extension<Arc<Auth>>,
    Json(req): Json<RefreshRequest>,
) -> StatusCode {
    auth.logout(&user.claims, &req.refresh_token);
    StatusCode::NO_CONTENT
}

async fn me(user: AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        sub: user.claims.sub,
        role: user.claims.role,
    })
}

async fn operator_ping(user: AuthUser) -> Result<StatusCode, AuthError> {
    user.require(Role::Operator)?;
    Ok(StatusCode::OK)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // Recover from poisoning rather than panic: a poisoned auth lock must not
    // take down the service.
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::getrandom(&mut bytes).expect("OS CSPRNG unavailable");
    bytes
}

fn hmac_sha256(secret: &[u8], message: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

fn encode_jwt(secret: &[u8], claims: &Claims) -> String {
    let header = URL_SAFE_NO_PAD.encode(HEADER_JSON);
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).expect("claims serialize"));
    let signing_input = format!("{header}.{payload}");
    let signature = URL_SAFE_NO_PAD.encode(hmac_sha256(secret, signing_input.as_bytes()));
    format!("{signing_input}.{signature}")
}

fn decode_jwt(secret: &[u8], token: &str) -> Result<Claims, AuthError> {
    let mut parts = token.split('.');
    let (header, payload, signature) = match (parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(p), Some(s)) if parts.next().is_none() => (h, p, s),
        _ => return Err(AuthError::Invalid),
    };

    // Pin the header to reject algorithm-confusion (e.g. `alg: none`).
    let header_bytes = URL_SAFE_NO_PAD
        .decode(header)
        .map_err(|_| AuthError::Invalid)?;
    if header_bytes != HEADER_JSON {
        return Err(AuthError::Invalid);
    }

    // Constant-time signature verification via HMAC's `verify_slice`.
    let provided = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| AuthError::Invalid)?;
    let signing_input = format!("{header}.{payload}");
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&provided)
        .map_err(|_| AuthError::Invalid)?;

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AuthError::Invalid)?;
    let claims: Claims = serde_json::from_slice(&payload_bytes).map_err(|_| AuthError::Invalid)?;
    if claims.exp <= now_unix() {
        return Err(AuthError::Expired);
    }
    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::{decode_jwt, hash_password, Auth, AuthError, Role};
    use crate::config::{AuthConfig, UserConfig};

    fn auth_with(user: &str, password: &str, role: Role, access_ttl: u64) -> Auth {
        let config = AuthConfig {
            access_ttl_secs: access_ttl,
            refresh_ttl_secs: 3600,
            users: vec![UserConfig {
                name: user.to_owned(),
                role,
                password_hash: hash_password(password).unwrap(),
            }],
        };
        Auth::from_config(&config)
    }

    #[test]
    fn role_privilege_ordering() {
        assert!(Role::Admin.allows(Role::Operator));
        assert!(Role::Operator.allows(Role::Observer));
        assert!(!Role::Observer.allows(Role::Operator));
        assert!(Role::Operator.allows(Role::Operator));
    }

    #[test]
    fn login_then_verify_round_trip() {
        let auth = auth_with("op", "s3cret", Role::Operator, 900);
        let pair = auth.login("op", "s3cret").unwrap();
        let claims = auth.verify(&pair.access_token).unwrap();
        assert_eq!(claims.sub, "op");
        assert_eq!(claims.role, Role::Operator);
    }

    #[test]
    fn login_rejects_bad_password_and_unknown_user() {
        let auth = auth_with("op", "s3cret", Role::Operator, 900);
        assert!(matches!(
            auth.login("op", "wrong"),
            Err(AuthError::InvalidCredentials)
        ));
        assert!(matches!(
            auth.login("nobody", "s3cret"),
            Err(AuthError::InvalidCredentials)
        ));
    }

    #[test]
    fn expired_access_token_is_rejected() {
        // FR-AUTH-02: access_ttl of 0 => token expires immediately.
        let auth = auth_with("op", "s3cret", Role::Operator, 0);
        let pair = auth.login("op", "s3cret").unwrap();
        assert!(matches!(
            auth.verify(&pair.access_token),
            Err(AuthError::Expired)
        ));
    }

    #[test]
    fn tampered_signature_is_rejected() {
        let auth = auth_with("op", "s3cret", Role::Operator, 900);
        let pair = auth.login("op", "s3cret").unwrap();
        // Flip the first character of the signature segment. (Flipping the last
        // base64url char is unreliable: no-pad trailing bits can decode to the
        // same signature bytes.)
        let dot = pair.access_token.rfind('.').unwrap();
        let mut bytes = pair.access_token.into_bytes();
        let i = dot + 1;
        bytes[i] = if bytes[i] == b'A' { b'B' } else { b'A' };
        let token = String::from_utf8(bytes).unwrap();
        assert!(matches!(auth.verify(&token), Err(AuthError::Invalid)));
    }

    #[test]
    fn refresh_rotates_and_invalidates_old_token() {
        // FR-AUTH-03: refresh issues a new pair; the old refresh token is dead.
        let auth = auth_with("op", "s3cret", Role::Operator, 900);
        let first = auth.login("op", "s3cret").unwrap();
        let second = auth.refresh(&first.refresh_token).unwrap();
        assert_ne!(first.refresh_token, second.refresh_token);
        assert!(matches!(
            auth.refresh(&first.refresh_token),
            Err(AuthError::Invalid)
        ));
        // The new access token still verifies.
        assert!(auth.verify(&second.access_token).is_ok());
    }

    #[test]
    fn logout_revokes_access_and_refresh() {
        // FR-AUTH-05: after logout the access token is revoked and the refresh
        // token can no longer be used.
        let auth = auth_with("op", "s3cret", Role::Operator, 900);
        let pair = auth.login("op", "s3cret").unwrap();
        let claims = auth.verify(&pair.access_token).unwrap();
        auth.logout(&claims, &pair.refresh_token);
        assert!(matches!(
            auth.verify(&pair.access_token),
            Err(AuthError::Invalid)
        ));
        assert!(matches!(
            auth.refresh(&pair.refresh_token),
            Err(AuthError::Invalid)
        ));
    }

    #[test]
    fn tokens_are_high_entropy_and_unique() {
        // NFR-SEC-02: refresh tokens are 256-bit random (43 base64url chars) and
        // never repeat.
        let auth = auth_with("op", "s3cret", Role::Operator, 900);
        let a = auth.login("op", "s3cret").unwrap();
        let b = auth.login("op", "s3cret").unwrap();
        assert_ne!(a.refresh_token, b.refresh_token);
        assert_eq!(a.refresh_token.len(), 43); // 32 bytes, base64url no-pad
    }

    #[test]
    fn alg_none_header_is_rejected() {
        // Algorithm-confusion guard: a token with a different header must fail
        // even before signature checks matter.
        let auth = auth_with("op", "s3cret", Role::Operator, 900);
        let forged = "eyJhbGciOiJub25lIn0.eyJzdWIiOiJvcCJ9.";
        assert!(matches!(
            decode_jwt(&[0u8; 32], forged),
            Err(AuthError::Invalid)
        ));
        let _ = auth;
    }
}
