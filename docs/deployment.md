---
title: Deployment Profiles and Decision Record
project: landline
doc_type: deployment-decision
license: AGPL-3.0-only
status: draft
version: 0.3.0
owner: ""
last_updated: 2026-05-13
---

# Deployment Profiles and Decision Record

License notice: This project is licensed under AGPL-3.0-only. See the top-level LICENSE file.

## 1. Purpose

This document defines split-host deployment profiles for running frontend and backend on different machines with secure connectivity.

This file provides:
- A decision framework for profile selection.
- Profile-specific setup and hardening steps.
- Verification criteria mapped to requirements and tests.

## 2. Scope and Traceability

### Requirements covered

- REQ-F-060, REQ-F-061, REQ-F-062, REQ-F-063
- REQ-S-013, REQ-S-014, REQ-S-015
- REQ-D-007, REQ-D-008
- REQ-L-002

### Tests covered

- TST-F-060, TST-F-061, TST-F-062
- TST-S-012, TST-S-013, TST-S-014
- TST-D-007
- TST-L-002

## 3. Decision Summary

| Profile | Transport | Trust Model | Operational Cost | Recommended Use |
|---|---|---|---|---|
| A | WireGuard | Self-hosted peer keys | Medium | Default reference profile |
| B | Tailscale | WireGuard + identity/ACL control plane | Low | Operator-friendly alternative |
| C | SSH tunnel | SSH key trust + manual ops | Medium/High | Fallback only, non-default |

Current decision status:
- Primary: Profile A (WireGuard).
- Secondary: Profile B (Tailscale).
- Fallback only: Profile C (SSH tunnel).

## 4. Common Split-Host Baseline

Applies to all profiles:
- Backend host (Pi) runs API/WSS service.
- Frontend host serves static frontend assets and connects to backend over secure channel.
- Frontend endpoint base URLs are runtime-configurable (REQ-F-063).
- Backend does not bind public interfaces by default in split-host mode (REQ-S-013).
- Mutual peer identity is required (REQ-S-014).
- No credentials in URL query strings or logs.

Required runtime parameters:
- LANDLINE_API_BASE_URL
- LANDLINE_WSS_BASE_URL
- LANDLINE_ALLOWED_ORIGINS
- LANDLINE_BIND_ADDR

## 5. Profile A - WireGuard (Primary)

### Topology

- Backend host: WireGuard interface wg0 with static private tunnel IP.
- Frontend host: WireGuard peer with static private tunnel IP.
- Frontend connects only to backend wg0 address for HTTPS/WSS.

### Security controls

- Unique private/public key pair per peer.
- Strict AllowedIPs for each peer.
- Backend service binds to wg0 tunnel IP only.
- Host firewall allows API/WSS from tunnel CIDR only.
- Optional mTLS on top of WireGuard for service-level identity.

### Setup checklist

1. Generate peer keys on both hosts.
2. Configure wg0 on backend and frontend hosts.
3. Set AllowedIPs minimally (peer-only or exact subnets).
4. Enable forwarding only if explicitly needed.
5. Configure backend bind address to tunnel IP.
6. Configure frontend runtime API/WSS base URLs to tunnel endpoint.
7. Restrict firewall ingress to WireGuard interface and service ports.

### Command templates

Backend host (Pi): install and key generation

```bash
sudo apt update
sudo apt install -y wireguard
umask 077
wg genkey | tee /etc/wireguard/backend.key | wg pubkey > /etc/wireguard/backend.pub
```

Frontend host: install and key generation

```bash
sudo apt update
sudo apt install -y wireguard
umask 077
wg genkey | tee /etc/wireguard/frontend.key | wg pubkey > /etc/wireguard/frontend.pub
```

Backend config template (`/etc/wireguard/wg0.conf`)

```ini
[Interface]
Address = 10.20.30.1/24
ListenPort = 51820
PrivateKey = <BACKEND_PRIVATE_KEY>

[Peer]
PublicKey = <FRONTEND_PUBLIC_KEY>
AllowedIPs = 10.20.30.2/32
PersistentKeepalive = 25
```

Frontend config template (`/etc/wireguard/wg0.conf`)

```ini
[Interface]
Address = 10.20.30.2/24
PrivateKey = <FRONTEND_PRIVATE_KEY>

[Peer]
PublicKey = <BACKEND_PUBLIC_KEY>
Endpoint = <BACKEND_PUBLIC_IP_OR_DNS>:51820
AllowedIPs = 10.20.30.1/32
PersistentKeepalive = 25
```

Bring tunnel up and enable at boot

```bash
sudo systemctl enable --now wg-quick@wg0
sudo wg show
```

Restrict API/WSS ingress to tunnel interface (example)

```bash
sudo ufw allow in on wg0 to any port 443 proto tcp
sudo ufw deny 443/tcp
```

Runtime endpoint template (frontend host)

```bash
export LANDLINE_API_BASE_URL="https://10.20.30.1"
export LANDLINE_WSS_BASE_URL="wss://10.20.30.1/ws"
```

### Hardening checklist

- [ ] Key rotation interval defined and documented.
- [ ] Unused peers removed.
- [ ] Keepalive configured only where NAT traversal requires it.
- [ ] Service ports closed on non-tunnel interfaces.
- [ ] Audit logs include source tunnel IP.

### Verification

- TST-F-060: split-host control and telemetry works.
- TST-F-061: runtime endpoint switch works without rebuild.
- TST-F-062: traffic flows through WireGuard tunnel.
- TST-S-012: backend not exposed on public bind.
- TST-S-013: unknown peer cannot connect.
- TST-D-007: runbook execution succeeds.

## 6. Profile B - Tailscale (Alternative)

### Topology

- Backend and frontend hosts join same tailnet.
- Frontend targets backend Tailscale IP or MagicDNS name.
- ACLs restrict frontend-backend communication to required ports only.

### Security controls

- Device identity and ACL policy enforced in tailnet.
- Tagged devices for role separation (frontend-host, backend-host).
- Funnel/exit-node/public exposure disabled by default.
- Backend service binds to tailscale0 or loopback + userspace proxy model.

### Setup checklist

1. Enroll both hosts in tailnet.
2. Apply ACL policy for least privilege traffic.
3. Disable features not needed for this deployment.
4. Configure backend bind and allowed origins for tailnet endpoint.
5. Configure frontend runtime API/WSS base URLs to tailnet endpoint.

### Command templates

Install and join tailnet (both hosts)

```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up --ssh=false
tailscale status
```

Tag hosts (example)

```bash
sudo tailscale set --advertise-tags=tag:backend-host
# On frontend host:
sudo tailscale set --advertise-tags=tag:frontend-host
```

ACL template (tailnet policy)

```json
{
	"tagOwners": {
		"tag:backend-host": ["autogroup:admin"],
		"tag:frontend-host": ["autogroup:admin"]
	},
	"acls": [
		{
			"action": "accept",
			"src": ["tag:frontend-host"],
			"dst": ["tag:backend-host:443"]
		}
	],
	"ssh": []
}
```

Disable public exposure features explicitly (example)

```bash
sudo tailscale set --advertise-exit-node=false --accept-routes=false
```

Runtime endpoint template (frontend host)

```bash
export LANDLINE_API_BASE_URL="https://backend-host.tailnet-name.ts.net"
export LANDLINE_WSS_BASE_URL="wss://backend-host.tailnet-name.ts.net/ws"
```

### Hardening checklist

- [ ] ACL policy reviewed and version-controlled.
- [ ] Device approval flow enabled.
- [ ] Expired devices removed from tailnet.
- [ ] Service ports inaccessible from non-authorized tailnet nodes.
- [ ] Tailnet auth events retained for audit.

### Verification

- TST-F-060: split-host control and telemetry works.
- TST-F-061: runtime endpoint switch works without rebuild.
- TST-F-062: traffic flows through Tailscale (WireGuard-based) path.
- TST-S-012: backend not exposed on public bind.
- TST-S-013: unauthorized identity denied by ACL.
- TST-D-007: runbook execution succeeds.

## 7. Profile C - SSH Tunnel (Fallback Only)

This profile is not production-default (REQ-S-015, REQ-D-008).

### Topology

- SSH local forward or reverse tunnel between frontend and backend hosts.
- Frontend points API/WSS to local forwarded endpoint.

### Security controls

- Key-only auth, password login disabled.
- Dedicated restricted SSH user for tunnel process.
- ForceCommand or PermitOpen restrictions.
- Optional jump host with explicit allowlist.

### Setup checklist

1. Create dedicated tunnel user and SSH key pair.
2. Restrict sshd config for tunnel user.
3. Define explicit local/remote forward ports.
4. Use autossh or supervised unit for tunnel health.
5. Configure frontend runtime API/WSS base URLs to forwarded endpoint.

### Command templates

Backend host: create restricted tunnel user

```bash
sudo useradd --create-home --shell /usr/sbin/nologin tunnel
sudo mkdir -p /home/tunnel/.ssh
sudo chmod 700 /home/tunnel/.ssh
```

Frontend host: create dedicated key pair

```bash
ssh-keygen -t ed25519 -f ~/.ssh/landline_tunnel -C "landline-tunnel"
ssh-copy-id -i ~/.ssh/landline_tunnel.pub tunnel@<BACKEND_HOST>
```

Backend sshd hardening snippet (`/etc/ssh/sshd_config.d/landline-tunnel.conf`)

```conf
Match User tunnel
		PasswordAuthentication no
		PermitTTY no
		X11Forwarding no
		AllowTcpForwarding yes
		PermitOpen 127.0.0.1:443
		ForceCommand /usr/sbin/nologin
```

Apply sshd config

```bash
sudo sshd -t && sudo systemctl reload ssh
```

Tunnel command template (frontend host)

```bash
autossh -M 0 -N \
	-i ~/.ssh/landline_tunnel \
	-o ServerAliveInterval=30 \
	-o ServerAliveCountMax=3 \
	-L 127.0.0.1:8443:127.0.0.1:443 \
	tunnel@<BACKEND_HOST>
```

Runtime endpoint template (frontend host)

```bash
export LANDLINE_API_BASE_URL="https://127.0.0.1:8443"
export LANDLINE_WSS_BASE_URL="wss://127.0.0.1:8443/ws"
```

### Hardening checklist

- [ ] Password authentication disabled.
- [ ] Root login disabled.
- [ ] PermitOpen limited to required host:port pairs.
- [ ] Tunnel user has no shell access.
- [ ] Tunnel restart and failure alerts configured.

### Verification

- TST-F-060: split-host control and telemetry works through tunnel.
- TST-F-061: runtime endpoint switch works without rebuild.
- TST-S-014: fallback mode documented and disabled by default.
- TST-D-007: fallback runbook executes successfully when enabled.

## 8. Selection Guidance

Choose profile by environment:
- Use Profile A (WireGuard) when you want full self-hosted control and predictable networking.
- Use Profile B (Tailscale) when operator simplicity and identity-based ACL management are priority.
- Use Profile C (SSH tunnel) only for temporary recovery/maintenance paths.

## 9. Validation Command Checklist

Use this section to execute repeatable checks for split-host tests.

### TST-F-060 - Split-host control and telemetry

Frontend host:

```bash
curl -sS "$LANDLINE_API_BASE_URL/health"
```

Expected result: HTTP 200-equivalent response from backend through selected profile.

### TST-F-061 - Runtime endpoint switch without rebuild

Frontend host:

```bash
export LANDLINE_API_BASE_URL="https://<NEW_ENDPOINT>"
export LANDLINE_WSS_BASE_URL="wss://<NEW_ENDPOINT>/ws"
curl -sS "$LANDLINE_API_BASE_URL/health"
```

Expected result: frontend uses new endpoint after restart/reload with no frontend rebuild.

### TST-F-062 - Transport path validation

WireGuard profile:

```bash
sudo wg show
ss -tnp | grep ':443\|:51820'
```

Tailscale profile:

```bash
tailscale status
tailscale ping <BACKEND_TAILNET_NAME_OR_IP>
```

Expected result: active session and successful connectivity over selected private transport.

### TST-S-012 - Backend bind exposure check

Backend host:

```bash
ss -ltnp | grep ':443'
ip -br a
```

Expected result: API/WSS listener bound only to tunnel interface or loopback strategy, not public wildcard bind.

### TST-S-013 - Unauthorized peer/identity denial

WireGuard profile (untrusted source host):

```bash
curl -vk --connect-timeout 5 https://<BACKEND_TUNNEL_IP>/health
```

Tailscale profile (non-authorized tailnet node):

```bash
curl -vk --connect-timeout 5 https://<BACKEND_TAILNET_NAME_OR_IP>/health
```

Expected result: access denied or unreachable from unauthorized peer/identity.

### TST-S-014 - SSH fallback disabled by default

Frontend host:

```bash
pgrep -af autossh || true
systemctl --user status landline-ssh-tunnel.service || true
```

Expected result: no active SSH fallback tunnel unless explicitly enabled for maintenance.

### TST-D-007 - Runbook execution

Operator checklist:
- Bring selected profile up.
- Confirm health endpoint and websocket path are reachable.
- Capture evidence outputs from checks above.

Expected result: profile setup and verification complete with captured evidence.

## 10. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.3.0 | 2026-05-13 | - | Added test-oriented validation command checklist for split-host profiles |
| 0.2.0 | 2026-05-13 | - | Added concrete command/config templates for WireGuard, Tailscale ACLs, and SSH fallback |
| 0.1.0 | 2026-05-13 | - | Initial deployment decision document with WireGuard, Tailscale, and SSH profiles |
