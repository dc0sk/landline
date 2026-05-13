---
title: Test Specification
project: landline
doc_type: test-specification
license: AGPL-3.0-only
status: draft
version: 0.4.0
owner: ""
last_updated: 2026-05-13
---

# Test Specification

## 1. Purpose

This document defines the test strategy, test levels, test cases, and requirement traceability for the landline system. Every requirement in docs/requirements-spec.md must have at least one test entry here before any release is approved.

---

## 2. Test Strategy

### 2.1 Principles

- Security tests are mandatory blockers; a failing security test blocks all other release gates.
- Tests are linked to requirement IDs; untraceable tests carry no release weight.
- Test evidence (pass/fail, environment, date) must be recorded before a phase exit is approved.

### 2.2 Test Levels

| Level | Scope | Tooling |
|---|---|---|
| Unit | Individual Rust functions and modules | `cargo test` |
| Integration | Backend API endpoints, WebSocket message handling, rigctld adapter | `cargo test` + mock rigctld |
| System | Full stack on Raspberry Pi hardware with connected or simulated rig | Manual + automated scripts |
| Browser compatibility | UI and audio across browser/device matrix | Manual + Playwright (where applicable) |
| Security | Auth bypass, replay, rate-limit, malformed-frame, injection | Manual + `websocat` / custom scripts |
| Performance / soak | Sustained load on Pi 4/5: latency, CPU, memory, thermal | Custom load scripts + `top` / `htop` |

### 2.3 Definition of Test Pass

A test passes when:
- The observed result matches the expected result.
- No unexpected panic, crash, or error log occurs.
- For security tests: the attack is blocked and an audit log event is produced.

---

## 3. Test Environment

| Environment | Description |
|---|---|
| Dev (local) | Developer machine; mock rigctld; self-signed TLS cert |
| Pi 4 integration | Raspberry Pi 4 (4 GB, RPiOS Bookworm 64-bit); real or simulated rig |
| Pi 5 integration | Raspberry Pi 5 (RPiOS Bookworm 64-bit); real or simulated rig |
| Browser matrix | See section 5 |
| Container | Docker or Podman on Raspberry Pi OS |

---

## 4. Traceability Matrix

> Status values: `Not written` | `Draft` | `Ready` | `Pass` | `Fail` | `Blocked`

### 4.1 Rig Control

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-F-001 | REQ-F-001 | Read current frequency from rig via rigctld mock | Integration | Not written |
| TST-F-002 | REQ-F-002 | Set valid frequency; verify rig receives command; verify out-of-range is rejected | Integration | Not written |
| TST-F-003 | REQ-F-003, REQ-F-004 | Read and set operating mode; verify unsupported mode is rejected | Integration | Not written |
| TST-F-004 | REQ-F-005 | Activate PTT as Operator; verify rig state; deactivate | Integration | Not written |
| TST-F-005 | REQ-F-005, REQ-S-007 | PTT activation attempt by Observer role; verify rejection and audit log | Integration | Not written |
| TST-F-006 | REQ-F-006 | S-meter value streams to client at configured interval | Integration | Not written |
| TST-F-007 | REQ-F-008 | rigctld TCP adapter connects, sends command, parses response | Unit | Not written |
| TST-F-008 | REQ-F-009 | Send frequency below 0 Hz; verify server rejects with 400-equivalent error | Integration | Not written |
| TST-F-009 | REQ-F-010 | Two concurrent Operator clients; verify only one can hold rig control at a time | System | Not written |

### 4.2 Spectrum and Waterfall

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-F-020 | REQ-F-020 | WebSocket spectrum stream delivers FFT bins at configured rate | Integration | Not written |
| TST-F-021 | REQ-F-021 | Change spectrum update rate via config; verify delivery rate changes | Integration | Not written |
| TST-F-022 | REQ-F-022 | Waterfall canvas renders in Firefox and Chromium without errors | Browser | Not written |
| TST-F-023 | REQ-F-022 | Waterfall renders on iOS Safari without WebGL dependency | Browser | Not written |

### 4.3 Audio

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-F-030 | REQ-F-030 | Received audio streams from backend to browser client over WSS | System | Not written |
| TST-F-031 | REQ-F-031 | Microphone audio streams from browser to backend over WSS | System | Not written |
| TST-F-032 | REQ-F-032, REQ-F-033 | Audio device selector lists available input and output devices in browser | Browser | Not written |
| TST-F-033 | REQ-C-003 | Audio device selection functions on iOS Safari (mic permission flow) | Browser | Not written |
| TST-F-034 | REQ-F-034 | Opus-encoded audio decoded without artefacts at 16 kbps | System | Not written |
| TST-F-035 | REQ-F-035 | Simulate 5 % packet loss; verify audio degrades gracefully without crash | System | Not written |

### 4.4 Authentication and Session

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-F-040 | REQ-F-040 | Unauthenticated WebSocket connection attempt; verify rejection and no data leak | Security | Not written |
| TST-F-041 | REQ-F-041 | Issue token; wait for expiry; verify subsequent requests are rejected | Integration | Not written |
| TST-F-042 | REQ-F-042 | Token refresh flow; verify new token issued and old token invalidated | Integration | Not written |
| TST-F-043 | REQ-F-043, REQ-S-012 | Observer attempts rig command; verify 403-equivalent rejection; no credential in logs | Security | Not written |
| TST-F-044 | REQ-F-044 | Explicit logout; verify session invalidated; re-use of old token rejected | Integration | Not written |

### 4.5 Audit and Logging

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-F-050 | REQ-F-050, REQ-F-051 | Execute rig command; verify audit log entry with timestamp, IP, user, action, params | Integration | Not written |
| TST-F-051 | REQ-F-053 | Failed login attempt; verify log entry with IP and timestamp; no password in log | Security | Not written |

### 4.6 Split-Host Frontend Deployment

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-F-060 | REQ-F-060 | Run frontend assets from a separate host; verify full control and telemetry flow to backend | System | Not written |
| TST-F-061 | REQ-F-061, REQ-F-063 | Configure frontend API/WSS base URLs for remote backend; verify no rebuild required | System | Not written |
| TST-F-062 | REQ-F-062 | Validate WireGuard/Tailscale tunnel profile carries API and WSS traffic between hosts | System | Not written |

### 4.7 GPIO Digital I/O

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-F-070 | REQ-F-070 | Configure and control at least 5 Raspberry Pi GPIO pins; verify readback and output level changes | System | Not written |

### 4.8 Non-Functional - Performance

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-NF-001 | REQ-NF-001 | Measure control command RTT over LAN; p95 must be < 100 ms | Performance | Not written |
| TST-NF-002 | REQ-NF-002 | Measure end-to-end audio latency LAN; target < 300 ms | Performance | Not written |
| TST-NF-003 | REQ-NF-003 | Connect 3 concurrent clients; verify all receive spectrum and control responds normally | Performance | Not written |
| TST-NF-004 | REQ-NF-004 | Sustained 30-minute load on Pi 4 with 3 clients; CPU must remain < 50 % | Performance | Not written |
| TST-NF-005 | REQ-NF-012 | Run system continuously for 24 hours; verify no crash, no memory leak trend | Soak | Not written |

### 4.9 Non-Functional - Reliability

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-NF-010 | REQ-NF-010 | Force TCP disconnect; verify client reconnects within 30 s with exponential backoff | System | Not written |
| TST-NF-011 | REQ-NF-011 | Kill rigctld process; verify backend recovers within 5 s on restart | System | Not written |

### 4.10 Security

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-S-001 | REQ-S-001 | Connect via plain `ws://`; verify backend rejects or redirects | Security | Not written |
| TST-S-002 | REQ-S-002 | Inspect issued tokens; verify ≥ 256-bit entropy, no predictable pattern | Security | Not written |
| TST-S-003 | REQ-S-003 | Check file permissions on token secret and TLS key files; must be 0600 | Security | Not written |
| TST-S-004 | REQ-S-004 | Send 20 control commands/s from one client; verify rate limit triggers after 10/s | Security | Not written |
| TST-S-005 | REQ-S-005 | Send oversized WebSocket frame (> 64 KB); verify backend closes connection cleanly | Security | Not written |
| TST-S-006 | REQ-S-006 | Send request with disallowed Origin header; verify CORS rejection | Security | Not written |
| TST-S-007 | REQ-S-007 | PTT timeout: leave PTT active; verify server deactivates after safety timeout | Security | Not written |
| TST-S-008 | REQ-S-008 | Send mode parameter with shell metacharacters; verify rejection without command execution | Security | Not written |
| TST-S-009 | REQ-S-009 | Trigger server error; verify response body contains no stack trace or file paths | Security | Not written |
| TST-S-010 | REQ-S-012 | Capture HTTP/WS traffic; verify no token or credential in URL query strings | Security | Not written |
| TST-S-011 | REQ-S-004 | Replay captured authenticated WebSocket message 60 s later; verify rejection | Security | Not written |
| TST-S-012 | REQ-S-013 | Verify backend bind/address policy in split-host mode exposes service only on tunnel interface(s) | Security | Not written |
| TST-S-013 | REQ-S-014 | Attempt split-host connection with untrusted peer identity; verify rejected access | Security | Not written |
| TST-S-014 | REQ-S-015 | Validate SSH tunnel fallback documentation and controls; verify fallback is disabled by default | Security | Not written |
| TST-S-015 | REQ-S-016 | Attempt control of non-allowlisted GPIO pins and verify denial; verify configured startup states are safe on service start | Security | Not written |

### 4.11 Compatibility - Browser Matrix

| Test ID | Requirement(s) | Browser / Platform | Controls | Spectrum | Audio | Status |
|---|---|---|---|---|---|---|
| TST-C-001 | REQ-C-001 | Firefox latest, Linux desktop | — | — | — | Not written |
| TST-C-002 | REQ-C-002 | Chrome latest, Linux desktop | — | — | — | Not written |
| TST-C-003 | REQ-C-002 | Edge latest, Windows desktop | — | — | — | Not written |
| TST-C-004 | REQ-C-003 | Safari iOS 16+, iPhone | — | — | — | Not written |
| TST-C-005 | REQ-C-003 | Safari iOS 16+, iPad | — | — | — | Not written |
| TST-C-006 | REQ-C-004 | Firefox Android latest | — | — | — | Not written |
| TST-C-007 | REQ-C-005 | Chrome Android latest | — | — | — | Not written |

### 4.12 Deployment

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-D-001 | REQ-D-001 | Build and run on Pi 4 (RPiOS 64-bit Bookworm); verify service starts and rig connects | System | Not written |
| TST-D-002 | REQ-D-001 | Build and run on Pi 5; verify same binary or equivalent runs correctly | System | Not written |
| TST-D-003 | REQ-D-002 | Install systemd unit; verify auto-start on reboot and clean stop on `systemctl stop` | System | Not written |
| TST-D-004 | REQ-D-003 | Build and run container image; verify audio device access and latency within threshold | System | Not written |
| TST-D-005 | REQ-D-003 | Verify container runs as non-root with read-only rootfs | Security | Not written |
| TST-D-006 | REQ-D-005 | Execute rollback procedure; verify previous version restores and service resumes | System | Not written |
| TST-D-007 | REQ-D-007, REQ-D-008 | Execute split-host deployment runbook with WireGuard or Tailscale profile and verify successful connectivity | System | Not written |

### 4.13 Licensing

| Test ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| TST-L-001 | REQ-L-001 | Verify repository license identifier and declared license are AGPL-3.0-only | Integration | Not written |
| TST-L-002 | REQ-L-002 | Verify top-level LICENSE file exists and contains AGPL-3.0 text; verify docs include short license notice | Integration | Not written |

---

## 5. Test Execution Record Template

For each test execution, record the following alongside the test case:

```
Test ID    : TST-xxx-nnn
Date       : YYYY-MM-DD
Executor   : 
Environment: 
Result     : Pass / Fail / Blocked
Evidence   : (log excerpt, screenshot path, or command output)
Notes      :
```

---

## 6. Release Gate Checklist

Before any phase exit is approved:

- [ ] All Must-priority tests in the phase scope have status `Pass`.
- [ ] All security tests relevant to the phase have status `Pass`.
- [ ] No requirement ID in the phase scope has zero mapped tests.
- [ ] Test execution records are filled with date, executor, and evidence.
- [ ] Any `Fail` or `Blocked` tests have a tracked issue with a disposition (fix, defer with justification).

---

## 7. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.4.0 | 2026-05-13 | — | Added GPIO security test for allowlisted pins and safe startup states |
| 0.3.0 | 2026-05-13 | — | Added GPIO digital I/O system test for Raspberry Pi (minimum 5 pins) |
| 0.2.0 | 2026-05-13 | — | Added tests for split-host frontend deployment/security and AGPL licensing requirements |
| 0.1.0 | 2026-05-12 | — | Initial draft; all test cases at status Not written |
