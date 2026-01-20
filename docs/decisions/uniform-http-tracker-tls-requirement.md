# Decision: Uniform TLS Requirement for HTTP Trackers

## Status

Accepted

## Date

2026-01-20

## Context

The deployer supports configuring multiple HTTP trackers in a single deployment:

```json
{
  "tracker": {
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "domain": "http1.example.com",
        "use_tls_proxy": true
      },
      {
        "bind_address": "0.0.0.0:7071",
        "domain": "http2.example.com",
        "use_tls_proxy": true
      },
      { "bind_address": "0.0.0.0:7072" }
    ]
  }
}
```

During HTTPS implementation, we discovered a **critical limitation** in the Torrust Tracker configuration: the `on_reverse_proxy` setting is **global**, not per-tracker.

### The Problem

When a tracker runs behind a reverse proxy (like Caddy), it needs to read `X-Forwarded-For` headers to determine the real client IP address. The tracker has a configuration option for this:

```toml
[core.net]
on_reverse_proxy = true  # Applies to ALL HTTP trackers
```

This creates a conflict:

- **Trackers behind Caddy**: Need `on_reverse_proxy = true` to read client IPs from headers
- **Direct HTTP trackers**: Need `on_reverse_proxy = false` to read client IPs from socket

With the global setting, you cannot mix these configurations.

### What Happens with Mixed Configuration

**If `on_reverse_proxy = true` (for proxied trackers)**:

- Direct HTTP trackers will look for `X-Forwarded-For` header
- Header won't exist for direct connections
- Client IP detection fails or uses wrong IP

**If `on_reverse_proxy = false` (for direct trackers)**:

- Proxied trackers will see Caddy's IP as the client IP
- All peers appear to come from the same IP (the proxy)
- BitTorrent swarm peer identification breaks

## Decision

We enforce a **uniform TLS configuration** for all HTTP trackers in a deployment:

> **If ANY HTTP tracker uses `use_tls_proxy: true`, ALL HTTP trackers MUST use `use_tls_proxy: true`.**

### Implementation

Validation is performed at environment creation time:

```rust
fn validate_uniform_http_tracker_tls(
    http_trackers: &[HttpTrackerSection],
) -> Result<(), CreateConfigError> {
    let any_uses_tls = http_trackers.iter().any(|t| t.use_tls_proxy.unwrap_or(false));
    let all_use_tls = http_trackers.iter().all(|t| t.use_tls_proxy.unwrap_or(false));

    if any_uses_tls && !all_use_tls {
        return Err(CreateConfigError::MixedHttpTrackerTls {
            message: "All HTTP trackers must use the same TLS proxy setting".into(),
            hint: "Either set use_tls_proxy: true for all trackers, or remove it from all".into(),
        });
    }

    Ok(())
}
```

### Error Message

When users attempt mixed configuration:

```text
Configuration Error: Mixed HTTP tracker TLS settings

All HTTP trackers must use the same TLS proxy configuration due to a
limitation in the Torrust Tracker's global `on_reverse_proxy` setting.

Current configuration:
  - http1.example.com (port 7070): use_tls_proxy = true
  - http2.example.com (port 7071): use_tls_proxy = true
  - Port 7072: use_tls_proxy = false

Fix: Either set use_tls_proxy: true for all HTTP trackers,
     or remove TLS proxy from all HTTP trackers.

See: docs/external-issues/tracker/on-reverse-proxy-global-setting.md
```

## Consequences

### Positive

- **Prevents silent failures**: Users get clear error instead of broken peer detection
- **Consistent behavior**: All trackers behave identically
- **Future-proof**: When tracker fixes the issue, we can remove this constraint

### Negative

- **Reduced flexibility**: Cannot mix HTTP and HTTPS trackers in same deployment
- **All-or-nothing**: Users must decide on proxy for entire tracker deployment

### Neutral

- **Workaround exists**: Users can run separate deployments for different TLS requirements

## Upstream Issue

We have filed an issue with the Torrust Tracker to request per-tracker `on_reverse_proxy` configuration:

**Issue**: [torrust/torrust-tracker#1640](https://github.com/torrust/torrust-tracker/issues/1640)

### Proposed Tracker Fix

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

When this is implemented in the tracker, we can:

1. Remove the uniform TLS validation
2. Generate per-tracker `on_reverse_proxy` settings
3. Allow mixed HTTP/HTTPS tracker configurations

## Alternatives Considered

### 1. Silent Acceptance (No Validation)

Allow mixed configuration and let it fail at runtime.

**Rejected because**:

- Peer IP detection would silently fail
- Hard to debug (symptoms appear in BitTorrent client behavior)
- Poor user experience

### 2. Auto-Proxy All Trackers

If any tracker needs TLS, automatically proxy all trackers.

**Rejected because**:

- Changes user intent without explicit consent
- Might not match user's network architecture
- Could expose internal trackers unintentionally

### 3. Warning Instead of Error

Allow mixed config with a warning.

**Rejected because**:

- Users would proceed and hit runtime issues
- Warning fatigue leads to ignoring important messages
- Fails principle of "actionable errors"

## Related Decisions

- [caddy-for-tls-termination.md](./caddy-for-tls-termination.md) - Why Caddy was chosen for TLS
- [per-service-tls-configuration.md](./per-service-tls-configuration.md) - Per-service TLS configuration pattern

## References

- [Issue #272 - Add HTTPS Support with Caddy](https://github.com/torrust/torrust-tracker-deployer/issues/272)
- [Upstream Issue: torrust/torrust-tracker#1640](https://github.com/torrust/torrust-tracker/issues/1640)
- [External Issue Documentation](../external-issues/tracker/on-reverse-proxy-global-setting.md)
- [Torrust Tracker Network Configuration](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/v2_0_0/network/struct.Network.html)
