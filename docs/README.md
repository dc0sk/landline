---
title: "landline — Documentation & RE Process"
status: Draft
version: "0.6"
updated: 2026-06-26
authors:
  - Simon Keimer (DC0SK)
owns: []
---

# landline — Documentation & Requirements Engineering Process

A secure, browser-native **web remote for amateur-radio transceivers**, written in **Rust**
(Axum/Tokio backend on a Raspberry Pi) with a TypeScript frontend: authenticated rig control,
spectrum + waterfall, full-duplex audio, GPIO, and split-host deployment — under an
explicit **security-first, documentation-first** release discipline.

This `docs/` tree is the single source of truth for the requirements-engineering (RE) and
concept phase. Development is **requirements-driven and test-driven (TDD)** with **strict,
consistent traceability** from stakeholder needs down to test cases and test results, enforced
by a build-breaking gate.

| Status | Version | Date | Author |
|---|---|---|---|
| Draft baseline for review | 0.6 | 2026-06-26 | Simon Keimer (DC0SK) |

> **License notice:** landline is licensed under **AGPL-3.0-only**. See the top-level
> [LICENSE](../LICENSE).

---

## 1. Document map

| Doc | Purpose | IDs owned |
|---|---|---|
| [requirements/vision-and-scope.md](requirements/vision-and-scope.md) | Problem, vision, stakeholders, goals, scope, phasing, assumptions, constraints, risks | `SH-`, `ASM-`, `CON-`, `RISK-` |
| [requirements/stakeholder-requirements.md](requirements/stakeholder-requirements.md) | What stakeholders need (solution-independent) | `STK-` |
| [requirements/system-requirements.md](requirements/system-requirements.md) | Functional + non-functional software requirements (testable) | `FR-`, `NFR-` |
| [concept/architecture.md](concept/architecture.md) | Concept, components, data flow, trust boundaries, ADRs | `ARC-`, `ADR-` |
| [test/test-strategy.md](test/test-strategy.md) | Test approach, levels, verification methods, traceability matrix | `TC-` |
| [security.md](security.md) | Threat model, controls, security release gates, secrets handling | — |
| [governance.md](governance.md) | Security-first / documentation-first governance, phase-gate rules, change control | — |
| [backlog.md](backlog.md) | Prioritised product/engineering backlog with dependencies | `EP-`, `BL-` |
| [roadmap.md](roadmap.md) | Release/phase plan, milestones, entry/exit gates | — |
| [deployment.md](deployment.md) | Split-host deployment profiles + decision record | — |
| [documentation-review.md](documentation-review.md) | Documentation gaps and improvement backlog | — |

## 1a. Document conventions

**Every Markdown document under `docs/` MUST begin with a YAML frontmatter block.** Required
keys: `title`, `status` (`Draft` · `Approved` · `Withdrawn`), `version`, `updated` (ISO date),
`authors` (list). Documents that own an ID prefix also carry `owns: [PREFIX, …]`. The frontmatter
is the machine-readable header; the human-readable Status/Version line in the body may mirror it.
The pre-commit hook ([`.githooks/pre-commit`](../.githooks/pre-commit)) rejects any `docs/*.md`
missing required frontmatter keys.

## 2. Identifier scheme

All artifacts carry a stable, never-reused ID. IDs are immutable once published in a
baseline; if a requirement is dropped it is marked `Withdrawn`, not deleted, and its ID is
retired. (The 0.x flat scheme — `REQ-*`/`TST-*` — was never baselined; its one-time migration
to the scheme below is recorded in [Appendix A](#appendix-a--migration-from-the-0x-flat-scheme).)

| Prefix | Artifact | Example |
|---|---|---|
| `SH-N` | Stakeholder | `SH-1` |
| `STK-NN` | Stakeholder requirement | `STK-03` |
| `ASM-NN` | Assumption | `ASM-02` |
| `CON-NN` | Constraint | `CON-01` |
| `RISK-NN` | Risk | `RISK-01` |
| `FR-<AREA>-NN` | Functional software requirement | `FR-RIG-02` |
| `NFR-<AREA>-NN` | Non-functional requirement | `NFR-SEC-01` |
| `ADR-NN` | Architecture decision record | `ADR-04` |
| `ARC-NN` | Architecture element / component | `ARC-06` |
| `TC-<AREA>-NN` | Test case | `TC-RIG-02` |
| `EP-NN` / `BL-NNN` | Backlog epic / item | `EP-03` / `BL-021` |

### Functional areas (`<AREA>`)

| Area | Meaning |
|---|---|
| `RIG` | Rig control: frequency, mode, PTT, metering, hamlib/rigctld adapter, command validation, exclusive access |
| `SPEC` | Spectrum & waterfall (FFT stream, Canvas render) |
| `AUD` | Audio streaming (RX/TX, Opus, device selection) |
| `AUTH` | Authentication, session tokens, RBAC |
| `AUDIT` | Audit logging of state-changing actions |
| `HOST` | Distributed / split-host frontend deployment (functional connectivity) |
| `GPIO` | Raspberry Pi GPIO digital I/O |

### Non-functional areas

| Area | Meaning |
|---|---|
| `PERF` | Performance (latency, throughput, CPU, spectrum cadence) |
| `REL` | Reliability (reconnect, recovery, soak) |
| `MAINT` | Maintainability (lint, test coverage, update/rollback) |
| `SEC` | Security |
| `COMPAT` | Browser/device compatibility |
| `DEPLOY` | Deployment targets & packaging |
| `LIC` | Licensing |

## 3. Requirement attributes

Every `FR`/`NFR` is recorded with:

- **ID**, **Title**, **Statement** (single "shall", testable, unambiguous)
- **Rationale**
- **Source / Trace-up** — upstream `STK-` (and standard reference where applicable)
- **Priority** — `M` (must, v1) · `S` (should, v1 if time) · `C` (could) · `W` (won't, this release)
- **Verification method** — `T` test (automated) · `D` demonstration · `I` inspection · `A` analysis
- **Acceptance criteria** — the conditions a test asserts
- **Status** — `Proposed` · `Approved` · `Implemented` · `Verified` · `Withdrawn`
- **Trace-down** — `ARC-` element(s) and `TC-` test case(s)

## 4. Traceability model (the V)

```
STK (need)  ──►  FR / NFR (system req)  ──►  ARC / ADR (design)  ──►  TC (test)  ──►  Result
   ▲                    │                                                  │
   └──────── every FR/NFR traces up to ≥1 STK ──────────────────┘         │
                        └──────── every M/S FR/NFR traces down to ≥1 TC ───┘
```

Tracing rules (enforced — see [test/test-strategy.md](test/test-strategy.md) §Coverage gate
and [`scripts/trace-gate`](../scripts/trace-gate)):

- **R1** Every `STK` is satisfied by ≥1 `FR`/`NFR`. (no orphan needs)
- **R2** Every `FR`/`NFR` traces up to ≥1 `STK`. (no gold-plating)
- **R3** Every `M`/`S` `FR`/`NFR` is covered by ≥1 `TC`. (no untested requirement)
- **R4** Every `TC` names the requirement ID(s) it verifies. (no dangling test)
- **R5** Every implemented `FR` is realized by ≥1 named `ARC` element.

`scripts/trace-gate` parses the requirement tables (SRS) and the test traceability matrix and
**fails the build** on **R4** (a `TC` naming an unknown requirement) and on any **uncovered
`M`/`S`** requirement (**R3**). `Could`-priority gaps are reported informationally. This makes
traceability a build-breaking invariant, not a manual spreadsheet. The gate is language-agnostic
today (docs-only repo); once the Rust workspace lands it is promoted into `cargo xtask` and the
`hardware`-free test suite per the latent hooks.

Verification-method note: requirements verified by `I`/`A` (e.g. `NFR-MAINT-01` clippy,
`NFR-LIC-*`) still carry a `TC` whose method column records `I`/`A` rather than `T`, so R3 holds
uniformly.

## 5. TDD workflow per requirement

1. Pick an `Approved` `FR`/`NFR`.
2. Write the `TC` first (red), tagging the requirement ID in the test name/annotation.
3. Implement the minimal `ARC` code to pass (green).
4. Refactor; keep the trace annotation.
5. Update the requirement `Status` → `Verified` and the matrix once the `TC` passes in CI.

Security tests are mandatory blockers: a failing `NFR-SEC-*` test fails the phase gate
regardless of other results (see [governance.md](governance.md)).

## 6. Baseline & change control

- Each published version is a **baseline** (git tag `req-vX.Y`).
- Changes after baseline go through a change note appended to the affected doc's
  **Change History** with date, author, affected IDs, and reason.
- Any change that touches scope, security posture, architecture, deployment model, or release
  criteria MUST update requirements, tests, backlog, and roadmap **in the same change set**
  (enforced by review per [governance.md](governance.md) §Change Control).
- This is a **Draft baseline (0.6)** intended for review; nothing is `Approved` until the
  stakeholder sign-off recorded in the §Change History of each document.

---

## Appendix A — Migration from the 0.x flat scheme

The 0.x documents used a flat, prefix-only scheme (`REQ-F-*`, `REQ-NF-*`, `REQ-S-*`, `REQ-C-*`,
`REQ-D-*`, `REQ-L-*`, `TST-*`). Because nothing was ever `Approved`/baselined, these IDs were
renumbered once into the area-coded scheme above. The full one-to-one map is below; old IDs are
**retired** and must not be reused.

| Old prefix | New scheme |
|---|---|
| `REQ-F-0xx` (rig) | `FR-RIG-0x` |
| `REQ-F-02x` (spectrum) | `FR-SPEC-0x` |
| `REQ-F-03x` (audio) | `FR-AUD-0x` |
| `REQ-F-04x` (auth) | `FR-AUTH-0x` |
| `REQ-F-05x` (audit) | `FR-AUDIT-0x` |
| `REQ-F-06x` (split-host) | `FR-HOST-0x` |
| `REQ-F-070` (GPIO) | `FR-GPIO-01` |
| `REQ-NF-00x` (perf) | `NFR-PERF-0x` |
| `REQ-NF-01x` (reliability) | `NFR-REL-0x` |
| `REQ-NF-02x` (maintainability) | `NFR-MAINT-0x` |
| `REQ-S-0xx` (security) | `NFR-SEC-0x` |
| `REQ-C-00x` (compatibility) | `NFR-COMPAT-0x` |
| `REQ-D-00x` (deployment) | `NFR-DEPLOY-0x` |
| `REQ-L-00x` (licensing) | `NFR-LIC-0x` |
| `TST-*` | `TC-<AREA>-NN` |

The exhaustive per-ID table is maintained alongside the migration commit; the
[system-requirements](requirements/system-requirements.md) and
[test-strategy](test/test-strategy.md) documents carry the authoritative current IDs.

## Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.6 | 2026-06-26 | DC0SK | Adopted area-coded ID scheme, layered RE doc tree (vision/stakeholder/system/concept/test), R1–R5 traceability gate, and frontmatter conventions. Migrated 0.x flat IDs (Appendix A). |
