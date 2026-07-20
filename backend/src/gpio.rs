//! ARC-08 GPIO adapter (action A17).
//!
//! Controls Raspberry Pi digital I/O (FR-GPIO-01) behind a strict allowlist:
//! only pins configured in `[gpio]` are reachable; every other pin is
//! inaccessible, and configured outputs are driven to their safe state at
//! startup (NFR-SEC-16).
//!
//! The pin state is held behind a backend so the hardware layer is swappable.
//! This build ships an in-memory backend (used on non-Pi hosts and in tests); a
//! Raspberry Pi sysfs/gpiod backend is a thin deployment-time adapter that
//! replaces the storage without changing the allowlist/validation logic here.

use std::collections::HashMap;
use std::sync::Mutex;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};

use crate::audit::{AuditLog, ClientIp};
use crate::auth::{AuthUser, Role};
use crate::config::GpioConfig;

/// Pin direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Input (read-only).
    In,
    /// Output (settable).
    Out,
}

/// Digital level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Logic low (0).
    Low,
    /// Logic high (1).
    High,
}

/// Errors from GPIO control.
#[derive(Debug, thiserror::Error)]
pub enum GpioError {
    /// GPIO control is disabled in configuration.
    #[error("gpio disabled")]
    Disabled,
    /// The pin is not in the configured allowlist (NFR-SEC-16).
    #[error("pin not allowed")]
    NotAllowed,
    /// Attempted to drive an input pin.
    #[error("pin is not an output")]
    NotAnOutput,
    /// The hardware operation failed (ioctl error, line unavailable). Reported
    /// rather than swallowed: a pin that did not move must not read as success,
    /// or the audit log records a state change that never happened.
    #[error("gpio hardware error")]
    Hardware,
}

impl IntoResponse for GpioError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            GpioError::Disabled => (StatusCode::SERVICE_UNAVAILABLE, "gpio disabled"),
            // Non-allowlisted pins are inaccessible (NFR-SEC-16).
            GpioError::NotAllowed => (StatusCode::FORBIDDEN, "pin not allowed"),
            GpioError::NotAnOutput => (StatusCode::BAD_REQUEST, "pin is not an output"),
            GpioError::Hardware => (StatusCode::BAD_GATEWAY, "gpio hardware error"),
        };
        (status, body).into_response()
    }
}

struct PinSpec {
    direction: Direction,
}

/// Storage/hardware backend for raw pin levels. The allowlist and direction
/// rules live in [`GpioController`]; a backend only reads/writes levels, so the
/// hardware layer is swappable (in-memory off-Pi and in tests, character-device
/// gpiod on the Pi with `--features gpio-device`).
trait PinBackend: Send + Sync {
    /// Read a pin's level, or [`GpioError::Hardware`] if the hardware operation
    /// failed. A failure must never be reported as a level: an unreadable pin
    /// defaulting to `Low` would read as "safe" for an interlock input.
    fn read(&self, pin: u8) -> Result<Level, GpioError>;
    /// Drive a pin, or [`GpioError::Hardware`] if the hardware operation failed.
    fn write(&self, pin: u8, level: Level) -> Result<(), GpioError>;
}

/// In-memory backend: holds levels in a map (used off-Pi and in tests).
struct MemoryBackend {
    state: Mutex<HashMap<u8, Level>>,
}

impl MemoryBackend {
    fn seeded(config: &GpioConfig) -> Self {
        let mut state = HashMap::new();
        for pin in &config.pins {
            state.insert(pin.pin, pin.safe_state);
        }
        Self {
            state: Mutex::new(state),
        }
    }
}

impl PinBackend for MemoryBackend {
    fn read(&self, pin: u8) -> Result<Level, GpioError> {
        Ok(self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&pin)
            .copied()
            .unwrap_or(Level::Low))
    }

    fn write(&self, pin: u8, level: Level) -> Result<(), GpioError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(pin, level);
        Ok(())
    }
}

/// Choose the pin backend: the real character-device backend when built with
/// `--features gpio-device` and GPIO is enabled, otherwise in-memory. A failed
/// hardware open degrades to in-memory rather than aborting startup.
#[cfg(not(feature = "gpio-device"))]
fn select_backend(config: &GpioConfig) -> Box<dyn PinBackend> {
    Box::new(MemoryBackend::seeded(config))
}

#[cfg(feature = "gpio-device")]
fn select_backend(config: &GpioConfig) -> Box<dyn PinBackend> {
    if config.enabled {
        match device::GpiodBackend::new(config) {
            Ok(backend) => {
                tracing::info!(pins = config.pins.len(), "gpio-device backend active");
                return Box::new(backend);
            }
            Err(err) => {
                tracing::warn!(error = %err, "gpio hardware init failed; using in-memory backend");
            }
        }
    }
    Box::new(MemoryBackend::seeded(config))
}

/// The GPIO controller (ARC-08): allowlist + safe-state enforcement over a
/// pin-state backend.
pub struct GpioController {
    enabled: bool,
    pins: HashMap<u8, PinSpec>,
    backend: Box<dyn PinBackend>,
}

impl GpioController {
    /// Build from configuration, driving every allowlisted output to its safe
    /// startup state (NFR-SEC-16).
    #[must_use]
    pub fn from_config(config: &GpioConfig) -> Self {
        let mut pins = HashMap::new();
        for pin in &config.pins {
            pins.insert(
                pin.pin,
                PinSpec {
                    direction: pin.direction,
                },
            );
        }
        Self {
            enabled: config.enabled,
            pins,
            backend: select_backend(config),
        }
    }

    fn spec(&self, pin: u8) -> Result<&PinSpec, GpioError> {
        if !self.enabled {
            return Err(GpioError::Disabled);
        }
        self.pins.get(&pin).ok_or(GpioError::NotAllowed)
    }

    /// Read a pin's current level.
    ///
    /// # Errors
    /// Returns [`GpioError`] if GPIO is disabled or the pin is not allowlisted.
    pub fn read(&self, pin: u8) -> Result<Level, GpioError> {
        self.spec(pin)?; // enabled + allowlist check
        self.backend.read(pin)
    }

    /// Set an output pin's level.
    ///
    /// # Errors
    /// Returns [`GpioError`] if GPIO is disabled, the pin is not allowlisted, or
    /// the pin is not an output.
    pub fn set(&self, pin: u8, level: Level) -> Result<(), GpioError> {
        let spec = self.spec(pin)?;
        if spec.direction != Direction::Out {
            return Err(GpioError::NotAnOutput);
        }
        self.backend.write(pin, level)
    }

    /// List every allowlisted pin with its direction and current level, sorted by
    /// pin number. Empty when GPIO is disabled (so the UI can render "no pins").
    #[must_use]
    pub fn list(&self) -> Vec<PinInfo> {
        if !self.enabled {
            return Vec::new();
        }
        let mut infos: Vec<PinInfo> = self
            .pins
            .iter()
            .map(|(pin, spec)| PinInfo {
                pin: *pin,
                direction: spec.direction,
                // `None` when the hardware read failed: the UI must show
                // "unknown", never a level the pin may not actually be at.
                level: self.backend.read(*pin).ok(),
            })
            .collect();
        infos.sort_by_key(|info| info.pin);
        infos
    }
}

/// Character-device (`/dev/gpiochipN`) backend, built only with the
/// `gpio-device` feature. Pure-Rust (gpio-cdev over ioctl), so the default
/// build stays free of extra deps and never touches real hardware.
#[cfg(feature = "gpio-device")]
mod device {
    use std::collections::HashMap;

    use gpio_cdev::{Chip, LineHandle, LineRequestFlags};

    use super::{Direction, GpioError, Level, PinBackend};
    use crate::config::GpioConfig;

    pub struct GpiodBackend {
        handles: HashMap<u8, LineHandle>,
    }

    impl GpiodBackend {
        pub fn new(config: &GpioConfig) -> Result<Self, String> {
            let path = config.chip.as_deref().unwrap_or("/dev/gpiochip0");
            let mut chip = Chip::new(path).map_err(|e| e.to_string())?;
            let mut handles = HashMap::new();
            for pin in &config.pins {
                let line = chip
                    .get_line(u32::from(pin.pin))
                    .map_err(|e| e.to_string())?;
                // Outputs are requested with their safe state as the initial value,
                // so the pin is driven safe the instant it is claimed (NFR-SEC-16).
                let handle = match pin.direction {
                    Direction::Out => line
                        .request(
                            LineRequestFlags::OUTPUT,
                            u8::from(pin.safe_state == Level::High),
                            "landline",
                        )
                        .map_err(|e| e.to_string())?,
                    Direction::In => line
                        .request(LineRequestFlags::INPUT, 0, "landline")
                        .map_err(|e| e.to_string())?,
                };
                handles.insert(pin.pin, handle);
            }
            Ok(Self { handles })
        }
    }

    impl PinBackend for GpiodBackend {
        fn read(&self, pin: u8) -> Result<Level, GpioError> {
            let handle = self.handles.get(&pin).ok_or(GpioError::Hardware)?;
            let value = handle.get_value().map_err(|err| {
                tracing::error!(pin, error = %err, "gpio read failed");
                GpioError::Hardware
            })?;
            Ok(if value == 1 { Level::High } else { Level::Low })
        }

        fn write(&self, pin: u8, level: Level) -> Result<(), GpioError> {
            let handle = self.handles.get(&pin).ok_or(GpioError::Hardware)?;
            handle
                .set_value(u8::from(level == Level::High))
                .map_err(|err| {
                    tracing::error!(pin, error = %err, "gpio write failed");
                    GpioError::Hardware
                })
        }
    }
}

/// Router for the `/api/gpio/*` endpoints (Operator-gated).
pub fn router() -> Router {
    Router::new()
        .route("/api/gpio", get(list_pins))
        .route("/api/gpio/{pin}", get(read_pin).post(set_pin))
}

#[derive(Serialize)]
struct PinResponse {
    pin: u8,
    level: Level,
}

/// One allowlisted pin's direction and current level (the `GET /api/gpio` list).
#[derive(Serialize)]
pub struct PinInfo {
    /// BCM pin number.
    pub pin: u8,
    /// Whether the pin is an input or output.
    pub direction: Direction,
    /// Current level, or `None` if the pin could not be read.
    pub level: Option<Level>,
}

async fn list_pins(
    user: AuthUser,
    Extension(gpio): Extension<std::sync::Arc<GpioController>>,
) -> Response {
    if let Err(err) = user.require(Role::Operator) {
        return err.into_response();
    }
    Json(gpio.list()).into_response()
}

#[derive(Deserialize)]
struct SetPinRequest {
    level: Level,
}

async fn read_pin(
    user: AuthUser,
    Extension(gpio): Extension<std::sync::Arc<GpioController>>,
    Path(pin): Path<u8>,
) -> Response {
    if let Err(err) = user.require(Role::Operator) {
        return err.into_response();
    }
    match gpio.read(pin) {
        Ok(level) => Json(PinResponse { pin, level }).into_response(),
        Err(err) => err.into_response(),
    }
}

async fn set_pin(
    user: AuthUser,
    Extension(gpio): Extension<std::sync::Arc<GpioController>>,
    Extension(audit): Extension<std::sync::Arc<AuditLog>>,
    ClientIp(ip): ClientIp,
    Path(pin): Path<u8>,
    Json(req): Json<SetPinRequest>,
) -> Response {
    if let Err(err) = user.require(Role::Operator) {
        return err.into_response();
    }
    match gpio.set(pin, req.level) {
        Ok(()) => {
            audit.record_action(
                ip.as_deref(),
                &user.claims.sub,
                "gpio.set",
                &format!("pin={pin} level={:?}", req.level),
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, GpioController, GpioError, Level, MemoryBackend, PinBackend, PinSpec};
    use crate::config::{GpioConfig, GpioPinConfig};
    use std::collections::HashMap;

    /// A backend whose hardware operations always fail, standing in for a real
    /// ioctl error (wrong chip, line contention, unplugged expander). The gpiod
    /// backend cannot run in CI, so this is how the propagation path is proven.
    struct FailingBackend;

    impl PinBackend for FailingBackend {
        fn read(&self, _pin: u8) -> Result<Level, GpioError> {
            Err(GpioError::Hardware)
        }

        fn write(&self, _pin: u8, _level: Level) -> Result<(), GpioError> {
            Err(GpioError::Hardware)
        }
    }

    /// A controller over the given backend, with the same allowlist as
    /// [`controller`], so a test can exercise hardware failure independently of
    /// the allowlist and direction rules.
    fn controller_with(backend: Box<dyn PinBackend>) -> GpioController {
        let mut pins = HashMap::new();
        pins.insert(
            17,
            PinSpec {
                direction: Direction::Out,
            },
        );
        pins.insert(
            27,
            PinSpec {
                direction: Direction::In,
            },
        );
        GpioController {
            enabled: true,
            pins,
            backend,
        }
    }

    #[test]
    fn hardware_failure_is_reported_not_swallowed() {
        // A failed ioctl must surface as an error, not as a successful set. The
        // caller audits `gpio.set` on Ok, so swallowing the failure would record
        // a state change that never reached the wire.
        let gpio = controller_with(Box::new(FailingBackend));
        assert!(matches!(
            gpio.set(17, Level::High),
            Err(GpioError::Hardware)
        ));
        assert!(matches!(gpio.read(27), Err(GpioError::Hardware)));
    }

    #[test]
    fn unreadable_pin_lists_as_unknown_not_low() {
        // `Low` is the safe state for these pins, so defaulting an unreadable
        // pin to Low would render as "safe" in the UI for a pin nobody can read.
        let gpio = controller_with(Box::new(FailingBackend));
        let list = gpio.list();
        assert_eq!(list.len(), 2);
        assert!(
            list.iter().all(|info| info.level.is_none()),
            "an unreadable pin must list as unknown, not as a level"
        );
    }

    #[test]
    fn memory_backend_reports_success() {
        // Guards the negative control: the failure tests above must be failing
        // because of the backend, not because the propagation is broken for all.
        let gpio = controller_with(Box::new(MemoryBackend::seeded(&GpioConfig {
            enabled: true,
            pins: vec![GpioPinConfig {
                pin: 17,
                direction: Direction::Out,
                safe_state: Level::Low,
            }],
            ..GpioConfig::default()
        })));
        assert!(gpio.set(17, Level::High).is_ok());
        assert_eq!(gpio.read(17).unwrap(), Level::High);
    }

    fn controller() -> GpioController {
        GpioController::from_config(&GpioConfig {
            enabled: true,
            pins: vec![
                GpioPinConfig {
                    pin: 17,
                    direction: Direction::Out,
                    safe_state: Level::Low,
                },
                GpioPinConfig {
                    pin: 27,
                    direction: Direction::In,
                    safe_state: Level::High,
                },
            ],
            chip: None,
        })
    }

    #[test]
    fn outputs_start_at_safe_state() {
        // NFR-SEC-16: configured startup states are safe on service start.
        let gpio = controller();
        assert_eq!(gpio.read(17).unwrap(), Level::Low);
        assert_eq!(gpio.read(27).unwrap(), Level::High);
    }

    #[test]
    fn non_allowlisted_pin_is_inaccessible() {
        // TC-SEC-15 / NFR-SEC-16
        let gpio = controller();
        assert!(matches!(gpio.read(5), Err(GpioError::NotAllowed)));
        assert!(matches!(
            gpio.set(5, Level::High),
            Err(GpioError::NotAllowed)
        ));
    }

    #[test]
    fn set_and_read_output() {
        let gpio = controller();
        gpio.set(17, Level::High).unwrap();
        assert_eq!(gpio.read(17).unwrap(), Level::High);
    }

    #[test]
    fn cannot_drive_an_input_pin() {
        let gpio = controller();
        assert!(matches!(
            gpio.set(27, Level::Low),
            Err(GpioError::NotAnOutput)
        ));
    }

    #[test]
    fn list_returns_sorted_pins_with_levels() {
        let gpio = controller();
        gpio.set(17, Level::High).unwrap();
        let list = gpio.list();
        assert_eq!(list.len(), 2);
        assert_eq!((list[0].pin, list[0].level), (17, Some(Level::High)));
        assert_eq!((list[1].pin, list[1].direction), (27, Direction::In));
    }

    #[test]
    fn list_is_empty_when_disabled() {
        let gpio = GpioController::from_config(&GpioConfig {
            enabled: false,
            pins: vec![GpioPinConfig {
                pin: 17,
                direction: Direction::Out,
                safe_state: Level::Low,
            }],
            chip: None,
        });
        assert!(gpio.list().is_empty());
    }

    #[test]
    fn disabled_gpio_rejects_all() {
        let gpio = GpioController::from_config(&GpioConfig {
            enabled: false,
            pins: vec![GpioPinConfig {
                pin: 17,
                direction: Direction::Out,
                safe_state: Level::Low,
            }],
            chip: None,
        });
        assert!(matches!(gpio.read(17), Err(GpioError::Disabled)));
        assert!(matches!(
            gpio.set(17, Level::High),
            Err(GpioError::Disabled)
        ));
    }
}
