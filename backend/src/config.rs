//! ARC-09 configuration loader (action A5).
//!
//! All configuration is sourced from a single TOML file (NFR-DEPLOY-04, default
//! `~/.config/landline/config.toml`). The config is secret-free — token secrets
//! and TLS keys live in separate 0600 files (NFR-SEC-03) and never in logs or
//! URLs (NFR-SEC-12). The default bind is loopback, never a public `0.0.0.0`
//! bind (NFR-SEC-13, split-host ingress hardening).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::auth::Role;

/// Top-level configuration, sourced from a single TOML file (NFR-DEPLOY-04).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Network binding for the HTTP/WS server (ARC-01).
    pub server: ServerConfig,
    /// Authentication, session, and RBAC settings (ARC-02).
    pub auth: AuthConfig,
    /// Security middleware settings (ARC-03).
    pub security: SecurityConfig,
    /// Audit log settings (ARC-07).
    pub audit: AuditConfig,
    /// Rig adapter settings (ARC-04).
    pub rig: RigConfig,
}

/// Rig adapter configuration (ARC-04): how to reach hamlib/rigctld.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RigConfig {
    /// rigctld host (trusted, typically loopback — ASM-05).
    pub host: String,
    /// rigctld TCP port (hamlib default 4532).
    pub port: u16,
    /// Per-command timeout in milliseconds.
    pub timeout_ms: u64,
}

impl Default for RigConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 4532,
            timeout_ms: 2000,
        }
    }
}

/// Audit log configuration (ARC-07).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuditConfig {
    /// Append-only audit log file path. `None` keeps the log in memory only
    /// (development); production should set a durable path.
    pub path: Option<String>,
    /// Minimum retention in days (FR-AUDIT-03). Enforced by deployment log
    /// rotation (logrotate/systemd); the app writes append-only.
    pub retention_days: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            path: None,
            retention_days: 30,
        }
    }
}

/// Security middleware configuration (ARC-03).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Per-client control-endpoint rate limit, commands/second (NFR-SEC-04).
    pub rate_limit_per_sec: u32,
    /// Maximum accepted request body size in bytes (NFR-SEC-05, default 64 KiB).
    pub max_body_bytes: usize,
    /// CORS origin allowlist; only these origins are permitted (NFR-SEC-06).
    /// Empty means no cross-origin requests are allowed.
    pub allowed_origins: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_sec: 10,
            max_body_bytes: 64 * 1024,
            allowed_origins: Vec::new(),
        }
    }
}

/// Authentication and session configuration (ARC-02).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Access-token lifetime in seconds (FR-AUTH-02, short-lived).
    pub access_ttl_secs: u64,
    /// Refresh-token lifetime in seconds (FR-AUTH-03).
    pub refresh_ttl_secs: u64,
    /// Configured user accounts. Passwords are stored only as argon2 hashes;
    /// plaintext credentials never appear in config, logs, or URLs (NFR-SEC-12).
    pub users: Vec<UserConfig>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            access_ttl_secs: 900,     // 15 minutes
            refresh_ttl_secs: 86_400, // 24 hours
            users: Vec::new(),
        }
    }
}

/// A single configured user account.
#[derive(Debug, Clone, Deserialize)]
pub struct UserConfig {
    /// Login name.
    pub name: String,
    /// Role granted to this user (Admin, Operator, Observer).
    pub role: Role,
    /// argon2 password hash (PHC string). Generate with the crate's
    /// [`crate::auth::hash_password`] helper.
    pub password_hash: String,
}

/// Server network binding.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Bind address. Defaults to loopback; a public `0.0.0.0` bind must be an
    /// explicit operator choice (NFR-SEC-13).
    pub bind: IpAddr,
    /// TCP port.
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8443,
        }
    }
}

impl ServerConfig {
    /// The resolved socket address to bind (`bind:port`).
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind, self.port)
    }
}

/// Errors returned while loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The config file exists but could not be read.
    #[error("failed to read config file {path}")]
    Read {
        /// Path that failed to read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The config file exists but is not valid TOML.
    #[error("failed to parse config file {path}")]
    Parse {
        /// Path that failed to parse.
        path: PathBuf,
        /// Underlying parse error.
        #[source]
        source: toml::de::Error,
    },
}

impl Config {
    /// The default configuration path `~/.config/landline/config.toml`
    /// (NFR-DEPLOY-04). Returns `None` if `$HOME` is unset.
    #[must_use]
    pub fn default_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|home| {
            Path::new(&home)
                .join(".config")
                .join("landline")
                .join("config.toml")
        })
    }

    /// Load configuration from `path`.
    ///
    /// A missing file yields the [`Default`] configuration, so a fresh install
    /// runs with safe defaults. An existing-but-unreadable or malformed file is
    /// an error rather than a silent fallback.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|source| ConfigError::Parse {
                path: path.to_path_buf(),
                source,
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Read {
                path: path.to_path_buf(),
                source,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn default_bind_is_loopback() {
        // NFR-SEC-13: never a public 0.0.0.0 bind by default.
        assert!(Config::default().server.bind.is_loopback());
    }

    #[test]
    fn parses_a_minimal_toml() {
        let cfg: Config = toml::from_str("[server]\nport = 9000\n").unwrap();
        assert_eq!(cfg.server.port, 9000);
        assert!(cfg.server.bind.is_loopback());
    }
}
