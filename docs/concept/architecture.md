---
title: "landline — Concept & Architecture"
status: Draft
version: "0.6"
updated: 2026-06-26
authors:
  - Simon Keimer (DC0SK)
owns: [ARC, ADR]
---

# landline — Concept & Architecture

> **License notice:** landline is licensed under **AGPL-3.0-only**. See the top-level
> [LICENSE](../../LICENSE).

| Status | Version | Date | Author |
|---|---|---|---|
| Draft baseline for review | 0.6 | 2026-06-26 | Simon Keimer (DC0SK) |

This document defines the solution concept: the component model (`ARC-`), the data flow and trust
boundaries, and the architecture decision records (`ADR-`). It realises the requirements in
[system-requirements.md](../requirements/system-requirements.md), which in turn trace up to the
needs in [stakeholder-requirements.md](../requirements/stakeholder-requirements.md).

---

## 1. Overview / concept

landline is a two-tier system. A **Rust backend** runs on a Raspberry Pi at the rig site: an
Axum/Tokio/Tower service that authenticates clients, enforces RBAC and security middleware, talks
to the transceiver through a hamlib/rigctld TCP adapter, runs the audio and spectrum pipelines,
controls GPIO, and writes a tamper-evident audit log. A **browser-native TypeScript frontend**
provides the control UI, audio I/O, and the spectrum/waterfall renderer, communicating over
TLS/WSS — control, audio, and spectrum multiplexed on WebSocket with per-message-type access
control.

The frontend may be served from the same host or, in **split-host** deployments, from a separate
machine reaching the backend over a WireGuard or Tailscale private network (SSH tunnel as fallback
only). The backend defaults to least exposure (private-interface bind, no public 0.0.0.0) and least
privilege (RBAC, validation allowlists, GPIO allowlist, safe defaults). Security controls are
treated as core components, not add-ons, consistent with the security-first governance charter.

## 2. Component model

Each architecture element carries a stable `ARC-` ID and names the requirement areas it realises.
Per rule **R5**, every implemented `FR` is realised by ≥ 1 named `ARC` element once code lands.

| ID | Component | Responsibility | Realises (requirement areas) |
|---|---|---|---|
| ARC-01 | Axum HTTP/WS server + Tower middleware | HTTP/WSS endpoints, WebSocket lifecycle, message multiplexing and routing, Tracing | `FR-RIG-*`, `FR-SPEC-*`, `FR-AUD-*` transport; `NFR-PERF-01`, `NFR-PERF-03` |
| ARC-02 | Auth & session | JWT issue/verify, short-lived tokens + refresh, session invalidation, RBAC guards (Admin/Operator/Observer) | `FR-AUTH-01..05`, `FR-RIG-09` (role gating), `NFR-SEC-02` |
| ARC-03 | Security middleware | Rate limiting, request/WS-frame size limits, CORS/origin allowlist, error sanitisation | `NFR-SEC-04`, `NFR-SEC-05`, `NFR-SEC-06`, `NFR-SEC-09` |
| ARC-04 | Rig adapter | hamlib/rigctld TCP client, command allowlist + numeric-range validation, circuit breaker/timeouts, exclusive-access mutex | `FR-RIG-01..10`, `NFR-SEC-07`, `NFR-SEC-08`, `NFR-REL-02` |
| ARC-05 | Audio pipeline | CPAL capture/playback on the Pi, Opus encode/decode, jitter buffering, loss tolerance | `FR-AUD-01..06`, `NFR-PERF-02` |
| ARC-06 | Spectrum/FFT pipeline + WS stream | FFT bin computation, configurable cadence, bounded WS spectrum stream | `FR-SPEC-01`, `FR-SPEC-02`, `NFR-PERF-05` |
| ARC-07 | Audit log subsystem | Tamper-evident append-only log of state-changing actions and auth failures, retention | `FR-AUDIT-01..04` |
| ARC-08 | GPIO adapter | Allowlisted pin read/set, role-gated, safe default startup states | `FR-GPIO-01`, `NFR-SEC-16` |
| ARC-09 | Config loader | Single TOML source, secret-free, 0600 permission checks, no credentials in logs/URLs | `NFR-DEPLOY-04`, `NFR-SEC-03`, `NFR-SEC-12`, `FR-HOST-04` |
| ARC-10 | Frontend app (TS) | Session bootstrap, control UI, WS client with reconnect + exponential backoff, auth/error states | `FR-RIG-*`/`FR-AUTH-*` UI, `NFR-REL-01`, `NFR-COMPAT-06` |
| ARC-11 | Frontend audio module | MediaDevices capture/playback, device selection, Opus, gesture-gated mic activation | `FR-AUD-03`, `FR-AUD-04`, `NFR-COMPAT-07`, `NFR-COMPAT-03` |
| ARC-12 | Frontend spectrum/waterfall renderer | HTML5 Canvas waterfall, palette selection, bounded frame/update rate | `FR-SPEC-03`, `FR-SPEC-04`, `NFR-COMPAT-03` |
| ARC-13 | Deployment artifacts | systemd unit (reference), evaluated container, reverse proxy (TLS), split-host profiles | `NFR-DEPLOY-01..08`, `NFR-SEC-01`, `NFR-SEC-10`, `NFR-SEC-11`, `NFR-SEC-13`, `NFR-SEC-14`, `NFR-SEC-15` |

## 3. Data flow & trust boundaries

```
                    TRUSTED PRIVATE NETWORK (LAN / WireGuard / Tailscale)
  ┌─────────────────┐                              ┌──────────────────────────────────┐
  │  Browser client │   TLS / WSS (control,        │  Raspberry Pi backend host        │
  │  ARC-10/11/12    │◄──────audio, spectrum)──────►│  ARC-01 Axum + Tower              │
  │  (untrusted in)  │   per-message-type ACLs      │   ├─ ARC-02 Auth & RBAC           │
  └─────────────────┘                              │   ├─ ARC-03 Security middleware   │
        ▲                                           │   ├─ ARC-04 Rig adapter ──┐       │
        │ reverse proxy (TLS term, ARC-13)          │   ├─ ARC-05 Audio         │       │
        │                                           │   ├─ ARC-06 Spectrum/FFT  │       │
        │                                           │   ├─ ARC-07 Audit log     │       │
        │                                           │   ├─ ARC-08 GPIO adapter ─┼──┐    │
        │                                           │   └─ ARC-09 Config loader │  │    │
        │                                           └───────────────────────────┼──┼───┘
        │                                                      TCP (loopback)    │  │ GPIO
        │                                                              ┌─────────▼┐ │ pins
   split-host: frontend host                                          │ rigctld /│ ▼
   over WireGuard/Tailscale                                           │  hamlib  │ station HW
   (SSH fallback only)                                                └──────────┘
```

**Trust boundaries**

- **Browser ↔ backend** — the browser is an untrusted input source. Everything crossing this
  boundary is authenticated (ARC-02), rate/size-limited and origin-checked (ARC-03), and validated
  before it reaches the rig (ARC-04) or GPIO (ARC-08). Transport is TLS/WSS only (ARC-13, `NFR-SEC-01`).
- **Backend ↔ rigctld** — the rig adapter is the sole authority for command validation and
  exclusive access; rigctld is reached over a trusted local/loopback TCP link (ASM-05).
- **Backend ↔ GPIO** — only allowlisted pins are reachable; pins initialise to safe states.
- **Frontend host ↔ backend host (split-host)** — the backend binds to the private tunnel interface
  by default (never public 0.0.0.0) with mutual peer authentication over WireGuard/Tailscale
  (`NFR-SEC-13`, `NFR-SEC-14`); SSH tunnelling is an operator-enabled fallback, disabled by default
  (`NFR-SEC-15`).

## 4. Architecture Decision Records

Each ADR records Context / Decision / Status / Consequences. Status values follow the requirement
attribute model (`Proposed` · `Accepted` · `Superseded`).

### ADR-01 — Browser-native TypeScript frontend (reject egui/WASM for MVP)

- **Context:** landline must run on desktop and mobile browsers with no install, including iOS
  Safari and Android. An egui/WASM frontend was considered.
- **Decision:** Build the frontend as browser-native TypeScript. egui/WASM is rejected for the MVP.
- **Status:** Accepted.
- **Consequences:** Better mobile/browser fit (MediaDevices, Canvas, touch) and lower operational
  risk; satisfies `NFR-COMPAT-*` directly. egui/WASM is out of scope (see vision-and-scope §5.3);
  the team owns browser-compatibility testing as a first-class concern.

### ADR-02 — WebSocket binary transport with per-message-type ACLs

- **Context:** Control, audio, and spectrum all need a low-latency bidirectional channel; mixing
  them must not let an Observer issue control commands.
- **Decision:** Use a WebSocket binary transport carrying control, audio, and spectrum messages with
  explicit message schemas, bounds checking, and per-message-type access-control checks tied to RBAC.
- **Status:** Accepted.
- **Consequences:** One authenticated connection serves all channels; reconnection/session semantics
  must never bypass auth (ARC-02). Realises `NFR-SEC-05`, `NFR-SEC-08`, and supports `NFR-PERF-01`.

### ADR-03 — hamlib/rigctld TCP adapter as the rig interface

- **Context:** landline must control a wide range of transceivers without per-rig drivers.
- **Decision:** Interface with the rig exclusively through hamlib/rigctld over TCP, behind a backend
  adapter that validates every command and serialises access.
- **Status:** Accepted.
- **Consequences:** Broad rig coverage via hamlib; landline depends on rigctld being reachable
  (ASM-05). The adapter (ARC-04) is the single choke point for validation, exclusivity, and the
  circuit breaker. Multi-rig support remains out of scope.

### ADR-04 — Auth model: short-lived JWT with refresh

- **Context:** The MVP may be reachable beyond a single trusted LAN; static bearer tokens give weak
  session control.
- **Decision:** Use short-lived JWTs with a refresh mechanism rather than static bearer tokens.
- **Status:** Accepted.
- **Consequences:** Stronger session control, expiry, and revocation on logout; satisfies
  `FR-AUTH-02/03/05` and `NFR-SEC-02`. Adds refresh-flow complexity in ARC-02 and the frontend.

### ADR-05 — Split-host transport: WireGuard primary / Tailscale alternative / SSH fallback-only

- **Context:** Split-host operation must reach the backend securely without public exposure.
- **Decision:** WireGuard is the primary split-host transport; Tailscale (WireGuard-based mesh) is
  the supported alternative; SSH tunnelling is a fallback only, disabled by default.
- **Status:** Accepted.
- **Consequences:** Private-interface bind and mutual peer authentication by default
  (`NFR-SEC-13/14`, `NFR-DEPLOY-07/08`); SSH carries an explicit `NFR-SEC-15` fallback-only
  constraint. Public internet exposure without VPN/proxy hardening is out of scope.

### ADR-06 — Native systemd as reference deployment; container evaluated, not default

- **Context:** Audio and GPIO device access plus latency budgets are sensitive to the runtime;
  containers complicate device passthrough.
- **Decision:** Ship a systemd service unit as the reference native deployment. Evaluate a container
  profile in P3 and support it only if it passes audio-latency and hardware-access thresholds.
- **Status:** Accepted.
- **Consequences:** Native deployment is the baseline (`NFR-DEPLOY-02`); the container profile is
  conditional (`NFR-DEPLOY-03`, `NFR-SEC-10/11`) with the decision recorded in P3 (RISK-05).

### ADR-07 — AGPL-3.0-only licensing

- **Context:** A network-served application should keep source obligations to its users; licensing
  is a release-gated artifact from inception.
- **Decision:** License the project under AGPL-3.0-only, with a top-level LICENSE file and notices in
  key docs.
- **Status:** Accepted.
- **Consequences:** Network-use source obligations apply; satisfies `NFR-LIC-01/02`. Downstream
  packagers (SH-7) inherit AGPL terms; the release checklist includes a licence-compliance gate.

> **R5 note:** rule R5 (every implemented `FR` is realised by ≥ 1 named `ARC` element) is enforced
> once the Rust/TypeScript workspace lands and the trace gate is promoted into `cargo xtask`. Until
> code exists, the `FR/NFR → ARC` mapping in §2 is the design-time contract that R5 will check.

## Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.6 | 2026-06-26 | DC0SK | Initial concept/architecture baseline: ARC-01..ARC-13 component model, data-flow & trust boundaries, ADR-01..ADR-07, and the R5 design-time mapping. |
