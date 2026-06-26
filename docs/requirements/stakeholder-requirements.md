---
title: "landline — Stakeholder Requirements"
status: Draft
version: "0.6"
updated: 2026-06-26
authors:
  - Simon Keimer (DC0SK)
owns: [STK]
---

# landline — Stakeholder Requirements

> **License notice:** landline is licensed under **AGPL-3.0-only**. See the top-level
> [LICENSE](../../LICENSE).

| Status | Version | Date | Author |
|---|---|---|---|
| Draft baseline for review | 0.6 | 2026-06-26 | Simon Keimer (DC0SK) |

This document records the **solution-independent** stakeholder needs (`STK-`) derived from the
stakeholders (`SH-`) and goals in [vision-and-scope.md](vision-and-scope.md). Each `STK-` states
*what* a stakeholder needs, not *how* landline provides it; the testable "how" lives as `FR-`/`NFR-`
items in [system-requirements.md](system-requirements.md). Priority is `M` (must, v1) or `S`
(should, v1 if time).

## 1. Stakeholder requirements

| ID | Need | Source SH | Priority | Rationale |
|---|---|---|---|---|
| STK-01 | Operate the transceiver remotely: tune frequency, set mode, key/transmit | SH-1 | M | The core purpose — a remote that cannot safely control and key the rig has no value. |
| STK-02 | Monitor receive in real time: RX audio, spectrum/waterfall, S-meter | SH-1, SH-2 | M | Operators and observers must hear and see the band to operate or follow activity. |
| STK-03 | Only authenticated, authorised users gain access; roles separate control from view | SH-1, SH-5 | M | A network-reachable transmitter must never be controllable by an unauthenticated or under-privileged party. |
| STK-04 | All access attempts and rig-changing actions are accountable and auditable | SH-6 | M | The licensee is personally accountable for emissions; actions must be attributable after the fact. |
| STK-05 | Usable from common desktop and mobile browsers with no client install | SH-1, SH-2 | M | "From anywhere a browser runs" depends on broad, install-free browser reach. |
| STK-06 | The link is confidential and tamper-resistant over untrusted networks | SH-1, SH-5 | M | Credentials, control, and audio cross networks the operator does not fully control. |
| STK-07 | Run the frontend remotely from the rig site over a secure private network | SH-5 | M | Split-host operation must work without exposing the backend to the public internet. |
| STK-08 | Run reliably on a Raspberry Pi as an always-on appliance | SH-4 | M | The station host needs an unattended appliance that stays up and within CPU budget. |
| STK-09 | Drive station auxiliary hardware via GPIO safely | SH-4 | M | Antenna/PA/relay control via GPIO must be allowlisted and start in safe states. |
| STK-10 | Maintainable, updatable, and recoverable (rollback) | SH-3 | S | A single-developer project must stay lintable, testable, and safely reversible. |
| STK-11 | Free/open under AGPL; transmit use remains lawful | SH-6, SH-7 | M | Licensing obligations and regulatory compliance are non-negotiable release criteria. |
| STK-12 | Responsive, low-latency control feedback for safe operating | SH-1 | M | Operating decisions (tuning, keying) need prompt feedback to be made safely. |

## 2. Satisfaction note (rule R1)

Per traceability rule **R1** (see [README §4](../README.md#4-traceability-model-the-v)), **every
`STK-` is satisfied by ≥ 1 `FR`/`NFR`** in [system-requirements.md](system-requirements.md). No
stakeholder need is left orphaned. The satisfying requirement areas below are the inverse of the
`FR/NFR → STK` trace-up map and must stay consistent with it.

| STK | Satisfied by (requirement areas / IDs) |
|---|---|
| STK-01 | `FR-RIG-01..05`, `FR-RIG-07`, `FR-RIG-08`, `FR-RIG-10`, `FR-AUD-02`, `NFR-SEC-07` |
| STK-02 | `FR-RIG-06`, `FR-SPEC-01..04`, `FR-AUD-01`, `FR-AUD-03`, `FR-AUD-04`, `FR-AUD-05`, `FR-AUD-06`, `NFR-PERF-05` |
| STK-03 | `FR-RIG-09`, `FR-AUTH-01..05`, `NFR-SEC-08` |
| STK-04 | `FR-AUDIT-01..04` |
| STK-05 | `NFR-COMPAT-01..07` |
| STK-06 | `NFR-SEC-01..06`, `NFR-SEC-09`, `NFR-SEC-12` |
| STK-07 | `FR-HOST-01..04`, `NFR-SEC-13`, `NFR-SEC-14`, `NFR-SEC-15`, `NFR-DEPLOY-07`, `NFR-DEPLOY-08` |
| STK-08 | `NFR-PERF-03`, `NFR-PERF-04`, `NFR-REL-02`, `NFR-REL-03`, `NFR-DEPLOY-01`, `NFR-DEPLOY-02`, `NFR-DEPLOY-04` |
| STK-09 | `FR-GPIO-01`, `NFR-SEC-16` |
| STK-10 | `NFR-MAINT-01..03`, `NFR-SEC-10`, `NFR-SEC-11`, `NFR-DEPLOY-03`, `NFR-DEPLOY-05`, `NFR-DEPLOY-06` |
| STK-11 | `NFR-LIC-01`, `NFR-LIC-02` |
| STK-12 | `NFR-PERF-01`, `NFR-PERF-02`, `NFR-REL-01` |

## Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.6 | 2026-06-26 | DC0SK | Initial stakeholder-requirements baseline: STK-01..STK-12 with source SH, priority, rationale, and R1 satisfaction map. |
