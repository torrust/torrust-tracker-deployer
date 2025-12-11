# Port Zero Not Supported in Bind Addresses

**Status**: Accepted

**Date**: December 11, 2025

**Author**: Development Team

**Related Issues**: [#220]

---

## Context

The Torrust Tracker Deployer allows users to configure tracker services with bind addresses (e.g., `0.0.0.0:6969` for UDP tracker, `0.0.0.0:7070` for HTTP tracker). These bind addresses are used throughout the deployment lifecycle:

1. **Environment Creation (`create` command)**: Configuration is validated and stored
2. **Configuration (`configure` command)**: Firewall rules are established based on specified ports
3. **Software Release (`release` command)**: Tracker is configured with bind addresses
4. **Service Execution (`run` command)**: Tracker services are started with configured ports

### The Port Zero Problem

Port `0` is a special value in network programming that means "let the operating system assign any available ephemeral port dynamically." While this is useful for applications where the specific port doesn't matter, it creates significant challenges in our deployment workflow:

**Firewall Configuration Conflict**: The `configure` command must open specific firewall ports **before** the tracker starts. With port `0`, we don't know which port the OS will assign until the tracker actually starts, creating a chicken-and-egg problem:

- We can't configure the firewall without knowing the port
- We can't start the tracker without opening the firewall
- We can't know the port without starting the tracker

**User Expectations**: Users specify bind addresses expecting those exact ports to be used consistently across:

- Firewall rules (UFW configuration)
- Service configuration (tracker TOML files)
- Health checks (validation commands)
- External access (port forwarding, client connections)

Dynamic port assignment would break this expectation and make the system unpredictable.

## Decision

We **explicitly reject port 0** in all tracker bind address configurations. This validation occurs at the **DTO-to-Domain boundary** when converting `TrackerSection` (application layer DTO) to `TrackerConfig` (domain type).

### Implementation Location

Validation is performed in the conversion methods of each tracker section:

- `UdpTrackerSection::to_udp_tracker_config()`
- `HttpTrackerSection::to_http_tracker_config()`
- `HttpApiSection::to_http_api_config()`

### Error Handling

When port 0 is detected, we return a clear, actionable error:

```rust
CreateConfigError::DynamicPortNotSupported {
    bind_address: "0.0.0.0:0".to_string(),
}
```

The error message includes:

- What went wrong (dynamic port assignment not supported)
- Why it's not supported (conflicts with firewall configuration)
- How to fix it (specify an explicit port number)

## Consequences

### Positive

1. **Predictable Deployment**: Users know exactly which ports will be used
2. **Consistent Configuration**: Same ports across all deployment phases
3. **Firewall Compatibility**: Can configure firewall rules before service starts
4. **Clear Documentation**: Users understand port requirements upfront
5. **Fail Fast**: Errors appear at environment creation, not during service startup

### Negative

1. **Port Conflicts**: Users must manually choose available ports
2. **Multi-Instance Deployments**: Each instance needs unique ports

### Neutral

1. **Validation Overhead**: Minimal - single integer comparison per bind address
2. **Test Coverage**: Requires additional test cases for port 0 rejection

## Alternatives Considered

### Alternative 1: Support Dynamic Ports with Runtime Discovery

**Approach**: Allow port 0, then discover the assigned port after service starts.

**How It Would Work**:

1. User specifies port 0 in configuration
2. Tracker starts and OS assigns ephemeral port
3. Parse Docker container logs or query Docker port mappings
4. Extract dynamically assigned port
5. Update firewall rules with discovered port

**Rejected Because**:

- Adds significant complexity to the deployment workflow
- Creates timing dependencies (must wait for service to start before configuring firewall)
- Breaks the "configure before deploy" model
- Requires Docker-specific inspection logic
- Makes health checks and validation more complex
- Could be revisited in future if there's strong user demand

### Alternative 2: Auto-Assign Sequential Ports

**Approach**: If port 0 is specified, automatically assign the next available port from a predefined range.

**Rejected Because**:

- Requires port availability checking across potentially remote systems
- Introduces race conditions in multi-deployment scenarios
- Hides port selection from users, reducing transparency
- Adds complexity without clear benefits

### Alternative 3: Port Range Specification

**Approach**: Allow users to specify port ranges (e.g., `6969-6979`) and pick the first available.

**Rejected Because**:

- More complex than current single-port model
- Still requires availability checking
- Doesn't solve the fundamental firewall configuration problem
- Adds unnecessary flexibility for most use cases

## Implementation Notes

### Where Validation Happens

```text
JSON Configuration (String)
  ↓
TrackerSection (DTO with String bind_address)
  ↓
[VALIDATION POINT - Reject port 0]
  ↓
TrackerConfig (Domain with SocketAddr bind_address)
```

### Example Error Output

```text
Error: Dynamic port assignment (port 0) is not supported in bind address '0.0.0.0:0'

Why: Port 0 tells the OS to assign any available port dynamically. This conflicts
with our firewall configuration which needs to know exact ports before services start.

Solution: Specify an explicit port number in your configuration:
  - UDP Tracker: Use a port like 6969 (default)
  - HTTP Tracker: Use a port like 7070 (default)
  - HTTP API: Use a port like 1212 (default)

Example:
  "udp_trackers": [
    { "bind_address": "0.0.0.0:6969" }  ← Explicit port, not 0
  ]
```

## Future Considerations

If there's strong user demand for dynamic port assignment:

1. Could implement runtime port discovery as an optional feature
2. Would require:
   - Docker port mapping inspection
   - Delayed firewall configuration
   - Updated health check logic
   - Clear documentation of limitations
3. Would be a **separate feature**, not a change to current behavior

For now, the explicit port requirement provides the best balance of:

- Simplicity
- Predictability
- Compatibility with existing deployment workflow

## References

- [Issue #220]: Tracker Slice - Release and Run Commands
- `docs/implementation-plans/issue-220-test-command-architecture.md`: Implementation plan
- `docs/contributing/error-handling.md`: Error handling principles
- [UFW Documentation](https://help.ubuntu.com/community/UFW): Firewall configuration

---

**Decision Made**: December 11, 2025  
**Last Updated**: December 11, 2025
