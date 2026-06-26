---
title: Product Backlog
status: Draft
version: 0.5.1
updated: 2026-06-26
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
| BL-004 | Create roadmap.md with phase/release plan | Must | 0 | S | BL-001 | — | — | In Progress |
| BL-005 | Define change control process for doc updates | Must | 0 | S | BL-001 | — | — | Proposed |
| BL-006 | Create governance charter with security-first policy | Must | 0 | S | BL-001 | NFR-SEC-*, NFR-LIC-* | TC-SEC-* | Proposed |

**Acceptance Criteria — BL-005:** Change control procedure is documented in docs/governance.md or equivalent; any PR touching requirements, tests, backlog, or roadmap must update all four artifacts together.
**Acceptance Criteria — BL-006:** docs/governance.md defines security-first as a release-gated governance rule and includes exception handling requirements.

---

### EP-02 — Security baseline and threat model

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-010 | Document trust boundaries and deployment modes | Must | 0 | S | BL-001 | NFR-SEC-* | — | Proposed |
| BL-011 | Define security release gates and acceptance criteria | Must | 0 | S | BL-010 | NFR-SEC-* | TC-SEC-* | Proposed |
| BL-012 | Define secrets storage and rotation policy | Must | 0 | S | BL-010 | NFR-SEC-03 | TC-SEC-03 | Proposed |
| BL-013 | Create docs/security.md (threat model + controls) | Must | 0 | M | BL-010 | NFR-SEC-* | — | Proposed |

**Acceptance Criteria — BL-011:** Documented list of security gates with pass/fail criteria; referenced in phase exit checklists in roadmap.

---

### EP-03 — Backend foundation and rig control

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-020 | Initialize Rust workspace: Tokio + Axum + Tower | Must | 1 | S | BL-011 | NFR-MAINT-01 | — | Proposed |
| BL-021 | Implement auth middleware (JWT, expiry, role claims) | Must | 1 | M | BL-020, BL-012 | FR-AUTH-01–FR-AUTH-05, NFR-SEC-01–NFR-SEC-02 | TC-AUTH-01–TC-AUTH-05, TC-SEC-01–TC-SEC-02 | Proposed |
| BL-022 | Implement rate limiting and frame/size limits | Must | 1 | S | BL-021 | NFR-SEC-04–NFR-SEC-05 | TC-SEC-04–TC-SEC-05 | Proposed |
| BL-023 | Implement CORS origin policy | Must | 1 | S | BL-021 | NFR-SEC-06 | TC-SEC-06 | Proposed |
| BL-024 | Implement audit log subsystem | Must | 1 | M | BL-020 | FR-AUDIT-01–FR-AUDIT-04 | TC-AUDIT-01–TC-AUDIT-02 | Proposed |
| BL-025 | Implement rigctld TCP adapter with command sanitisation | Must | 1 | M | BL-020 | FR-RIG-08–FR-RIG-09 | TC-RIG-07–TC-RIG-08 | Proposed |
| BL-026 | Implement frequency read/set handlers | Must | 1 | S | BL-025, BL-021 | FR-RIG-01–FR-RIG-02 | TC-RIG-01–TC-RIG-02 | Proposed |
| BL-027 | Implement mode read/set handlers | Must | 1 | S | BL-025, BL-021 | FR-RIG-03–FR-RIG-04 | TC-RIG-03 | Proposed |
| BL-028 | Implement PTT handler with role check and safety timeout | Must | 1 | M | BL-025, BL-021 | FR-RIG-05, NFR-SEC-07 | TC-RIG-04–TC-RIG-05, TC-SEC-07 | Proposed |
| BL-029 | Implement S-meter streaming | Should | 1 | S | BL-025 | FR-RIG-06 | TC-RIG-06 | Proposed |
| BL-030 | Implement rig access mutex for concurrent clients | Must | 1 | S | BL-025 | FR-RIG-10 | TC-RIG-09 | Proposed |
| BL-031 | Implement rigctld reconnect/circuit-breaker | Should | 1 | M | BL-025 | NFR-REL-02 | TC-REL-02 | Proposed |
| BL-032 | Structured tracing/logging integration | Must | 1 | S | BL-020 | NFR-SEC-09, NFR-SEC-12 | TC-SEC-09–TC-SEC-10 | Proposed |
| BL-033 | Implement Raspberry Pi GPIO control API for at least 5 digital pins | Must | 1 | M | BL-020, BL-021 | FR-GPIO-01, NFR-SEC-16 | TC-GPIO-01, TC-SEC-15 | Proposed |

---

### EP-04 — Frontend: control UI and session management

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-040 | Initialize TypeScript/HTML5 frontend project | Must | 1 | S | BL-021 | NFR-COMPAT-01–NFR-COMPAT-02 | — | Proposed |
| BL-041 | Implement authenticated session bootstrap (login, token storage, logout) | Must | 1 | M | BL-040, BL-021 | FR-AUTH-01–FR-AUTH-05 | TC-AUTH-01–TC-AUTH-05 | Proposed |
| BL-042 | Implement frequency display and tuning control | Must | 1 | M | BL-040, BL-026 | FR-RIG-01–FR-RIG-02, NFR-COMPAT-06 | TC-RIG-01–TC-RIG-02, TC-COMPAT-01–TC-COMPAT-02 | Proposed |
| BL-043 | Implement mode selector | Must | 1 | S | BL-040, BL-027 | FR-RIG-03–FR-RIG-04 | TC-RIG-03 | Proposed |
| BL-044 | Implement PTT button with visual transmit indicator | Must | 1 | S | BL-040, BL-028 | FR-RIG-05 | TC-RIG-04 | Proposed |
| BL-045 | Implement S-meter display | Should | 1 | S | BL-040, BL-029 | FR-RIG-06 | TC-RIG-06 | Proposed |
| BL-046 | Implement WebSocket client with reconnect/backoff | Must | 1 | M | BL-040 | NFR-REL-01 | TC-REL-01 | Proposed |
| BL-047 | Responsive CSS layout (desktop 3-column, mobile vertical stack) | Must | 1 | M | BL-040 | NFR-COMPAT-03–NFR-COMPAT-06 | TC-COMPAT-04–TC-COMPAT-07 | Proposed |

---

### EP-05 — Spectrum and waterfall pipeline

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-050 | Implement FFT pipeline (rustfft) on audio capture thread | Must | 2 | M | BL-025 | FR-SPEC-01 | TC-SPEC-01 | Proposed |
| BL-051 | Stream FFT bin data to clients over WebSocket at configurable rate | Must | 2 | M | BL-050, BL-021 | FR-SPEC-01–FR-SPEC-02 | TC-SPEC-01–TC-SPEC-02 | Proposed |
| BL-052 | Implement Canvas 2D waterfall renderer in frontend | Must | 2 | M | BL-040, BL-051 | FR-SPEC-03 | TC-SPEC-03 | Proposed |
| BL-053 | Verify waterfall rendering on iOS Safari (no WebGL requirement) | Must | 2 | S | BL-052 | FR-SPEC-03 | TC-SPEC-04 | Proposed |
| BL-054 | Add spectrum update rate configuration option | Should | 2 | S | BL-051 | FR-SPEC-02 | TC-SPEC-02 | Proposed |
| BL-055 | Add colour palette selector for waterfall | Could | 2 | S | BL-052 | FR-SPEC-04 | — | Proposed |

---

### EP-06 — Mobile browser compatibility

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-060 | Validate full browser matrix (see TC-COMPAT-01–TC-COMPAT-07) | Must | 2 | M | BL-047, BL-052 | NFR-COMPAT-01–NFR-COMPAT-07 | TC-COMPAT-01–TC-COMPAT-07 | Proposed |
| BL-061 | Touch optimisation: tuning slider, PTT button sizing | Must | 2 | S | BL-047 | NFR-COMPAT-06 | TC-COMPAT-04–TC-COMPAT-07 | Proposed |
| BL-062 | Implement audio device selector UI (MediaDevices API) | Must | 2 | S | BL-040 | FR-AUD-03–FR-AUD-04, NFR-COMPAT-07 | TC-AUD-03–TC-AUD-04 | Proposed |

---

### EP-07 — Audio pipeline

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-070 | Implement Pi-side audio capture (CPAL) → Opus encode | Must | 3 | L | BL-025 | FR-AUD-01, FR-AUD-05 | TC-AUD-01 | Proposed |
| BL-071 | Stream encoded audio to browser client over WSS | Must | 3 | M | BL-070, BL-021 | FR-AUD-01 | TC-AUD-01 | Proposed |
| BL-072 | Browser-side Opus decode and audio playback | Must | 3 | M | BL-040 | FR-AUD-01 | TC-AUD-01 | Proposed |
| BL-073 | Browser-side mic capture and Opus encode | Must | 3 | M | BL-062 | FR-AUD-02 | TC-AUD-02 | Proposed |
| BL-074 | Pi-side Opus decode and audio playback (CPAL) | Must | 3 | M | BL-073 | FR-AUD-02 | TC-AUD-02 | Proposed |
| BL-075 | Per-session auth check on audio WebSocket channel | Must | 3 | S | BL-021, BL-071 | FR-AUTH-01, NFR-SEC-01 | TC-AUTH-01, TC-SEC-01 | Proposed |
| BL-076 | Bitrate/sample-rate profile for constrained mobile clients | Should | 3 | S | BL-071 | FR-AUD-05 | TC-AUD-05 | Proposed |
| BL-077 | Audio drop/retry and watchdog | Should | 3 | M | BL-071 | FR-AUD-06 | TC-AUD-06 | Proposed |
| BL-078 | Measure and document end-to-end audio latency on Pi 4 | Must | 3 | S | BL-074 | NFR-PERF-02 | TC-PERF-02 | Proposed |

---

### EP-08 — Deployment: native (systemd)

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-080 | Write systemd service unit (start/stop/restart, resource limits) | Must | 1 | S | BL-020 | NFR-DEPLOY-02 | TC-DEPLOY-03 | Proposed |
| BL-081 | Configure TOML config file with defaults | Must | 1 | S | BL-020 | NFR-DEPLOY-04 | — | Proposed |
| BL-082 | Document rollback procedure for native deployment | Must | 4 | S | BL-080 | NFR-DEPLOY-05 | TC-DEPLOY-06 | Proposed |
| BL-083 | Cross-compile release binary for aarch64-unknown-linux-gnu | Must | 1 | S | BL-020 | NFR-DEPLOY-01 | TC-DEPLOY-01–TC-DEPLOY-02 | Proposed |

---

### EP-09 — Deployment: container evaluation

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-090 | Write Dockerfile (non-root, read-only rootfs, minimal base) | Should | 3 | M | BL-083 | NFR-DEPLOY-03, NFR-SEC-10 | TC-DEPLOY-04–TC-DEPLOY-05 | Proposed |
| BL-091 | Write compose.yml for local orchestration | Should | 3 | S | BL-090 | NFR-DEPLOY-03 | TC-DEPLOY-04 | Proposed |
| BL-092 | Evaluate ALSA/PipeWire device passthrough in container | Should | 3 | M | BL-090 | NFR-DEPLOY-03 | TC-DEPLOY-04 | Proposed |
| BL-093 | Benchmark audio latency: native vs container on Pi 4 | Should | 3 | M | BL-092, BL-078 | NFR-DEPLOY-03, NFR-PERF-02 | TC-PERF-02, TC-DEPLOY-04 | Proposed |
| BL-094 | Write container deployment decision record in docs/deployment.md | Should | 3 | S | BL-093 | NFR-DEPLOY-03 | — | Proposed |
| BL-095 | Validate secret injection in container (no secrets in image layers) | Should | 3 | S | BL-090 | NFR-SEC-03, NFR-SEC-10 | TC-SEC-03, TC-DEPLOY-05 | Proposed |

---

### EP-10 — Operations and release hardening

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-100 | Production TLS setup (Let's Encrypt or self-signed + nginx reverse proxy) | Must | 4 | M | BL-080 | NFR-SEC-01 | TC-SEC-01 | Proposed |
| BL-101 | Write nginx reverse proxy config (TLS termination, WS proxy headers) | Must | 4 | S | BL-100 | NFR-SEC-01 | TC-SEC-01 | Proposed |
| BL-102 | Soak test: 24 h continuous operation on Pi 4 | Must | 4 | L | EP-07 | NFR-REL-03 | TC-REL-03 | Proposed |
| BL-103 | Pi 4 load test: 3 clients, full features; CPU < 50 % | Must | 4 | M | EP-07 | NFR-PERF-04 | TC-PERF-04 | Proposed |
| BL-104 | Final documentation alignment (all FR/NFR IDs traced) | Must | 4 | M | All | All FR/NFR | All TC | Proposed |
| BL-105 | Write ops runbook (start, stop, update, token rotation, log access) | Must | 4 | M | BL-080, BL-094 | NFR-DEPLOY-02, NFR-DEPLOY-05 | — | Proposed |
| BL-106 | Create release checklist referencing all release gate criteria | Must | 4 | S | BL-104 | — | — | Proposed |

---

### EP-11 — Split-host frontend deployment security

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-110 | Define split-host network topology (frontend host + backend host) | Must | 3 | S | BL-010 | FR-HOST-01–FR-HOST-02, NFR-DEPLOY-07 | TC-HOST-01, TC-DEPLOY-07 | Proposed |
| BL-111 | Add WireGuard profile for frontend-host to backend-host secure connectivity | Must | 3 | M | BL-110 | FR-HOST-03, NFR-SEC-13–NFR-SEC-14, NFR-DEPLOY-08 | TC-HOST-03, TC-SEC-12–TC-SEC-13, TC-DEPLOY-07 | Proposed |
| BL-112 | Add Tailscale profile as operator-friendly WireGuard-based alternative | Should | 3 | S | BL-110 | FR-HOST-03, NFR-SEC-14, NFR-DEPLOY-08 | TC-HOST-03, TC-SEC-13, TC-DEPLOY-07 | Proposed |
| BL-113 | Implement frontend runtime configuration for remote API/WSS endpoints | Must | 3 | S | BL-040 | FR-HOST-04 | TC-HOST-02 | Proposed |
| BL-114 | Document SSH tunnel profile as non-default fallback only | Should | 3 | S | BL-110 | NFR-SEC-15, NFR-DEPLOY-08 | TC-SEC-14 | Proposed |

---

### EP-12 — Licensing and compliance

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-120 | Add top-level LICENSE with AGPL-3.0-only text | Must | 0 | S | — | NFR-LIC-01–NFR-LIC-02 | TC-LIC-02 | Proposed |
| BL-121 | Add AGPL license notice in core project docs | Must | 0 | S | BL-120 | NFR-LIC-02 | TC-LIC-02 | Proposed |
| BL-122 | Add license compliance check to release checklist | Should | 4 | S | BL-120 | NFR-LIC-01–NFR-LIC-02 | TC-LIC-01–TC-LIC-02 | Proposed |

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
| 0.5.1 | 2026-06-26 | DC0SK | Migrated to area-coded FR/NFR/TC ids and new doc-tree frontmatter. |
| 0.5.0 | 2026-05-13 | — | Linked GPIO backlog item to security requirement/test (allowlist + safe startup states) |
| 0.4.0 | 2026-05-13 | — | Added Phase 1 backlog item for Raspberry Pi GPIO digital I/O control |
| 0.3.0 | 2026-05-13 | — | Added governance charter backlog item and aligned change-control acceptance criteria |
| 0.2.0 | 2026-05-13 | — | Added split-host secure deployment and AGPL licensing epics/items |
| 0.1.0 | 2026-05-12 | — | Initial draft; all items at Proposed or Done |
