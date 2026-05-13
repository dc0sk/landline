---
title: Project Governance Charter
project: landline
doc_type: governance
license: AGPL-3.0-only
status: draft
version: 0.1.0
owner: ""
last_updated: 2026-05-13
---

# Project Governance Charter

License notice: This project is licensed under AGPL-3.0-only. See the top-level LICENSE file.

## 1. Purpose

This charter defines mandatory governance rules for project planning, implementation, and release decisions.

## 2. Governance Principles

- Security-first: security controls and security tests are release blockers, never optional polish.
- Documentation-first: requirements, tests, backlog, roadmap, and governance docs are required release artifacts.
- License-first: AGPL obligations are tracked as release criteria.
- Traceability-first: every requirement must map to at least one test before release.

## 3. Security-First Policy

Security-first means:
- Feature work is not considered complete without passing required security tests.
- Security regressions block merges unless explicitly accepted with documented risk owner and expiry.
- Default deployment posture is least exposure and least privilege.
- Secrets, auth controls, transport encryption, and auditability are treated as core functionality.

## 4. Phase Gate Rules

- Phase exit is denied if scoped security tests are failing or missing.
- Must-priority security backlog items cannot be deferred without documented decision and replacement mitigation.
- Security exceptions require:
  - Issue ID
  - Risk owner
  - Compensating controls
  - Expiration date

## 5. Change Control Rules

Any PR that changes scope, architecture, security posture, deployment model, or release criteria must update, in the same change set:
- docs/requirements-spec.md
- docs/test-spec.md
- docs/backlog.md
- docs/roadmap.md
- docs/governance.md when governance policy is impacted

## 6. Ownership and Review

- At least one owner is accountable for governance updates.
- Security-impacting changes require explicit reviewer sign-off for security implications and test coverage.

## 7. Compliance Checklist

Before release:
- [ ] All scoped security tests are Pass.
- [ ] No open Must security backlog items.
- [ ] Requirement-to-test traceability is complete for Must/Should scope.
- [ ] AGPL license artifacts are present and accurate.
- [ ] Open exceptions (if any) are valid and unexpired.

## 8. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.1.0 | 2026-05-13 | - | Initial governance charter with security-first policy |
