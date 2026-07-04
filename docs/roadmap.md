---
title: Roadmap and Release Plan
status: Draft
version: 0.5.2
updated: 2026-07-04
authors:
  - Simon Keimer (DC0SK)
---

# Roadmap and Release Plan

## 1. Purpose

This document defines the phased development roadmap, milestone scope, entry and exit criteria, and release decision process for landline. It is updated at the end of each phase and whenever the backlog or requirements change in scope.

---

## 2. Guiding Principles

- **Documentation-first**: phase gates are blocked until docs are current and traceable.
- **Security-first**: security gates are mandatory blockers at every phase; they cannot be deferred.
- **License-first governance**: AGPL obligations are part of release readiness, not post-release cleanup.
- **Scope discipline**: items not listed in a phase's in-scope list are not worked in that phase.
- **Exit criteria govern release**: a phase does not close until all exit criteria are confirmed.

---

## 3. Phase Overview

```
Phase 0 ─── Foundation & Docs
    │
Phase 1 ─── Secure Control MVP
    │
Phase 2 ─── Spectrum, Waterfall & Mobile
    │
Phase 3 ─── Audio & Container Evaluation
    │
Phase 4 ─── Release Candidate & Operations
```

---

## 4. Phase 0 — Foundation and Documentation

**Goal:** Establish all project governance artifacts and security baseline before any implementation begins.

### In Scope

| Backlog IDs | Deliverable |
|---|---|
| BL-001–004 | docs/requirements/system-requirements.md, docs/test/test-strategy.md, docs/backlog.md, docs/roadmap.md |
| BL-005–006 | Change control procedure and governance charter |
| BL-010–013 | Threat model, trust boundaries, security gates, docs/security.md |
| BL-120–121 | AGPL license file and license notices in core project docs |

### Entry Criteria

- Repository initialised with version control.
- At least one project owner assigned.

### Exit Criteria

- [x] docs/requirements/system-requirements.md v1 complete with all functional, non-functional, security, compatibility, and deployment requirements carrying IDs and priority.
- [x] docs/test/test-strategy.md v1 complete with all requirement IDs mapped to at least one test case (status: Not written is acceptable at Phase 0 exit).
- [x] docs/backlog.md v1 complete; all Phase 1 items reviewed and assigned.
- [x] docs/roadmap.md v1 complete (this document).
- [x] docs/governance.md v1 complete with security-first governance policy and exception handling rules.
- [x] docs/security.md v1 complete with threat model, trust boundaries, and security gate criteria documented.
- [x] Security gates list approved; referenced in phase exit checklist for Phase 1.
- [x] Top-level LICENSE file added with AGPL-3.0-only text.
- [x] License notice added to core project docs.

> **Phase 0 status (2026-07-04): complete — ready for Phase 1 entry.** All exit criteria above
> are satisfied by the existing RE baseline; the traceability gate (`scripts/trace-gate.py`) is
> green with 78 requirements declared and all 76 Must/Should requirements covered by tests.
> The two Could-priority requirements without a test (FR-RIG-07, FR-SPEC-04) are intentionally
> uncovered per rule R3, which enforces coverage for Must/Should scope only. Remaining Phase 0
> follow-up: the secrets *rotation* policy (BL-012) is deferred to before production release —
> see docs/backlog.md note on BL-012.

### Risks

| Risk | Mitigation |
|---|---|
| Requirements churn before Phase 1 starts | Time-box Phase 0 to 1 week; allow living updates but freeze MVP scope |
| Security gate criteria too vague to verify | Use TC-SEC-* test IDs as the definition of "gate passed" |

---

## 5. Phase 1 — Secure Control MVP

**Goal:** Working authenticated rig control on Raspberry Pi 4, accessible from Firefox and Chromium desktop browsers over HTTPS/WSS.

### In Scope

| Backlog IDs | Deliverable |
|---|---|
| BL-020–033 | Backend: auth, rate limiting, CORS, audit log, rigctld adapter, control handlers, GPIO control API |
| BL-040–047 | Frontend: session bootstrap, frequency/mode/PTT/S-meter UI, WebSocket client, responsive layout |
| BL-080–081, BL-083 | Systemd service unit, config file, cross-compiled aarch64 release binary |

### Out of Scope (Phase 1)

- Spectrum/waterfall (Phase 2)
- Mobile browser testing beyond layout (Phase 2)
- Audio pipeline (Phase 3)
- Container evaluation (Phase 3)
- Production TLS/nginx (Phase 4)

### Entry Criteria

- Phase 0 exit criteria all confirmed.
- Raspberry Pi 4 hardware available for testing.

### Exit Criteria

- [ ] All TC-RIG-01–TC-RIG-09 (rig control) pass.
- [ ] TC-GPIO-01 (GPIO digital I/O control on Raspberry Pi, minimum 5 pins) pass.
- [ ] All TC-AUTH-01–TC-AUTH-05 (auth/session) pass.
- [ ] All TC-SEC-01–TC-SEC-11 and TC-SEC-15 (security) pass.
- [ ] TC-PERF-01 (control latency < 100 ms p95) pass on LAN.
- [ ] TC-DEPLOY-01 and TC-DEPLOY-03 (Pi 4 deployment, systemd) pass.
- [ ] Audit log verified: all rig-changing commands produce log entries with required fields.
- [ ] No Must backlog items (BL-020–033, BL-040–047, BL-080–081, BL-083) remain open.
- [ ] docs/requirements/system-requirements.md updated with any Phase 1 scope changes.
- [ ] docs/test/test-strategy.md updated with Phase 1 test execution records.

### Risks

| Risk | Mitigation |
|---|---|
| hamlib FFI instability on Pi | Use rigctld TCP adapter first; FFI can follow later |
| JWT library choice adds complexity | Evaluate `jsonwebtoken` crate early; accept simple bearer token if JWT adds >1 week |
| Pi cross-compilation issues | Test `cross` toolchain in CI from day one; unblock early |

---

## 6. Phase 2 — Spectrum, Waterfall, and Mobile Compatibility

**Goal:** Add spectrum/waterfall display and validate the full browser/device compatibility matrix.

### In Scope

| Backlog IDs | Deliverable |
|---|---|
| BL-050–055 | Backend FFT pipeline, spectrum WebSocket stream, frontend Canvas waterfall |
| BL-060–062 | Full browser matrix testing, touch optimisation, audio device selector UI |

### Entry Criteria

- Phase 1 exit criteria all confirmed.

### Exit Criteria

- [ ] TC-SPEC-01–TC-SPEC-04 (spectrum/waterfall) pass on Firefox and Chromium desktop.
- [ ] TC-SPEC-04 (iOS Safari Canvas rendering) pass.
- [ ] TC-COMPAT-01–TC-COMPAT-07 (full browser matrix) complete with pass status.
- [ ] TC-AUD-03–TC-AUD-04 (audio device selector) pass on at least Firefox desktop, Chrome Android, and Safari iOS.
- [ ] Spectrum update rate ≥ 2 Hz verified on Pi 4 under load (NFR-PERF-05).
- [ ] No Must backlog items in Phase 2 scope remain open.
- [ ] docs/test/test-strategy.md updated with Phase 2 test execution records.

### Risks

| Risk | Mitigation |
|---|---|
| iOS Safari mic permission flow differs from desktop | Test early; prototype audio device selector on iOS before full audio work (Phase 3) |
| FFT CPU load degrades Pi 4 under concurrent clients | Profile on Pi 4 before completing Phase 2; adjust bin count/update rate as needed |
| Canvas waterfall performance on low-end Android | Limit waterfall height and update rate on mobile; add device detection hint |

---

## 7. Phase 3 — Audio Pipeline and Container Evaluation

**Goal:** Bidirectional audio streaming over WSS and a completed container deployment evaluation.

### In Scope

| Backlog IDs | Deliverable |
|---|---|
| BL-070–078 | Full audio pipeline: Pi CPAL capture, Opus encode, WSS stream, browser decode/playback; reverse path |
| BL-090–095 | Dockerfile, compose.yml, device passthrough evaluation, latency benchmark, decision record |
| BL-110–114 | Split-host deployment topology and secure connectivity profiles (WireGuard primary, Tailscale alternative, SSH fallback docs) |

### Entry Criteria

- Phase 2 exit criteria all confirmed.

### Exit Criteria

- [ ] TC-AUD-01–TC-AUD-06 (audio) pass.
- [ ] TC-PERF-02 (audio latency < 300 ms) pass on Pi 4 LAN.
- [ ] TC-DEPLOY-04–TC-DEPLOY-05 (container run, non-root/read-only-rootfs) pass.
- [ ] TC-HOST-01–TC-HOST-03 (split-host frontend deployment) pass.
- [ ] TC-SEC-12–TC-SEC-14 (split-host security controls) pass.
- [ ] TC-DEPLOY-07 (split-host runbook execution) pass.
- [ ] docs/deployment.md container decision record written: accept or defer container deployment profile with rationale.
- [ ] Audio security tests (TC-SEC-01, TC-AUTH-01 for audio channel) pass.
- [ ] No Must backlog items in Phase 3 scope remain open.
- [ ] docs/test/test-strategy.md updated with Phase 3 test execution records.

### Risks

| Risk | Mitigation |
|---|---|
| Opus ARM encoding latency exceeds budget | Benchmark on Pi 4 in week 1 of phase; reduce bitrate or frame size if needed |
| ALSA/PipeWire passthrough breaks in container | Document as known limitation; native deployment remains the reference |
| Audio sync drift in long sessions | Implement simple NTP-based drift compensation; measure drift over 1-hour session |

---

## 8. Phase 4 — Release Candidate and Operations

**Goal:** Production-ready deployment, full documentation, soak tests, and release approval.

### In Scope

| Backlog IDs | Deliverable |
|---|---|
| BL-100–101 | Production TLS and nginx reverse proxy config |
| BL-102–103 | 24 h soak test, Pi 4 load test |
| BL-082 | Rollback procedure |
| BL-104–106 | Final doc alignment, ops runbook, release checklist |
| BL-122 | License compliance check integrated in release checklist |

### Entry Criteria

- Phase 3 exit criteria all confirmed.
- Container deployment decision recorded.

### Exit Criteria

- [ ] TC-REL-03 (24 h soak) pass on Pi 4.
- [ ] TC-PERF-04 (CPU < 50 % under 3-client load) pass.
- [ ] TC-DEPLOY-06 (rollback procedure) pass.
- [ ] TC-SEC-01 (HTTPS/WSS enforcement via nginx) pass.
- [ ] Every requirement with status Must or Should has at least one test with status Pass.
- [ ] No open Must-priority backlog items.
- [ ] docs/requirements/system-requirements.md updated to reflect final shipped scope.
- [ ] docs/test/test-strategy.md updated with all Phase 4 test execution records and final traceability.
- [ ] Ops runbook complete and reviewed.
- [ ] Go/no-go decision recorded in release checklist.
- [ ] License compliance check complete (AGPL present and documented).

### Risks

| Risk | Mitigation |
|---|---|
| Soak test reveals memory leak | Instrument with `valgrind` / `heaptrack` in earlier phases to catch early |
| TLS certificate renewal not automated | Document manual renewal; add renewal reminder to ops runbook |
| Pi thermal throttling under 24 h load | Monitor temperature; add note to deployment requirements if cooling is needed |

---

## 9. Release Decision Process

At Phase 4 exit:

1. Confirm all exit criteria checkboxes are ticked.
2. Review open Should/Could items; categorise as: (a) include, (b) defer with ID recorded in Won't list, (c) schedule for v0.2.
3. Record go/no-go decision in docs/backlog.md release section with date and approver.
4. Tag release in version control.

---

## 10. Version History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.5.2 | 2026-07-04 | DC0SK | Ticked all Phase 0 exit criteria (verified against existing docs) and recorded Phase 0 as complete / ready for Phase 1 entry. |
| 0.5.1 | 2026-06-26 | DC0SK | Migrated to area-coded FR/NFR/TC ids and new doc-tree frontmatter. |
| 0.5.0 | 2026-05-13 | — | Added GPIO security test gate (TC-SEC-15) to Phase 1 exit criteria |
| 0.4.0 | 2026-05-13 | — | Added Raspberry Pi GPIO requirement to Phase 1 scope and exit criteria |
| 0.3.0 | 2026-05-13 | — | Added Phase 0 governance charter deliverable with security-first policy gate |
| 0.2.0 | 2026-05-13 | — | Added AGPL governance and split-host secure deployment to phase scope and exits |
| 0.1.0 | 2026-05-12 | — | Initial draft |
