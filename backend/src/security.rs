//! ARC-03 security middleware (actions A7, A8).
//!
//! Realises:
//! - **NFR-SEC-04** — per-client rate limiting on control endpoints (token
//!   bucket, default 10 commands/s). [`RateLimiter`] + [`rate_limit`].
//! - **NFR-SEC-05** — request body size limit (HTTP analogue of the WebSocket
//!   frame limit; the WS-frame cap lands with the WS endpoints in Phase 2/3).
//!   Applied via `tower_http::limit::RequestBodyLimitLayer` in [`crate::app`].
//! - **NFR-SEC-06** — CORS allowlist; only configured origins are permitted.
//!   [`cors_layer`].
//!
//! The rate limiter keys on the peer IP ([`ConnectInfo`]). Behind the Phase-4
//! reverse proxy this must switch to a trusted `X-Forwarded-For` (with a
//! trusted-proxy allowlist) so clients are not collapsed to the proxy's IP.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tower_http::cors::CorsLayer;

/// A per-key token-bucket rate limiter (NFR-SEC-04).
///
/// Burst capacity equals the per-second rate, refilled continuously. Keys are
/// opaque (peer IP in the middleware); each key has an independent bucket.
pub struct RateLimiter {
    capacity: f64,
    refill_per_sec: f64,
    buckets: Mutex<HashMap<String, Bucket>>,
}

struct Bucket {
    tokens: f64,
    last: Instant,
}

impl RateLimiter {
    /// Create a limiter allowing `rate_per_sec` requests per second per key,
    /// with a burst of the same size.
    #[must_use]
    pub fn new(rate_per_sec: u32) -> Self {
        let rate = f64::from(rate_per_sec);
        Self {
            capacity: rate,
            refill_per_sec: rate,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Account for one request from `key` at time `now`; return `true` if it is
    /// within the limit (a token was available), `false` if it should be
    /// rejected.
    #[must_use]
    pub fn check(&self, key: &str, now: Instant) -> bool {
        let mut buckets = self.buckets.lock().unwrap_or_else(PoisonError::into_inner);
        let bucket = buckets.entry(key.to_owned()).or_insert(Bucket {
            tokens: self.capacity,
            last: now,
        });
        let elapsed = now.saturating_duration_since(bucket.last).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        bucket.last = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Rate-limiting middleware (NFR-SEC-04). Keys on the peer IP (read from the
/// [`ConnectInfo`] extension); requests without connection info share a single
/// fallback bucket. Wire it with
/// `axum::middleware::from_fn_with_state(limiter, rate_limit)`.
pub async fn rate_limit(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let key = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map_or_else(|| "unknown".to_owned(), |info| info.0.ip().to_string());
    if limiter.check(&key, Instant::now()) {
        next.run(request).await
    } else {
        (StatusCode::TOO_MANY_REQUESTS, "rate limited").into_response()
    }
}

/// Build the CORS layer from the configured origin allowlist (NFR-SEC-06).
///
/// Only origins that parse successfully are allowed; an empty (or all-invalid)
/// list permits no cross-origin requests.
pub fn cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let origins: Vec<_> = allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

#[cfg(test)]
mod tests {
    use super::RateLimiter;
    use std::time::{Duration, Instant};

    #[test]
    fn allows_up_to_rate_then_blocks() {
        // NFR-SEC-04: 10/s => 10 pass at one instant, the 11th is rejected.
        let limiter = RateLimiter::new(10);
        let now = Instant::now();
        for _ in 0..10 {
            assert!(limiter.check("client-a", now));
        }
        assert!(!limiter.check("client-a", now));
    }

    #[test]
    fn refills_over_time() {
        let limiter = RateLimiter::new(10);
        let now = Instant::now();
        for _ in 0..10 {
            assert!(limiter.check("client-a", now));
        }
        assert!(!limiter.check("client-a", now));
        // One second later the bucket has refilled.
        assert!(limiter.check("client-a", now + Duration::from_secs(1)));
    }

    #[test]
    fn keys_are_independent() {
        let limiter = RateLimiter::new(1);
        let now = Instant::now();
        assert!(limiter.check("client-a", now));
        assert!(!limiter.check("client-a", now));
        // A different client has its own bucket.
        assert!(limiter.check("client-b", now));
    }
}
