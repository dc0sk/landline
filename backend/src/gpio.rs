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
}

impl IntoResponse for GpioError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            GpioError::Disabled => (StatusCode::SERVICE_UNAVAILABLE, "gpio disabled"),
            // Non-allowlisted pins are inaccessible (NFR-SEC-16).
            GpioError::NotAllowed => (StatusCode::FORBIDDEN, "pin not allowed"),
            GpioError::NotAnOutput => (StatusCode::BAD_REQUEST, "pin is not an output"),
        };
        (status, body).into_response()
    }
}

struct PinSpec {
    direction: Direction,
}

/// The GPIO controller (ARC-08): allowlist + safe-state enforcement over a
/// pin-state backend.
pub struct GpioController {
    enabled: bool,
    pins: HashMap<u8, PinSpec>,
    state: Mutex<HashMap<u8, Level>>,
}

impl GpioController {
    /// Build from configuration, driving every allowlisted output to its safe
    /// startup state (NFR-SEC-16).
    #[must_use]
    pub fn from_config(config: &GpioConfig) -> Self {
        let mut pins = HashMap::new();
        let mut state = HashMap::new();
        for pin in &config.pins {
            // Drive outputs (and seed input readbacks) to the safe startup state.
            state.insert(pin.pin, pin.safe_state);
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
            state: Mutex::new(state),
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
        Ok(self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&pin)
            .copied()
            .unwrap_or(Level::Low))
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
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(pin, level);
        Ok(())
    }
}

/// Router for the `/api/gpio/*` endpoints (Operator-gated).
pub fn router() -> Router {
    Router::new().route("/api/gpio/{pin}", get(read_pin).post(set_pin))
}

#[derive(Serialize)]
struct PinResponse {
    pin: u8,
    level: Level,
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
    use super::{Direction, GpioController, GpioError, Level};
    use crate::config::{GpioConfig, GpioPinConfig};

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
    fn disabled_gpio_rejects_all() {
        let gpio = GpioController::from_config(&GpioConfig {
            enabled: false,
            pins: vec![GpioPinConfig {
                pin: 17,
                direction: Direction::Out,
                safe_state: Level::Low,
            }],
        });
        assert!(matches!(gpio.read(17), Err(GpioError::Disabled)));
        assert!(matches!(
            gpio.set(17, Level::High),
            Err(GpioError::Disabled)
        ));
    }
}
