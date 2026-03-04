# DNS Setup

> **Status**: 🔄 In progress — both floating IPs assigned; VM-side configuration and DNS records pending.

Set up DNS records so that all domain names in the environment config resolve to
the floating IP before running `configure`.

## Why Floating IPs?

The deployment uses **Hetzner floating IPs** (static IPs that can be reassigned across servers)
rather than the server's direct IP (`46.225.234.201`). This means:

- DNS records always point to the same IP, even if the underlying server is ever recreated.
- To rebuild the server, you reassign the floating IP — no DNS changes needed.

## Floating IPs

| Type | IP                      | Notes                                                                                |
| ---- | ----------------------- | ------------------------------------------------------------------------------------ |
| IPv4 | `116.202.176.169`       | Assign as A records                                                                  |
| IPv6 | `2a01:4f8:1c0c:9aae::1` | First usable address from `/64` block `2a01:4f8:1c0c:9aae::/64`; use as AAAA records |

## Step 1: Assign Floating IPs to the Server

In the [Hetzner Console](https://console.hetzner.cloud/):

1. Open the project `torrust-tracker-demo.com`.
2. Go to **Networking → Floating IPs**.
3. For the IPv4 floating IP (`116.202.176.169`):
   - Click **⋯ → Assign**.
   - Select server `torrust-tracker-vm-torrust-tracker-demo`.
   - Confirm.
4. For the IPv6 floating IP (`2a01:4f8:1c0c:9aae::/64`):
   - Same procedure — assign to the same server.

### IPv4 Assigned (2026-03-04)

The IPv4 floating IP was assigned successfully. Hetzner showed a confirmation popup:

![Hetzner console popup after assigning the IPv4 floating IP](../media/hetzner-console-assign-floating-ipv4-popup.png)

The popup text:

> **Configure Floating IP**
>
> The Floating IP has been successfully assigned. You now need to configure it on
> your server in order for it to work.
>
> **Command for temporary configuration**
>
> `sudo ip addr add 116.202.176.169 dev eth0`
>
> A temporary configuration will only work until the next reboot. To permanently
> configure the IP have a look at our Docs.

### IPv6 Assigned (2026-03-04)

The IPv6 floating IP was assigned successfully. Hetzner showed a confirmation popup:

![Hetzner console popup after assigning the IPv6 floating IP](../media/hetzner-console-assign-floating-ipv6-popup.png)

The popup text:

> **Configure Floating IP**
>
> The Floating IP has been successfully assigned. You now need to configure it on
> your server in order for it to work.
>
> **Command for temporary configuration**
>
> `sudo ip addr add 2a01:4f8:1c0c:9aae::1 dev eth0`
>
> A temporary configuration will only work until the next reboot. To permanently
> configure the IP have a look at our Docs.

### Both Floating IPs Assigned

Both IPs now appear in the Hetzner console Floating IPs list assigned to the server:

![Hetzner console showing both floating IPs assigned to the server](../media/hetzner-console-floating-ips-assigned-to-server.png)

### Step 1.5: Configure the Floating IPs Inside the VM

Hetzner's assignment only updates their routing — the VM's network interface still needs to
know about the new IPs. The Hetzner console popup shows a **temporary** command that works
until the next reboot. We need the **permanent** configuration instead.

Reference: [Hetzner — Persistent Floating IP Configuration](https://docs.hetzner.com/cloud/floating-ips/persistent-configuration/)

**Temporary (shown by Hetzner popup — lost on reboot):**

```bash
sudo ip addr add 116.202.176.169 dev eth0
sudo ip addr add 2a01:4f8:1c0c:9aae::1 dev eth0
```

**Permanent (survives reboot) — recommended:**

SSH into the server:

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
```

Create `/etc/netplan/60-floating-ip.yaml` with both floating IPs. The Hetzner docs specify
`renderer: networkd` and separate IPv4 (`/32`) and IPv6 (`/64`) prefixes:

```bash
sudo tee /etc/netplan/60-floating-ip.yaml > /dev/null << 'EOF'
network:
  version: 2
  renderer: networkd
  ethernets:
    eth0:
      addresses:
        - 116.202.176.169/32
        - 2a01:4f8:1c0c:9aae::1/64
EOF
```

Apply (this will briefly reset the network connection):

```bash
sudo netplan apply
```

Verify both IPs appear on the interface:

```bash
ip addr show eth0
# Look for:
#   inet 116.202.176.169/32
#   inet6 2a01:4f8:1c0c:9aae::1/64
```

> **Note**: The `configure` command does not configure floating IPs — this must be done
> manually before running `configure`.

## Step 2: Create DNS Records

In the [Hetzner DNS Console](https://dns.hetzner.com/), open the `torrust-tracker-demo.com` zone
and create the following records:

| Subdomain | Type | Value                   |
| --------- | ---- | ----------------------- |
| `http1`   | A    | `116.202.176.169`       |
| `http1`   | AAAA | `2a01:4f8:1c0c:9aae::1` |
| `http2`   | A    | `116.202.176.169`       |
| `http2`   | AAAA | `2a01:4f8:1c0c:9aae::1` |
| `api`     | A    | `116.202.176.169`       |
| `api`     | AAAA | `2a01:4f8:1c0c:9aae::1` |
| `grafana` | A    | `116.202.176.169`       |
| `grafana` | AAAA | `2a01:4f8:1c0c:9aae::1` |
| `udp1`    | A    | `116.202.176.169`       |
| `udp1`    | AAAA | `2a01:4f8:1c0c:9aae::1` |
| `udp2`    | A    | `116.202.176.169`       |
| `udp2`    | AAAA | `2a01:4f8:1c0c:9aae::1` |

Use the default TTL (300 s is fine for initial setup; increase to 3600 s once stable).

## Step 3: Verify DNS Propagation

From your local machine, check that all records resolve correctly:

```bash
# IPv4 records (A)
for subdomain in http1 http2 api grafana udp1 udp2; do
  echo -n "$subdomain.torrust-tracker-demo.com A: "
  dig +short A "$subdomain.torrust-tracker-demo.com"
done

# IPv6 records (AAAA)
for subdomain in http1 http2 api grafana udp1 udp2; do
  echo -n "$subdomain.torrust-tracker-demo.com AAAA: "
  dig +short AAAA "$subdomain.torrust-tracker-demo.com"
done
```

Expected output for each:

```text
http1.torrust-tracker-demo.com A: 116.202.176.169
http2.torrust-tracker-demo.com A: 116.202.176.169
api.torrust-tracker-demo.com A: 116.202.176.169
grafana.torrust-tracker-demo.com A: 116.202.176.169
udp1.torrust-tracker-demo.com A: 116.202.176.169
udp2.torrust-tracker-demo.com A: 116.202.176.169

http1.torrust-tracker-demo.com AAAA: 2a01:4f8:1c0c:9aae::1
...
```

> DNS propagation with Hetzner's nameservers (`helium.ns.hetzner.de`, `hydrogen.ns.hetzner.com`,
> `oxygen.ns.hetzner.com`) is typically fast (under 1 minute). If you get `NXDOMAIN` or empty
> results, wait a minute and retry.

## Outcome

Once all subdomains resolve to `116.202.176.169` / `2a01:4f8:1c0c:9aae::1`, DNS is ready and
you can proceed to [volume-setup.md](volume-setup.md).

## Problems

<!-- Document any issues encountered during DNS setup here. -->

## Improvements

<!-- Document any recommended improvements here. -->
