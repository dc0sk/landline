# Container profile (evaluated)

An **evaluated** container deployment for the landline backend (NFR-DEPLOY-03,
ADR-06). The native [systemd unit](../systemd/landline.service) remains the
**reference** deployment; the container profile becomes *supported* only if it
passes the audio-latency and hardware-access thresholds below.

## Build & run

```sh
# From the repo root (the build context is the root):
docker build -f deploy/container/Dockerfile -t landline-backend:latest .

# Or with compose (place a 0600 config.toml next to compose.yml first):
cd deploy/container
chmod 600 config.toml   # the backend rejects a group/world-readable config (NFR-SEC-03)
docker compose up -d
```

## Security posture (NFR-SEC-10)

The [`compose.yml`](compose.yml) runs the container:

- **non-root** (`user: 10001:10001`),
- with a **read-only root filesystem** (`read_only: true`),
- **all capabilities dropped** (`cap_drop: [ALL]`) and **no privilege
  escalation** (`no-new-privileges`),
- writable state confined to a named volume (`/var/lib/landline`, the audit log)
  and a `tmpfs` (`/tmp`); the config is mounted read-only.

Publish the port **only on the private tunnel interface** (WireGuard/Tailscale),
never `0.0.0.0` (NFR-SEC-13). Rebuild the image within 7 days of upstream
OS/dependency security patches (NFR-SEC-11).

## Container evaluation — decision record (NFR-DEPLOY-03)

**Status: OPEN — pending hardware-in-the-loop evaluation on a Raspberry Pi 4/5.**

The profile is supported only if it meets these thresholds; each requires the
container running on real hardware:

| Criterion | Threshold | Method | Result |
|---|---|---|---|
| Audio latency (container vs native) | Within the `NFR-PERF-02` budget (< 300 ms) | TC-PERF-02, TC-DEPLOY-04 | *pending HIL* |
| Audio device access (ALSA/PipeWire passthrough) | RX/TX audio works in-container | TC-DEPLOY-04 | *pending HIL* |
| Serial/USB/GPIO access (if used by the rig interface) | rigctld/GPIO reachable | TC-DEPLOY-04 | *pending HIL* |
| Non-root + read-only rootfs | Enforced; service runs | TC-DEPLOY-05 | **met (compose)** |
| Secret injection | No secrets baked into image layers | TC-SEC-03, TC-DEPLOY-05 | **met (config mounted, not built in)** |

**Decision:** deferred. Record accept/defer here once the Pi HIL benchmark is
run (RISK-05: ALSA/PipeWire passthrough may break in-container — native remains
the reference either way).
