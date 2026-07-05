# Operations runbook

Day-to-day operation of the landline backend on the native systemd deployment
(BL-105). Assumes the install from [`systemd/landline.service`](systemd/landline.service).

## Service control

```sh
systemctl status  landline      # health / last logs
systemctl start   landline
systemctl stop    landline      # SIGTERM -> graceful shutdown
systemctl restart landline      # also rotates the in-memory JWT signing secret
journalctl -u landline -f       # follow logs (RUST_LOG controls verbosity)
```

Health probes (no auth): `GET /healthz` → `{"status":"ok"}`, `GET /version`.

## Update

```sh
sudo systemctl stop landline
sudo cp /usr/local/bin/landline /usr/local/bin/landline.prev   # keep the last-good binary
sudo install -Dm755 <new-binary> /usr/local/bin/landline
sudo systemctl start landline
systemctl status landline && curl -fsS http://127.0.0.1:8443/healthz
```

## Rollback (BL-082, NFR-DEPLOY-05, TC-DEPLOY-06)

The audit log (`/var/lib/landline/`) and `config.toml` are independent of the
binary, so a binary rollback loses no data.

```sh
sudo systemctl stop landline
sudo install -Dm755 /usr/local/bin/landline.prev /usr/local/bin/landline
sudo systemctl start landline
# Verify: service active, health OK, version is the previous one.
systemctl is-active landline && curl -fsS http://127.0.0.1:8443/version
```

If a config change caused the failure, restore the previous `config.toml`
(keep a `config.toml.prev` alongside updates) and restart. The audit log is
append-only and hash-chained, so no rollback can silently drop history —
verify chain integrity after any incident.

## Secret / token rotation

Follow [security.md §8.2](../docs/security.md). Quick reference:

- **Force all sessions to re-auth:** `systemctl restart landline` (rotates the
  in-memory JWT signing secret).
- **Rotate a user password:** regenerate the argon2 hash
  (`landline_backend::auth::hash_password`), update `config.toml` (0600), restart.
- **Rotate TLS:** renew the cert, replace key+chain (0600), `nginx -t &&
  systemctl reload nginx`.

## Logs & audit

- Service logs: `journalctl -u landline` (structured; never contain secrets).
- Audit log: `/var/lib/landline/audit.log` (append-only, hash-chained), or via
  the Admin-only `GET /api/audit`. 30-day retention is enforced by log rotation.

## Common issues

| Symptom | Likely cause | Action |
|---|---|---|
| Service won't start, "insecure permissions on …config.toml" | config not 0600 | `chmod 600 /etc/landline/config.toml` |
| `502 rig unavailable` on control endpoints | rigctld down/unreachable | check rigctld; the circuit breaker retries automatically |
| `503 rig unavailable` | circuit breaker open (repeated failures) | wait out the cooldown; fix rigctld |
| Clients rejected right after restart | JWT secret rotated on restart | expected — clients re-authenticate |
