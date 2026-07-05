# Deployment

Native **systemd** service is the reference deployment for the landline backend
(ADR-06, NFR-DEPLOY-02). A **container profile** is evaluated in
[`container/`](container/) (ADR-06, NFR-DEPLOY-03), a **split-host profile** is in
[`split-host/`](split-host/) (ADR-05, NFR-DEPLOY-07/08), and production TLS /
reverse proxy is Phase 4 (BL-100–101).

| Profile | Where | Status |
|---|---|---|
| Native (systemd) — reference | [`systemd/landline.service`](systemd/landline.service) | supported |
| Container (evaluated) | [`container/`](container/) | evaluated; supported pending Pi HIL |
| Split-host (WireGuard/Tailscale) | [`split-host/`](split-host/) | documented profile |
| TLS reverse proxy (nginx) | [`nginx/`](nginx/) | production TLS/WSS (NFR-SEC-01) |

**Operations:** [`RUNBOOK.md`](RUNBOOK.md) (start/stop, update, rollback, rotation,
logs) and the release gate in [`../docs/release-checklist.md`](../docs/release-checklist.md).

> **Config permissions (NFR-SEC-03):** the backend **rejects a `config.toml` that
> is group- or world-accessible** — install it `0600`, owned by the service user.

## Native (systemd)

The hardened unit is [`systemd/landline.service`](systemd/landline.service); its
header documents the install steps. In short:

1. Cross-compile the aarch64 release binary (see the root `README.md`).
2. Create a dedicated `landline` service user.
3. Install the binary to `/usr/local/bin/landline` and a config to
   `/etc/landline/config.toml` (start from
   [`../backend/config.example.toml`](../backend/config.example.toml)).
4. Install and enable the unit.

The unit runs unprivileged with an empty capability set, a read-only root
filesystem, a syscall allowlist, and a private `/var/lib/landline` for the audit
log. The service binds to loopback by default (NFR-SEC-13) — put TLS/WSS
termination (reverse proxy) in front of it, or reach it over a WireGuard /
Tailscale tunnel for split-host operation (ADR-05).

**GPIO note:** GPIO is disabled by default and the unit blocks device access
(`PrivateDevices=true`). On a Raspberry Pi that needs GPIO, set
`PrivateDevices=false` and add a specific `DeviceAllow=` for the gpiochip device.

## Frontend

The [`../frontend`](../frontend) build (`npm run build` → `dist/`) plus
`index.html` and `styles.css` are static assets. Serve them from the same host,
or from a separate frontend host reaching the backend over the private tunnel
(split-host, ADR-05). Set `window.LANDLINE_API_BASE` for a non-same-origin
backend.
