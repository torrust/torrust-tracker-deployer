# Tracker Allows UDP and TCP Trackers on Same Port

**Issue Date**: December 23, 2025  
**Affected Component**: Torrust Tracker Application (`torrust/tracker:develop`)  
**Status**: Documented - Behavior by design (UDP and TCP are different protocols)

## Problem Description

The Torrust Tracker application **allows** both a UDP tracker and an HTTP (TCP) tracker to bind to the same port number when using the wildcard IP address (`0.0.0.0`).

Example configuration that is accepted:

```toml
[[udp_trackers]]
bind_address = "0.0.0.0:7070"

[[http_trackers]]
bind_address = "0.0.0.0:7070"
```

Both trackers start successfully:

```text
2025-12-23T16:06:08.094597Z  INFO UDP TRACKER: Starting on: 0.0.0.0:7070
2025-12-23T16:06:08.094660Z  INFO UDP TRACKER: Started on: udp://0.0.0.0:7070
2025-12-23T16:06:08.094818Z  INFO HTTP TRACKER: Starting on: http://0.0.0.0:7070
2025-12-23T16:06:08.094894Z  INFO HTTP TRACKER: Started on: http://0.0.0.0:7070
```

## Root Cause

This behavior is **technically correct** from an operating system perspective:

- **UDP and TCP are different transport protocols**
- The OS maintains separate port spaces for UDP and TCP
- A UDP socket on port 7070 does not conflict with a TCP socket on port 7070
- Both can coexist and operate independently

The tracker application simply requests socket bindings from the OS, and the OS allows both because they use different protocol stacks.

## Impact

### Positive Impact

- Technically valid configuration that works correctly
- Allows advanced users to intentionally share port numbers across protocols
- No runtime errors or crashes

### Potential Confusion

While technically valid, this configuration is **likely unintentional** in most use cases:

1. **User Intent**: Users typically expect different services to use different port numbers
2. **Port Management**: Makes it harder to manage and document which services use which ports
3. **Firewall Rules**: May complicate firewall configurations when UDP and TCP use same port
4. **Monitoring**: Can be confusing when monitoring port usage and service health
5. **Documentation**: Requires careful documentation to explain which protocol uses which port

### When This is Valid

Scenarios where sharing ports across protocols makes sense:

- **Testing**: Quick testing with limited port ranges
- **Cloud Environments**: Some cloud providers have port restrictions
- **Container Environments**: Port mapping limitations in container orchestration
- **Intentional Design**: User specifically wants to use same port for both protocols

## Current Behavior in Deployer

The deployer currently **allows** this configuration because:

1. The tracker accepts it without error
2. Both services start and run successfully
3. No deployment failures occur
4. The configuration is technically valid

## Recommended Approach

### For the Tracker Repository

**No changes recommended** - the current behavior is correct. UDP and TCP are different protocols and can legitimately share port numbers.

If the tracker maintainers want to prevent this, they could add an optional validation check with a configuration flag to warn users about port sharing across protocols.

### For the Deployer Repository

**Consider adding validation** with appropriate context:

1. **Strict Mode** (default): Prevent same port across any tracker types

   - Reject: UDP tracker + HTTP tracker on same port
   - Reject: UDP tracker + API on same port
   - Reject: HTTP tracker + API on same port
   - Allow: Different IPs on same port (e.g., `192.168.1.10:7070` + `192.168.1.20:7070`)

2. **Permissive Mode** (opt-in via flag): Allow port sharing across different protocols

   - Allow: UDP tracker + HTTP tracker on same port (different protocols)
   - Reject: Two UDP trackers on same port (same protocol)
   - Reject: Two HTTP trackers on same port (same protocol)
   - Reject: HTTP tracker + API on same port (both are TCP)

3. **Warning Mode**: Accept the configuration but warn the user
   - Display informational message about port sharing
   - Suggest different ports for clarity
   - Proceed with deployment

## Validation Rules

The deployer should validate socket address uniqueness based on:

```text
Socket Address = IP + Port + Protocol
```

### Invalid Configurations (Same Protocol + Same Socket)

❌ **Two UDP trackers on same IP:Port**:

```toml
[[udp_trackers]]
bind_address = "0.0.0.0:7070"

[[udp_trackers]]
bind_address = "0.0.0.0:7070"  # INVALID: Same protocol, same IP, same port
```

❌ **Two HTTP trackers on same IP:Port**:

```toml
[[http_trackers]]
bind_address = "0.0.0.0:7070"

[[http_trackers]]
bind_address = "0.0.0.0:7070"  # INVALID: Same protocol, same IP, same port
```

❌ **HTTP tracker and API on same IP:Port** (both use TCP):

```toml
[[http_trackers]]
bind_address = "0.0.0.0:7070"

[http_api]
bind_address = "0.0.0.0:7070"  # INVALID: Both are TCP, same IP, same port
```

### Valid Configurations

✅ **UDP and HTTP on same port** (different protocols):

```toml
[[udp_trackers]]
bind_address = "0.0.0.0:7070"  # UDP protocol

[[http_trackers]]
bind_address = "0.0.0.0:7070"  # TCP protocol - VALID but potentially confusing
```

✅ **Same port, different IPs**:

```toml
[[http_trackers]]
bind_address = "192.168.1.10:7070"

[[http_trackers]]
bind_address = "192.168.1.20:7070"  # VALID: Different IP addresses
```

✅ **Different ports, same protocol**:

```toml
[[http_trackers]]
bind_address = "0.0.0.0:7070"

[[http_trackers]]
bind_address = "0.0.0.0:8080"  # VALID: Different ports
```

## Testing Evidence

### Test Configuration

File: `envs/bug-test-duplicate-port.json`

```json
{
  "tracker": {
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ]
  }
}
```

### Test Results

**Deployment**: ✅ SUCCESS

- Provision: Completed
- Configure: Completed
- Release: Completed
- Run: Completed

**Tracker Startup**: ✅ SUCCESS

```bash
$ docker logs tracker 2>&1 | grep -E "(UDP TRACKER|HTTP TRACKER).*Started"
2025-12-23T16:06:08.094660Z  INFO UDP TRACKER: Started on: udp://0.0.0.0:7070
2025-12-23T16:06:08.094894Z  INFO HTTP TRACKER: Started on: http://0.0.0.0:7070
```

**Service Status**: ✅ HEALTHY

```bash
$ docker compose ps
NAME      STATUS
tracker   Up 2 minutes (healthy)
```

**Health Checks**: ✅ PASSING

- UDP tracker responding on port 7070
- HTTP tracker responding on port 7070
- No port binding conflicts
- No application errors

## References

- [Tracker Configuration Schema](https://github.com/torrust/torrust-tracker/blob/develop/docs/config.md)
- [UDP Tracker Implementation](https://github.com/torrust/torrust-tracker/tree/develop/packages/udp-tracker)
- [HTTP Tracker Implementation](https://github.com/torrust/torrust-tracker/tree/develop/packages/http-tracker)
- [OS Socket Binding Documentation](https://man7.org/linux/man-pages/man2/bind.2.html)
- Test Environment: `envs/bug-test-duplicate-port.json`
- Test Date: December 23, 2025

## Conclusion

The tracker's behavior is **correct by design** - UDP and TCP can share port numbers because they are different protocols. However, the deployer **should add validation** to prevent potentially confusing configurations, especially for users who may not understand the protocol-level distinction.

The validation should focus on preventing:

1. Same protocol + same IP + same port (actual conflicts)
2. Optionally warn about cross-protocol port sharing (clarity)
