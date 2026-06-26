---
title: "System Requirements (SRS)"
status: Draft
version: "0.6"
updated: 2026-06-26
authors:
  - Simon Keimer (DC0SK)
owns: [FR, NFR]
---

# System Requirements (SRS)

> **License notice:** landline is licensed under **AGPL-3.0-only**. See the top-level
> [LICENSE](../../LICENSE).

## Purpose

This document defines the functional (`FR-`) and non-functional (`NFR-`) software requirements
for **landline** — a secure, browser-native web remote for amateur-radio transceivers, built in
Rust (Axum/Tokio) on a Raspberry Pi with a TypeScript browser frontend. Each requirement is a
single, testable "shall" statement carrying a priority, verification method, an upstream
stakeholder trace (`Up`), and a downstream test trace (`Down`).

The machine-readable source of truth for each area is its **table**. The `ID` cell is a
backtick-quoted identifier; the [`scripts/trace-gate.py`](../../scripts/trace-gate.py) gate parses
these tables (and the [test traceability matrix](../test/test-strategy.md)) to enforce rules
**R3** (every M/S requirement covered by ≥1 `TC`) and **R4** (no dangling test traces). See
[docs/README.md](../README.md) §3–§4 for attribute and traceability conventions.

Columns: **ID** · **Statement** (the single-"shall" text) · **Prio** (`M`/`S`/`C`/`W`) ·
**Verif** (`T` test · `D` demonstration · `I` inspection · `A` analysis) · **Up** (upstream
`STK-`) · **Down** (verifying `TC-`) · **Status**.

## Glossary

| Term | Definition |
|---|---|
| Rig | An amateur-radio transceiver controlled via hamlib/rigctld |
| rigctld | hamlib daemon exposing rig control over TCP |
| Backend | The Rust (Axum/Tokio) service running on the Raspberry Pi |
| Client | A web browser session connected to the backend |
| Frontend host | A separate machine serving frontend assets and connecting to backend APIs |
| Operator | Authenticated user permitted to operate the rig |
| Observer | Authenticated user with read-only access (spectrum, status) |
| Admin | Full access: configuration, user management, key rotation |
| PTT | Push-to-transmit; activates the rig transmitter |
| WSS | WebSocket over TLS |

---

## Functional Requirements

### RIG — Rig control

Operator-facing control of the transceiver via the hamlib/rigctld TCP adapter: frequency, mode,
PTT, and metering. All commands are validated server-side and rig access is exclusive across
clients.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `FR-RIG-01` | The system shall allow an Operator to read the current frequency from the rig. | M | T | STK-01 | TC-RIG-01 | Proposed |
| `FR-RIG-02` | The system shall allow an Operator to set the rig frequency within valid band limits. | M | T | STK-01 | TC-RIG-02 | Proposed |
| `FR-RIG-03` | The system shall allow an Operator to read the current operating mode (USB, LSB, CW, FM, AM, etc.). | M | T | STK-01 | TC-RIG-03 | Proposed |
| `FR-RIG-04` | The system shall allow an Operator to set the operating mode. | M | T | STK-01 | TC-RIG-03 | Proposed |
| `FR-RIG-05` | The system shall allow an Operator to activate and deactivate PTT. | M | T | STK-01 | TC-RIG-04, TC-RIG-05 | Proposed |
| `FR-RIG-06` | The system shall display received signal strength (S-meter) to Operators and Observers. | S | T | STK-02 | TC-RIG-06 | Proposed |
| `FR-RIG-07` | The system shall support passband tuning/filter width where the rig supports it. | C | T | STK-01 | — | Proposed |
| `FR-RIG-08` | The system shall interface with the rig via hamlib/rigctld over TCP. | M | T | STK-01 | TC-RIG-07 | Proposed |
| `FR-RIG-09` | The system shall validate all rig commands server-side and reject invalid or out-of-range commands. | M | T | STK-03 | TC-RIG-08 | Proposed |
| `FR-RIG-10` | The system shall maintain exclusive rig access when multiple clients are connected. | M | T | STK-01 | TC-RIG-09 | Proposed |

### SPEC — Spectrum & waterfall

Streamed FFT spectrum and a browser-rendered scrolling waterfall (HTML5 Canvas).

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `FR-SPEC-01` | The system shall stream spectrum data (FFT bins) to connected clients. | M | T | STK-02 | TC-SPEC-01 | Proposed |
| `FR-SPEC-02` | The spectrum update rate shall be configurable between 1 and 10 Hz. | S | T | STK-02 | TC-SPEC-02 | Proposed |
| `FR-SPEC-03` | The system shall render a scrolling waterfall display in the browser using HTML5 Canvas. | M | T | STK-02 | TC-SPEC-03, TC-SPEC-04 | Proposed |
| `FR-SPEC-04` | The waterfall shall support colour palette selection. | C | T | STK-02 | — | Proposed |

### AUD — Audio streaming

Full-duplex RX/TX audio over WSS, Opus-encoded, with browser device selection and graceful
degradation under packet loss.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `FR-AUD-01` | The system shall stream received audio from the rig to connected Operator/Observer clients. | M | T | STK-02 | TC-AUD-01 | Proposed |
| `FR-AUD-02` | The system shall stream microphone audio from the Operator client to the rig transmit input. | M | T | STK-01 | TC-AUD-02 | Proposed |
| `FR-AUD-03` | The client shall allow the user to select the local audio input device. | M | T | STK-02 | TC-AUD-03 | Proposed |
| `FR-AUD-04` | The client shall allow the user to select the local audio output device. | M | T | STK-02 | TC-AUD-03 | Proposed |
| `FR-AUD-05` | Audio shall be encoded with Opus at a configurable bitrate (default 16 kbps). | S | T | STK-02 | TC-AUD-05 | Proposed |
| `FR-AUD-06` | The audio path shall tolerate packet loss with graceful degradation. | S | T | STK-02 | TC-AUD-06 | Proposed |

### AUTH — Authentication & session

No endpoint is reachable without authentication; short-lived tokens, refresh, RBAC
(Admin/Operator/Observer), and session invalidation.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `FR-AUTH-01` | The system shall require authentication before granting access to any control, audio, or spectrum endpoint. | M | T | STK-03 | TC-AUTH-01 | Proposed |
| `FR-AUTH-02` | The system shall issue short-lived session tokens with expiry. | M | T | STK-03 | TC-AUTH-02 | Proposed |
| `FR-AUTH-03` | The system shall support token refresh without full re-authentication. | S | T | STK-03 | TC-AUTH-03 | Proposed |
| `FR-AUTH-04` | The system shall enforce role-based access control (Admin, Operator, Observer). | M | T | STK-03 | TC-AUTH-04 | Proposed |
| `FR-AUTH-05` | The system shall invalidate sessions on logout or token expiry. | M | T | STK-03 | TC-AUTH-05 | Proposed |

### AUDIT — Audit & logging

Tamper-evident audit of state-changing actions, structured event fields, retention, and
authentication-failure logging.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `FR-AUDIT-01` | The system shall produce a tamper-evident audit log of all rig state-changing actions. | M | T | STK-04 | TC-AUDIT-01 | Proposed |
| `FR-AUDIT-02` | Each audit event shall include timestamp, client IP, user identity, action, and parameter values. | M | T | STK-04 | TC-AUDIT-01 | Proposed |
| `FR-AUDIT-03` | Audit logs shall be retained for at least 30 days. | S | T | STK-04 | TC-AUDIT-03 | Proposed |
| `FR-AUDIT-04` | Authentication failures shall be logged with client IP and timestamp. | M | T | STK-04 | TC-AUDIT-02 | Proposed |

### HOST — Split-host frontend deployment (functional connectivity)

The frontend may run on a machine separate from the backend, reachable over a secure private
network without public exposure, with runtime-configurable backend base URLs.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `FR-HOST-01` | The system shall support running the frontend from a machine separate from the backend host. | M | T | STK-07 | TC-HOST-01 | Proposed |
| `FR-HOST-02` | The backend API and WSS endpoints shall be reachable by the frontend host without requiring public internet exposure. | M | T | STK-07 | TC-HOST-02 | Proposed |
| `FR-HOST-03` | The deployment shall support at least one secure private-network profile based on WireGuard-compatible tunnels (WireGuard or Tailscale). | M | T | STK-07 | TC-HOST-03 | Proposed |
| `FR-HOST-04` | The frontend host shall be configurable to target backend API/WSS base URLs without code changes. | M | T | STK-07 | TC-HOST-02 | Proposed |

### GPIO — Raspberry Pi GPIO digital I/O

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `FR-GPIO-01` | On Raspberry Pi deployment targets, the system shall support controlling at least 5 digital GPIO pins (read state and set output level). | M | T | STK-09 | TC-GPIO-01 | Proposed |

---

## Non-Functional Requirements

### PERF — Performance

Low-latency control feedback and adequate throughput on Raspberry Pi reference hardware.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `NFR-PERF-01` | Control command round-trip latency shall be < 100 ms at p95 on LAN. | M | T | STK-12 | TC-PERF-01 | Proposed |
| `NFR-PERF-02` | End-to-end audio latency (microphone to rig input) shall be < 300 ms on LAN. | S | T | STK-12 | TC-PERF-02 | Proposed |
| `NFR-PERF-03` | The backend shall sustain ≥ 3 concurrent clients without degradation. | S | T | STK-08 | TC-PERF-03 | Proposed |
| `NFR-PERF-04` | CPU usage on Raspberry Pi 4 shall be < 50 % under full load (3 clients, audio, spectrum). | S | T | STK-08 | TC-PERF-04 | Proposed |
| `NFR-PERF-05` | Spectrum data shall be updated at a minimum of 2 Hz under normal load. | M | T | STK-02 | TC-SPEC-05 | Proposed |

### REL — Reliability

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `NFR-REL-01` | The client shall implement WebSocket reconnection with exponential backoff (max 30 s). | M | T | STK-12 | TC-REL-01 | Proposed |
| `NFR-REL-02` | The backend service shall recover and resume rig access within 5 s after a transient rigctld disconnect. | S | T | STK-08 | TC-REL-02 | Proposed |
| `NFR-REL-03` | The system shall sustain 24-hour continuous operation without restart. | M | T | STK-08 | TC-REL-03 | Proposed |

### MAINT — Maintainability

Lint discipline, backend test coverage, and safe update/rollback.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `NFR-MAINT-01` | The codebase shall pass Clippy (pedantic) lint without warnings. | S | I | STK-10 | TC-MAINT-01 | Proposed |
| `NFR-MAINT-02` | All public backend APIs shall have integration tests. | M | A | STK-10 | TC-MAINT-02 | Proposed |
| `NFR-MAINT-03` | The deployment shall support rolling update/rollback without data loss. | S | T | STK-10 | TC-DEPLOY-06 | Proposed |

### SEC — Security

TLS-only transport, strong tokens, secret hygiene, rate/size limits, input validation, safe PTT
and GPIO behaviour, and split-host bind/peer-authentication defaults. Security tests are mandatory
release blockers.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `NFR-SEC-01` | All client-server communication shall use TLS (HTTPS/WSS); plaintext connections shall be rejected. | M | T | STK-06 | TC-SEC-01 | Proposed |
| `NFR-SEC-02` | Session tokens shall be cryptographically random (≥ 256 bits entropy). | M | T | STK-06 | TC-SEC-02 | Proposed |
| `NFR-SEC-03` | Token secrets and TLS private keys shall be stored in files with permissions 0600 owned by the service user. | M | T | STK-06 | TC-SEC-03 | Proposed |
| `NFR-SEC-04` | The backend shall enforce rate limiting on control endpoints (max 10 commands/s per client). | M | T | STK-06 | TC-SEC-04, TC-SEC-11 | Proposed |
| `NFR-SEC-05` | WebSocket frames shall be subject to maximum payload size enforcement (configurable, default 64 KB). | M | T | STK-06 | TC-SEC-05 | Proposed |
| `NFR-SEC-06` | The backend shall enforce CORS policy; only configured origins shall be allowed. | M | T | STK-06 | TC-SEC-06 | Proposed |
| `NFR-SEC-07` | PTT activation shall require Operator role and be subject to a server-side safety timeout. | M | T | STK-01 | TC-RIG-05, TC-SEC-07 | Proposed |
| `NFR-SEC-08` | All rig command parameters shall be validated against an allowlist of values and numeric ranges. | M | T | STK-03 | TC-SEC-08 | Proposed |
| `NFR-SEC-09` | The system shall not leak stack traces, internal paths, or configuration details in error responses. | M | T | STK-06 | TC-SEC-09 | Proposed |
| `NFR-SEC-10` | Container images (if used) shall run as a non-root user with a read-only root filesystem. | S | T | STK-10 | TC-DEPLOY-05 | Proposed |
| `NFR-SEC-11` | Container images shall be rebuilt on upstream OS/dependency security patches within 7 days. | S | T | STK-10 | TC-DEPLOY-05 | Proposed |
| `NFR-SEC-12` | Authentication credentials shall never appear in URL query strings or log output. | M | T | STK-06 | TC-AUTH-04, TC-SEC-10 | Proposed |
| `NFR-SEC-13` | In distributed frontend deployments, backend ingress shall default to private tunnel interfaces only (not 0.0.0.0 public bind). | M | T | STK-07 | TC-SEC-12 | Proposed |
| `NFR-SEC-14` | Connections between frontend host and backend shall enforce mutual authentication (WireGuard peer keys, Tailscale identity ACLs, or mTLS). | M | T | STK-07 | TC-SEC-13 | Proposed |
| `NFR-SEC-15` | SSH tunnelling may be used only as an operator-maintained fallback mode and shall not be the default production profile. | S | T | STK-07 | TC-SEC-14 | Proposed |
| `NFR-SEC-16` | GPIO control shall enforce an allowlist of configured pins; all non-allowlisted pins shall be inaccessible and default to safe startup states. | M | T | STK-09 | TC-SEC-15 | Proposed |

### COMPAT — Browser/device compatibility

The frontend must run on current desktop and mobile browsers with no client install, fully
touch-operable, using the standard MediaDevices API for audio selection.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `NFR-COMPAT-01` | The web frontend shall function on Firefox (latest two major releases) on desktop Linux/Windows/macOS. | M | T | STK-05 | TC-COMPAT-01 | Proposed |
| `NFR-COMPAT-02` | The web frontend shall function on Chromium-based browsers (Chrome, Edge; latest two major releases) on desktop. | M | T | STK-05 | TC-COMPAT-02, TC-COMPAT-03 | Proposed |
| `NFR-COMPAT-03` | The web frontend shall function on Safari iOS 16+ on iPhone and iPad. | M | T | STK-05 | TC-COMPAT-04, TC-COMPAT-05, TC-AUD-04 | Proposed |
| `NFR-COMPAT-04` | The web frontend shall function on Firefox for Android (latest release). | M | T | STK-05 | TC-COMPAT-06 | Proposed |
| `NFR-COMPAT-05` | The web frontend shall function on Chrome for Android (latest release). | M | T | STK-05 | TC-COMPAT-07 | Proposed |
| `NFR-COMPAT-06` | All UI controls shall be operable by touch on mobile devices. | M | T | STK-05 | TC-COMPAT-06, TC-COMPAT-07 | Proposed |
| `NFR-COMPAT-07` | Audio device selection shall use the standard browser MediaDevices API. | M | T | STK-05 | TC-AUD-03 | Proposed |

### DEPLOY — Deployment targets & packaging

Raspberry Pi 4/5 reference targets, systemd native delivery, optional container profile, single
config file, documented rollback, split-host profile, and portability.

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `NFR-DEPLOY-01` | The reference deployment target shall be Raspberry Pi 4 (4 GB) and Raspberry Pi 5 running Raspberry Pi OS (64-bit, Bookworm). | M | T | STK-08 | TC-DEPLOY-01, TC-DEPLOY-02 | Proposed |
| `NFR-DEPLOY-02` | The system shall ship a systemd service unit as the native deployment method. | M | T | STK-08 | TC-DEPLOY-03 | Proposed |
| `NFR-DEPLOY-03` | The system shall be evaluated as a containerized deployment; if it meets audio latency and hardware-access thresholds it shall be a supported deployment profile. | S | T | STK-10 | TC-DEPLOY-04, TC-DEPLOY-05 | Proposed |
| `NFR-DEPLOY-04` | All configuration shall be sourced from a single file (default: ~/.config/landline/config.toml). | M | T | STK-08 | TC-DEPLOY-08 | Proposed |
| `NFR-DEPLOY-05` | The deployment shall include a documented rollback procedure. | M | T | STK-10 | TC-DEPLOY-06 | Proposed |
| `NFR-DEPLOY-06` | The backend architecture shall not preclude future non-Pi deployment targets. | S | A | STK-10 | TC-DEPLOY-09 | Proposed |
| `NFR-DEPLOY-07` | Deployment documentation shall include a dedicated profile for split-host operation (frontend host + backend host) with secure connectivity setup steps. | M | T | STK-07 | TC-DEPLOY-07 | Proposed |
| `NFR-DEPLOY-08` | The recommended split-host profile shall use WireGuard or Tailscale as the primary transport; the SSH tunnel profile shall be documented as fallback only. | M | T | STK-07 | TC-DEPLOY-07 | Proposed |

### LIC — Licensing

| ID | Statement | Prio | Verif | Up | Down | Status |
|---|---|---|---|---|---|---|
| `NFR-LIC-01` | The project shall be licensed under GNU Affero General Public License v3.0 (AGPL-3.0-only). | M | I | STK-11 | TC-LIC-01 | Proposed |
| `NFR-LIC-02` | The repository shall include a top-level LICENSE file containing the full AGPL-3.0 license text and a short license notice in key project docs. | M | I | STK-11 | TC-LIC-02 | Proposed |

---

## Coverage notes

`FR-RIG-07` (passband/filter width) and `FR-SPEC-04` (waterfall palette) are **Could**-priority and
intentionally carry no test at this draft; the gate reports them informationally and does not fail
on them. Every **Must/Should** requirement above traces down to at least one `TC-` in the
[test traceability matrix](../test/test-strategy.md).

## Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.6 | 2026-06-26 | DC0SK | Migrated requirements-spec.md to area-coded FR/NFR scheme with trace-up/down columns. |
