//! ARC-04 rig adapter (action A10).
//!
//! The single choke point between landline and the transceiver. It speaks the
//! hamlib **rigctld** simple TCP protocol (ADR-03, FR-RIG-08) and exposes a
//! **typed** API: every operation is a method taking numeric or enum parameters,
//! validated against an allowlist and numeric ranges before anything is sent
//! (FR-RIG-09, NFR-SEC-08). Because parameters are never free-form strings,
//! shell/metacharacter injection is impossible by construction (TC-SEC-08).
//!
//! Access is serialised through an async mutex so concurrent clients cannot
//! interleave commands on the shared rigctld connection (FR-RIG-10). A failed
//! exchange drops the connection so the next call transparently reconnects,
//! which is the basis for transient-disconnect recovery (NFR-REL-02; the full
//! circuit breaker is action A16).

use std::time::Duration;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::config::RigConfig;

/// Lowest accepted frequency (1 kHz) and highest (500 MHz). Values outside this
/// range are rejected before reaching the rig (FR-RIG-09).
const FREQ_MIN_HZ: u64 = 1_000;
const FREQ_MAX_HZ: u64 = 500_000_000;

/// An allowlisted operating mode (FR-RIG-03/04). Only these exact tokens are
/// accepted; anything else — including strings with metacharacters — is rejected
/// (NFR-SEC-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Mode {
    /// Upper sideband.
    Usb,
    /// Lower sideband.
    Lsb,
    /// CW (normal).
    Cw,
    /// CW (reverse).
    Cwr,
    /// AM.
    Am,
    /// Narrow FM.
    Fm,
    /// Wide FM.
    Wfm,
    /// RTTY.
    Rtty,
    /// Packet/data USB.
    Pktusb,
    /// Packet/data LSB.
    Pktlsb,
}

impl Mode {
    /// The rigctld protocol token for this mode.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Usb => "USB",
            Mode::Lsb => "LSB",
            Mode::Cw => "CW",
            Mode::Cwr => "CWR",
            Mode::Am => "AM",
            Mode::Fm => "FM",
            Mode::Wfm => "WFM",
            Mode::Rtty => "RTTY",
            Mode::Pktusb => "PKTUSB",
            Mode::Pktlsb => "PKTLSB",
        }
    }

    /// Parse a rigctld mode token against the allowlist.
    ///
    /// # Errors
    /// Returns [`RigError::InvalidMode`] for any token not in the allowlist.
    pub fn parse(token: &str) -> Result<Self, RigError> {
        let mode = match token {
            "USB" => Mode::Usb,
            "LSB" => Mode::Lsb,
            "CW" => Mode::Cw,
            "CWR" => Mode::Cwr,
            "AM" => Mode::Am,
            "FM" => Mode::Fm,
            "WFM" => Mode::Wfm,
            "RTTY" => Mode::Rtty,
            "PKTUSB" => Mode::Pktusb,
            "PKTLSB" => Mode::Pktlsb,
            _ => return Err(RigError::InvalidMode),
        };
        Ok(mode)
    }
}

/// Validate a requested frequency (FR-RIG-09). Rejects negatives and anything
/// outside `[1 kHz, 500 MHz]`.
///
/// # Errors
/// Returns [`RigError::OutOfRange`] if the value is negative or out of band.
pub fn validate_frequency(hz: i64) -> Result<u64, RigError> {
    let hz = u64::try_from(hz).map_err(|_| RigError::OutOfRange)?;
    if (FREQ_MIN_HZ..=FREQ_MAX_HZ).contains(&hz) {
        Ok(hz)
    } else {
        Err(RigError::OutOfRange)
    }
}

/// Errors from the rig adapter.
#[derive(Debug, thiserror::Error)]
pub enum RigError {
    /// Could not connect to rigctld.
    #[error("failed to connect to rigctld")]
    Connect(#[source] std::io::Error),
    /// I/O error on the rigctld connection.
    #[error("rigctld I/O error")]
    Io(#[source] std::io::Error),
    /// rigctld returned an unexpected or error response.
    #[error("rigctld protocol error: {0}")]
    Protocol(String),
    /// A parameter was out of the accepted range.
    #[error("value out of range")]
    OutOfRange,
    /// A mode token was not in the allowlist.
    #[error("invalid mode")]
    InvalidMode,
    /// The rigctld exchange timed out.
    #[error("rigctld timeout")]
    Timeout,
}

impl IntoResponse for RigError {
    fn into_response(self) -> Response {
        // Sanitised: client input errors -> 400; rig-side faults -> 502. No
        // internal details are leaked (NFR-SEC-09).
        let (status, body) = match self {
            RigError::OutOfRange | RigError::InvalidMode => {
                (StatusCode::BAD_REQUEST, "invalid rig command")
            }
            RigError::Connect(_) | RigError::Io(_) | RigError::Protocol(_) | RigError::Timeout => {
                (StatusCode::BAD_GATEWAY, "rig unavailable")
            }
        };
        (status, body).into_response()
    }
}

/// The rig adapter (ARC-04): a serialised hamlib/rigctld TCP client.
pub struct RigAdapter {
    host: String,
    port: u16,
    timeout: Duration,
    conn: Mutex<Option<BufReader<TcpStream>>>,
}

impl RigAdapter {
    /// Build the adapter from configuration. No connection is made until the
    /// first command (lazy connect).
    #[must_use]
    pub fn from_config(config: &RigConfig) -> Self {
        Self {
            host: config.host.clone(),
            port: config.port,
            timeout: Duration::from_millis(config.timeout_ms),
            conn: Mutex::new(None),
        }
    }

    /// Read the current frequency in Hz (FR-RIG-01).
    ///
    /// # Errors
    /// Returns [`RigError`] on connection/timeout/protocol failure.
    pub async fn get_frequency(&self) -> Result<u64, RigError> {
        let lines = self.request("f", 1).await?;
        lines[0]
            .parse::<u64>()
            .map_err(|_| RigError::Protocol(format!("bad frequency: {}", lines[0])))
    }

    /// Set the frequency in Hz (FR-RIG-02), validated first (FR-RIG-09).
    ///
    /// # Errors
    /// Returns [`RigError::OutOfRange`] if invalid, or a transport error.
    pub async fn set_frequency(&self, hz: i64) -> Result<(), RigError> {
        let hz = validate_frequency(hz)?;
        self.set_command(&format!("F {hz}")).await
    }

    /// Read the current mode (FR-RIG-03).
    ///
    /// # Errors
    /// Returns [`RigError`] on transport failure or an unknown mode token.
    pub async fn get_mode(&self) -> Result<Mode, RigError> {
        // rigctld `m` returns two lines: mode token and passband.
        let lines = self.request("m", 2).await?;
        Mode::parse(&lines[0])
    }

    /// Set the mode and passband (FR-RIG-04).
    ///
    /// # Errors
    /// Returns [`RigError`] on transport failure or rigctld error.
    pub async fn set_mode(&self, mode: Mode, passband_hz: u32) -> Result<(), RigError> {
        self.set_command(&format!("M {} {passband_hz}", mode.as_str()))
            .await
    }

    /// Activate or deactivate PTT (FR-RIG-05).
    ///
    /// # Errors
    /// Returns [`RigError`] on transport failure or rigctld error.
    pub async fn set_ptt(&self, transmit: bool) -> Result<(), RigError> {
        self.set_command(&format!("T {}", u8::from(transmit))).await
    }

    /// Read the S-meter strength (FR-RIG-06).
    ///
    /// # Errors
    /// Returns [`RigError`] on transport failure or an unparseable value.
    pub async fn get_strength(&self) -> Result<i32, RigError> {
        let lines = self.request("l STRENGTH", 1).await?;
        lines[0]
            .parse::<i32>()
            .map_err(|_| RigError::Protocol(format!("bad strength: {}", lines[0])))
    }

    /// Send a `set`-style command and check the `RPRT` result code.
    async fn set_command(&self, command: &str) -> Result<(), RigError> {
        let lines = self.request(command, 1).await?;
        parse_rprt(&lines[0])
    }

    /// Send `command` (a newline is appended) and read `expected_lines` reply
    /// lines. On any failure the connection is dropped so the next call
    /// reconnects.
    async fn request(&self, command: &str, expected_lines: usize) -> Result<Vec<String>, RigError> {
        let mut guard = self.conn.lock().await;
        if guard.is_none() {
            let stream = timeout(
                self.timeout,
                TcpStream::connect((self.host.as_str(), self.port)),
            )
            .await
            .map_err(|_| RigError::Timeout)?
            .map_err(RigError::Connect)?;
            *guard = Some(BufReader::new(stream));
        }
        let conn = guard
            .as_mut()
            .ok_or_else(|| RigError::Protocol("no connection".to_owned()))?;

        let result = exchange(conn, command, expected_lines, self.timeout).await;
        if result.is_err() {
            *guard = None;
        }
        result
    }
}

async fn exchange(
    conn: &mut BufReader<TcpStream>,
    command: &str,
    expected_lines: usize,
    within: Duration,
) -> Result<Vec<String>, RigError> {
    timeout(within, async {
        let stream = conn.get_mut();
        stream
            .write_all(command.as_bytes())
            .await
            .map_err(RigError::Io)?;
        stream.write_all(b"\n").await.map_err(RigError::Io)?;
        stream.flush().await.map_err(RigError::Io)?;

        let mut lines = Vec::with_capacity(expected_lines);
        for _ in 0..expected_lines {
            let mut line = String::new();
            let read = conn.read_line(&mut line).await.map_err(RigError::Io)?;
            if read == 0 {
                return Err(RigError::Protocol("connection closed".to_owned()));
            }
            lines.push(line.trim_end().to_owned());
        }
        Ok(lines)
    })
    .await
    .map_err(|_| RigError::Timeout)?
}

fn parse_rprt(line: &str) -> Result<(), RigError> {
    let code = line
        .strip_prefix("RPRT ")
        .and_then(|rest| rest.trim().parse::<i32>().ok())
        .ok_or_else(|| RigError::Protocol(format!("unexpected response: {line}")))?;
    if code == 0 {
        Ok(())
    } else {
        Err(RigError::Protocol(format!("rigctld error {code}")))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_rprt, validate_frequency, Mode, RigError};

    #[test]
    fn frequency_validation_rejects_negative_and_out_of_range() {
        // TC-RIG-08 / FR-RIG-09
        assert!(matches!(validate_frequency(-1), Err(RigError::OutOfRange)));
        assert!(matches!(validate_frequency(0), Err(RigError::OutOfRange)));
        assert!(matches!(
            validate_frequency(600_000_000),
            Err(RigError::OutOfRange)
        ));
        assert_eq!(validate_frequency(14_074_000).unwrap(), 14_074_000);
    }

    #[test]
    fn mode_parse_allowlist_rejects_metacharacters() {
        // TC-SEC-08 / NFR-SEC-08: injection attempts are not valid modes.
        assert_eq!(Mode::parse("USB").unwrap(), Mode::Usb);
        assert!(matches!(
            Mode::parse("USB; rm -rf /"),
            Err(RigError::InvalidMode)
        ));
        assert!(matches!(
            Mode::parse("`reboot`"),
            Err(RigError::InvalidMode)
        ));
        assert!(matches!(Mode::parse(""), Err(RigError::InvalidMode)));
    }

    #[test]
    fn rprt_parsing() {
        assert!(parse_rprt("RPRT 0").is_ok());
        assert!(matches!(parse_rprt("RPRT -1"), Err(RigError::Protocol(_))));
        assert!(matches!(parse_rprt("garbage"), Err(RigError::Protocol(_))));
    }
}
