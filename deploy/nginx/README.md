# TLS reverse proxy (nginx)

Production TLS/WSS termination for landline (BL-100/101, NFR-SEC-01). The backend
binds loopback and speaks plaintext HTTP/WS on `127.0.0.1:8443`; **nginx is its
only public face** and terminates TLS, so no cleartext ever leaves the host.

## Install

1. Obtain a certificate (Let's Encrypt, or an internal CA) and place it at
   `/etc/landline/tls/fullchain.pem` + `privkey.pem`. The **private key must be
   `0600`** and owned by the nginx user (NFR-SEC-03).
2. Build the frontend (`cd frontend && npm run build`) and copy `index.html`,
   `styles.css`, and `dist/` to `/var/www/landline`.
3. Install [`nginx.conf`](nginx.conf) as a site (edit `server_name`), then
   `nginx -t && systemctl reload nginx`.

## What it enforces

- **HTTPS only (NFR-SEC-01):** `:80` is `308`-redirected to `:443`; the backend
  is unreachable over cleartext. (TC-SEC-01.)
- **Modern TLS:** TLS 1.2/1.3 only, GCM ciphers, HSTS.
- **WSS upgrade:** `/ws` forwards the `Upgrade`/`Connection` headers so the
  authenticated WebSocket telemetry works end-to-end over `wss://`.
- **`X-Forwarded-For`:** passes the real client IP so the backend rate-limiter
  can key on it behind the proxy (the BL-022 Phase-4 follow-up).

For split-host, run this proxy on the frontend host and `proxy_pass` to the
backend's tunnel address instead of loopback (see [`../split-host`](../split-host)).
