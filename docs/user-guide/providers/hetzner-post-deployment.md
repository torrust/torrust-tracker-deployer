# Hetzner Post-Deployment: Floating IPs and IPv6

This guide documents the manual steps required **after** running the deployer when the server uses
**Hetzner floating IPs** and/or needs **IPv6 UDP tracker support**.

These steps are not planned to be automated by the deployer. They are specific to multi-IP setups
where separate floating IPs are used for separate tracker endpoints (e.g. one IP for the HTTP
tracker, one for the UDP tracker) so that both can be listed independently on
[newTrackon](https://newtrackon.com/), which tracks one tracker per IP.

The reference implementation is
[torrust/torrust-tracker-demo](https://github.com/torrust/torrust-tracker-demo), which uses this
setup with two floating IPs:

- HTTP tracker: `http1.torrust-tracker-demo.com` → `116.202.176.169` / `2a01:4f8:1c0c:9aae::1`
- UDP tracker: `udp1.torrust-tracker-demo.com` → `116.202.177.184` / `2a01:4f8:1c0c:828e::1`

The full incident investigation that led to this documentation is in
[torrust-tracker-demo#2](https://github.com/torrust/torrust-tracker-demo/issues/2).

## Which Steps Are Needed for Which Scenario

| Scenario                          | Step 1 | Step 2 | Step 3 | Step 4 |
| --------------------------------- | ------ | ------ | ------ | ------ |
| Floating IPv4 only                | ✅     | —      | —      | —      |
| IPv6 UDP, primary IP only         | —      | ✅     | ✅     | —      |
| IPv6 UDP, floating IP             | —      | ✅     | ✅     | ✅     |
| Floating IPv4 + IPv6 UDP floating | ✅     | ✅     | ✅     | ✅     |

---

## Why Floating IPs Require Manual Steps

The deployer configures the tracker to listen on the server's **primary public IP** only. When
traffic arrives on a **Hetzner floating IP**, the kernel's default routing uses the primary IP as
the reply source. The client then receives a reply from a different address than it sent to and
treats it as a timeout (asymmetric routing).

This applies to **both IPv4 and IPv6** floating IPs.

---

## Step 1 — Floating IP Policy Routing

> **Required for**: each floating IP (IPv4 or IPv6)

For each floating IP, add a policy routing rule so that packets arriving on that IP also leave via
that IP.

On Hetzner, this means adding routing tables (e.g. `100` for IPv4, `200` for IPv6) with a default
route via the floating IP gateway, then adding `ip rule` / `ip -6 rule` entries that match source
addresses on those tables.

**Persist via netplan** in `/etc/netplan/60-floating-ip.yaml`:

```yaml
network:
  version: 2
  renderer: networkd
  ethernets:
    eth0:
      addresses:
        - 116.202.177.184/32 # floating IPv4 (UDP1)
        - 2a01:4f8:1c0c:828e::1/64 # floating IPv6 (UDP1)
      routing-policy:
        - from: 116.202.177.184
          table: 100
        - from: 2a01:4f8:1c0c:828e::1
          table: 200
      routes:
        - to: default
          via: 172.31.1.1
          table: 100
        - to: default
          via: fe80::1
          table: 200
```

Apply:

```bash
sudo netplan apply
```

Verify:

```bash
ip rule list
ip route show table 100
ip -6 rule list
ip -6 route show table 200
```

> Repeat for every new floating IP pair. Without this, replies from floating IP endpoints leave
> via the wrong source address.

---

## Step 2 — Enable Docker ip6tables Management

> **Required for**: IPv6 UDP tracker

By default, Docker has `ip6tables: false`. This means:

- Docker does not insert ip6tables rules for published ports (unlike IPv4 where it does this
  automatically via iptables).
- Every time Docker starts or restarts a container, it rewrites its own chain tables. This flush
  wipes ufw's live ip6tables rules from the kernel. ufw does not automatically reload after this,
  so IPv6 UDP traffic is silently dropped after every container restart.

**Fix**: create `/etc/docker/daemon.json`:

```json
{
  "ip6tables": true
}
```

Apply:

```bash
sudo systemctl restart docker
```

Verify:

```bash
sudo ip6tables -L ufw6-user-input -n
# Must show: ACCEPT  17  --  ::/0  ::/0  udp dpt:6969
```

---

## Step 3 — Enable IPv6 on the Docker Bridge Network

> **Required for**: IPv6 UDP tracker

Even with `ip6tables: true`, native IPv6 UDP still fails. Docker spawns `docker-proxy` processes
for each published port. For IPv6, docker-proxy receives packets on an `::` socket but the
container only has an IPv4 address — docker-proxy cannot relay across address families and silently
drops all native IPv6 UDP.

**Fix**: add `enable_ipv6: true` and a ULA subnet to the bridge network in `docker-compose.yml`:

```yaml
proxy_network:
  driver: bridge
  enable_ipv6: true
  ipam:
    config:
      - subnet: "fd01:db8:1::/64"
```

With an IPv6 address on the container, Docker creates ip6tables DNAT rules that route native IPv6
traffic directly to the container, bypassing docker-proxy entirely.

Apply:

```bash
cd /opt/torrust
docker compose down
docker compose up -d
```

Verify the container has an IPv6 address:

```bash
docker inspect tracker --format '{{range .NetworkSettings.Networks}}{{.GlobalIPv6Address}} {{end}}'
# Expected: fd01:db8:1::x  (non-empty)
```

Verify the DNAT rule exists:

```bash
sudo ip6tables -t nat -L DOCKER -n -v | grep 6969
# Expected: DNAT rule for dpt:6969
```

---

## Step 4 — SNAT for IPv6 UDP Replies via Floating IP

> **Required for**: floating IPv6 UDP

After Step 3, the container has a ULA IPv6 address (`fd01:db8:1::x`). When it replies, Docker's
MASQUERADE rule rewrites the source to the server's **primary** IPv6 address
(`2a01:4f8:1c19:620b::1`). Clients that probed the **floating** IPv6 (`2a01:4f8:1c0c:828e::1`)
receive a reply from the wrong address and time out.

**Fix**: prepend a SNAT rule to `/etc/ufw/before6.rules` **before** the existing `*filter`
section:

```text
# NAT: rewrite source of Docker UDP tracker IPv6 replies to the floating IP
*nat
:POSTROUTING ACCEPT [0:0]
-A POSTROUTING -s fd01:db8:1::/64 -o eth0 -p udp --sport 6969 \
    -j SNAT --to-source 2a01:4f8:1c0c:828e::1
COMMIT
```

Apply:

```bash
sudo ufw reload
```

Verify:

```bash
sudo ip6tables -t nat -L POSTROUTING -n -v | grep 6969
# Expected: SNAT  ...  fd01:db8:1::/64  ...  udp spt:6969  to:2a01:4f8:1c0c:828e::1
```

> This rule must be in `before6.rules` (not added via `ufw` CLI) so it persists in the `*nat`
> table. ufw loads this file at startup, before Docker starts. The SNAT fires before Docker's
> MASQUERADE and takes precedence.
>
> If you change the `subnet` in `docker-compose.yml`, update the `-s` match here too.
> If you add a second floating IPv6, add a second SNAT rule for its subnet/address.

---

## References

- [torrust/torrust-tracker-demo](https://github.com/torrust/torrust-tracker-demo) — full working configuration
- [torrust-tracker-demo#2](https://github.com/torrust/torrust-tracker-demo/issues/2) — incident that produced this documentation
- [torrust-tracker-demo/docs/docker-ipv6.md](https://github.com/torrust/torrust-tracker-demo/blob/main/docs/docker-ipv6.md) — detailed explanation with packet-flow diagram
- [torrust-tracker-demo/docs/post-deployment.md](https://github.com/torrust/torrust-tracker-demo/blob/main/docs/post-deployment.md) — step-by-step instructions
- [Hetzner Provider Guide](hetzner.md) — Hetzner Cloud configuration for the deployer
- [IPv6 UDP Tracker Issue Investigation](../../../docs/deployments/hetzner-demo-tracker/post-provision/ipv6-udp-tracker-issue.md) — root-cause analysis
