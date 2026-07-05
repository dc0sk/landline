# Split-host deployment profile

Run the **frontend from a machine separate from the backend host** (FR-HOST-01),
with the backend reachable over a **private tunnel — no public internet
exposure** (FR-HOST-02). Primary transport is **WireGuard** (or **Tailscale**);
**SSH is a documented fallback only** (ADR-05, NFR-DEPLOY-07/08, NFR-SEC-13/14/15).

```
   Frontend host                     private tunnel                Backend host (Pi)
 ┌────────────────┐   WireGuard / Tailscale (mutually       ┌──────────────────────┐
 │ static frontend│     authenticated, encrypted)           │ landline backend      │
 │ (index.html +  │◄═══════════ 10.10.0.0/24 ══════════════►│ binds 10.10.0.1:8443  │
 │  dist/, served │                                         │ (tunnel iface only,   │
 │  by any HTTP)  │   API + WSS to 10.10.0.1:8443            │  never 0.0.0.0)       │
 └────────────────┘                                         └──────────────────────┘
```

## 1. Backend: bind to the tunnel interface only (NFR-SEC-13)

In `config.toml`, set the server to the backend's **tunnel** address — never a
public `0.0.0.0` bind:

```toml
[server]
bind = "10.10.0.1"   # the backend's WireGuard/Tailscale address
port = 8443
```

The default is loopback; this is the one place you widen it, and only to the
private interface. (TC-SEC-12.)

## 2. Primary: WireGuard (FR-HOST-03, NFR-SEC-14)

Mutual authentication is by peer keys — each side lists only the other's public
key, so the link is encrypted and mutually authenticated by construction.

Templates: [`wireguard/backend-host.conf.example`](wireguard/backend-host.conf.example)
and [`wireguard/frontend-host.conf.example`](wireguard/frontend-host.conf.example).

```sh
# On each host: generate a keypair
wg genkey | tee privatekey | wg pubkey > publickey

# Fill each peer's [Peer] PublicKey with the OTHER host's publickey, then:
sudo cp backend-host.conf   /etc/wireguard/wg0.conf   # (on the backend host)
sudo cp frontend-host.conf  /etc/wireguard/wg0.conf   # (on the frontend host)
sudo systemctl enable --now wg-quick@wg0
```

Verify: `wg show` lists the peer with a recent handshake; the backend
(`10.10.0.1:8443`) answers **only** over the tunnel. (TC-HOST-01/03, TC-SEC-13,
TC-DEPLOY-07.)

## 3. Alternative: Tailscale (FR-HOST-03, NFR-SEC-14)

Tailscale is WireGuard-based with identity-based ACLs:

```sh
sudo tailscale up            # on both hosts
```

Restrict access with an ACL so only the frontend host can reach the backend's
landline port, e.g.:

```json
{
  "acls": [
    { "action": "accept", "src": ["tag:landline-frontend"], "dst": ["tag:landline-backend:8443"] }
  ]
}
```

Bind the backend to its Tailscale address and set `LANDLINE_API_BASE` to the
backend's `*.ts.net` name.

## 4. Fallback only: SSH tunnel (NFR-SEC-15, NFR-DEPLOY-08)

Use **only** when WireGuard/Tailscale is unavailable, and never as the default
production profile:

```sh
# From the frontend host, forward local 8443 to the backend's loopback:
ssh -N -L 8443:127.0.0.1:8443 operator@backend-host
```

Then point the frontend at `http://127.0.0.1:8443`. This keeps the backend on
loopback, but SSH is operator-maintained and out of scope for the hardened
production path. (TC-SEC-14.)

## 5. Frontend: target the backend without code changes (FR-HOST-04)

The frontend reads a runtime base URL — set `window.LANDLINE_API_BASE` before
`dist/main.js` loads (e.g. inject a small `<script>` in the served `index.html`):

```html
<script>window.LANDLINE_API_BASE = "http://10.10.0.1:8443";</script>
```

The client derives the WSS URL from it automatically (`http`→`ws`). No rebuild
is needed to retarget a different backend. (TC-HOST-02; implemented in
`frontend/src/main.ts`.)

## Verification checklist

- [ ] `wg show` (or `tailscale status`) shows a live, mutually-authenticated peer.
- [ ] Backend answers on the tunnel address and **fails** from any public
      interface (`curl` from off-tunnel is refused). (TC-SEC-12/13.)
- [ ] Frontend host loads the UI and reaches API + WSS over the tunnel. (TC-HOST-01/02.)
- [ ] SSH profile documented as fallback only; not the default. (TC-SEC-14.)
