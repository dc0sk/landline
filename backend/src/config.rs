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
use crate::gpio::{Direction, Level};

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
    /// GPIO settings (ARC-08).
    pub gpio: GpioConfig,
    /// Spectrum/FFT + WebSocket telemetry settings (ARC-06).
    pub spectrum: SpectrumConfig,
    /// Audio pipeline settings (ARC-05).
    pub audio: AudioConfig,
}

/// Audio pipeline configuration (ARC-05).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    /// Sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Frame duration in milliseconds.
    pub frame_ms: u32,
    /// Opus target bitrate in bits/second (FR-AUD-05, default 16 kbps).
    pub bitrate_bps: u32,
    /// Jitter-buffer target depth (frames to buffer before playout).
    pub jitter_target_frames: usize,
    /// Jitter-buffer maximum depth before a missing frame is concealed.
    pub jitter_max_frames: usize,
    /// Capture device name substring (audio-device feature); `None` = default.
    #[serde(default)]
    pub capture_device: Option<String>,
    /// Playback device name substring (audio-device feature); `None` = default.
    #[serde(default)]
    pub playback_device: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: 48_000,
            frame_ms: 20,
            bitrate_bps: 16_000,
            jitter_target_frames: 3,
            jitter_max_frames: 10,
            capture_device: None,
            playback_device: None,
        }
    }
}

/// Spectrum/FFT and WebSocket telemetry configuration (ARC-06).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SpectrumConfig {
    /// FFT size (bins in = samples per frame); output is `fft_size / 2` bins.
    pub fft_size: usize,
    /// Nominal sample rate of the source in Hz.
    pub sample_rate_hz: u32,
    /// Spectrum frame rate in Hz; clamped to 1–10 (FR-SPEC-02, NFR-PERF-05).
    pub update_rate_hz: f32,
    /// Nominal centre frequency reported with each frame, in Hz.
    pub center_hz: u64,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            fft_size: 1024,
            sample_rate_hz: 48_000,
            update_rate_hz: 5.0,
            center_hz: 0,
        }
    }
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
    /// PTT safety timeout in seconds: the server auto-unkeys if PTT is left
    /// active this long without a refresh (NFR-SEC-07).
    pub ptt_timeout_secs: u64,
    /// Consecutive failures before the circuit breaker opens (NFR-REL-02).
    pub breaker_threshold: u32,
    /// Circuit-breaker cooldown in milliseconds before a retry is allowed.
    pub breaker_cooldown_ms: u64,
}

impl Default for RigConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 4532,
            timeout_ms: 2000,
            ptt_timeout_secs: 120,
            breaker_threshold: 3,
            breaker_cooldown_ms: 1000,
        }
    }
}

/// GPIO configuration (ARC-08): the allowlist of controllable pins and their
/// safe startup states (NFR-SEC-16).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GpioConfig {
    /// Whether GPIO control is enabled (Raspberry Pi deployments).
    pub enabled: bool,
    /// Allowlisted pins. Any pin not listed is inaccessible (NFR-SEC-16).
    pub pins: Vec<GpioPinConfig>,
}

/// A single allowlisted GPIO pin.
#[derive(Debug, Clone, Deserialize)]
pub struct GpioPinConfig {
    /// BCM pin number.
    pub pin: u8,
    /// Whether the pin is an input or an output.
    pub direction: Direction,
    /// The safe level applied to outputs at startup (NFR-SEC-16).
    pub safe_state: Level,
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
    /// Optional directory of static frontend files to serve at `/` (single-host
    /// deployments). Unset = API only; the split-host topology serves the UI
    /// from a separate origin behind the reverse proxy instead.
    #[serde(default)]
    pub static_dir: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8443,
            static_dir: None,
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
    /// The config file is readable/writable by group or others (NFR-SEC-03).
    #[error("insecure permissions on {path}: {mode:#o} (require owner-only, e.g. 0600)")]
    InsecurePermissions {
        /// Offending path.
        path: PathBuf,
        /// The file's permission bits.
        mode: u32,
    },
}

/// Whether a Unix mode grants no group/other access (owner-only, NFR-SEC-03).
#[must_use]
#[allow(clippy::verbose_bit_mask)] // the octal mask reads clearer than trailing_zeros here
fn mode_is_owner_only(mode: u32) -> bool {
    mode & 0o077 == 0
}

#[cfg(unix)]
fn check_permissions(path: &Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(path)
        .map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?
        .permissions()
        .mode();
    if mode_is_owner_only(mode) {
        Ok(())
    } else {
        Err(ConfigError::InsecurePermissions {
            path: path.to_path_buf(),
            mode: mode & 0o777,
        })
    }
}

#[cfg(not(unix))]
fn check_permissions(_path: &Path) -> Result<(), ConfigError> {
    Ok(())
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
            Ok(text) => {
                // Fail closed if the file is group/other-accessible (NFR-SEC-03):
                // the config may reference secret files and must not be exposed.
                check_permissions(path)?;
                toml::from_str(&text).map_err(|source| ConfigError::Parse {
                    path: path.to_path_buf(),
                    source,
                })
            }
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

    #[test]
    fn parses_users_under_auth_table() {
        // Users MUST live under `[[auth.users]]` (config.example.toml). A bare
        // top-level `[[users]]` is a different array and leaves auth.users empty
        // — the mistake that made every login 401 in first HIL bring-up.
        let cfg: Config = toml::from_str(concat!(
            "[auth]\n",
            "[[auth.users]]\n",
            "name = \"op\"\n",
            "role = \"operator\"\n",
            "password_hash = \"$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$aGFzaA\"\n",
        ))
        .unwrap();
        assert_eq!(cfg.auth.users.len(), 1);
        assert_eq!(cfg.auth.users[0].name, "op");

        let wrong: Config = toml::from_str(
            "[auth]\n[[users]]\nname = \"op\"\nrole = \"operator\"\npassword_hash = \"x\"\n",
        )
        .unwrap();
        assert!(
            wrong.auth.users.is_empty(),
            "bare [[users]] must not populate auth.users"
        );
    }

    #[test]
    fn owner_only_mode_detection() {
        // NFR-SEC-03: owner-only (no group/other bits).
        assert!(super::mode_is_owner_only(0o600));
        assert!(super::mode_is_owner_only(0o700));
        assert!(!super::mode_is_owner_only(0o640));
        assert!(!super::mode_is_owner_only(0o644));
        assert!(!super::mode_is_owner_only(0o666));
    }

    #[cfg(unix)]
    #[test]
    fn load_rejects_group_readable_config() {
        use super::ConfigError;
        use std::os::unix::fs::PermissionsExt;
        let path =
            std::env::temp_dir().join(format!("landline-cfg-{}-insecure.toml", std::process::id()));
        std::fs::write(&path, "[server]\nport = 8443\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let result = Config::load(&path);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(
            result,
            Err(ConfigError::InsecurePermissions { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn load_accepts_owner_only_config() {
        use std::os::unix::fs::PermissionsExt;
        let path =
            std::env::temp_dir().join(format!("landline-cfg-{}-secure.toml", std::process::id()));
        std::fs::write(&path, "[server]\nport = 9000\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let config = Config::load(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(config.server.port, 9000);
    }
}
