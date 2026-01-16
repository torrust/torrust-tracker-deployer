# Tracker `on_reverse_proxy` is Global Instead of Per-HTTP-Tracker

**Issue Date**: January 16, 2026
**Affected Component**: Torrust Tracker Configuration
**Status**: Documented - Issue filed in tracker repository
**Upstream Issue**: [torrust/torrust-tracker#1640](https://github.com/torrust/torrust-tracker/issues/1640)

## Problem Description

The Torrust Tracker has a configuration option `[core.net].on_reverse_proxy` that controls whether the tracker expects `X-Forwarded-For` HTTP headers to determine the real client IP address. This setting is **global** and applies to **all HTTP trackers** in the deployment.

This creates a limitation: you cannot have some HTTP trackers behind a reverse proxy while others are accessed directly in the same deployment.

## How We Discovered This

While implementing HTTPS support with Caddy as a TLS-terminating reverse proxy in the [Torrust Tracker Deployer](https://github.com/torrust/torrust-tracker-deployer), we needed to configure the tracker to work behind Caddy.

Our use case:

- Multiple HTTP trackers on different ports (e.g., 7070, 7071, 7072)
- Some trackers exposed via HTTPS through Caddy (TLS termination)
- Some trackers exposed directly via HTTP (no proxy)

**Example configuration intent**:

```json
{
  "http_trackers": [
    {
      "bind_address": "0.0.0.0:7070",
      "domain": "http1.tracker.local",
      "use_tls_proxy": true
    },
    {
      "bind_address": "0.0.0.0:7071",
      "domain": "http2.tracker.local",
      "use_tls_proxy": true
    },
    {
      "bind_address": "0.0.0.0:7072"
    }
  ]
}
```

In this scenario:

- Trackers on ports 7070 and 7071 are behind Caddy (need `on_reverse_proxy = true`)
- Tracker on port 7072 is direct (needs `on_reverse_proxy = false`)

However, the current tracker configuration only allows:

```toml
[core.net]
on_reverse_proxy = true  # Applies to ALL HTTP trackers
```

## Root Cause

The `on_reverse_proxy` setting is defined in `[core.net]` which is a global network configuration section, not per-tracker. Looking at the tracker's network configuration structure:

**Reference**: [Torrust Tracker Network Configuration](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/v2_0_0/network/struct.Network.html)

```rust
pub struct Network {
    // ...
    pub on_reverse_proxy: bool,  // Global setting
    // ...
}
```

Each HTTP tracker configuration does not have its own `on_reverse_proxy` field.

## Impact

### For Deployer Users

When `on_reverse_proxy = true` is set globally:

1. **All HTTP trackers expect `X-Forwarded-For` headers**
2. Trackers accessed directly (without proxy) will **fail to identify client IPs correctly**
3. The tracker will see the absence of `X-Forwarded-For` and may log warnings or behave unexpectedly

When `on_reverse_proxy = false` is set globally:

1. **All HTTP trackers ignore `X-Forwarded-For` headers**
2. Trackers behind a reverse proxy will **see the proxy's IP as the client IP**
3. All peers from different clients will appear to come from the same IP (the proxy)
4. This breaks peer identification in swarms

### Current Workaround in Deployer

We enforce a rule in the deployer:

> **If ANY HTTP tracker uses a TLS proxy, ALL HTTP trackers must use the TLS proxy.**

This is documented as a known limitation and validated during environment creation:

```text
Known Limitation (due to tracker's global setting):

If you have multiple HTTP trackers where some use use_tls_proxy and others don't,
the ones without it will still receive the global on_reverse_proxy = true setting
and may fail if they receive direct requests without X-Forwarded-For headers.

Workaround: Ensure all HTTP trackers in a deployment either ALL use the TLS proxy
or NONE use it.
```

This limitation reduces deployment flexibility and forces users into an all-or-nothing approach.

## Recommended Solution

Add an optional `on_reverse_proxy` field to each HTTP tracker configuration, allowing per-tracker control:

### Proposed Configuration Structure

```toml
[core.net]
on_reverse_proxy = false  # Default for trackers without explicit setting

[[http_trackers]]
bind_address = "0.0.0.0:7070"
on_reverse_proxy = true  # Override: this tracker is behind a proxy

[[http_trackers]]
bind_address = "0.0.0.0:7071"
on_reverse_proxy = true  # Override: this tracker is behind a proxy

[[http_trackers]]
bind_address = "0.0.0.0:7072"
# No override: uses global default (false) - direct access
```

### Behavior

1. If `on_reverse_proxy` is specified on an HTTP tracker, use that value
2. If not specified, fall back to `[core.net].on_reverse_proxy` (backward compatible)
3. Each HTTP tracker independently decides whether to read `X-Forwarded-For`

### Implementation Considerations

The HTTP tracker request handler would need to check its own `on_reverse_proxy` setting when extracting the client IP, rather than checking the global setting.

**Pseudocode change**:

```rust
// Before (global check)
fn get_client_ip(request: &Request, config: &Config) -> IpAddr {
    if config.core.net.on_reverse_proxy {
        extract_from_x_forwarded_for(request)
    } else {
        request.peer_addr()
    }
}

// After (per-tracker check)
fn get_client_ip(request: &Request, tracker_config: &HttpTrackerConfig) -> IpAddr {
    let on_reverse_proxy = tracker_config.on_reverse_proxy
        .unwrap_or(config.core.net.on_reverse_proxy);

    if on_reverse_proxy {
        extract_from_x_forwarded_for(request)
    } else {
        request.peer_addr()
    }
}
```

## Benefits of This Change

1. **Flexible deployments**: Mix proxied and direct HTTP trackers in one deployment
2. **Backward compatible**: Global setting remains the default
3. **Clearer intent**: Each tracker explicitly declares its network topology
4. **Better for edge cases**: Internal trackers (localhost) vs external (behind proxy)

## Use Cases Enabled

1. **Mixed TLS/non-TLS deployment**: Some trackers via HTTPS (Caddy), some via direct HTTP
2. **Internal monitoring**: Direct localhost tracker for Prometheus, proxied trackers for public access
3. **Gradual migration**: Move trackers behind proxy one at a time during migration
4. **Multi-tenant**: Different trackers for different networks with different proxy configurations

## References

- [Torrust Tracker Network Configuration Docs](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/v2_0_0/network/struct.Network.html)
- [Torrust Tracker Repository](https://github.com/torrust/torrust-tracker)
- [Deployer Issue #272 - Add HTTPS Support](https://github.com/torrust/torrust-tracker-deployer/issues/272)
- [Deployer PR #273 - HTTPS Implementation](https://github.com/torrust/torrust-tracker-deployer/pull/273)
