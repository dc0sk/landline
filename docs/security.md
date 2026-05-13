---
title: Security Model and Controls
project: landline
doc_type: security
license: AGPL-3.0-only
status: draft
version: 0.1.0
owner: ""
last_updated: 2026-05-13
---

# Security Model and Controls

License notice: This project is licensed under AGPL-3.0-only. See the top-level LICENSE file.

## 1. Purpose

This document defines threat model assumptions, trust boundaries, security controls, and release-gated security criteria.

## 2. Threat Model Scope

In scope:
- Unauthorized rig control attempts.
- Session/token abuse and replay.
- Malformed WebSocket payloads and protocol misuse.
- Excessive command rates and resource abuse.
- Misconfiguration risks in split-host deployments.
- GPIO misuse on Raspberry Pi (unsafe pin selection or startup states).

Out of scope for MVP:
- Physical access attacks on Raspberry Pi hardware.
- Nation-state level side-channel attacks.
- Full internet-exposed deployment without VPN/proxy hardening.

## 3. Trust Boundaries

- Browser client boundary: untrusted input source.
- Frontend host boundary: trusted deployment component, untrusted network path.
- Backend boundary: authority for authz, validation, rig and GPIO actions.
- Rig/hamlib boundary: external control interface requiring strict command validation.
- GPIO boundary: hardware control surface requiring allowlist and safe default states.
- Network boundary: TLS/WSS and private tunnel constraints for split-host deployment.

## 4. Deployment Security Modes

- LAN-only: backend and frontend on trusted local network.
- Split-host private network: frontend and backend on separate hosts over WireGuard/Tailscale.
- SSH fallback mode: operator-enabled temporary maintenance path only, disabled by default.

## 5. Control Baseline

- TLS-only communication for API and WebSocket channels.
- Short-lived session tokens and controlled refresh behavior.
- RBAC enforcement (Admin, Operator, Observer).
- Input validation and bounded payload/frame sizes.
- Rate limiting on control endpoints.
- Tamper-evident audit logs for state-changing operations.
- Split-host private interface binding by default.
- GPIO pin allowlist enforcement and safe startup states.

## 6. GPIO Security Policy

- Only configured allowlisted GPIO pins may be controlled.
- Non-allowlisted pins are inaccessible to API callers.
- Service startup initializes GPIO to configured safe states.
- GPIO actions are role-gated and auditable.

Mapped IDs:
- REQ-F-070
- REQ-S-016
- TST-F-070
- TST-S-015

## 7. Security Release Gates

A release is blocked if any applicable security test fails.

| Gate | Requirement IDs | Test IDs |
|---|---|---|
| Transport security | REQ-S-001 | TST-S-001 |
| Session and token safety | REQ-S-002, REQ-S-012 | TST-S-002, TST-S-010 |
| Command and payload hardening | REQ-S-004, REQ-S-005, REQ-S-008 | TST-S-004, TST-S-005, TST-S-008, TST-S-011 |
| Access control and safety controls | REQ-S-007, REQ-F-043 | TST-S-007, TST-F-043 |
| Split-host security posture | REQ-S-013, REQ-S-014, REQ-S-015 | TST-S-012, TST-S-013, TST-S-014 |
| GPIO safety controls | REQ-S-016 | TST-S-015 |

## 8. Secrets and Key Handling

- Secrets and private keys stored with file mode 0600 under service-owned account.
- No credentials in URLs or logs.
- Rotation policy defined before production release.
- No secrets embedded in container images.

## 9. Open Security Documentation TODOs

- Add incident response workflow and severity model.
- Add key/token rotation runbook with operational cadence.
- Add secure default configuration examples per deployment profile.
- Add security exceptions register template.

## 10. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.1.0 | 2026-05-13 | - | Initial security model and controls document |
