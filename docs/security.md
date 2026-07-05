---
title: Security Model and Controls
status: Draft
version: 0.2.0
updated: 2026-07-05
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

### 8.1 Storage (NFR-SEC-03)

- Secrets and private keys are stored in files with mode **0600**, owned by the
  service user. The backend **fails closed** on a group/world-accessible
  `config.toml` (see `config::Config::load`), and the deployment installs it 0600.
- No credentials appear in URLs or logs (NFR-SEC-12): the audit log stores only
  the username/action/params (never passwords), the WS auth token travels in the
  message body rather than the query string, and tracing never emits secrets.
- No secrets are embedded in container image layers — the config is mounted at
  runtime, not built in (NFR-SEC-10).

### 8.2 Rotation policy (NFR-SEC-03, TC-SEC-03) — BL-012

| Secret | Where | Cadence | Procedure |
|---|---|---|---|
| **JWT signing secret** | In-memory, generated per process start (256-bit CSPRNG) | Every restart; force with a rolling restart | Restart the service; all existing access tokens become invalid and clients re-authenticate. When a persisted signing key is later introduced, rotate it on the same cadence as the TLS key and restart. |
| **User password hashes** | `config.toml` (argon2) | On compromise, on operator change, and at least every 180 days | Regenerate with `landline_backend::auth::hash_password`, update `config.toml` (0600), reload. |
| **TLS private key** | `/etc/landline/tls/privkey.pem` (0600, nginx) | Per certificate lifetime (≤ 90 days with Let's Encrypt) | Renew the certificate, replace key+chain (0600), `nginx -t && systemctl reload nginx`. |
| **WireGuard/Tailscale keys** | Tunnel host config (0600) | On host decommission or compromise; annual review | Regenerate the keypair, update both peers, `systemctl restart wg-quick@wg0`. |

**Triggers for immediate rotation (any secret):** suspected compromise, operator
departure, or a lost/retired device. Rotations are recorded in the operations
log; a rotation is verified by confirming old material no longer authenticates.

## 9. Open Security Documentation TODOs

- Add incident response workflow and severity model.
- Add security exceptions register template.
- Add secure default configuration examples per deployment profile (partially
  covered by `backend/config.example.toml` and the `deploy/` profiles).

## 10. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.2.0 | 2026-07-05 | DC0SK | Added §8.2 secrets rotation policy (BL-012, closing the last Phase-0 remainder); expanded §8 storage with the enforced 0600 config check and NFR-SEC-10/12 notes. |
| 0.1.1 | 2026-06-26 | DC0SK | Migrated to area-coded FR/NFR/TC ids and new doc-tree frontmatter. |
| 0.1.0 | 2026-05-13 | - | Initial security model and controls document |
