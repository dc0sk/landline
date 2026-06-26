---
title: Security Model and Controls
status: Draft
version: 0.1.1
updated: 2026-06-26
authors:
  - Simon Keimer (DC0SK)
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
- FR-GPIO-01
- NFR-SEC-16
- TC-GPIO-01
- TC-SEC-15

## 7. Security Release Gates

A release is blocked if any applicable security test fails.

| Gate | Requirement IDs | Test IDs |
|---|---|---|
| Transport security | NFR-SEC-01 | TC-SEC-01 |
| Session and token safety | NFR-SEC-02, NFR-SEC-12 | TC-SEC-02, TC-SEC-10 |
| Command and payload hardening | NFR-SEC-04, NFR-SEC-05, NFR-SEC-08 | TC-SEC-04, TC-SEC-05, TC-SEC-08, TC-SEC-11 |
| Access control and safety controls | NFR-SEC-07, FR-AUTH-04 | TC-SEC-07, TC-AUTH-04 |
| Split-host security posture | NFR-SEC-13, NFR-SEC-14, NFR-SEC-15 | TC-SEC-12, TC-SEC-13, TC-SEC-14 |
| GPIO safety controls | NFR-SEC-16 | TC-SEC-15 |

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
| 0.1.1 | 2026-06-26 | DC0SK | Migrated to area-coded FR/NFR/TC ids and new doc-tree frontmatter. |
| 0.1.0 | 2026-05-13 | - | Initial security model and controls document |
