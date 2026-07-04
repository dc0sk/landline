//! landline backend entry point (ARC-01).
//!
//! Initialises tracing, loads the single-file configuration, builds the Axum
//! application, and serves it with graceful shutdown.

use std::path::Path;
use std::process::ExitCode;

use landline_backend::config::{Config, ConfigError};
use landline_backend::{app, telemetry};

#[tokio::main]
async fn main() -> ExitCode {
    telemetry::init();

    let config = match load_config() {
        Ok(config) => config,
        Err(err) => {
            tracing::error!(error = %err, "failed to load configuration");
            return ExitCode::FAILURE;
        }
    };

    let addr = config.server.socket_addr();
    let app = app(&config);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!(error = %err, %addr, "failed to bind listener");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(%addr, "landline backend listening");

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!(error = %err, "server error");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Resolve and load configuration: `$LANDLINE_CONFIG` overrides the default
/// path (NFR-DEPLOY-04); a missing file yields safe defaults.
fn load_config() -> Result<Config, ConfigError> {
    if let Some(path) = std::env::var_os("LANDLINE_CONFIG") {
        return Config::load(Path::new(&path));
    }
    match Config::default_path() {
        Some(path) => Config::load(&path),
        None => Ok(Config::default()),
    }
}

/// Complete when the process receives SIGINT (Ctrl-C) or SIGTERM, triggering
/// graceful shutdown of in-flight requests. systemd stops the service with
/// SIGTERM (A26), so both must unblock shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(err) => tracing::error!(error = %err, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
