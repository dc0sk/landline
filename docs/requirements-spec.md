---
title: Requirements Specification
project: landline
doc_type: requirements-specification
license: AGPL-3.0-only
status: draft
version: 0.4.0
owner: ""
last_updated: 2026-05-13
---

# Requirements Specification

## 1. Purpose

This document defines the functional, non-functional, security, deployment, and compatibility requirements for the landline system. All requirements carry a unique ID. Every ID must be traceable to at least one test in docs/test-spec.md before any release is approved.

## 2. Glossary

| Term | Definition |
|---|---|
| Rig | A hamradio transceiver controlled via hamlib/rigctld |
| Operator | Authenticated user with permission to operate the rig |
| Observer | Authenticated user with read-only access (spectrum, status) |
| Backend | The Rust service running on the Raspberry Pi |
| Client | A web browser session connected to the backend |
| Frontend Host | A separate machine that serves frontend assets and connects to backend APIs |
| WSS | WebSocket over TLS |
| PTT | Push-to-transmit; activates transmitter on the rig |
| rigctld | hamlib daemon exposing rig control over TCP |

---

## 3. Stakeholders and Roles

| Role | Description |
|---|---|
| Admin | Full system access: configuration, user management, key rotation |
| Operator | Can operate rig: change frequency, mode, PTT, audio |
| Observer | Read-only: view spectrum, waterfall, rig status |

---

## 4. Functional Requirements

### 4.1 Rig Control

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-F-001 | The system shall allow an Operator to read the current frequency from the rig | Must | Draft |
| REQ-F-002 | The system shall allow an Operator to set the rig frequency within valid band limits | Must | Draft |
| REQ-F-003 | The system shall allow an Operator to read the current operating mode (USB, LSB, CW, FM, AM, etc.) | Must | Draft |
| REQ-F-004 | The system shall allow an Operator to set the operating mode | Must | Draft |
| REQ-F-005 | The system shall allow an Operator to activate and deactivate PTT | Must | Draft |
| REQ-F-006 | The system shall display received signal strength (S-meter) to Operators and Observers | Should | Draft |
| REQ-F-007 | The system shall support passband tuning/filter width where the rig supports it | Could | Draft |
| REQ-F-008 | The system shall interface with the rig via hamlib/rigctld over TCP | Must | Draft |
| REQ-F-009 | The system shall validate all rig commands server-side; invalid or out-of-range commands shall be rejected | Must | Draft |
| REQ-F-010 | The system shall maintain exclusive rig access when multiple clients are connected | Must | Draft |

### 4.2 Spectrum and Waterfall

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-F-020 | The system shall stream spectrum data (FFT bins) to connected clients | Must | Draft |
| REQ-F-021 | The spectrum update rate shall be configurable between 1 and 10 Hz | Should | Draft |
| REQ-F-022 | The system shall render a scrolling waterfall display in the browser using HTML5 Canvas | Must | Draft |
| REQ-F-023 | The waterfall shall support colour palette selection | Could | Draft |

### 4.3 Audio

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-F-030 | The system shall stream received audio from the rig to connected Operator/Observer clients | Must | Draft |
| REQ-F-031 | The system shall stream microphone audio from the Operator client to the rig transmit input | Must | Draft |
| REQ-F-032 | The client shall allow the user to select the local audio input device | Must | Draft |
| REQ-F-033 | The client shall allow the user to select the local audio output device | Must | Draft |
| REQ-F-034 | Audio shall be encoded with Opus at a configurable bitrate (default 16 kbps) | Should | Draft |
| REQ-F-035 | The audio path shall tolerate packet loss with graceful degradation | Should | Draft |

### 4.4 Authentication and Session

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-F-040 | The system shall require authentication before granting access to any control, audio, or spectrum endpoint | Must | Draft |
| REQ-F-041 | The system shall issue short-lived session tokens with expiry | Must | Draft |
| REQ-F-042 | The system shall support token refresh without full re-authentication | Should | Draft |
| REQ-F-043 | The system shall enforce role-based access control (Admin, Operator, Observer) | Must | Draft |
| REQ-F-044 | The system shall invalidate sessions on logout or token expiry | Must | Draft |

### 4.5 Audit and Logging

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-F-050 | The system shall produce a tamper-evident audit log of all rig state-changing actions | Must | Draft |
| REQ-F-051 | Each audit event shall include: timestamp, client IP, user identity, action, parameter values | Must | Draft |
| REQ-F-052 | Audit logs shall be retained for at least 30 days | Should | Draft |
| REQ-F-053 | Authentication failures shall be logged with client IP and timestamp | Must | Draft |

### 4.6 Distributed Frontend Deployment

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-F-060 | The system shall support running the frontend from a machine separate from the backend host | Must | Draft |
| REQ-F-061 | The backend API and WSS endpoints shall be reachable by the frontend host without requiring public internet exposure | Must | Draft |
| REQ-F-062 | The deployment shall support at least one secure private-network profile based on WireGuard-compatible tunnels (WireGuard or Tailscale) | Must | Draft |
| REQ-F-063 | The frontend host shall be configurable to target backend API/WSS base URLs without code changes | Must | Draft |

### 4.7 GPIO Digital I/O (Raspberry Pi)

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-F-070 | On Raspberry Pi deployment targets, the system shall support controlling at least 5 digital GPIO pins (read state and set output level) | Must | Draft |

---

## 5. Non-Functional Requirements

### 5.1 Performance

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-NF-001 | Control command round-trip latency shall be < 100 ms at p95 on LAN | Must | Draft |
| REQ-NF-002 | End-to-end audio latency (microphone to rig input) shall be < 300 ms on LAN | Should | Draft |
| REQ-NF-003 | The backend shall sustain ≥ 3 concurrent clients without degradation | Should | Draft |
| REQ-NF-004 | CPU usage on Raspberry Pi 4 shall be < 50 % under full load (3 clients, audio, spectrum) | Should | Draft |
| REQ-NF-005 | Spectrum data shall be updated at a minimum of 2 Hz under normal load | Must | Draft |

### 5.2 Reliability

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-NF-010 | The client shall implement WebSocket reconnection with exponential backoff (max 30 s) | Must | Draft |
| REQ-NF-011 | The backend service shall recover and resume rig access within 5 s after a transient rigctld disconnect | Should | Draft |
| REQ-NF-012 | The system shall sustain 24-hour continuous operation without restart | Must | Draft |

### 5.3 Maintainability

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-NF-020 | The codebase shall pass Clippy (pedantic) lint without warnings | Should | Draft |
| REQ-NF-021 | All public backend APIs shall have integration tests | Must | Draft |
| REQ-NF-022 | The deployment shall support rolling update/rollback without data loss | Should | Draft |

---

## 6. Security Requirements

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-S-001 | All client-server communication shall use TLS (HTTPS/WSS); plaintext connections shall be rejected | Must | Draft |
| REQ-S-002 | Session tokens shall be cryptographically random (≥ 256 bits entropy) | Must | Draft |
| REQ-S-003 | Token secrets and TLS private keys shall be stored in files with permissions 0600 owned by the service user | Must | Draft |
| REQ-S-004 | The backend shall enforce rate limiting on control endpoints (max 10 commands/s per client) | Must | Draft |
| REQ-S-005 | WebSocket frames shall be subject to maximum payload size enforcement (configurable, default 64 KB) | Must | Draft |
| REQ-S-006 | The backend shall enforce CORS policy; only configured origins shall be allowed | Must | Draft |
| REQ-S-007 | PTT activation shall require Operator role and be subject to a server-side safety timeout | Must | Draft |
| REQ-S-008 | All rig command parameters shall be validated against an allowlist of values and numeric ranges | Must | Draft |
| REQ-S-009 | The system shall not leak stack traces, internal paths, or configuration details in error responses | Must | Draft |
| REQ-S-010 | Container images (if used) shall run as a non-root user with a read-only root filesystem | Should | Draft |
| REQ-S-011 | Container images shall be rebuilt on upstream OS/dependency security patches within 7 days | Should | Draft |
| REQ-S-012 | Authentication credentials shall never appear in URL query strings or log output | Must | Draft |
| REQ-S-013 | In distributed frontend deployments, backend ingress shall default to private tunnel interfaces only (not 0.0.0.0 public bind) | Must | Draft |
| REQ-S-014 | Connections between frontend host and backend shall enforce mutual authentication (WireGuard peer keys, Tailscale identity ACLs, or mTLS) | Must | Draft |
| REQ-S-015 | SSH tunnelling may be used only as an operator-maintained fallback mode and shall not be the default production profile | Should | Draft |
| REQ-S-016 | GPIO control shall enforce an allowlist of configured pins; all non-allowlisted pins shall be inaccessible and default to safe startup states | Must | Draft |

---

## 7. Compatibility Requirements

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-C-001 | The web frontend shall function on Firefox (latest two major releases) on desktop Linux/Windows/macOS | Must | Draft |
| REQ-C-002 | The web frontend shall function on Chromium-based browsers (Chrome, Edge; latest two major releases) on desktop | Must | Draft |
| REQ-C-003 | The web frontend shall function on Safari iOS 16+ on iPhone and iPad | Must | Draft |
| REQ-C-004 | The web frontend shall function on Firefox for Android (latest release) | Must | Draft |
| REQ-C-005 | The web frontend shall function on Chrome for Android (latest release) | Must | Draft |
| REQ-C-006 | All UI controls shall be operable by touch on mobile devices | Must | Draft |
| REQ-C-007 | Audio device selection shall use the standard browser MediaDevices API | Must | Draft |

---

## 8. Deployment Requirements

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-D-001 | The reference deployment target is Raspberry Pi 4 (4 GB) and Raspberry Pi 5 running Raspberry Pi OS (64-bit, Bookworm) | Must | Draft |
| REQ-D-002 | The system shall ship a systemd service unit as the native deployment method | Must | Draft |
| REQ-D-003 | The system shall be evaluated as a containerized deployment; if it meets audio latency and hardware-access thresholds it shall be a supported deployment profile | Should | Draft |
| REQ-D-004 | All configuration shall be sourced from a single file (default: ~/.config/landline/config.toml) | Must | Draft |
| REQ-D-005 | The deployment shall include a documented rollback procedure | Must | Draft |
| REQ-D-006 | The backend architecture shall not preclude future non-Pi deployment targets | Should | Draft |
| REQ-D-007 | Deployment documentation shall include a dedicated profile for split-host operation (frontend host + backend host) with secure connectivity setup steps | Must | Draft |
| REQ-D-008 | The recommended split-host profile shall use WireGuard or Tailscale as the primary transport; SSH tunnel profile shall be documented as fallback only | Must | Draft |

---

## 9. Licensing Requirements

| ID | Requirement | Priority | Status |
|---|---|---|---|
| REQ-L-001 | The project shall be licensed under GNU Affero General Public License v3.0 (AGPL-3.0-only) | Must | Draft |
| REQ-L-002 | The repository shall include a top-level LICENSE file containing the full AGPL-3.0 license text and a short license notice in key project docs | Must | Draft |

---

## 10. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.4.0 | 2026-05-13 | — | Added GPIO security requirement for pin allowlist and safe default startup states |
| 0.3.0 | 2026-05-13 | — | Added Raspberry Pi GPIO digital I/O requirement (minimum 5 controllable pins) |
| 0.2.0 | 2026-05-13 | — | Added AGPL licensing requirements and split-host frontend deployment/security requirements |
| 0.1.0 | 2026-05-12 | — | Initial draft |
