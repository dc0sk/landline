---
title: Roadmap and Release Plan
project: landline
doc_type: roadmap
status: draft
version: 0.1.0
owner: ""
last_updated: 2026-05-12
---

# Roadmap and Release Plan

## 1. Purpose

This document defines the phased development roadmap, milestone scope, entry and exit criteria, and release decision process for landline. It is updated at the end of each phase and whenever the backlog or requirements change in scope.

---

## 2. Guiding Principles

- **Documentation-first**: phase gates are blocked until docs are current and traceable.
- **Security-first**: security gates are mandatory blockers at every phase; they cannot be deferred.
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
| BL-001–004 | docs/requirements-spec.md, docs/test-spec.md, docs/backlog.md, docs/roadmap.md |
| BL-005 | Change control procedure |
| BL-010–013 | Threat model, trust boundaries, security gates, docs/security.md |

### Entry Criteria

- Repository initialised with version control.
- At least one project owner assigned.

### Exit Criteria

- [ ] docs/requirements-spec.md v1 complete with all functional, non-functional, security, compatibility, and deployment requirements carrying IDs and priority.
- [ ] docs/test-spec.md v1 complete with all requirement IDs mapped to at least one test case (status: Not written is acceptable at Phase 0 exit).
- [ ] docs/backlog.md v1 complete; all Phase 1 items reviewed and assigned.
- [ ] docs/roadmap.md v1 complete (this document).
- [ ] docs/security.md v1 complete with threat model, trust boundaries, and security gate criteria documented.
- [ ] Security gates list approved; referenced in phase exit checklist for Phase 1.

### Risks

| Risk | Mitigation |
|---|---|
| Requirements churn before Phase 1 starts | Time-box Phase 0 to 1 week; allow living updates but freeze MVP scope |
| Security gate criteria too vague to verify | Use TST-S-* test IDs as the definition of "gate passed" |

---

## 5. Phase 1 — Secure Control MVP

**Goal:** Working authenticated rig control on Raspberry Pi 4, accessible from Firefox and Chromium desktop browsers over HTTPS/WSS.

### In Scope

| Backlog IDs | Deliverable |
|---|---|
| BL-020–032 | Backend: auth, rate limiting, CORS, audit log, rigctld adapter, control handlers |
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

- [ ] All TST-F-001–009 (rig control) pass.
- [ ] All TST-F-040–044 (auth/session) pass.
- [ ] All TST-S-001–011 (security) pass.
- [ ] TST-NF-001 (control latency < 100 ms p95) pass on LAN.
- [ ] TST-D-001 and TST-D-003 (Pi 4 deployment, systemd) pass.
- [ ] Audit log verified: all rig-changing commands produce log entries with required fields.
- [ ] No Must backlog items (BL-020–032, BL-040–047, BL-080–081, BL-083) remain open.
- [ ] docs/requirements-spec.md updated with any Phase 1 scope changes.
- [ ] docs/test-spec.md updated with Phase 1 test execution records.

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

- [ ] TST-F-020–023 (spectrum/waterfall) pass on Firefox and Chromium desktop.
- [ ] TST-F-023 (iOS Safari Canvas rendering) pass.
- [ ] TST-C-001–007 (full browser matrix) complete with pass status.
- [ ] TST-F-032–033 (audio device selector) pass on at least Firefox desktop, Chrome Android, and Safari iOS.
- [ ] Spectrum update rate ≥ 2 Hz verified on Pi 4 under load (REQ-NF-005).
- [ ] No Must backlog items in Phase 2 scope remain open.
- [ ] docs/test-spec.md updated with Phase 2 test execution records.

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

### Entry Criteria

- Phase 2 exit criteria all confirmed.

### Exit Criteria

- [ ] TST-F-030–035 (audio) pass.
- [ ] TST-NF-002 (audio latency < 300 ms) pass on Pi 4 LAN.
- [ ] TST-D-004–005 (container run, non-root/read-only-rootfs) pass.
- [ ] docs/deployment.md container decision record written: accept or defer container deployment profile with rationale.
- [ ] Audio security tests (TST-S-001, TST-F-040 for audio channel) pass.
- [ ] No Must backlog items in Phase 3 scope remain open.
- [ ] docs/test-spec.md updated with Phase 3 test execution records.

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

### Entry Criteria

- Phase 3 exit criteria all confirmed.
- Container deployment decision recorded.

### Exit Criteria

- [ ] TST-NF-005 (24 h soak) pass on Pi 4.
- [ ] TST-NF-004 (CPU < 50 % under 3-client load) pass.
- [ ] TST-D-006 (rollback procedure) pass.
- [ ] TST-S-001 (HTTPS/WSS enforcement via nginx) pass.
- [ ] Every requirement with status Must or Should has at least one test with status Pass.
- [ ] No open Must-priority backlog items.
- [ ] docs/requirements-spec.md updated to reflect final shipped scope.
- [ ] docs/test-spec.md updated with all Phase 4 test execution records and final traceability.
- [ ] Ops runbook complete and reviewed.
- [ ] Go/no-go decision recorded in release checklist.

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
| 0.1.0 | 2026-05-12 | — | Initial draft |
