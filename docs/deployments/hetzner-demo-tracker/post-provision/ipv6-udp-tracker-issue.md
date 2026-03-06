# IPv6 UDP Tracker — Known Issue

> **Status**: 🔴 Unresolved — newTrackon submission rejected with "UDP timeout" on IPv6 probe

## Context

During issue #407 (submitting the UDP1 tracker to newTrackon), the tracker was rejected with a
"UDP timeout" error. The newTrackon probe used the AAAA record
(`2a01:4f8:1c0c:828e::1`) to reach the tracker via IPv6. IPv4 probes (tested locally) work fine.

This document records the investigation, likely root cause, and the fix required.

## Symptom

- `udp://udp1.torrust-tracker-demo.com:6969/announce` submitted to newTrackon
- newTrackon probed via IPv6: `2a01:4f8:1c0c:828e::1`
- Result: ❌ **Rejected — UDP timeout**
- Local test via IPv4 (`116.202.177.184`): ✅ Works

## What Was Ruled Out

| Hypothesis                   | Evidence                                                        | Verdict      |
| ---------------------------- | --------------------------------------------------------------- | ------------ |
| Asymmetric routing           | Must rule out — see below                                       | 🔍 Possible  |
| Wrong IP in DNS              | `dig AAAA` returns `2a01:4f8:1c0c:828e::1` ✅                   | ❌ Ruled out |
| Floating IP not on interface | `ip addr show eth0` shows all four IPs with `valid_lft forever` | ❌ Ruled out |
| BEP 34 TXT record missing    | `dig TXT udp1.torrust-tracker-demo.com` returns correct value   | ❌ Ruled out |
| Caddy proxy intercepting UDP | UDP tracker bypasses reverse proxy entirely                     | ❌ Ruled out |

## Most Likely Root Cause — Docker IPv6 Disabled

By default, Docker does **not** enable IPv6. When Docker is started without IPv6 support, the
tracker container binds to `0.0.0.0:6969` (IPv4 only). UDP packets arriving on the IPv6 floating
IP are received by the kernel but never forwarded to the container — they are silently dropped.

This would also explain why:

- IPv4 tests (local machine, clients) work fine — the container is reachable on `0.0.0.0:6969`
- IPv6 probes (newTrackon) fail — the kernel has no listener to forward them to

### Verification Steps

SSH into the server and run:

```bash
# 1. Check what the tracker is actually listening on
sudo ss -ulnp | grep 6969
# Expected if broken: 0.0.0.0:6969  (IPv4 only, no :::6969)
# Expected if working: 0.0.0.0:6969 AND :::6969

# 2. Check if Docker IPv6 is enabled in the daemon
cat /etc/docker/daemon.json

# 3. Check if the Docker network has IPv6
docker network inspect bridge | grep -A5 EnableIPv6

# 4. Check running container port bindings
docker compose ps
```

## Alternative Root Cause — Asymmetric Routing

Even if Docker is IPv6-enabled, the kernel may route reply packets via the wrong interface.
When a UDP probe arrives on `2a01:4f8:1c0c:828e::1`, the reply could leave via the primary
interface IP (`2a01:4f8:1c19:620b::1`) rather than the floating IP. The remote host (newTrackon)
discards packets with an unexpected source IP.

This requires **policy-based routing**: a separate routing table per floating IP that forces
replies to use the correct source.

## Historical Context

### Old Demo Tracker (torrust-demo.com, Digital Ocean)

The previous Torrust demo tracker was deployed on Digital Ocean with a reserved IPv4
(`144.126.245.19`). That deployment only served **IPv4** — no IPv6 floating IPs were configured.
This means the asymmetric routing / IPv6 Docker issue was never encountered.

### This Deployment (torrust-tracker-demo.com, Hetzner)

This is the **first Torrust deployment routing UDP tracker traffic over IPv6 floating IPs**.
The combination of:

1. Multiple floating IPs (both IPv4 and IPv6)
2. Docker with default network settings
3. UDP tracker on port 6969

…is new territory. Either or both root causes above may apply.

### Proxy Difference (Nginx vs Caddy)

The old demo used Nginx as a reverse proxy; this deployment uses Caddy. This is **irrelevant
for UDP tracker traffic** — UDP does not go through the reverse proxy (HTTP only). Both
setups are equivalent from the UDP tracker's perspective.

## Required Fix

### Fix 1 — Enable Docker IPv6 (likely required)

On the server:

```bash
# Check current daemon.json
cat /etc/docker/daemon.json
```

If IPv6 is not enabled, add it:

```json
{
  "ipv6": true,
  "fixed-cidr-v6": "fd00::/80"
}
```

Then restart Docker and re-check:

```bash
sudo systemctl restart docker
sudo ss -ulnp | grep 6969
```

After this, the tracker should show `:::6969` in the `ss` output.

### Fix 2 — Policy-Based Routing (may also be required)

If replies still go via the wrong source IP:

```bash
# Get the default IPv6 gateway
ip -6 route show default

# Add a routing table for the UDP1 floating IPv6
ip -6 route add default via <ipv6_gateway> dev eth0 table 200
ip -6 rule add from 2a01:4f8:1c0c:828e::1 table 200

# Make persistent via netplan or /etc/networkd-dispatcher
```

## Impact

This issue blocks the UDP1 tracker from being accepted by newTrackon. It does **not** affect:

- HTTP tracker functionality (goes through Caddy → Docker on IPv4)
- IPv4 UDP tracker functionality
- Existing HTTP1 tracker newTrackon listing

## Cross-Repository Note

This issue should also be documented in the
[torrust-tracker](https://github.com/torrust/torrust-tracker) repository, as it involves
the tracker's network configuration requirements when running behind Docker with IPv6 floating
IPs. Any future deployment guide covering IPv6 should mention:

1. Docker daemon needs `"ipv6": true` in `daemon.json`
2. Policy-based routing may be needed for multiple IPv6 floating IPs

## Related

- [Issue #407 — Submit UDP1 Tracker to newTrackon](../../../issues/407-submit-udp1-tracker-to-newtrackon.md)
- [newTrackon Prerequisites](newtrackon-prerequisites.md)
- [Netplan Configuration](newtrackon-prerequisites.md#step-3--configure-all-floating-ips-permanently-via-netplan-)
