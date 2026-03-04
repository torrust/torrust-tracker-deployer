# Deployer Commands — Improvements

Improvements identified during the Hetzner demo tracker deployment that apply
across multiple commands, not just one specific command.

---

## Improvement: Deployer is not aware of floating IPs

**Status**: Open
**Affects**: `test` command (DNS checks), potentially `configure`, `release`, `run`
**Observed in**: `test` command DNS warnings

### Observation

Hetzner and other providers support **floating IPs** — IP addresses that are
owned independently of any single server and can be reassigned between servers
instantly without changing DNS. In this deployment:

- **Instance IP**: `46.225.234.201` — the bare VM's IP, not published in DNS
- **Floating IP**: `116.202.176.169` — the IP published in all DNS A records

The deployer `test` command resolves each configured domain and compares the
result against the **instance IP**. Because DNS points to the floating IP, every
domain triggers a warning:

```text
⚠️  DNS check: api.torrust-tracker-demo.com resolves to [116.202.176.169]
    but expected 46.225.234.201
```

This is not a real problem — DNS is configured correctly and traffic reaches the
server. The warning is a false positive produced by the deployer's lack of
floating IP awareness.

### Why Floating IPs Matter

Using a floating IP provides two key operational benefits:

1. **Zero-downtime failover**: If the primary server fails, the floating IP can
   be reassigned to a standby server in seconds. DNS does not need to change and
   clients are not affected by TTL delays.
2. **Maintenance without downtime**: A new server can be fully provisioned and
   configured before cutting traffic over by reassigning the floating IP.

These benefits are lost if DNS records point directly to the instance IP.

### Proposed Fix

The environment JSON config (or a separate provider config section) should allow
specifying a **floating IP** (or more generally, a **public IP**) that is
distinct from the instance IP. For example:

```json
"provider": {
  ...
  "floating_ip": "116.202.176.169"
}
```

The deployer should then:

- Use the floating IP (when present) as the expected DNS target in `test` DNS
  checks, treating a match as a pass rather than a warning.
- Optionally, during provisioning, automatically assign the floating IP to the
  newly created instance.
- Optionally, expose the floating IP in the `Running` state output so operators
  can confirm which IP is serving traffic.

### Current Workaround

The DNS warnings in `test` output can be safely ignored when a floating IP is in
use. Verify manually that the domains resolve to the expected floating IP and
that the services are reachable through those domains (see
[../verify/](../verify/) for service verification procedures).
