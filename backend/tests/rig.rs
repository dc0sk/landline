//! Integration tests for the ARC-04 rig adapter against a mock rigctld
//! (NFR-MAINT-02). Traces TC-RIG-07 (connect, send command, parse response).

use std::net::SocketAddr;

use landline_backend::config::RigConfig;
use landline_backend::rig::{Mode, RigAdapter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// Spawn a minimal rigctld emulator on an ephemeral port. It answers the simple
/// protocol commands the adapter sends, over a single reused connection.
async fn spawn_mock_rigctld() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
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
            let command = line.trim_end();
            let response: &[u8] = if command == "f" {
                b"14074000\n"
            } else if command == "m" {
                b"USB\n2400\n"
            } else if command == "l STRENGTH" {
                b"-54\n"
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
    addr
}

fn adapter_for(addr: SocketAddr) -> RigAdapter {
    RigAdapter::from_config(&RigConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        timeout_ms: 2000,
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
    });
    assert!(rig.set_frequency(-1).await.is_err());
}
