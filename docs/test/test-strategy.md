---
title: "Test Strategy & Traceability"
status: Draft
version: "0.19"
updated: 2026-07-20
authors:
  - Simon Keimer (DC0SK)
owns: [TC]
---

# Test Strategy & Traceability

> **License notice:** landline is licensed under **AGPL-3.0-only**. See the top-level
> [LICENSE](../../LICENSE).

## 1. Purpose

This document defines the test strategy, levels, environments, and the requirement traceability
matrix for **landline**. Every **Must/Should** requirement in
[system-requirements.md](../requirements/system-requirements.md) must be covered by at least one
test case (`TC-`) here, and every `TC-` must name the requirement ID(s) it verifies. These
invariants (rules **R3** and **R4** in [docs/README.md](../README.md) §4) are enforced by
[`scripts/trace-gate.py`](../../scripts/trace-gate.py), which fails the build on any uncovered M/S
requirement or any dangling/untraced test.

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
| Browser | UI and audio across the browser/device matrix | Manual + Playwright (where applicable) |
| Security | Auth bypass, replay, rate-limit, malformed-frame, injection | Manual + `websocat` / custom scripts |
| Performance / Soak | Sustained load on Pi 4/5: latency, CPU, memory, thermal | Custom load scripts + `top` / `htop` |
| Static | Lint / source inspection | `cargo clippy`, CI inspection |

### 2.3 Verification methods

Per [docs/README.md](../README.md) §3, each requirement is verified by `T` (automated test),
`D` (demonstration), `I` (inspection), or `A` (analysis). Requirements verified by `I`/`A` still
carry a `TC` whose **Method** note records `I`/`A` rather than `T`, so R3 holds uniformly.

### 2.4 Definition of Test Pass

A test passes when:
- The observed result matches the expected result.
- No unexpected panic, crash, or error log occurs.
- For security tests: the attack is blocked and an audit log event is produced.

---

## 3. Test Environments

| Environment | Description |
|---|---|
| Dev (local) | Developer machine; mock rigctld; self-signed TLS cert |
| Pi 4 integration | Raspberry Pi 4 (4 GB, RPiOS Bookworm 64-bit); real or simulated rig |
| Pi 5 integration | Raspberry Pi 5 (RPiOS Bookworm 64-bit); real or simulated rig |
| Browser matrix | See the COMPAT section of the traceability matrix |
| Container | Docker or Podman on Raspberry Pi OS |
| Portability | Non-Pi target (x86_64 Linux) for build/run portability checks |

---

## 4. Traceability Matrix

> Status values: `Not written` | `Draft` | `Ready` | `Pass` | `Fail` | `Blocked`.
> Each row: **ID** (backtick-quoted `TC-`), **Requirement(s)** (the `FR-`/`NFR-` id(s) it verifies),
> **Description**, **Level**, **Status**.

### 4.1 RIG — Rig Control

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-RIG-01` | `FR-RIG-01` | Read current frequency from rig via rigctld mock | Integration | Not written |
| `TC-RIG-02` | `FR-RIG-02` | Set valid frequency; verify rig receives command; verify out-of-range is rejected | Integration | Not written |
| `TC-RIG-03` | `FR-RIG-03`, `FR-RIG-04` | Read and set operating mode; verify unsupported mode is rejected | Integration | Not written |
| `TC-RIG-04` | `FR-RIG-05` | Activate PTT as Operator; verify rig state; deactivate | Integration | Not written |
| `TC-RIG-05` | `FR-RIG-05`, `NFR-SEC-07` | PTT activation attempt by Observer role; verify rejection and audit log | Integration | Pass (integration) |
| `TC-RIG-06` | `FR-RIG-06` | S-meter value streams to client at configured interval | Integration | Not written |
| `TC-RIG-07` | `FR-RIG-08` | rigctld TCP adapter connects, sends command, parses response | Unit | Not written |
| `TC-RIG-08` | `FR-RIG-09` | Send frequency below 0 Hz; verify server rejects with 400-equivalent error | Integration | Not written |
| `TC-RIG-09` | `FR-RIG-10` | Two concurrent Operator clients; verify only one can hold rig control at a time | System | Not written |

### 4.2 SPEC — Spectrum and Waterfall

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-SPEC-01` | `FR-SPEC-01` | WebSocket spectrum stream delivers FFT bins at configured rate | Integration | Not written |
| `TC-SPEC-02` | `FR-SPEC-02` | Change spectrum update rate via config; verify delivery rate changes | Integration | Not written |
| `TC-SPEC-03` | `FR-SPEC-03` | Waterfall canvas renders in Firefox and Chromium without errors | Browser | Not written |
| `TC-SPEC-04` | `FR-SPEC-03` | Waterfall renders on iOS Safari without WebGL dependency | Browser | Not written |
| `TC-SPEC-05` | `NFR-PERF-05` | Measure spectrum update rate on Pi 4 under load; verify ≥ 2 Hz sustained | Performance | Not written |

### 4.3 AUD — Audio

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-AUD-01` | `FR-AUD-01` | Received audio streams from backend to browser client over WSS | System | Not written |
| `TC-AUD-02` | `FR-AUD-02` | Microphone audio streams from browser to backend over WSS | System | Not written |
| `TC-AUD-03` | `FR-AUD-03`, `FR-AUD-04`, `NFR-COMPAT-07` | Audio device selector lists available input and output devices in browser via the MediaDevices API | Browser | Not written |
| `TC-AUD-04` | `NFR-COMPAT-03` | Audio device selection functions on iOS Safari (mic permission flow) | Browser | Not written |
| `TC-AUD-05` | `FR-AUD-05` | Opus-encoded audio decoded without artefacts at 16 kbps | System | Not written |
| `TC-AUD-06` | `FR-AUD-06` | Simulate 5 % packet loss; verify audio degrades gracefully without crash | System | Not written |

### 4.4 AUTH — Authentication and Session

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-AUTH-01` | `FR-AUTH-01` | Unauthenticated WebSocket connection attempt; verify rejection and no data leak | Security | Not written |
| `TC-AUTH-02` | `FR-AUTH-02` | Issue token; wait for expiry; verify subsequent requests are rejected **and that an already-open WebSocket stops streaming** | Integration | Pass (integration) |
| `TC-AUTH-03` | `FR-AUTH-03` | Token refresh flow; verify new token issued and old token invalidated | Integration | Not written |
| `TC-AUTH-04` | `FR-AUTH-04`, `NFR-SEC-12` | Observer attempts rig command; verify 403-equivalent rejection; no credential in logs | Security | Not written |
| `TC-AUTH-05` | `FR-AUTH-05` | Explicit logout; verify session invalidated; re-use of old token rejected; **an already-open WebSocket is closed** | Integration | Pass (integration) |

### 4.5 AUDIT — Audit and Logging

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-AUDIT-01` | `FR-AUDIT-01`, `FR-AUDIT-02` | Execute rig command; verify audit log entry with timestamp, IP, user, action, params | Integration | Not written |
| `TC-AUDIT-02` | `FR-AUDIT-04` | Failed login attempt; verify log entry with IP and timestamp; no password in log | Security | Not written |
| `TC-AUDIT-03` | `FR-AUDIT-03` | Audit log rotation/retention check; verify entries are retained ≥ 30 days | Integration | Not written |

### 4.6 HOST — Split-Host Frontend Deployment

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-HOST-01` | `FR-HOST-01` | Run frontend assets from a separate host; verify full control and telemetry flow to backend | System | Not written |
| `TC-HOST-02` | `FR-HOST-02`, `FR-HOST-04` | Configure frontend API/WSS base URLs for remote backend; verify no rebuild required | System | Not written |
| `TC-HOST-03` | `FR-HOST-03` | Validate WireGuard/Tailscale tunnel profile carries API and WSS traffic between hosts | System | Not written |

### 4.7 GPIO — Digital I/O

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-GPIO-01` | `FR-GPIO-01` | Configure and control at least 5 Raspberry Pi GPIO pins; verify readback and output level changes | System | Not written |

### 4.8 PERF — Performance

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-PERF-01` | `NFR-PERF-01` | Measure control command RTT over LAN; p95 must be < 100 ms | Performance | Not written |
| `TC-PERF-02` | `NFR-PERF-02` | Measure end-to-end audio latency LAN; target < 300 ms | Performance | Not written |
| `TC-PERF-03` | `NFR-PERF-03` | Connect 3 concurrent clients; verify all receive spectrum and control responds normally | Performance | Not written |
| `TC-PERF-04` | `NFR-PERF-04` | Sustained 30-minute load on Pi 4 with 3 clients; CPU must remain < 50 % | Performance | Not written |

### 4.9 REL — Reliability

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-REL-01` | `NFR-REL-01` | Force TCP disconnect; verify client reconnects within 30 s with exponential backoff | System | Not written |
| `TC-REL-02` | `NFR-REL-02` | Kill rigctld process; verify backend recovers within 5 s on restart | System | Not written |
| `TC-REL-03` | `NFR-REL-03` | Run system continuously for 24 hours; verify no crash, no memory leak trend | Soak | Not written |

### 4.10 MAINT — Maintainability

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-MAINT-01` | `NFR-MAINT-01` | Run `cargo clippy` (pedantic) in CI; verify zero warnings (Method: Inspection) | Static | Not written |
| `TC-MAINT-02` | `NFR-MAINT-02` | Enumerate public backend API surface and confirm each endpoint has an integration test (Method: Analysis) | Analysis | Not written |

### 4.11 SEC — Security

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-SEC-01` | `NFR-SEC-01` | Connect via plain `ws://`; verify backend rejects or redirects | Security | Not written |
| `TC-SEC-02` | `NFR-SEC-02` | Inspect issued tokens; verify ≥ 256-bit entropy, no predictable pattern | Security | Not written |
| `TC-SEC-03` | `NFR-SEC-03` | Check file permissions on token secret and TLS key files; must be 0600 | Security | Not written |
| `TC-SEC-04` | `NFR-SEC-04` | Send 20 control commands/s from one client; verify rate limit triggers after 10/s | Security | Not written |
| `TC-SEC-05` | `NFR-SEC-05` | Send oversized WebSocket frame (> 64 KB); verify backend closes connection cleanly | Security | Not written |
| `TC-SEC-06` | `NFR-SEC-06` | Send request with disallowed Origin header; verify CORS rejection | Security | Not written |
| `TC-SEC-07` | `NFR-SEC-07` | PTT timeout: leave PTT active; verify the server sends the unkey **to the rig** after the safety timeout, keeps PTT reported active if the rig does not confirm, and unkeys on shutdown | Security | Pass (unit/integration, mock rigctld) |
| `TC-SEC-08` | `NFR-SEC-08` | Send mode parameter with shell metacharacters; verify rejection without command execution | Security | Not written |
| `TC-SEC-09` | `NFR-SEC-09` | Trigger server error; verify response body contains no stack trace or file paths | Security | Not written |
| `TC-SEC-10` | `NFR-SEC-12` | Capture HTTP/WS traffic; verify no token or credential in URL query strings | Security | Not written |
| `TC-SEC-11` | `NFR-SEC-04` | Replay captured authenticated WebSocket message 60 s later; verify rejection | Security | Not written |
| `TC-SEC-12` | `NFR-SEC-13` | Verify backend bind/address policy in split-host mode exposes service only on tunnel interface(s) | Security | Not written |
| `TC-SEC-13` | `NFR-SEC-14` | Attempt split-host connection with untrusted peer identity; verify rejected access | Security | Not written |
| `TC-SEC-14` | `NFR-SEC-15` | Validate SSH tunnel fallback documentation and controls; verify fallback is disabled by default | Security | Not written |
| `TC-SEC-15` | `NFR-SEC-16` | Attempt control of non-allowlisted GPIO pins and verify denial; verify configured startup states are safe on service start | Security | Not written |

### 4.12 COMPAT — Browser Matrix

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-COMPAT-01` | `NFR-COMPAT-01` | Firefox latest, Linux desktop: controls, spectrum, audio | Browser | Not written |
| `TC-COMPAT-02` | `NFR-COMPAT-02` | Chrome latest, Linux desktop: controls, spectrum, audio | Browser | Not written |
| `TC-COMPAT-03` | `NFR-COMPAT-02` | Edge latest, Windows desktop: controls, spectrum, audio | Browser | Not written |
| `TC-COMPAT-04` | `NFR-COMPAT-03` | Safari iOS 16+, iPhone: controls, spectrum, audio | Browser | Not written |
| `TC-COMPAT-05` | `NFR-COMPAT-03` | Safari iOS 16+, iPad: controls, spectrum, audio | Browser | Not written |
| `TC-COMPAT-06` | `NFR-COMPAT-04`, `NFR-COMPAT-06` | Firefox Android latest: controls, spectrum, audio; verify all controls are touch-operable | Browser | Not written |
| `TC-COMPAT-07` | `NFR-COMPAT-05`, `NFR-COMPAT-06` | Chrome Android latest: controls, spectrum, audio; verify all controls are touch-operable | Browser | Not written |

### 4.13 DEPLOY — Deployment

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-DEPLOY-01` | `NFR-DEPLOY-01` | Build and run on Pi 4 (RPiOS 64-bit Bookworm); verify service starts and rig connects | System | Not written |
| `TC-DEPLOY-02` | `NFR-DEPLOY-01` | Build and run on Pi 5; verify same binary or equivalent runs correctly | System | Not written |
| `TC-DEPLOY-03` | `NFR-DEPLOY-02` | Install systemd unit; verify auto-start on reboot and clean stop on `systemctl stop` | System | Not written |
| `TC-DEPLOY-04` | `NFR-DEPLOY-03` | Build and run container image; verify audio device access and latency within threshold | System | Not written |
| `TC-DEPLOY-05` | `NFR-DEPLOY-03`, `NFR-SEC-10`, `NFR-SEC-11` | Verify container runs as non-root with read-only rootfs; verify image is rebuilt on security patches within 7 days | Security | Not written |
| `TC-DEPLOY-06` | `NFR-DEPLOY-05`, `NFR-MAINT-03` | Execute rollback procedure; verify previous version restores and service resumes without data loss | System | Not written |
| `TC-DEPLOY-07` | `NFR-DEPLOY-07`, `NFR-DEPLOY-08` | Execute split-host deployment runbook with WireGuard or Tailscale profile and verify successful connectivity; verify SSH documented as fallback only | System | Not written |
| `TC-DEPLOY-08` | `NFR-DEPLOY-04` | Load a single config file (~/.config/landline/config.toml); verify documented keys and defaults are applied | Integration | Not written |
| `TC-DEPLOY-09` | `NFR-DEPLOY-06` | Build and run on a non-Pi target (x86_64 Linux) to prove the architecture does not preclude portability (Method: Analysis) | System | Not written |

### 4.14 LIC — Licensing

| ID | Requirement(s) | Description | Level | Status |
|---|---|---|---|---|
| `TC-LIC-01` | `NFR-LIC-01` | Verify repository license identifier and declared license are AGPL-3.0-only (Method: Inspection) | Static | Not written |
| `TC-LIC-02` | `NFR-LIC-02` | Verify top-level LICENSE file exists and contains AGPL-3.0 text; verify docs include short license notice (Method: Inspection) | Static | Not written |

---

## 5. Test Execution Record Template

For each test execution, record the following alongside the test case:

```
Test ID    : TC-XXX-nn
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

- [ ] `scripts/trace-gate.py` exits 0 (R3 M/S coverage and R4 no dangling traces satisfied).
- [ ] All Must-priority tests in the phase scope have status `Pass`.
- [ ] All security tests relevant to the phase have status `Pass`.
- [ ] No requirement ID in the phase scope has zero mapped tests.
- [ ] Test execution records are filled with date, executor, and evidence.
- [ ] Any `Fail` or `Blocked` tests have a tracked issue with a disposition (fix, defer with justification).

---

## 6a. Phase 1 execution record (2026-07-05)

Snapshot taken at the Phase 1 exit review (roadmap A27). "Automated" = covered by a
green test in the CI suite (55 Rust tests + 22 frontend tests). "HIL" = requires
hardware-in-the-loop (real Raspberry Pi + rigctld + transceiver). "Browser" = manual
browser-matrix run. "Deferred" = delivered in a later phase.

| Test cases | Level | Status | Evidence / note |
|---|---|---|---|
| TC-AUTH-01..05 | Unit + HTTP | **Automated — pass** | `backend/src/auth.rs`, `backend/tests/auth.rs`. TC-AUTH-01 WS variant is Phase 2. |
| TC-SEC-02, -04, -06, -07, -08, -10, -15 | Unit + HTTP | **Automated — pass** | `auth.rs`, `security.rs`, `rig.rs`, `gpio.rs` + tests. |
| TC-SEC-01 | Security | **Deferred (Phase 4)** | TLS/WSS enforcement via reverse proxy. |
| TC-SEC-05, TC-SEC-11 | Security | **Deferred (Phase 2)** | WS frame-size cap / WS replay — land with the WS transport. |
| TC-SEC-03, TC-SEC-09 | Security | **Partial** | Secret storage / error sanitisation documented + auth/rig paths sanitised; in-app 0600 check (BL-081) and a global sanitiser (BL-032) remain. |
| TC-RIG-01..05, -07, -08 | Unit + HTTP | **Automated — pass (mock rigctld)** | `backend/tests/control.rs`, `rig.rs`; real-radio is HIL (ASM-05). |
| TC-RIG-06 | Integration | **Partial** | S-meter read path automated; streaming at interval is Phase 2. |
| TC-RIG-09 | Integration | **Automated (mechanism)** | Exclusive access via the adapter's async mutex; concurrency HIL. |
| TC-AUDIT-01, -02 | Integration + Security | **Automated — pass** | `backend/tests/control.rs`, `audit.rs`. |
| TC-REL-01 | System | **Automated (logic)** | `frontend/src/socket.ts` backoff + reconnect; live TCP-drop run is HIL. |
| TC-REL-02 | System | **HIL** | Kill/restart rigctld on a real host. |
| TC-GPIO-01 | System | **HIL** | Real-pin readback; allowlist/safe-state logic verified in-memory (TC-SEC-15). |
| TC-PERF-01 | Performance | **HIL** | Control-latency measurement on a real LAN deployment. |
| TC-DEPLOY-01, -03 | System | **Partial / HIL** | aarch64 cross-build verified; systemd unit written; Pi start/stop is HIL. |
| TC-COMPAT-01..07 | Browser | **Browser (manual)** | Firefox/Chromium/mobile E2E of the built frontend. |

**Disposition:** Phase 1 is software-complete and green under automation; the formal exit gate
is held pending the HIL, browser-matrix, and Phase-4 TLS items above (no `Fail` results — all
open items are `Deferred`/`Blocked-on-hardware` with a tracked cause in the backlog).

## 6b. Phase 2 execution record (2026-07-05)

Snapshot at the Phase 2 exit review. Same legend as §6a.

| Test cases | Level | Status | Evidence / note |
|---|---|---|---|
| TC-SPEC-01 | Integration | **Automated — pass** | `backend/tests/ws.rs`: authed WS delivers FFT-bin spectrum frames. |
| TC-SPEC-02 | Integration | **Partial** | Update rate is configurable + clamped 1–10 Hz (`spectrum` config, WS loop); an automated delivery-rate assertion is not yet written. |
| TC-SPEC-03, TC-SPEC-04 | Browser | **Browser (manual)** | Canvas 2D waterfall renders; pure colour-map logic unit-tested (`frontend/src/waterfall.test.ts`). No WebGL → iOS-Safari-safe by construction; on-device render is a browser-matrix run. |
| TC-SPEC-05 | Performance | **HIL** | ≥ 2 Hz sustained under load on a Pi 4 (default rate 5 Hz). |
| TC-AUTH-01 (WS) | Security | **Automated — pass** | `backend/tests/ws.rs`: unauth / bad-token WS handshakes rejected. |
| TC-SEC-05 (WS frame) | Security | **Automated (config) / HIL** | WS upgrade caps message/frame size; an oversized-frame close is exercised on the real transport. |
| TC-AUD-03, TC-AUD-04 | Browser | **Software done / browser** | MediaDevices enumeration + partition implemented and unit-tested (`frontend/src/audio-devices.test.ts`, NFR-COMPAT-07); on-device selection + iOS mic-permission flow is a browser-matrix run. |
| TC-COMPAT-01..07 | Browser | **Browser (manual)** | Responsive layout + touch targets implemented; full matrix needs real devices. |

**Disposition:** Phase 2 is software-complete and green under automation; the gate is held
pending the browser-matrix and Pi HIL items (no `Fail` results).

---

## 6c. Rig HIL execution record (2026-07-05)

First hardware-in-the-loop run: the release backend (static-musl aarch64 build) on the target
Raspberry Pi, driving a real **Yaesu FT-991A** through `rigctld` (hamlib model 1035, CAT
`/dev/ttyUSB0` @ 38400, NET on `127.0.0.1:4532`). Exercised via the HTTP API with an
`operator` token. **No transmit performed**; frequencies constrained to 2 m / 10 m per the
operator's safety limit.

| Test cases | Level | Status | Evidence / note |
|---|---|---|---|
| TC-AUTH-01/02 | Security | **HIL — pass** | Login against the real deployment issues a working JWT; `authorization: Bearer` accepted; missing token → 401. |
| TC-RIG-01 (frequency) | Integration | **HIL — pass** | `GET` reads 144.600 MHz; `POST` set to 145.500 MHz (2 m) and 28.400 MHz (10 m) → 204, readback confirms; restored to 144.600. |
| TC-RIG-02 (mode) | Integration | **HIL — pass** | `GET` reads `USB`; `POST` mode `USB` passband 2400 → 204, readback confirms. |
| TC-RIG (S-meter) | Integration | **HIL — pass** | `GET /api/rig/smeter` returns live strength (−54) from the rig. |
| TC-SEC (input validation) | Security | **HIL — pass** | Negative frequency and a mode-injection string (`USB;rm -rf /`) both rejected with 400 before reaching rigctld. |
| TC-AUDIT-01 | Security | **HIL — pass** | Hash-chained audit log records `auth.login`, `rig.set_freq`, `rig.set_mode` with `outcome="success"`, seq 0–4. |
| TC-RIG (PTT) | Integration | **Deferred (safety)** | Not run — requires a dummy load; PTT/TX withheld pending operator confirmation. |
| TC-SPEC-01 (real RF) | Integration | **HIL — pass** | With `--features audio-device`, the `CpalCapture` adapter opens the rig's USB codec and the spectrum WS delivers **live** FFT frames — peak bin and noise floor vary frame-to-frame (real audio), unlike the fixed synthetic tone. |
| TC-AUD (real capture) | Integration | **HIL — pass** | Binary audio WS frames stream from the same capture tap (seq-incrementing, non-empty). With `--features audio-device,opus` they are Opus-encoded; content-listening is a browser test. |
| Static frontend serving | Integration | **HIL — pass** | `[server] static_dir` serves the UI at `/` on the API origin: `GET /`, `/styles.css`, `/dist/main.js`, `/healthz` all 200. |
| TC-GPIO-01 / NFR-SEC-16 | Integration | **HIL — pass** | With `--features gpio-device`, the `GpiodBackend` drives real pins on `/dev/gpiochip0`. Independently verified via kernel debugfs (`/sys/kernel/debug/gpio`, not the API's own readback): pin 17 claimed `out lo` at startup (safe state); API HIGH → debugfs `out hi`; LOW → `out lo`. Allowlist: non-listed pin 5 → 403; input pin 27 driven → 400; `gpio.set` audited. Pin 17 confirmed unconnected by the operator. |

**Bug found & fixed:** login initially 401'd — the config example used `[[users]]` instead of
`[[auth.users]]`, so no users loaded (fixed + regression-tested, PR #31).

**Bug found & fixed (audit, 2026-07-20):** the PTT safety timeout cleared its internal `active`
flag *before* — and regardless of whether — the rig confirmed the unkey, and the sole TC-SEC-07
test asserted that flag rather than the command reaching the rig; deleting the real unkey call
left the suite green. Shutdown also never unkeyed, so a SIGTERM mid-transmission dropped the
safety timer with the rig still keyed. Both fixed and now covered by mock-rigctld tests that
assert on the recorded `T 0` command. **Evidence tier: mock rigctld only** — the PTT path still
has no HIL evidence and remains deferred pending a dummy load.

**Disposition:** rig-control, real spectrum/audio-capture, **and GPIO** HIL are **green**
(CpalCapture/CpalSink + GpiodBackend adapters validated on the FT-991A / Pi hardware). Remaining
HIL: PTT (needs dummy load), in-browser audio *playback* (PCM works; Opus needs browser-side
decode, BL-072), and the browser matrix.

---

## 7. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.19 | 2026-07-21 | DC0SK | GPIO hardware-init failure now runs degraded-but-honest (operator decision): the station keeps serving, GPIO reports the fault on every operation, /healthz and the UI surface it. Previously it silently simulated working pins. |
| 0.18 | 2026-07-21 | DC0SK | WebSocket sessions now carry a per-session message budget and a concurrent-session cap; the HTTP limiter only ever ran on the upgrade. |
| 0.17 | 2026-07-21 | DC0SK | Login hardened against user enumeration by timing (measured 773,000:1 before, ~1:1 after) and Argon2 moved off the async executor with a concurrency bound. |
| 0.16 | 2026-07-21 | DC0SK | Every role-gated endpoint now audits its denials through one seam, and WebSocket TX audio is audited (accepted and denied). Closes the "blocked AND audited" gap in §2.4 for GPIO/frequency/mode and gives the WS TX path its first test. |
| 0.15 | 2026-07-20 | DC0SK | Audio sample rate is now requested from the device and advertised to clients rather than assumed at both ends (audit finding). Rate-matching unit-tested; the device-negotiation half still needs HIL on the FT-991A codec. |
| 0.14 | 2026-07-20 | DC0SK | GPIO hardware failures now propagate instead of being swallowed (audit finding): fault-injection tests cover the error path the real gpiod backend cannot exercise in CI. |
| 0.13 | 2026-07-20 | DC0SK | TC-AUTH-02/05 extended to the WebSocket path: an open socket must stop streaming on token expiry or logout (audit finding — auth ran only at connect). |
| 0.12 | 2026-07-20 | DC0SK | TC-SEC-07 strengthened: asserts the unkey command reaching the rig (mock rigctld) rather than an internal flag; records the PTT unkey-confirmation and shutdown-unkey defects found by the loose-ends audit. |
| 0.11 | 2026-07-08 | DC0SK | §6c: GpiodBackend GPIO validated on real Pi hardware — kernel-debugfs-verified safe state + drive HIGH/LOW on pin 17, allowlist/direction enforcement (TC-GPIO-01/NFR-SEC-16). |
| 0.10 | 2026-07-05 | DC0SK | §6c: CpalCapture/CpalSink audio-device adapter validated on the FT-991A USB codec — live spectrum from real RF, real audio-capture frames, static frontend serving. |
| 0.9 | 2026-07-05 | DC0SK | Added §6c rig HIL execution record: rig control validated on a real Yaesu FT-991A (read/set freq 2 m+10 m, mode, S-meter, input-validation, audit); PTT + real spectrum/audio still deferred. |
| 0.8 | 2026-07-05 | DC0SK | Added §6b Phase 2 execution record: spectrum/WS automated vs. browser-matrix/HIL status at the Phase 2 exit review. |
| 0.7 | 2026-07-05 | DC0SK | Added §6a Phase 1 execution record (A27): automated / HIL / browser / deferred status per test-case group at the Phase 1 exit review. |
| 0.6 | 2026-06-26 | DC0SK | Migrated to TC ids; added TC-SPEC-05/AUDIT-03/MAINT-01/MAINT-02/DEPLOY-08/DEPLOY-09 to close M/S coverage. |
