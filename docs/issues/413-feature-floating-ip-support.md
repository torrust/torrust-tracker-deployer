# Feature: Floating IP Support

**Issue**: #413
**Parent Epic**: None
**Related**: #405 - Deploy Hetzner Demo Tracker and Document the Process

## Overview

The deployer is not aware of **floating IPs** (also called static IPs, reserved IPs,
or elastic IPs depending on the provider). A floating IP is an IP address owned
independently of any specific server and can be reassigned between servers without
changing DNS.

During the Hetzner Demo deployment (#405) floating IPs were used deliberately to allow
zero-downtime failover and maintenance:

- **Instance IP**: `46.225.234.201` — the bare VM's IP, stored in the deployer's
  internal environment state after provisioning
- **Floating IP**: `116.202.176.169` — the IP published in all DNS A records

The deployer has no concept of floating IPs. It records the instance IP during
`provision` and uses that IP as the expected DNS target in the `test` command's
DNS checks. Because DNS points to the floating IP, every domain in the environment
triggers a false-positive warning during `test`:

```text
⚠️  DNS check: api.torrust-tracker-demo.com resolves to [116.202.176.169]
    but expected 46.225.234.201
```

This is not an error. The deployment works correctly — traffic reaches the server
through the floating IP. The deployer simply lacks the notion of a separate "public"
IP that differs from the provisioned instance IP.

The DNS setup for this deployment (floating IP assignment, VM network configuration,
and DNS record creation) is documented in
[docs/deployments/hetzner-demo-tracker/post-provision/dns-setup.md](../deployments/hetzner-demo-tracker/post-provision/dns-setup.md).

## Goals

- [ ] Allow users to specify a floating IP (or more generally, a public IP) that is
      separate from the instance IP in the environment configuration
- [ ] Use the floating IP as the expected DNS target in `test` DNS checks when it
      is configured, eliminating false-positive warnings
- [ ] Consider using the floating IP during provisioning to automatically assign it to
      the newly created instance (provider-dependent)
- [ ] Expose the floating IP in `provision` and `Running` state output so operators
      can confirm which IP is serving traffic

## Specifications

### Motivation: Why Floating IPs?

Floating IPs decouple the publicly announced address from the physical server, providing:

1. **Zero-downtime failover**: If the primary server fails, the floating IP can be
   reassigned to a standby server in seconds. DNS TTLs are not a concern because the
   IP itself does not change.
2. **Maintenance without downtime**: A new server can be fully provisioned and
   configured before traffic is cut over by reassigning the floating IP.

Other providers use different terminology for the same concept:

| Provider     | Term             |
| ------------ | ---------------- |
| Hetzner      | Floating IP      |
| AWS          | Elastic IP       |
| GCP          | Reserved IP      |
| DigitalOcean | Reserved IP      |
| Azure        | Static Public IP |

### Current Behavior (Limitation)

1. `provision` stores the instance IP in the environment state.
2. `configure`, `release`, and `run` use the instance IP — no issue here since
   these commands operate on the instance directly.
3. `test` resolves each domain via DNS and compares the result against the instance IP.
   When DNS points to a floating IP instead, the comparison fails and the deployer
   emits a warning for every domain.
4. The deployer works correctly despite the warnings; the false positives are noise.

### Proposed Configuration Change

The environment config (or a provider-specific section) should accept an optional
`public_ip` (or `floating_ip`) field:

```json
"provider": {
  "name": "hetzner",
  "instance_ip": "46.225.234.201",
  "public_ip": "116.202.176.169"
}
```

When `public_ip` is present:

- The `test` command DNS checks compare resolved IPs against `public_ip`, not
  `instance_ip`.
- The `provision` output includes `public_ip` alongside `instance_ip`.
- Future: `provision` could automatically assign the floating IP to the instance
  via provider APIs.

### Current Workaround

Until this feature is implemented, `test` command DNS warnings can be safely ignored
when floating IPs are in use. Verify manually that:

1. Each domain resolves to the expected floating IP.
2. Services are reachable through those domains.

See [docs/deployments/hetzner-demo-tracker/commands/improvements.md](../deployments/hetzner-demo-tracker/commands/improvements.md)
and [docs/deployments/hetzner-demo-tracker/post-provision/dns-setup.md](../deployments/hetzner-demo-tracker/post-provision/dns-setup.md)
for the specific setup used in the Hetzner demo deployment.

## Implementation Plan

### Phase 1: Environment Config Schema

- [ ] Add optional `public_ip` field to the environment config schema
      (`schemas/environment-config.json`)
- [ ] Parse and store `public_ip` in the environment domain types
- [ ] Validate: if `public_ip` is present it must be a valid IPv4/IPv6 address

### Phase 2: `test` Command DNS Checks

- [ ] When `public_ip` is configured, use it as the expected target in DNS checks
      instead of the instance IP
- [ ] Update warning/success messages to indicate which IP was expected and which
      IP was found
- [ ] Add unit tests for the DNS check logic covering the floating-IP case

### Phase 3: `provision` Output

- [ ] Include `public_ip` in the `provision` command output when configured
- [ ] Include `public_ip` in the `Running` state output

### Phase 4: Documentation

- [ ] Update the environment config user guide to document the `public_ip` field
- [ ] Add a note in the provider guide explaining floating IP support

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.
> Use this as your pre-review checklist before submitting the PR to minimize
> back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `test` command emits no DNS warnings when `public_ip` is configured and DNS
      resolves to that IP
- [ ] `provision` output includes `public_ip` when configured
- [ ] Environment config schema validates `public_ip` as a valid IP address
- [ ] Unit tests cover DNS check with and without `public_ip`
- [ ] Documentation updated

## Related Documentation

- [docs/deployments/hetzner-demo-tracker/post-provision/dns-setup.md](../deployments/hetzner-demo-tracker/post-provision/dns-setup.md) — actual DNS setup using floating IPs
- [docs/deployments/hetzner-demo-tracker/commands/improvements.md](../deployments/hetzner-demo-tracker/commands/improvements.md) — original field observation
