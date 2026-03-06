# Bug: UDP Tracker Domains Missing from `provision` Output

**Issue**: #412
**Parent Epic**: None
**Related**: #405 - Deploy Hetzner Demo Tracker and Document the Process

## Overview

The `provision` command output includes a `domains` array listing the configured domain
names for the deployed environment. When UDP trackers have domains configured (via the
`domain` field of `udp_trackers[].domain` in the environment JSON), those domains are
absent from the list. Only HTTP-based service domains appear.

Example observed output (Hetzner demo deployment #405):

```json
{
  "domains": [
    "http1.torrust-tracker-demo.com",
    "http2.torrust-tracker-demo.com",
    "api.torrust-tracker-demo.com",
    "grafana.torrust-tracker-demo.com"
  ]
}
```

Expected — UDP domains should also appear:

```json
{
  "domains": [
    "http1.torrust-tracker-demo.com",
    "http2.torrust-tracker-demo.com",
    "udp1.torrust-tracker-demo.com",
    "udp2.torrust-tracker-demo.com",
    "api.torrust-tracker-demo.com",
    "grafana.torrust-tracker-demo.com"
  ]
}
```

The `domains` list is used as a DNS-setup reminder — it tells the operator which
`A`/`AAAA` records need to point at the server IP. A missing UDP domain means the
operator does not know they need to create that DNS record.

## Goals

- [ ] Include UDP tracker domain names in the `domains` list of the `provision` output
- [ ] Ensure the `dns_reminder` view also includes UDP domains
- [ ] Add or update tests to cover UDP domains appearing in both views

## Specifications

### Root Cause

The `domains` field in `ProvisionDetailsData` is populated in
`src/presentation/cli/views/commands/provision/view_data/provision_details.rs` by
calling `services.tls_domain_names()`:

```rust
let domains = if let Some(ip) = environment.instance_ip() {
    let tracker_config = environment.tracker_config();
    let grafana_config = environment.grafana_config();
    let services = ServiceInfo::from_tracker_config(tracker_config, ip, grafana_config);
    services
        .tls_domain_names()      // ← only returns TLS service domains
        .iter()
        .map(|s| (*s).to_string())
        .collect()
} else {
    vec![]
};
```

`tls_domain_names()` in
`src/application/command_handlers/show/info/tracker.rs` returns only the `tls_domains`
vector — domains associated with HTTPS services (HTTP trackers with TLS proxy, API with
TLS proxy, Grafana). UDP trackers are not TLS services and are never added to
`tls_domains`.

`ServiceInfo` already stores UDP tracker URLs in `udp_trackers: Vec<String>` (built by
`build_udp_tracker_urls`), but these are full announce URLs
(`udp://udp1.example.com:6969/announce`), not bare domain names. The domain name needs
to be extracted from those URLs, or a separate accessor for UDP domains needs to be
added.

The same issue exists in `dns_reminder.rs` which also calls `tls_domain_names()` to
build the DNS setup hint.

### Solution

Add a method `all_domain_names() -> Vec<&str>` (or rename the existing one) to
`ServiceInfo` that returns:

1. All TLS service domain names (HTTP trackers, API, Grafana) — currently in `tls_domains`
2. All UDP tracker domain names — extracted from `UdpTrackerConfig::domain()` (if set)

The `provision_details.rs` and `dns_reminder.rs` callers are then updated to call the
new method instead of `tls_domain_names()`.

An alternative is to keep `tls_domain_names()` unchanged (it is used for the HTTPS
hint that reads "configure these domains in /etc/hosts or your DNS before enabling TLS")
and introduce a separate `all_domain_names()` accessor used only for the DNS reminder
and provision output `domains` field. This keeps the TLS-specific semantic intact while
fixing the provision output.

### Affected Modules and Types

#### `src/application/command_handlers/show/info/tracker.rs`

- `ServiceInfo`: add `all_domain_names() -> Vec<&str>` that returns TLS domains plus
  UDP tracker domains that have a `domain` configured.
- `build_udp_tracker_urls`: no change needed; UDP domains are read directly from
  `UdpTrackerConfig::domain()`.

#### `src/presentation/cli/views/commands/provision/view_data/provision_details.rs`

- `From<&Environment<Provisioned>>` implementation: replace the `tls_domain_names()`
  call with `all_domain_names()`.

#### `src/presentation/cli/views/commands/provision/view_data/dns_reminder.rs`

- Replace the `tls_domain_names()` call with `all_domain_names()`.

### What Does Not Change

- `tls_domain_names()` is kept as-is for the `show` command's HTTPS/TLS hint, which
  should only list TLS domains (the hint is about configuring reverse proxy, not DNS).
- The UDP tracker URLs in `ServiceInfo.udp_trackers` are unchanged.
- The domain name in a UDP tracker config is optional; UDP trackers without a configured
  domain produce no entry in `all_domain_names()`.

## Implementation Plan

### Phase 1: Add `all_domain_names()` to `ServiceInfo`

- [ ] In `ServiceInfo` (`tracker.rs`): implement `all_domain_names() -> Vec<&str>` that
      returns the union of TLS domain names and UDP tracker domain names (where
      `udp.domain()` is `Some`)
- [ ] Keep `tls_domain_names()` unchanged

### Phase 2: Update callers

- [ ] `provision_details.rs`: replace `tls_domain_names()` call with `all_domain_names()`
- [ ] `dns_reminder.rs`: replace `tls_domain_names()` call with `all_domain_names()`

### Phase 3: Tests

- [ ] `tracker.rs`: add test `it_should_return_all_domain_names_including_udp` that
      asserts UDP tracker domains appear in `all_domain_names()` when a UDP domain is set
- [ ] `tracker.rs`: add test `it_should_exclude_udp_trackers_without_domain_from_all_domain_names`
      that asserts UDP trackers without a `domain` do not appear
- [ ] `provision_details.rs` or `text_view.rs`/`json_view.rs`: update or add test
      covering UDP domains in the rendered provision output
- [ ] Run `cargo test` to verify all tests pass

### Phase 4: Linting and pre-commit

- [ ] Run linters: `cargo run --bin linter all`
- [ ] Run pre-commit: `./scripts/pre-commit.sh`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.
> Use this as your pre-review checklist before submitting the PR to minimize
> back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `provision` output `domains` array includes UDP tracker domain names when the
      environment config supplies a `domain` for UDP trackers
- [ ] `provision` output `domains` array does **not** include entries for UDP trackers
      that have no `domain` configured (IP-only UDP trackers)
- [ ] The DNS setup reminder shown after `provision` also includes UDP tracker domains
- [ ] `tls_domain_names()` is unchanged — the HTTPS-specific TLS hint is not affected
- [ ] Unit tests for `all_domain_names()` pass for both the UDP-with-domain and
      UDP-without-domain cases

## Related Documentation

- [docs/deployments/hetzner-demo-tracker/commands/provision/bugs.md](../deployments/hetzner-demo-tracker/commands/provision/bugs.md) — original bug report
- [src/application/command_handlers/show/info/tracker.rs](../../src/application/command_handlers/show/info/tracker.rs) — `ServiceInfo`, `tls_domain_names()`
- [src/presentation/cli/views/commands/provision/view_data/provision_details.rs](../../src/presentation/cli/views/commands/provision/view_data/provision_details.rs) — `ProvisionDetailsData` and its `From` impl
- [src/presentation/cli/views/commands/provision/view_data/dns_reminder.rs](../../src/presentation/cli/views/commands/provision/view_data/dns_reminder.rs) — DNS reminder view
