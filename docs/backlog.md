---
title: Product Backlog
project: landline
doc_type: backlog
status: draft
version: 0.1.0
owner: ""
last_updated: 2026-05-12
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
- Requirements and test-spec updated if new/changed.
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

---

## 3. Backlog Items

### EP-01 — Project documentation and governance

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-001 | Create requirements-spec.md with all REQ-* IDs | Must | 0 | S | — | All REQ | — | Done |
| BL-002 | Create test-spec.md with traceability matrix | Must | 0 | S | BL-001 | All REQ | — | Done |
| BL-003 | Create backlog.md (this document) | Must | 0 | S | BL-001 | — | — | Done |
| BL-004 | Create roadmap.md with phase/release plan | Must | 0 | S | BL-001 | — | — | In Progress |
| BL-005 | Define change control process for doc updates | Must | 0 | S | BL-001 | — | — | Proposed |

**Acceptance Criteria — BL-005:** Change control procedure is documented in docs/contributing.md or equivalent; any PR touching requirements, tests, backlog, or roadmap must update all four artifacts together.

---

### EP-02 — Security baseline and threat model

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-010 | Document trust boundaries and deployment modes | Must | 0 | S | BL-001 | REQ-S-* | — | Proposed |
| BL-011 | Define security release gates and acceptance criteria | Must | 0 | S | BL-010 | REQ-S-* | TST-S-* | Proposed |
| BL-012 | Define secrets storage and rotation policy | Must | 0 | S | BL-010 | REQ-S-003 | TST-S-003 | Proposed |
| BL-013 | Create docs/security.md (threat model + controls) | Must | 0 | M | BL-010 | REQ-S-* | — | Proposed |

**Acceptance Criteria — BL-011:** Documented list of security gates with pass/fail criteria; referenced in phase exit checklists in roadmap.

---

### EP-03 — Backend foundation and rig control

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-020 | Initialize Rust workspace: Tokio + Axum + Tower | Must | 1 | S | BL-011 | REQ-NF-020 | — | Proposed |
| BL-021 | Implement auth middleware (JWT, expiry, role claims) | Must | 1 | M | BL-020, BL-012 | REQ-F-040–044, REQ-S-001–002 | TST-F-040–044, TST-S-001–002 | Proposed |
| BL-022 | Implement rate limiting and frame/size limits | Must | 1 | S | BL-021 | REQ-S-004–005 | TST-S-004–005 | Proposed |
| BL-023 | Implement CORS origin policy | Must | 1 | S | BL-021 | REQ-S-006 | TST-S-006 | Proposed |
| BL-024 | Implement audit log subsystem | Must | 1 | M | BL-020 | REQ-F-050–053 | TST-F-050–051 | Proposed |
| BL-025 | Implement rigctld TCP adapter with command sanitisation | Must | 1 | M | BL-020 | REQ-F-008–009 | TST-F-007–008 | Proposed |
| BL-026 | Implement frequency read/set handlers | Must | 1 | S | BL-025, BL-021 | REQ-F-001–002 | TST-F-001–002 | Proposed |
| BL-027 | Implement mode read/set handlers | Must | 1 | S | BL-025, BL-021 | REQ-F-003–004 | TST-F-003 | Proposed |
| BL-028 | Implement PTT handler with role check and safety timeout | Must | 1 | M | BL-025, BL-021 | REQ-F-005, REQ-S-007 | TST-F-004–005, TST-S-007 | Proposed |
| BL-029 | Implement S-meter streaming | Should | 1 | S | BL-025 | REQ-F-006 | TST-F-006 | Proposed |
| BL-030 | Implement rig access mutex for concurrent clients | Must | 1 | S | BL-025 | REQ-F-010 | TST-F-009 | Proposed |
| BL-031 | Implement rigctld reconnect/circuit-breaker | Should | 1 | M | BL-025 | REQ-NF-011 | TST-NF-011 | Proposed |
| BL-032 | Structured tracing/logging integration | Must | 1 | S | BL-020 | REQ-S-009, REQ-S-012 | TST-S-009–010 | Proposed |

---

### EP-04 — Frontend: control UI and session management

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-040 | Initialize TypeScript/HTML5 frontend project | Must | 1 | S | BL-021 | REQ-C-001–002 | — | Proposed |
| BL-041 | Implement authenticated session bootstrap (login, token storage, logout) | Must | 1 | M | BL-040, BL-021 | REQ-F-040–044 | TST-F-040–044 | Proposed |
| BL-042 | Implement frequency display and tuning control | Must | 1 | M | BL-040, BL-026 | REQ-F-001–002, REQ-C-006 | TST-F-001–002, TST-C-001–002 | Proposed |
| BL-043 | Implement mode selector | Must | 1 | S | BL-040, BL-027 | REQ-F-003–004 | TST-F-003 | Proposed |
| BL-044 | Implement PTT button with visual transmit indicator | Must | 1 | S | BL-040, BL-028 | REQ-F-005 | TST-F-004 | Proposed |
| BL-045 | Implement S-meter display | Should | 1 | S | BL-040, BL-029 | REQ-F-006 | TST-F-006 | Proposed |
| BL-046 | Implement WebSocket client with reconnect/backoff | Must | 1 | M | BL-040 | REQ-NF-010 | TST-NF-010 | Proposed |
| BL-047 | Responsive CSS layout (desktop 3-column, mobile vertical stack) | Must | 1 | M | BL-040 | REQ-C-003–006 | TST-C-004–007 | Proposed |

---

### EP-05 — Spectrum and waterfall pipeline

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-050 | Implement FFT pipeline (rustfft) on audio capture thread | Must | 2 | M | BL-025 | REQ-F-020 | TST-F-020 | Proposed |
| BL-051 | Stream FFT bin data to clients over WebSocket at configurable rate | Must | 2 | M | BL-050, BL-021 | REQ-F-020–021 | TST-F-020–021 | Proposed |
| BL-052 | Implement Canvas 2D waterfall renderer in frontend | Must | 2 | M | BL-040, BL-051 | REQ-F-022 | TST-F-022 | Proposed |
| BL-053 | Verify waterfall rendering on iOS Safari (no WebGL requirement) | Must | 2 | S | BL-052 | REQ-F-022 | TST-F-023 | Proposed |
| BL-054 | Add spectrum update rate configuration option | Should | 2 | S | BL-051 | REQ-F-021 | TST-F-021 | Proposed |
| BL-055 | Add colour palette selector for waterfall | Could | 2 | S | BL-052 | REQ-F-023 | — | Proposed |

---

### EP-06 — Mobile browser compatibility

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-060 | Validate full browser matrix (see TST-C-001–007) | Must | 2 | M | BL-047, BL-052 | REQ-C-001–007 | TST-C-001–007 | Proposed |
| BL-061 | Touch optimisation: tuning slider, PTT button sizing | Must | 2 | S | BL-047 | REQ-C-006 | TST-C-004–007 | Proposed |
| BL-062 | Implement audio device selector UI (MediaDevices API) | Must | 2 | S | BL-040 | REQ-F-032–033, REQ-C-007 | TST-F-032–033 | Proposed |

---

### EP-07 — Audio pipeline

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-070 | Implement Pi-side audio capture (CPAL) → Opus encode | Must | 3 | L | BL-025 | REQ-F-030, REQ-F-034 | TST-F-030 | Proposed |
| BL-071 | Stream encoded audio to browser client over WSS | Must | 3 | M | BL-070, BL-021 | REQ-F-030 | TST-F-030 | Proposed |
| BL-072 | Browser-side Opus decode and audio playback | Must | 3 | M | BL-040 | REQ-F-030 | TST-F-030 | Proposed |
| BL-073 | Browser-side mic capture and Opus encode | Must | 3 | M | BL-062 | REQ-F-031 | TST-F-031 | Proposed |
| BL-074 | Pi-side Opus decode and audio playback (CPAL) | Must | 3 | M | BL-073 | REQ-F-031 | TST-F-031 | Proposed |
| BL-075 | Per-session auth check on audio WebSocket channel | Must | 3 | S | BL-021, BL-071 | REQ-F-040, REQ-S-001 | TST-F-040, TST-S-001 | Proposed |
| BL-076 | Bitrate/sample-rate profile for constrained mobile clients | Should | 3 | S | BL-071 | REQ-F-034 | TST-F-034 | Proposed |
| BL-077 | Audio drop/retry and watchdog | Should | 3 | M | BL-071 | REQ-F-035 | TST-F-035 | Proposed |
| BL-078 | Measure and document end-to-end audio latency on Pi 4 | Must | 3 | S | BL-074 | REQ-NF-002 | TST-NF-002 | Proposed |

---

### EP-08 — Deployment: native (systemd)

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-080 | Write systemd service unit (start/stop/restart, resource limits) | Must | 1 | S | BL-020 | REQ-D-002 | TST-D-003 | Proposed |
| BL-081 | Configure TOML config file with defaults | Must | 1 | S | BL-020 | REQ-D-004 | — | Proposed |
| BL-082 | Document rollback procedure for native deployment | Must | 4 | S | BL-080 | REQ-D-005 | TST-D-006 | Proposed |
| BL-083 | Cross-compile release binary for aarch64-unknown-linux-gnu | Must | 1 | S | BL-020 | REQ-D-001 | TST-D-001–002 | Proposed |

---

### EP-09 — Deployment: container evaluation

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-090 | Write Dockerfile (non-root, read-only rootfs, minimal base) | Should | 3 | M | BL-083 | REQ-D-003, REQ-S-010 | TST-D-004–005 | Proposed |
| BL-091 | Write compose.yml for local orchestration | Should | 3 | S | BL-090 | REQ-D-003 | TST-D-004 | Proposed |
| BL-092 | Evaluate ALSA/PipeWire device passthrough in container | Should | 3 | M | BL-090 | REQ-D-003 | TST-D-004 | Proposed |
| BL-093 | Benchmark audio latency: native vs container on Pi 4 | Should | 3 | M | BL-092, BL-078 | REQ-D-003, REQ-NF-002 | TST-NF-002, TST-D-004 | Proposed |
| BL-094 | Write container deployment decision record in docs/deployment.md | Should | 3 | S | BL-093 | REQ-D-003 | — | Proposed |
| BL-095 | Validate secret injection in container (no secrets in image layers) | Should | 3 | S | BL-090 | REQ-S-003, REQ-S-010 | TST-S-003, TST-D-005 | Proposed |

---

### EP-10 — Operations and release hardening

| ID | Title | Priority | Phase | Est. | Deps | Req IDs | Test IDs | Status |
|---|---|---|---|---|---|---|---|---|
| BL-100 | Production TLS setup (Let's Encrypt or self-signed + nginx reverse proxy) | Must | 4 | M | BL-080 | REQ-S-001 | TST-S-001 | Proposed |
| BL-101 | Write nginx reverse proxy config (TLS termination, WS proxy headers) | Must | 4 | S | BL-100 | REQ-S-001 | TST-S-001 | Proposed |
| BL-102 | Soak test: 24 h continuous operation on Pi 4 | Must | 4 | L | EP-07 | REQ-NF-012 | TST-NF-005 | Proposed |
| BL-103 | Pi 4 load test: 3 clients, full features; CPU < 50 % | Must | 4 | M | EP-07 | REQ-NF-004 | TST-NF-004 | Proposed |
| BL-104 | Final documentation alignment (all REQ IDs traced) | Must | 4 | M | All | All REQ | All TST | Proposed |
| BL-105 | Write ops runbook (start, stop, update, token rotation, log access) | Must | 4 | M | BL-080, BL-094 | REQ-D-002, REQ-D-005 | — | Proposed |
| BL-106 | Create release checklist referencing all release gate criteria | Must | 4 | S | BL-104 | — | — | Proposed |

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
| 0.1.0 | 2026-05-12 | — | Initial draft; all items at Proposed or Done |
