---
title: Product Backlog
status: Draft
version: 0.5.21
updated: 2026-07-05
authors:
  - Simon Keimer (DC0SK)
---

# Product Backlog

## 1. Backlog Conventions

### Priority Classes (MoSCoW)

| Class | Meaning |
|---|---|
| Must | Required for the target release; blocks shipment if missing |
| Should | High value; included unless capacity forces deferral |
| Could | Nice to have; included only if Must and Should items are complete |
| Won't | Explicitly deferred; recorded to prevent scope creep |

### Item Fields

Each item carries: **ID**, **Title**, **Priority**, **Phase**, **Estimate** (S/M/L/XL), **Dependencies**, **Req IDs**, **Test IDs**, **Acceptance Criteria**, **Status**, **Notes**.

### Status Values

`Proposed` | `Approved` | `In Progress` | `Done` | `Deferred` | `Rejected`

### Definition of Done

- Implementation complete and merged.
- Security gate passed (if item touches auth, transport, input handling, or rig commands).
- Mapped tests written and passing.
- Requirements and test strategy updated if new/changed.
- Documentation updated if behaviour is user-visible.

---

## 2. Epics

| Epic ID | Title | Phase |
|---|---|---|
| EP-01 | Project documentation and governance | Phase 0 |
| EP-02 | Security baseline and threat model | Phase 0 |
| EP-03 | Backend foundation and rig control | Phase 1 |
| EP-04 | Frontend: control UI and session management | Phase 1 |
| EP-05 | Spectrum and waterfall pipeline | Phase 2 |
| EP-06 | Mobile browser compatibility | Phase 2 |
| EP-07 | Audio pipeline | Phase 3 |
| EP-08 | Deployment: native (systemd) | Phase 1 |
| EP-09 | Deployment: container evaluation | Phase 3 |
| EP-10 | Operations and release hardening | Phase 4 |
| EP-11 | Split-host frontend deployment security | Phase 3 |
| EP-12 | Licensing and compliance | Phase 0 |

---

## 3. Backlog Items

### EP-01 — Project documentation and governance

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-001 | Create system-requirements.md with all FR/NFR IDs | Must | 0 | S | — | All FR/NFR | — | Done |
| BL-002 | Create test-strategy.md with traceability matrix | Must | 0 | S | BL-001 | All FR/NFR | — | Done |
| BL-003 | Create backlog.md (this document) | Must | 0 | S | BL-001 | — | — | Done |
| BL-004 | Create roadmap.md with phase/release plan | Must | 0 | S | BL-001 | — | — | Done |
| BL-005 | Define change control process for doc updates | Must | 0 | S | BL-001 | — | — | Done |
| BL-006 | Create governance charter with security-first policy | Must | 0 | S | BL-001 | NFR-SEC-*, NFR-LIC-* | TC-SEC-* | Done |

**Acceptance Criteria — BL-005:** Change control procedure is documented in docs/governance.md or equivalent; any PR touching requirements, tests, backlog, or roadmap must update all four artifacts together.
**Acceptance Criteria — BL-006:** docs/governance.md defines security-first as a release-gated governance rule and includes exception handling requirements.

---

### EP-02 — Security baseline and threat model

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-010 | Document trust boundaries and deployment modes | Must | 0 | S | BL-001 | NFR-SEC-* | — | Done |
| BL-011 | Define security release gates and acceptance criteria | Must | 0 | S | BL-010 | NFR-SEC-* | TC-SEC-* | Done |
| BL-012 | Define secrets storage and rotation policy | Must | 0 | S | BL-010 | NFR-SEC-03 | TC-SEC-03 | Done |
| BL-013 | Create docs/security.md (threat model + controls) | Must | 0 | M | BL-010 | NFR-SEC-* | — | Done |

**Acceptance Criteria — BL-011:** Documented list of security gates with pass/fail criteria; referenced in phase exit checklists in roadmap.
**Note — BL-012:** Partially met, so kept Proposed: docs/security.md §8 defines secrets *storage* (0600 file mode, no credentials in URLs/logs, no secrets in container images) but the *rotation* policy is only deferred there ("defined before production release") and remains an open TODO in security.md §9 (key/token rotation runbook with operational cadence).

---

### EP-03 — Backend foundation and rig control

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-020 | Initialize Rust workspace: Tokio + Axum + Tower | Must | 1 | S | BL-011 | NFR-MAINT-01 | — | Done |
| BL-021 | Implement auth middleware (JWT, expiry, role claims) | Must | 1 | M | BL-020, BL-012 | FR-AUTH-01–FR-AUTH-05, NFR-SEC-01–NFR-SEC-02 | TC-AUTH-01–TC-AUTH-05, TC-SEC-01–TC-SEC-02 | Done |
| BL-022 | Implement rate limiting and frame/size limits | Must | 1 | S | BL-021 | NFR-SEC-04–NFR-SEC-05 | TC-SEC-04–TC-SEC-05 | Done |
| BL-023 | Implement CORS origin policy | Must | 1 | S | BL-021 | NFR-SEC-06 | TC-SEC-06 | Done |
| BL-024 | Implement audit log subsystem | Must | 1 | M | BL-020 | FR-AUDIT-01–FR-AUDIT-04 | TC-AUDIT-01–TC-AUDIT-02 | Done |
| BL-025 | Implement rigctld TCP adapter with command sanitisation | Must | 1 | M | BL-020 | FR-RIG-08–FR-RIG-09 | TC-RIG-07–TC-RIG-08 | Done |
| BL-026 | Implement frequency read/set handlers | Must | 1 | S | BL-025, BL-021 | FR-RIG-01–FR-RIG-02 | TC-RIG-01–TC-RIG-02 | Done |
| BL-027 | Implement mode read/set handlers | Must | 1 | S | BL-025, BL-021 | FR-RIG-03–FR-RIG-04 | TC-RIG-03 | Done |
| BL-028 | Implement PTT handler with role check and safety timeout | Must | 1 | M | BL-025, BL-021 | FR-RIG-05, NFR-SEC-07 | TC-RIG-04–TC-RIG-05, TC-SEC-07 | Done |
| BL-029 | Implement S-meter streaming | Should | 1 | S | BL-025 | FR-RIG-06 | TC-RIG-06 | In Progress |
| BL-030 | Implement rig access mutex for concurrent clients | Must | 1 | S | BL-025 | FR-RIG-10 | TC-RIG-09 | Done |
| BL-031 | Implement rigctld reconnect/circuit-breaker | Should | 1 | M | BL-025 | NFR-REL-02 | TC-REL-02 | Done |
| BL-032 | Structured tracing/logging integration | Must | 1 | S | BL-020 | NFR-SEC-09, NFR-SEC-12 | TC-SEC-09–TC-SEC-10 | Done |
| BL-033 | Implement Raspberry Pi GPIO control API for at least 5 digital pins | Must | 1 | M | BL-020, BL-021 | FR-GPIO-01, NFR-SEC-16 | TC-GPIO-01, TC-SEC-15 | Done |

**Note — BL-094 (container decision):** In Progress. The container artifacts (Dockerfile, compose) and a decision-record skeleton with the acceptance-threshold table are in `deploy/container/`; the accept/defer decision is deferred pending the Pi HIL benchmark (BL-092 device passthrough, BL-093 latency).

**Note — split-host (A34):** BL-110–114 Done as documentation/config artifacts in `deploy/split-host/` (topology, WireGuard templates, Tailscale ACL, SSH fallback, tunnel-interface bind). On-network verification (TC-HOST-01/02/03, TC-SEC-12/13/14, TC-DEPLOY-07) is hardware-in-the-loop. BL-113 (FR-HOST-04) is fully done in code (`LANDLINE_API_BASE`).

**Note — audio (A31):** The ARC-05 `audio` module and the WS audio transport are done in software: a reordering `JitterBuffer` with graceful loss concealment (FR-AUD-06, BL-077 In Progress), a `Codec` seam + `PcmCodec` with `[audio]` bitrate config (FR-AUD-05, BL-076 In Progress), and the **authenticated WS binary audio RX stream** (BL-071 In Progress — transport + per-session auth BL-075 Done; streams synthetic-source PCM frames, tested in `backend/tests/ws.rs`). The device ends and codec remain hardware-in-the-loop: libopus `OpusCodec` (feature-gated native adapter, keeps the default aarch64 cross-build C-free), CPAL capture/playback (BL-070/074), the browser Web Audio playback + mic TX (BL-072/073), and end-to-end latency (BL-078).

**Note — BL-060:** In Progress. The software targets are met — responsive layout, Canvas 2D waterfall (no WebGL), and MediaDevices-based device selection all implemented and unit-tested. Executing the full browser matrix (TC-COMPAT-01–07, TC-AUD-03/04 on Firefox/Chromium/Edge desktop + iOS Safari + Chrome Android) needs real devices and is the manual/HIL remainder.

**Note — BL-053:** In Progress. The waterfall renderer uses only the Canvas 2D context + `ImageData` — no WebGL — so it is structurally iOS-Safari-compatible (FR-SPEC-03). On-device confirmation on iOS Safari (TC-SPEC-04) is part of the browser-matrix run (BL-060), which needs real devices.

**Note — BL-029:** In Progress. The S-meter read path is done (`GET /api/rig/smeter`, Observer+, FR-RIG-06 display). Continuous streaming at a configured cadence (TC-RIG-06) rides the Phase-2 WebSocket telemetry channel (ADR-02/ARC-06) alongside the spectrum stream, so it lands in Phase 2.

**Note — BL-033:** Done. The ARC-08 GPIO controller enforces the pin allowlist and safe startup states (NFR-SEC-16) with Operator-gated, audited `/api/gpio/{pin}` endpoints, verified in-memory (TC-SEC-15). The Raspberry Pi sysfs/gpiod hardware backend is a thin deployment-time adapter; TC-GPIO-01 is a hardware-in-the-loop System test.

**Note — security remainders closed:** BL-081 (config now rejects group/world-accessible files, 0600 enforced on Unix — NFR-SEC-03), BL-032 (global panic sanitisation via `catch_panic_layer` completes NFR-SEC-09 alongside the typed sanitised errors), and BL-100/101 (nginx TLS reverse-proxy config in `deploy/nginx/` delivers NFR-SEC-01/TC-SEC-01) are Done. With TLS delivered by the proxy, BL-021 (auth middleware) is now Done. BL-012 (secrets *rotation* policy) is now Done — see [security.md §8.2](security.md).

**Note — ops/release docs (A35/A38):** BL-012 (rotation policy, security.md §8.2), BL-082 (rollback) + BL-105 (ops runbook) in `deploy/RUNBOOK.md`, and BL-106 (release checklist) + BL-122 (license-compliance gate) in `docs/release-checklist.md` are Done. BL-104 (final doc alignment / all IDs traced) stays In Progress until the first release, when the phase execution records are all filled from real test runs (currently HIL-gated).

**Note — BL-024:** Done. The ARC-07 audit subsystem (SHA-256 hash-chained tamper-evident events, durable append file, Admin `GET /api/audit`) logs auth failures (FR-AUDIT-04 / TC-AUDIT-02) and rig state-changes via the control handlers (FR-AUDIT-01 / TC-AUDIT-01, verified). FR-AUDIT-03 30-day retention (TC-AUDIT-03) is enforced by deployment log rotation.

**Note — BL-022:** Done. Per-client rate limiting (NFR-SEC-04) and the HTTP request body-size limit (ARC-03) plus the WebSocket **frame**-size cap (NFR-SEC-05 / TC-SEC-05, enforced on the ARC-01 WS upgrade in Phase 2) are all implemented. Remaining hardening: rate-limit keying on `X-Forwarded-For` behind the reverse proxy is a Phase-4 follow-up (BL-100/101).

---

### EP-04 — Frontend: control UI and session management

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-040 | Initialize TypeScript/HTML5 frontend project | Must | 1 | S | BL-021 | NFR-COMPAT-01–NFR-COMPAT-02 | — | Done |
| BL-041 | Implement authenticated session bootstrap (login, token storage, logout) | Must | 1 | M | BL-040, BL-021 | FR-AUTH-01–FR-AUTH-05 | TC-AUTH-01–TC-AUTH-05 | Done |
| BL-042 | Implement frequency display and tuning control | Must | 1 | M | BL-040, BL-026 | FR-RIG-01–FR-RIG-02, NFR-COMPAT-06 | TC-RIG-01–TC-RIG-02, TC-COMPAT-01–TC-COMPAT-02 | Done |
| BL-043 | Implement mode selector | Must | 1 | S | BL-040, BL-027 | FR-RIG-03–FR-RIG-04 | TC-RIG-03 | Done |
| BL-044 | Implement PTT button with visual transmit indicator | Must | 1 | S | BL-040, BL-028 | FR-RIG-05 | TC-RIG-04 | Done |
| BL-045 | Implement S-meter display | Should | 1 | S | BL-040, BL-029 | FR-RIG-06 | TC-RIG-06 | Done |
| BL-046 | Implement WebSocket client with reconnect/backoff | Must | 1 | M | BL-040 | NFR-REL-01 | TC-REL-01 | Done |
| BL-047 | Responsive CSS layout (desktop 3-column, mobile vertical stack) | Must | 1 | M | BL-040 | NFR-COMPAT-03–NFR-COMPAT-06 | TC-COMPAT-04–TC-COMPAT-07 | Done |

---

### EP-05 — Spectrum and waterfall pipeline

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-050 | Implement FFT pipeline (rustfft) on audio capture thread | Must | 2 | M | BL-025 | FR-SPEC-01 | TC-SPEC-01 | Done |
| BL-051 | Stream FFT bin data to clients over WebSocket at configurable rate | Must | 2 | M | BL-050, BL-021 | FR-SPEC-01–FR-SPEC-02 | TC-SPEC-01–TC-SPEC-02 | Done |
| BL-052 | Implement Canvas 2D waterfall renderer in frontend | Must | 2 | M | BL-040, BL-051 | FR-SPEC-03 | TC-SPEC-03 | Done |
| BL-053 | Verify waterfall rendering on iOS Safari (no WebGL requirement) | Must | 2 | S | BL-052 | FR-SPEC-03 | TC-SPEC-04 | In Progress |
| BL-054 | Add spectrum update rate configuration option | Should | 2 | S | BL-051 | FR-SPEC-02 | TC-SPEC-02 | Done |
| BL-055 | Add colour palette selector for waterfall | Could | 2 | S | BL-052 | FR-SPEC-04 | — | Proposed |

---

### EP-06 — Mobile browser compatibility

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-060 | Validate full browser matrix (see TC-COMPAT-01–TC-COMPAT-07) | Must | 2 | M | BL-047, BL-052 | NFR-COMPAT-01–NFR-COMPAT-07 | TC-COMPAT-01–TC-COMPAT-07 | In Progress |
| BL-061 | Touch optimisation: tuning slider, PTT button sizing | Must | 2 | S | BL-047 | NFR-COMPAT-06 | TC-COMPAT-04–TC-COMPAT-07 | Done |
| BL-062 | Implement audio device selector UI (MediaDevices API) | Must | 2 | S | BL-040 | FR-AUD-03–FR-AUD-04, NFR-COMPAT-07 | TC-AUD-03–TC-AUD-04 | Done |

---

### EP-07 — Audio pipeline

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-070 | Implement Pi-side audio capture (CPAL) → Opus encode | Must | 3 | L | BL-025 | FR-AUD-01, FR-AUD-05 | TC-AUD-01 | Proposed |
| BL-071 | Stream encoded audio to browser client over WSS | Must | 3 | M | BL-070, BL-021 | FR-AUD-01 | TC-AUD-01 | In Progress |
| BL-072 | Browser-side Opus decode and audio playback | Must | 3 | M | BL-040 | FR-AUD-01 | TC-AUD-01 | Proposed |
| BL-073 | Browser-side mic capture and Opus encode | Must | 3 | M | BL-062 | FR-AUD-02 | TC-AUD-02 | Proposed |
| BL-074 | Pi-side Opus decode and audio playback (CPAL) | Must | 3 | M | BL-073 | FR-AUD-02 | TC-AUD-02 | Proposed |
| BL-075 | Per-session auth check on audio WebSocket channel | Must | 3 | S | BL-021, BL-071 | FR-AUTH-01, NFR-SEC-01 | TC-AUTH-01, TC-SEC-01 | Done |
| BL-076 | Bitrate/sample-rate profile for constrained mobile clients | Should | 3 | S | BL-071 | FR-AUD-05 | TC-AUD-05 | In Progress |
| BL-077 | Audio drop/retry and watchdog | Should | 3 | M | BL-071 | FR-AUD-06 | TC-AUD-06 | In Progress |
| BL-078 | Measure and document end-to-end audio latency on Pi 4 | Must | 3 | S | BL-074 | NFR-PERF-02 | TC-PERF-02 | Proposed |

---

### EP-08 — Deployment: native (systemd)

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-080 | Write systemd service unit (start/stop/restart, resource limits) | Must | 1 | S | BL-020 | NFR-DEPLOY-02 | TC-DEPLOY-03 | Done |
| BL-081 | Configure TOML config file with defaults | Must | 1 | S | BL-020 | NFR-DEPLOY-04 | — | Done |
| BL-082 | Document rollback procedure for native deployment | Must | 4 | S | BL-080 | NFR-DEPLOY-05 | TC-DEPLOY-06 | Done |
| BL-083 | Cross-compile release binary for aarch64-unknown-linux-gnu | Must | 1 | S | BL-020 | NFR-DEPLOY-01 | TC-DEPLOY-01–TC-DEPLOY-02 | Done |

---

### EP-09 — Deployment: container evaluation

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-090 | Write Dockerfile (non-root, read-only rootfs, minimal base) | Should | 3 | M | BL-083 | NFR-DEPLOY-03, NFR-SEC-10 | TC-DEPLOY-04–TC-DEPLOY-05 | Done |
| BL-091 | Write compose.yml for local orchestration | Should | 3 | S | BL-090 | NFR-DEPLOY-03 | TC-DEPLOY-04 | Done |
| BL-092 | Evaluate ALSA/PipeWire device passthrough in container | Should | 3 | M | BL-090 | NFR-DEPLOY-03 | TC-DEPLOY-04 | Proposed |
| BL-093 | Benchmark audio latency: native vs container on Pi 4 | Should | 3 | M | BL-092, BL-078 | NFR-DEPLOY-03, NFR-PERF-02 | TC-PERF-02, TC-DEPLOY-04 | Proposed |
| BL-094 | Write container deployment decision record in docs/deployment.md | Should | 3 | S | BL-093 | NFR-DEPLOY-03 | — | In Progress |
| BL-095 | Validate secret injection in container (no secrets in image layers) | Should | 3 | S | BL-090 | NFR-SEC-03, NFR-SEC-10 | TC-SEC-03, TC-DEPLOY-05 | Proposed |

---

### EP-10 — Operations and release hardening

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-100 | Production TLS setup (Let's Encrypt or self-signed + nginx reverse proxy) | Must | 4 | M | BL-080 | NFR-SEC-01 | TC-SEC-01 | Done |
| BL-101 | Write nginx reverse proxy config (TLS termination, WS proxy headers) | Must | 4 | S | BL-100 | NFR-SEC-01 | TC-SEC-01 | Done |
| BL-102 | Soak test: 24 h continuous operation on Pi 4 | Must | 4 | L | EP-07 | NFR-REL-03 | TC-REL-03 | Proposed |
| BL-103 | Pi 4 load test: 3 clients, full features; CPU < 50 % | Must | 4 | M | EP-07 | NFR-PERF-04 | TC-PERF-04 | Proposed |
| BL-104 | Final documentation alignment (all FR/NFR IDs traced) | Must | 4 | M | All | All FR/NFR | All TC | In Progress |
| BL-105 | Write ops runbook (start, stop, update, token rotation, log access) | Must | 4 | M | BL-080, BL-094 | NFR-DEPLOY-02, NFR-DEPLOY-05 | — | Done |
| BL-106 | Create release checklist referencing all release gate criteria | Must | 4 | S | BL-104 | — | — | Done |

---

### EP-11 — Split-host frontend deployment security

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-110 | Define split-host network topology (frontend host + backend host) | Must | 3 | S | BL-010 | FR-HOST-01–FR-HOST-02, NFR-DEPLOY-07 | TC-HOST-01, TC-DEPLOY-07 | Done |
| BL-111 | Add WireGuard profile for frontend-host to backend-host secure connectivity | Must | 3 | M | BL-110 | FR-HOST-03, NFR-SEC-13–NFR-SEC-14, NFR-DEPLOY-08 | TC-HOST-03, TC-SEC-12–TC-SEC-13, TC-DEPLOY-07 | Done |
| BL-112 | Add Tailscale profile as operator-friendly WireGuard-based alternative | Should | 3 | S | BL-110 | FR-HOST-03, NFR-SEC-14, NFR-DEPLOY-08 | TC-HOST-03, TC-SEC-13, TC-DEPLOY-07 | Done |
| BL-113 | Implement frontend runtime configuration for remote API/WSS endpoints | Must | 3 | S | BL-040 | FR-HOST-04 | TC-HOST-02 | Done |
| BL-114 | Document SSH tunnel profile as non-default fallback only | Should | 3 | S | BL-110 | NFR-SEC-15, NFR-DEPLOY-08 | TC-SEC-14 | Done |

---

### EP-12 — Licensing and compliance

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-120 | Add top-level LICENSE with AGPL-3.0-only text | Must | 0 | S | — | NFR-LIC-01–NFR-LIC-02 | TC-LIC-02 | Done |
| BL-121 | Add AGPL license notice in core project docs | Must | 0 | S | BL-120 | NFR-LIC-02 | TC-LIC-02 | Done |
| BL-122 | Add license compliance check to release checklist | Should | 4 | S | BL-120 | NFR-LIC-01–NFR-LIC-02 | TC-LIC-01–TC-LIC-02 | Done |

---

## 4. Won't Have (This Release)

| ID | Title | Reason |
|---|---|---|
| BL-W-001 | egui / WASM frontend | Browser/mobile compatibility risk; Web-native TS frontend preferred for MVP |
| BL-W-002 | WebRTC audio | STUN/TURN complexity not justified for LAN/VPN use case in first release |
| BL-W-003 | OAuth2 / OIDC integration | Added complexity; bearer token sufficient for first release scope |
| BL-W-004 | Multi-rig support | Out of scope for initial release; architecture should not preclude it |
| BL-W-005 | Public internet direct exposure (no VPN/proxy) | Deferred until security review of internet-facing deployment is complete |

---

## 5. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.5.21 | 2026-07-05 | DC0SK | WS audio RX transport: BL-075 (per-session audio auth) → Done, BL-071 (stream audio over WS) → In Progress (authenticated binary audio frames from a synthetic source; codec/capture ends are HIL). |
| 0.5.20 | 2026-07-05 | DC0SK | Ops/release docs: BL-012 (rotation policy), BL-082 (rollback), BL-105 (runbook), BL-106 (release checklist), BL-122 (license gate) → Done. BL-104 In Progress (final trace pending HIL). |
| 0.5.19 | 2026-07-05 | DC0SK | Closed security remainders + TLS: BL-081 (0600 config check), BL-032 (panic sanitisation/NFR-SEC-09), BL-100/101 (nginx TLS proxy/NFR-SEC-01), BL-021 (auth) → Done. Deferral TC-SEC-01 now met by config. |
| 0.5.18 | 2026-07-05 | DC0SK | Split-host (BL-110–114) → Done; container artifacts BL-090/091 → Done, BL-094 In Progress (decision pending Pi HIL). Deployment breadth artifacts landed. |
| 0.5.17 | 2026-07-05 | DC0SK | Phase 3 start: audio software core (ARC-05 jitter buffer + codec seam + config) → BL-076/077 In Progress (FR-AUD-05/06). Native/HIL audio parts remain. |
| 0.5.16 | 2026-07-05 | DC0SK | BL-062 (audio device selector) + BL-061 (touch optimisation) → Done; BL-060 (browser matrix) → In Progress (software done; on-device matrix is HIL). Phase 2 development-complete. |
| 0.5.15 | 2026-07-05 | DC0SK | BL-052 (Canvas 2D waterfall) → Done; BL-053 (iOS Safari no-WebGL) → In Progress (structurally 2D-only; on-device verify is browser-matrix). 27 frontend tests. |
| 0.5.14 | 2026-07-05 | DC0SK | Phase 2 start: BL-050/051/054 (FFT + spectrum WS stream) → Done; BL-022 (frame/size limits) → Done (WS frame cap now enforced on the ARC-01 WS transport). |
| 0.5.13 | 2026-07-05 | DC0SK | BL-047 (responsive CSS) + BL-080 (systemd unit) → Done. All Phase 1 build actions complete; only the exit review (A27) remains. |
| 0.5.12 | 2026-07-05 | DC0SK | BL-043 (mode selector) + BL-044 (PTT button) + BL-045 (S-meter display) → Done. Full rig-control UI wired; 22 frontend tests. |
| 0.5.11 | 2026-07-05 | DC0SK | BL-046 (WS client reconnect/backoff, NFR-REL-01) + BL-042 (frequency display/tuning) → Done. Frontend now at 18 unit tests. |
| 0.5.10 | 2026-07-04 | DC0SK | BL-040 (TS frontend project) + BL-041 (session bootstrap) → Done: ARC-10 frontend started — typed API/session/backoff modules, login UI, 11 tests, CI frontend job. |
| 0.5.9 | 2026-07-04 | DC0SK | BL-031 (circuit breaker) + BL-033 (GPIO, ARC-08) → Done; BL-029 (S-meter) → In Progress (read path; streaming rides Phase-2 WS). Phase 1 backend complete. |
| 0.5.8 | 2026-07-04 | DC0SK | BL-026/027/028 (freq/mode/PTT handlers) + BL-030 (rig mutex) → Done; BL-024 (audit) → Done (rig-action auditing now verified, TC-AUDIT-01). Rig control endpoints RBAC-gated + audited; PTT safety timeout (NFR-SEC-07). |
| 0.5.7 | 2026-07-04 | DC0SK | BL-025 (rigctld adapter) → Done: ARC-04 typed async rigctld client with allowlist + range validation (injection-proof), serialised access, reconnect; mock-rigctld + validation tests. |
| 0.5.6 | 2026-07-04 | DC0SK | BL-024 (audit log) → In Progress: ARC-07 hash-chained tamper-evident audit subsystem + auth-failure logging + Admin view; rig-action auditing lands with rig handlers. |
| 0.5.5 | 2026-07-04 | DC0SK | BL-023 (CORS) → Done; BL-022 (rate/size limits) → In Progress (rate limiting + HTTP body limit done; WS frame cap deferred to WS endpoints). ARC-03 security middleware. |
| 0.5.4 | 2026-07-04 | DC0SK | BL-021 (auth middleware) → In Progress: ARC-02 auth/RBAC implemented (JWT + argon2 + refresh + logout); NFR-SEC-01/TC-SEC-01 (TLS transport) remain for Phase 4. |
| 0.5.3 | 2026-07-04 | DC0SK | Phase 1 kickoff: BL-020 (workspace) and BL-083 (aarch64 cross-build) → Done; BL-032 (tracing) and BL-081 (config) → In Progress. |
| 0.5.2 | 2026-07-04 | DC0SK | Phase 0 reconciliation: verified deliverables against repo and moved BL-004–BL-006, BL-010–BL-011, BL-013, BL-120–BL-121 to Done; BL-012 kept Proposed (rotation policy still open, see note). |
| 0.5.1 | 2026-06-26 | DC0SK | Migrated to area-coded FR/NFR/TC ids and new doc-tree frontmatter. |
| 0.5.0 | 2026-05-13 | — | Linked GPIO backlog item to security requirement/test (allowlist + safe startup states) |
| 0.4.0 | 2026-05-13 | — | Added Phase 1 backlog item for Raspberry Pi GPIO digital I/O control |
| 0.3.0 | 2026-05-13 | — | Added governance charter backlog item and aligned change-control acceptance criteria |
| 0.2.0 | 2026-05-13 | — | Added split-host secure deployment and AGPL licensing epics/items |
| 0.1.0 | 2026-05-12 | — | Initial draft; all items at Proposed or Done |
