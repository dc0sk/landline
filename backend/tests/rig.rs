//! Integration tests for the ARC-04 rig adapter against a mock rigctld
//! (NFR-MAINT-02). Traces TC-RIG-07 (connect, send command, parse response).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use landline_backend::config::RigConfig;
use landline_backend::rig::{Mode, PttGuard, RigAdapter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// Commands the mock rigctld received, in order. Asserting against this is what
/// makes a PTT test evidence about the *rig* rather than about an internal flag.
type CommandLog = Arc<Mutex<Vec<String>>>;

/// Spawn a minimal rigctld emulator on an ephemeral port. It answers the simple
/// protocol commands the adapter sends, over a single reused connection, and
/// records every command it saw. When `fail_unkey` is set it rejects `T 0`
/// (unkey) with a protocol error, emulating a rig that will not stop
/// transmitting.
async fn spawn_mock_rigctld_opts(fail_unkey: bool) -> (SocketAddr, CommandLog) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log: CommandLog = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            let command = line.trim_end().to_owned();
            sink.lock().unwrap().push(command.clone());
            let response: &[u8] = if command == "f" {
                b"14074000\n"
            } else if command == "m" {
                b"USB\n2400\n"
            } else if command == "l STRENGTH" {
                b"-54\n"
            } else if fail_unkey && command == "T 0" {
                b"RPRT -1\n"
            } else if command.starts_with("F ")
                || command.starts_with("M ")
                || command.starts_with("T ")
            {
                b"RPRT 0\n"
            } else {
                b"RPRT -1\n"
            };
            if reader.get_mut().write_all(response).await.is_err() {
                break;
            }
            let _ = reader.get_mut().flush().await;
        }
    });
    (addr, log)
}

async fn spawn_mock_rigctld() -> SocketAddr {
    spawn_mock_rigctld_opts(false).await.0
}

fn saw(log: &CommandLog, command: &str) -> bool {
    log.lock().unwrap().iter().any(|c| c == command)
}

fn adapter_for(addr: SocketAddr) -> RigAdapter {
    RigAdapter::from_config(&RigConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        timeout_ms: 2000,
        ..RigConfig::default()
    })
}

#[tokio::test]
async fn round_trips_against_mock_rigctld() {
    // TC-RIG-07: connect, send commands, parse responses.
    let addr = spawn_mock_rigctld().await;
    let rig = adapter_for(addr);

    assert_eq!(rig.get_frequency().await.unwrap(), 14_074_000);
    rig.set_frequency(14_100_000).await.unwrap();
    assert_eq!(rig.get_mode().await.unwrap(), Mode::Usb);
    rig.set_mode(Mode::Lsb, 2400).await.unwrap();
    rig.set_ptt(true).await.unwrap();
    rig.set_ptt(false).await.unwrap();
    assert_eq!(rig.get_strength().await.unwrap(), -54);
}

#[tokio::test]
async fn invalid_frequency_is_rejected_without_contacting_rig() {
    // TC-RIG-08 / FR-RIG-09: validation happens before any TCP connect, so this
    // fails even though no rigctld is listening on the (unused) port.
    let rig = RigAdapter::from_config(&RigConfig {
        host: "127.0.0.1".to_owned(),
        port: 1, // nothing listens here; we must never reach it
        timeout_ms: 200,
        ..RigConfig::default()
    });
    assert!(rig.set_frequency(-1).await.is_err());
}

#[tokio::test]
async fn ptt_safety_timeout_auto_unkeys() {
    // TC-SEC-07 / NFR-SEC-07: leave PTT active; the server auto-deactivates it
    // after the safety timeout. Asserting on the *command the rig received*, not
    // just on `is_active()`: the flag is server-side state, and a test that only
    // checks the flag still passes if the unkey command is never sent.
    let (addr, log) = spawn_mock_rigctld_opts(false).await;
    let rig = Arc::new(adapter_for(addr));
    let ptt = PttGuard::new(Arc::clone(&rig), Duration::from_millis(100));

    ptt.activate().await.unwrap();
    assert!(ptt.is_active());
    assert!(saw(&log, "T 1"), "activate must key the rig");

    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(
        saw(&log, "T 0"),
        "safety timeout must send the unkey command to the rig"
    );
    assert!(
        !ptt.is_active(),
        "PTT should auto-unkey after the safety timeout"
    );
}

#[tokio::test]
async fn manual_unkey_confirms_before_clearing_state() {
    // NFR-SEC-07: if the rig rejects the unkey, the transmitter may still be
    // keyed — the guard must report an error and keep reporting PTT active
    // rather than clearing state on an unconfirmed unkey.
    let (addr, log) = spawn_mock_rigctld_opts(true).await;
    let rig = Arc::new(adapter_for(addr));
    // Long timeout: this test is about the manual path, not the safety timer.
    let ptt = PttGuard::new(Arc::clone(&rig), Duration::from_secs(60));

    ptt.activate().await.unwrap();
    assert!(ptt.is_active());

    assert!(
        ptt.deactivate().await.is_err(),
        "a rejected unkey must surface as an error"
    );
    assert!(saw(&log, "T 0"), "the unkey must actually be attempted");
    assert!(
        ptt.is_active(),
        "PTT must stay active when the rig did not confirm the unkey"
    );
}

#[tokio::test]
async fn safety_timeout_keeps_ptt_active_when_unkey_fails() {
    // NFR-SEC-07: the auto-unkey path has the same honesty requirement as the
    // manual one — a failed unkey must not leave the server believing a
    // possibly-keyed transmitter is safe.
    let (addr, log) = spawn_mock_rigctld_opts(true).await;
    let rig = Arc::new(adapter_for(addr));
    let ptt = PttGuard::new(Arc::clone(&rig), Duration::from_millis(100));

    ptt.activate().await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(saw(&log, "T 0"), "safety timeout must attempt the unkey");
    assert!(
        ptt.is_active(),
        "PTT must remain active while the rig has not confirmed the unkey"
    );
}

#[tokio::test]
async fn shutdown_unkeys_an_active_ptt() {
    // The safety timer lives on the Tokio runtime; process shutdown drops it, so
    // shutdown must unkey explicitly or a SIGTERM mid-transmission leaves the
    // rig keyed with nothing left running to release it.
    let (addr, log) = spawn_mock_rigctld_opts(false).await;
    let rig = Arc::new(adapter_for(addr));
    let ptt = PttGuard::new(Arc::clone(&rig), Duration::from_secs(60));

    ptt.activate().await.unwrap();
    ptt.shutdown().await;

    assert!(saw(&log, "T 0"), "shutdown must unkey the rig");
    assert!(!ptt.is_active());
}
