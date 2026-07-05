---
title: Release Checklist
status: Draft
version: "0.1"
updated: 2026-07-05
authors:
  - Simon Keimer (DC0SK)
---

# Release Checklist

The go/no-go gate for a landline release (BL-104/106). A release is **blocked**
until every Must item below is checked. This operationalises the roadmap Phase 4
exit criteria and the security-first / documentation-first / license-first
governance rules.

License notice: This project is licensed under AGPL-3.0-only. See the top-level
[LICENSE](../LICENSE).

## 1. Traceability & tests

- [ ] `python3 scripts/trace-gate.py` exits 0 (R3 M/S coverage, R4 no dangling traces).
- [ ] `cargo xtask ci` green (fmt, clippy pedantic, tests) and the frontend job green (typecheck, tests, build).
- [ ] Every `Must`/`Should` requirement has ≥ 1 test recorded `Pass` in [test/test-strategy.md](test/test-strategy.md) (execution records for the phase filled in).
- [ ] No `Fail` tests; any `Blocked`/`Deferred` test has a tracked cause and disposition.

## 2. Security gates (mandatory blockers)

- [ ] All `NFR-SEC-*` tests in scope pass (auth, RBAC, rate/size limits, CORS, PTT timeout, input validation, GPIO allowlist, error sanitisation).
- [ ] TLS/WSS enforced at the edge; plaintext rejected (TC-SEC-01, [deploy/nginx](../deploy/nginx)).
- [ ] Secrets stored 0600; rotation policy current ([security.md §8](security.md)).
- [ ] Audit log verified: state-changing actions and auth failures recorded; chain intact.

## 3. Deployment & operations

- [ ] Reference (systemd) deployment starts/stops cleanly on the target (TC-DEPLOY-03).
- [ ] Rollback procedure executed and verified without data loss (TC-DEPLOY-06, [deploy/RUNBOOK.md](../deploy/RUNBOOK.md)).
- [ ] Container profile decision recorded (accept/defer) if evaluated ([deploy/container](../deploy/container)).
- [ ] Ops runbook current.

## 4. License compliance (BL-122, license-first gate)

- [ ] Top-level `LICENSE` present and contains the AGPL-3.0-only text (TC-LIC-02).
- [ ] Declared license identifier is `AGPL-3.0-only` in `Cargo.toml`/`package.json` (TC-LIC-01).
- [ ] License notices present in the core docs and the frontend (TC-LIC-02).

## 5. Go/No-Go decision

- [ ] All Must items above checked; open `Should`/`Could` items categorised (include / defer to Won't / schedule for next version).
- [ ] Decision recorded below with date and approver.
- [ ] Release tagged (`req-vX.Y` baseline + release tag).

| Version | Date | Decision (go/no-go) | Approver | Notes |
|---|---|---|---|---|
| _tbd_ | _tbd_ | _tbd_ | _tbd_ | _first release pending HIL + browser-matrix validation_ |

## Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.1 | 2026-07-05 | DC0SK | Initial release checklist: traceability, security, deployment, license, and go/no-go gates. |
