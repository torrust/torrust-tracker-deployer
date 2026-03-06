# Submit UDP1 Tracker to newTrackon

**Issue**: #407
**Parent Epic**: None
**Related**: [#405 - Deploy Hetzner Demo Tracker](405-deploy-hetzner-demo-tracker-and-document-process.md),
[docs/deployments/hetzner-demo-tracker/tracker-registry.md](../deployments/hetzner-demo-tracker/tracker-registry.md)

## Overview

After deploying the Hetzner demo tracker (issue #405), the HTTP1 tracker was successfully submitted to
[newTrackon](https://newtrackon.com/). However, the UDP1 tracker submission failed to be accepted.

Two prerequisites were missed during the initial submission:

1. **BEP 34 DNS TXT records**: newTrackon requires a DNS TXT record on the tracker's domain following
   [BEP 34](https://www.bittorrent.org/beps/bep_0034.html) to announce which ports the tracker uses.
2. **One tracker per IP policy**: newTrackon only accepts one tracker per IP address. The HTTP1 tracker
   already occupies the two floating IPs assigned to the server
   (`116.202.176.169` and `2a01:4f8:1c0c:9aae::1`), so two new floating IPs (IPv4 + IPv6) must be
   provisioned and assigned to support the UDP1 tracker.

This task documents and resolves both blockers so that the UDP1 tracker can be listed on newTrackon,
and updates the deployment documentation to make the newTrackon prerequisites explicit for future
deployments.

## Goals

- [x] Add BEP 34 DNS TXT records for `http1.torrust-tracker-demo.com` (port 443) and
      `udp1.torrust-tracker-demo.com` (port 6969)
- [x] Provision two new Hetzner floating IPs (IPv4 + IPv6) and assign them to the existing server
- [x] Configure the new IPs permanently inside the VM (netplan)
- [x] Configure DNS A/AAAA records so `udp1.torrust-tracker-demo.com` resolves to the new IPs
- [ ] Retry submission of `udp://udp1.torrust-tracker-demo.com:6969/announce` to newTrackon
      (Attempt 1: ❌ Rejected — UDP timeout; see Step 5 in `newtrackon-prerequisites.md`)
- [ ] Verify UDP1 tracker appears in the newTrackon public list
- [ ] Document the complete process (prerequisites, steps, outcomes) in the deployment docs

## Specifications

### BEP 34 DNS TXT Record

[BEP 34](https://www.bittorrent.org/beps/bep_0034.html) defines a DNS-based method for announcing
tracker availability. newTrackon uses this to validate that a domain is intentionally serving a
BitTorrent tracker on the submitted port.

The TXT record format is:

```text
"BITTORRENT UDP:<port> TCP:<port>"
```

For example, the old demo tracker (`tracker.torrust-demo.com`) has:

```text
"BITTORRENT UDP:6969 TCP:443"
```

Records to add for the new demo:

| Domain                           | TXT value             |
| -------------------------------- | --------------------- |
| `http1.torrust-tracker-demo.com` | `BITTORRENT TCP:443`  |
| `udp1.torrust-tracker-demo.com`  | `BITTORRENT UDP:6969` |

### One IP Per Tracker (newTrackon Policy)

newTrackon enforces that each listed tracker resolves to unique IP addresses not already used by
another listed tracker. The HTTP1 tracker already occupies:

- IPv4: `116.202.176.169`
- IPv6: `2a01:4f8:1c0c:9aae::1`

To add the UDP1 tracker, two new floating IPs must be provisioned in Hetzner and associated
exclusively with the `udp1` subdomain.

### Floating IP Configuration

New floating IPs must be made persistent inside the VM using netplan. The current floating IPs
were **not** configured with netplan — this task also covers making all four floating IPs
permanent (both existing and new ones).

Netplan configuration path: `/etc/netplan/60-floating-ip.yaml`

Example netplan stanza for a floating IPv4:

```yaml
network:
  version: 2
  renderer: networkd
  ethernets:
    eth0:
      addresses:
        - 116.202.176.169/32
```

> **Note**: Hetzner uses `/64` prefix for IPv6 floating IPs (not `/128`).

### DNS Records for New IPs

Once the new floating IPs are provisioned, A and AAAA records must be created for
`udp1.torrust-tracker-demo.com` pointing to those new IPs via the Hetzner DNS API.

## Implementation Plan

### Phase 1: DNS BEP 34 TXT Records

- [x] Task 1.1: Add TXT record `"BITTORRENT TCP:443"` to `http1.torrust-tracker-demo.com` via Hetzner DNS API
- [x] Task 1.2: Add TXT record `"BITTORRENT UDP:6969"` to `udp1.torrust-tracker-demo.com` via Hetzner DNS API
- [x] Task 1.3: Verify both TXT records resolve correctly with `dig TXT <domain>`
- [x] Task 1.4: Create `docs/deployments/hetzner-demo-tracker/post-provision/newtrackon-prerequisites.md`
      documenting the BEP 34 requirement and the TXT records added
- [x] Task 1.5: Update `docs/deployments/hetzner-demo-tracker/README.md` to reference the new document

### Phase 2: Provision New Floating IPs

- [x] Task 2.1: Book a new IPv4 floating IP in Hetzner Cloud Console (region `nbg1`) — `udp1-ipv4`: `116.202.177.184`
- [x] Task 2.2: Book a new IPv6 floating IP in Hetzner Cloud Console (region `nbg1`) — `udp1-ipv6`: `2a01:4f8:1c0c:828e::1`
- [x] Task 2.3: Assign both new floating IPs to the existing demo server in Hetzner Console
- [x] Task 2.4: Add the "one tracker per IP" policy section to `newtrackon-prerequisites.md`

### Phase 3: Configure New IPs Inside the VM

- [x] Task 3.1: SSH into the server
- [x] Task 3.2: Add all four floating IPs to netplan configuration (`/etc/netplan/60-floating-ip.yaml`)
      — both existing IPs (which were not previously configured via netplan) and the two new ones
- [x] Task 3.3: Apply netplan configuration (`sudo netplan apply`) and verify all IPs are active
- [x] Task 3.4: Confirm the new IPs receive traffic (IPv4 ping test from external host: ✅;
      IPv6 not testable from local machine — confirmed active on `eth0` via `ip addr`)
- [x] Task 3.5: Document the netplan configuration steps and file content in `newtrackon-prerequisites.md`

### Phase 4: Update DNS for UDP1 Subdomain

- [x] Task 4.1: Update (or add) A record for `udp1.torrust-tracker-demo.com` pointing to the new IPv4
- [x] Task 4.2: Update (or add) AAAA record for `udp1.torrust-tracker-demo.com` pointing to the new IPv6
- [x] Task 4.3: Verify DNS resolution with `dig A udp1.torrust-tracker-demo.com` and
      `dig AAAA udp1.torrust-tracker-demo.com`
- [x] Task 4.4: Update `docs/deployments/hetzner-demo-tracker/post-provision/dns-setup.md` with the
      new A/AAAA records added for `udp1.torrust-tracker-demo.com`

### Phase 5: Submit UDP1 Tracker to newTrackon

- [ ] Task 5.1: Go to <https://newtrackon.com/> and submit `udp://udp1.torrust-tracker-demo.com:6969/announce`
- [ ] Task 5.2: Verify submission is accepted (no error message from newTrackon)
- [ ] Task 5.3: Wait for the tracker to appear in the [newTrackon list](https://newtrackon.com/list)
- [ ] Task 5.4: Verify via newTrackon API: `curl https://newtrackon.com/api/stable`
- [ ] Task 5.5: Update `docs/deployments/hetzner-demo-tracker/tracker-registry.md` with the final
      submission status for the UDP1 tracker and link to `newtrackon-prerequisites.md`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your
> pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] BEP 34 TXT records are present and correct for both `http1` and `udp1` subdomains
      (verified with `dig TXT`)
- [ ] Two new floating IPs are provisioned in Hetzner and assigned to the server
- [ ] All four floating IPs (existing + new) are configured permanently via netplan
- [ ] `udp1.torrust-tracker-demo.com` resolves to the new IPs (A + AAAA records)
- [ ] `udp://udp1.torrust-tracker-demo.com:6969/announce` appears in the newTrackon public list
- [ ] `docs/deployments/hetzner-demo-tracker/post-provision/newtrackon-prerequisites.md` documents
      the prerequisites clearly
- [ ] `tracker-registry.md` is updated with the correct submission status

## Related Documentation

- [BEP 34 — DNS Tracker Preferences](https://www.bittorrent.org/beps/bep_0034.html)
- [newTrackon](https://newtrackon.com/)
- [Hetzner Demo Tracker — Deployment Journal](../deployments/hetzner-demo-tracker/README.md)
- [Hetzner Demo Tracker — Tracker Registry](../deployments/hetzner-demo-tracker/tracker-registry.md)
- [Hetzner Demo Tracker — DNS Setup](../deployments/hetzner-demo-tracker/post-provision/dns-setup.md)
- [Issue #405 — Deploy Hetzner Demo Tracker](405-deploy-hetzner-demo-tracker-and-document-process.md)

## Notes

The HTTP1 tracker (`https://http1.torrust-tracker-demo.com/announce`) was successfully submitted and
accepted by newTrackon on 2026-03-04. The newTrackon API can be used to verify current status:
`curl https://newtrackon.com/api/stable`.

The UDP2 tracker (`udp://udp2.torrust-tracker-demo.com:6868/announce`) is intentionally **not**
submitted to any public registry — it is reserved as a low-traffic endpoint for manual testing
and debugging.
