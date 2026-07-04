//! ARC-01 structured tracing initialisation (action A4).
//!
//! landline log statements never emit credential material (NFR-SEC-12); this
//! module only configures the sink and filter. Error responses are sanitised
//! separately by the security middleware (NFR-SEC-09), landing in a later action.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialise the global tracing subscriber.
///
/// The filter is read from `RUST_LOG`, defaulting to `info`. This must be called
/// exactly once, early in `main`; it is intentionally not invoked from tests.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
}
