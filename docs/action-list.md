---
title: Action List
status: Draft
version: "0.1"
updated: 2026-07-04
authors:
  - Simon Keimer (DC0SK)
owns: []
---

# Action List

This is the ordered, working checklist that turns the backlog and roadmap into concrete next
steps. It is the **operational companion** to [backlog.md](backlog.md) and
[roadmap.md](roadmap.md): every action cites the backlog item(s) and requirement/test IDs it
advances — this document **references** those IDs, it does not define new ones. Actions carry
stable labels (A1, A2, …) so they can be referenced in commits and reviews.

License notice: This project is licensed under AGPL-3.0-only. See the top-level LICENSE file.

---

## 1. Current state

- **Phase 0 (RE baseline): complete** — all governance, requirements, security, test, and
  planning docs exist; Phase 0 exit criteria are ticked in [roadmap.md](roadmap.md) §4.
- **Trace gate: green** — 78 requirements declared, all 76 Must/Should covered by tests
  (`python3 scripts/trace-gate.py`). FR-RIG-07 and FR-SPEC-04 are Could-priority and
  intentionally uncovered per rule R3.
- **Next up: Phase 1 — Secure Control MVP** (authenticated rig control on Raspberry Pi 4,
  desktop Firefox/Chromium over HTTPS/WSS).
- Open Phase 0 remainder: secrets *rotation* policy (BL-012) is deferred to before production
  release — tracked below under Phase 4 preparation.

---

## 2. Milestone: Phase 1 kickoff — backend foundation

Ordered to respect the backlog dependency graph (workspace → auth → security middleware →
rigctld adapter → control handlers → GPIO).

- [ ] A1. Initialize Rust workspace (Tokio + Axum + Tower), with `cargo fmt`/`clippy` clean baseline — BL-020 · NFR-MAINT-01
- [ ] A2. Activate latent tooling: rename `.github/workflows/ci.yml.disabled` → `ci.yml`, enable the commented-out Rust steps in `.githooks/pre-commit`/`.githooks/pre-push` (fmt, clippy `-D warnings`, test, audit), and promote the trace gate into `cargo xtask` — BL-020 · NFR-MAINT-01
- [ ] A3. Set up cross-compilation for `aarch64-unknown-linux-gnu` in CI from day one — BL-083 · NFR-DEPLOY-01 · TC-DEPLOY-01–TC-DEPLOY-02
- [ ] A4. Implement structured tracing/logging integration (no secrets in logs) — BL-032 · NFR-SEC-09, NFR-SEC-12 · TC-SEC-09–TC-SEC-10
- [ ] A5. Implement TOML config loader with secure defaults and 0600 permission checks — BL-081 · NFR-DEPLOY-04
- [ ] A6. Implement auth middleware (JWT issue/verify, expiry, role claims, RBAC) — BL-021 · FR-AUTH-01–FR-AUTH-05, NFR-SEC-01–NFR-SEC-02 · TC-AUTH-01–TC-AUTH-05, TC-SEC-01–TC-SEC-02
- [ ] A7. Implement rate limiting and request/WS-frame size limits — BL-022 · NFR-SEC-04–NFR-SEC-05 · TC-SEC-04–TC-SEC-05
- [ ] A8. Implement CORS origin allowlist policy — BL-023 · NFR-SEC-06 · TC-SEC-06
- [ ] A9. Implement audit log subsystem (append-only, state-changing actions + auth failures) — BL-024 · FR-AUDIT-01–FR-AUDIT-04 · TC-AUDIT-01–TC-AUDIT-02
- [ ] A10. Implement rigctld TCP adapter with command allowlist/sanitisation — BL-025 · FR-RIG-08–FR-RIG-09 · TC-RIG-07–TC-RIG-08
- [ ] A11. Implement frequency read/set handlers — BL-026 · FR-RIG-01–FR-RIG-02 · TC-RIG-01–TC-RIG-02
- [ ] A12. Implement mode read/set handlers — BL-027 · FR-RIG-03–FR-RIG-04 · TC-RIG-03
- [ ] A13. Implement PTT handler with role check and safety timeout — BL-028 · FR-RIG-05, NFR-SEC-07 · TC-RIG-04–TC-RIG-05, TC-SEC-07
- [ ] A14. Implement rig access mutex for concurrent clients — BL-030 · FR-RIG-10 · TC-RIG-09
- [ ] A15. Implement S-meter streaming — BL-029 · FR-RIG-06 · TC-RIG-06
- [ ] A16. Implement rigctld reconnect/circuit-breaker — BL-031 · NFR-REL-02 · TC-REL-02
- [ ] A17. Implement GPIO control API (≥ 5 digital pins, allowlist, safe startup states, role-gated) — BL-033 · FR-GPIO-01, NFR-SEC-16 · TC-GPIO-01, TC-SEC-15

## 3. Milestone: Phase 1 — frontend and deployment

Frontend bootstrap can start in parallel once the auth contract (A6) is stable.

- [ ] A18. Initialize TypeScript/HTML5 frontend project — BL-040 · NFR-COMPAT-01–NFR-COMPAT-02
- [ ] A19. Implement authenticated session bootstrap (login, token storage, logout) — BL-041 · FR-AUTH-01–FR-AUTH-05 · TC-AUTH-01–TC-AUTH-05
- [ ] A20. Implement WebSocket client with reconnect/backoff — BL-046 · NFR-REL-01 · TC-REL-01
- [ ] A21. Implement frequency display and tuning control — BL-042 · FR-RIG-01–FR-RIG-02, NFR-COMPAT-06 · TC-RIG-01–TC-RIG-02, TC-COMPAT-01–TC-COMPAT-02
- [ ] A22. Implement mode selector — BL-043 · FR-RIG-03–FR-RIG-04 · TC-RIG-03
- [ ] A23. Implement PTT button with visual transmit indicator — BL-044 · FR-RIG-05 · TC-RIG-04
- [ ] A24. Implement S-meter display — BL-045 · FR-RIG-06 · TC-RIG-06
- [ ] A25. Responsive CSS layout (desktop 3-column, mobile vertical stack) — BL-047 · NFR-COMPAT-03–NFR-COMPAT-06 · TC-COMPAT-04–TC-COMPAT-07
- [ ] A26. Write systemd service unit (start/stop/restart, resource limits) — BL-080 · NFR-DEPLOY-02 · TC-DEPLOY-03
- [ ] A27. Phase 1 exit review: run all scoped TC-RIG/TC-GPIO/TC-AUTH/TC-SEC/TC-PERF-01/TC-DEPLOY gates and tick roadmap.md §5 exit criteria — roadmap §5 · docs updated per governance change control

## 4. Milestone: Phase 2 — spectrum, waterfall, mobile (forward-looking)

- [ ] A28. FFT pipeline + spectrum WebSocket stream at configurable rate — BL-050, BL-051, BL-054 · FR-SPEC-01–FR-SPEC-02 · TC-SPEC-01–TC-SPEC-02
- [ ] A29. Canvas 2D waterfall renderer; verify on iOS Safari (no WebGL) — BL-052, BL-053 · FR-SPEC-03 · TC-SPEC-03–TC-SPEC-04
- [ ] A30. Full browser matrix validation, touch optimisation, audio device selector UI — BL-060–BL-062 · NFR-COMPAT-01–NFR-COMPAT-07, FR-AUD-03–FR-AUD-04 · TC-COMPAT-01–TC-COMPAT-07, TC-AUD-03–TC-AUD-04

## 5. Milestone: Phase 3 — audio, container evaluation, split-host (forward-looking)

- [ ] A31. Bidirectional Opus audio pipeline over WSS with per-session auth — BL-070–BL-077 · FR-AUD-01–FR-AUD-06, NFR-SEC-01 · TC-AUD-01–TC-AUD-06
- [ ] A32. Measure/document end-to-end audio latency on Pi 4 — BL-078 · NFR-PERF-02 · TC-PERF-02
- [ ] A33. Container evaluation (Dockerfile, compose, device passthrough, latency benchmark, decision record) — BL-090–BL-095 · NFR-DEPLOY-03, NFR-SEC-10 · TC-DEPLOY-04–TC-DEPLOY-05
- [ ] A34. Split-host topology + WireGuard/Tailscale profiles, SSH fallback docs, frontend runtime endpoint config — BL-110–BL-114 · FR-HOST-01–FR-HOST-04, NFR-SEC-13–NFR-SEC-15 · TC-HOST-01–TC-HOST-03, TC-SEC-12–TC-SEC-14

## 6. Milestone: Phase 4 — release candidate and operations (forward-looking)

- [ ] A35. Define secrets rotation policy and rotation runbook (closes the open BL-012 remainder) — BL-012, BL-105 · NFR-SEC-03 · TC-SEC-03
- [ ] A36. Production TLS + nginx reverse proxy config — BL-100–BL-101 · NFR-SEC-01 · TC-SEC-01
- [ ] A37. Soak test (24 h) and 3-client load test on Pi 4 — BL-102–BL-103 · NFR-REL-03, NFR-PERF-04 · TC-REL-03, TC-PERF-04
- [ ] A38. Rollback procedure, ops runbook, final doc alignment, release checklist incl. license compliance — BL-082, BL-104–BL-106, BL-122 · NFR-DEPLOY-05, NFR-LIC-01–NFR-LIC-02 · TC-DEPLOY-06, TC-LIC-01–TC-LIC-02

---

## 7. Definition of done per action

Each action is done per the **Definition of Done in [backlog.md](backlog.md) §1**
(implementation merged, security gate passed where applicable, mapped tests passing, docs
updated). In addition:

- Completing an action must keep the trace gate green (`python3 scripts/trace-gate.py`, or
  `cargo xtask` once promoted per A2).
- Requirements, tests, backlog, and roadmap must be updated **in the same change set** per
  [governance.md](governance.md) §5 change control.

## 8. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.1 | 2026-07-04 | DC0SK | Initial action list: Phase 1 kickoff ordering plus forward-looking Phase 2–4 milestones. |
