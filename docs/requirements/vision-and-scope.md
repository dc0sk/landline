---
title: "landline — Vision & Scope"
status: Draft
version: "0.6"
updated: 2026-06-26
authors:
  - Simon Keimer (DC0SK)
owns: [SH, ASM, CON, RISK]
---

# landline — Vision & Scope

> **License notice:** landline is licensed under **AGPL-3.0-only**. See the top-level
> [LICENSE](../../LICENSE).

| Status | Version | Date | Author |
|---|---|---|---|
| Draft baseline for review | 0.6 | 2026-06-26 | Simon Keimer (DC0SK) |

This document fixes the problem, vision, stakeholders, goals, scope, phasing, assumptions,
constraints, and risks for landline. It owns the `SH-`, `ASM-`, `CON-`, and `RISK-` identifier
prefixes. Stakeholder needs derived here are recorded as `STK-` items in
[stakeholder-requirements.md](stakeholder-requirements.md); they are made testable as `FR-`/`NFR-`
in [system-requirements.md](system-requirements.md).

---

## 1. Problem statement

Amateur-radio operators increasingly want to run their station from somewhere other than the
shack: another room, a quiet office, or a different site entirely. Existing remote-control options
are typically vendor-locked, desktop-only, install-heavy, or weak on security — and several expose
a transmitter to the network with little authentication, authorisation, validation, or audit. For
a transmitting station this is not a cosmetic concern: an unauthorised or malformed control action
can key the rig, drive it out of band, or otherwise produce an unlawful emission for which the
licensee is personally accountable.

landline addresses this gap with a secure, browser-native remote for hamlib/rigctld-controlled
transceivers, running as an always-on appliance on a Raspberry Pi. It must give a licensed operator
real-time control, monitoring, and audio from any common desktop or mobile browser with no client
install, while treating authentication, authorisation, transport encryption, input validation, and
auditability as core functionality gated at every release — not as late hardening.

## 2. Vision

> landline is the open, security-first way for a licensed amateur to operate their own station
> from anywhere a browser runs. It delivers low-latency rig control, live spectrum, waterfall, and
> full-duplex audio over an authenticated, encrypted, audited channel — defaulting to least
> exposure and least privilege, easy to run on a Raspberry Pi, and free under AGPL-3.0-only so the
> community can inspect, trust, and extend it.

## 3. Stakeholders

| ID | Stakeholder | Interest in landline |
|---|---|---|
| SH-1 | Operator / licensed end user | Operate the rig remotely (tune, mode, key/transmit) with responsive, safe feedback; usable from a browser with no install. |
| SH-2 | Observer / guest listener | Monitor receive — RX audio, spectrum, waterfall, S-meter — without any control authority. |
| SH-3 | Developer / maintainer | Build and evolve landline under a documentation-first, test-driven process within single-developer bandwidth; keep it lintable, testable, and recoverable. |
| SH-4 | Station / site host | Owns the transceiver and the Raspberry Pi appliance; needs reliable always-on operation and safe GPIO control of station auxiliary hardware. |
| SH-5 | Remote-site network owner | Administers the LAN/VPN/firewall path; needs the backend to bind privately, authenticate peers, and never require public internet exposure. |
| SH-6 | Regulator / amateur-licence authority | Requires that all transmit operation remains within the operator's licence and that actions are accountable and auditable. |
| SH-7 | Downstream packagers & contributors | Redistribute, package, and extend landline under AGPL-3.0-only with complete, accurate licence artifacts and notices. |

## 4. Goals & success criteria

| ID | Goal | Success measure (referenced requirement area) |
|---|---|---|
| G1 | Safe, low-latency remote control | Control round-trip < 100 ms p95 on LAN (`NFR-PERF`); PTT gated to Operator with a server-side safety timeout (`NFR-SEC-07`). |
| G2 | Real-time monitoring | Spectrum updates ≥ 2 Hz under load (`NFR-PERF-05`); end-to-end audio latency < 300 ms on LAN (`NFR-PERF`); waterfall renders in-browser (`FR-SPEC`). |
| G3 | Security-first by construction | Every applicable `NFR-SEC` test passes before any phase/release gate; a failing security test blocks release regardless of other results. |
| G4 | Broad browser reach, no install | Defined browser matrix (Firefox, Chromium, iOS Safari, Android) passes the compatibility test set (`NFR-COMPAT`). |
| G5 | Reliable Pi appliance | 24 h continuous operation without restart (`NFR-REL`); CPU < 50 % on a Pi 4 under full load (`NFR-PERF`). |
| G6 | Secure split-host operation | Backend defaults to private-tunnel bind with mutual peer auth (`NFR-SEC-13/14`); WireGuard/Tailscale profile documented (`NFR-DEPLOY-07/08`). |
| G7 | Maintainable & open | Clippy-clean, integration-tested, rollback-capable (`NFR-MAINT`); AGPL-3.0-only artifacts complete (`NFR-LIC`). |
| G8 | Full accountability | Every state-changing action and auth failure is captured in a tamper-evident audit log (`FR-AUDIT`). |

## 5. Scope

### 5.1 In scope — v1

- Single-rig control over a hamlib/rigctld TCP adapter: frequency, mode, PTT, metering, with
  server-side command validation and exclusive cross-client access.
- Spectrum/FFT stream and HTML5 Canvas waterfall.
- Full-duplex RX/TX audio with Opus encoding and browser device selection.
- Authentication, short-lived session tokens, and RBAC (Admin / Operator / Observer).
- Tamper-evident audit logging of state-changing actions and auth failures.
- Raspberry Pi GPIO digital I/O with a pin allowlist and safe startup states.
- Split-host deployment over a WireGuard/Tailscale private network; SSH tunnel as fallback only.
- Native systemd deployment on Raspberry Pi 4 (4 GB) / Pi 5, RPiOS Bookworm 64-bit.
- Single-file TOML configuration; documented rollback procedure.

### 5.2 In scope — later phases

- Containerized deployment **profile**, supported only if it passes audio-latency and
  hardware-access acceptance thresholds (evaluated, not assumed — see ADR-06).
- Spectrum colour-palette selection and passband/filter-width control where the rig supports it
  (`Could`-priority refinements).
- Hardening and documentation for additional, non-Pi build targets without committing support.

### 5.3 Out of scope

- Public internet exposure without VPN/reverse-proxy hardening.
- Multi-rig / multi-transceiver support.
- OAuth2 / OIDC / external identity-provider federation.
- WebRTC media transport.
- An egui / WASM desktop frontend (rejected for MVP — see ADR-01).
- Native mobile apps or any required client install.
- Logging/SIEM aggregation, fleet management, or cloud back-end services.

## 6. Phasing

| Phase | Theme | Key deliverables | Exit gate |
|---|---|---|---|
| P0 | Foundation & docs | Requirements, test strategy, backlog, roadmap, threat model, governance baseline | Documentation baseline reviewed; security gates defined |
| P1 | Secure control MVP | Authenticated rig control UI + backend control APIs + audit log + security middleware | Control-latency target met; unauthorised control blocked; `NFR-SEC` MVP gates pass |
| P2 | Spectrum, waterfall & mobile | Responsive UI, spectrum/FFT pipeline, Canvas waterfall, browser matrix validation | Browser matrix passes the defined compatibility set |
| P3 | Audio & container eval | Bidirectional Opus audio, TLS production setup, native-vs-container evaluation report | Stable long-run audio sessions; native/container decision recorded |
| P4 | Release candidate & ops | Release checklist, deployment runbooks, rollback plan, final doc/trace alignment | All Must requirements trace to passing tests; go/no-go approval complete |

## 7. Assumptions

| ID | Assumption | Impact if false |
|---|---|---|
| ASM-01 | The transceiver exposes control via hamlib/rigctld over TCP. | No rig adapter is possible without a different integration path; core control scope invalid. |
| ASM-02 | Raspberry Pi 4 (4 GB) / Pi 5 on RPiOS Bookworm 64-bit is available as the reference appliance. | Performance, CPU, and deployment targets must be re-baselined for other hardware. |
| ASM-03 | The operator reaches the backend over a trusted LAN or a VPN/private tunnel, never raw public internet. | Threat model and `NFR-SEC` split-host controls must expand to a hostile-network posture. |
| ASM-04 | Target browsers support the MediaDevices, Canvas, WebSocket, and Opus capabilities landline relies on. | Audio device selection, waterfall, or transport degrade or fail on affected browsers. |
| ASM-05 | rigctld is reachable from the backend host (loopback or trusted local link) and authoritative for the rig. | Exclusive access and validation guarantees cannot be enforced from the backend alone. |
| ASM-06 | The site network owner can provision a WireGuard or Tailscale peer for split-host use. | Split-host falls back to SSH-only, contradicting the recommended secure-transport posture. |
| ASM-07 | Development proceeds with single-developer bandwidth under a documentation-first discipline. | Scope must be cut or phased further; broad v1 scope becomes unrealistic (see RISK-06). |

## 8. Constraints

| ID | Constraint | Source |
|---|---|---|
| CON-01 | Implementation language is Rust. | Technical decision; revised plan |
| CON-02 | Backend is built on Axum / Tokio / Tower (with Tracing). | Architecture decision; revised plan |
| CON-03 | Frontend is browser-native TypeScript; no egui. | Decided — see ADR-01 |
| CON-04 | Rig interface is hamlib/rigctld over TCP. | Decided — see ADR-03 |
| CON-05 | Reference deployment target is Raspberry Pi 4 (4 GB) / Pi 5 on RPiOS Bookworm 64-bit. | Hardware baseline; revised plan |
| CON-06 | The project is licensed AGPL-3.0-only. | Licensing policy — see ADR-07 |
| CON-07 | Split-host primary transport is WireGuard/Tailscale; SSH is fallback only. | Security policy — see ADR-05 |
| CON-08 | Transmit operation must comply with the operator's amateur-radio licence. | Regulatory (SH-6) |
| CON-09 | Security-first release gating is mandatory: security tests are release blockers, not optional hardening. | Governance charter |
| CON-10 | Documentation-first: requirements, tests, backlog, and roadmap are release artifacts maintained in the same change set. | Governance charter |

## 9. Risks

| ID | Risk | L×I | Mitigation | Owner |
|---|---|---|---|---|
| RISK-01 | Audio latency / jitter over WAN exceeds usability for full-duplex operation. | M×H | Opus with bounded jitter buffer; bitrate/sample-rate profiles for constrained links; measure against `NFR-PERF-02`; degrade gracefully. | SH-3 |
| RISK-02 | Pi 4 CPU saturates under simultaneous FFT + audio + 3 clients. | M×H | Benchmark early against `NFR-PERF-04`; bound spectrum cadence and frame sizes; offer Pi 5 as headroom target. | SH-3 |
| RISK-03 | iOS Safari microphone / Canvas quirks break audio capture or waterfall on iPhone/iPad. | H×M | Early dedicated iOS Safari test path (`NFR-COMPAT-03`); gesture-gated mic activation; feature-detect and fall back. | SH-3 |
| RISK-04 | Internet-exposure / misconfiguration leads to an unauthorised or unlawful emission. | L×H | Default private-tunnel bind, mutual peer auth, RBAC + PTT safety timeout, validation allowlist, audit; out-of-scope public exposure. | SH-1 / SH-5 |
| RISK-05 | Container deployment cannot cleanly pass through audio/GPIO devices with acceptable latency. | M×M | Treat container as evaluated-only (ADR-06); keep native systemd as reference; record decision in P3. | SH-3 |
| RISK-06 | Single-developer bandwidth is outstripped by the breadth of v1 scope. | H×M | Strict Must/Should/Could prioritisation; phase gating P0–P4; defer Could-class items; documentation-first to limit rework. | SH-3 |

## Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.6 | 2026-06-26 | DC0SK | Initial vision-and-scope baseline: SH-1..SH-7, G1..G8, scope/phasing P0..P4, ASM-01..ASM-07, CON-01..CON-10, RISK-01..RISK-06 under the area-coded ID scheme. |
