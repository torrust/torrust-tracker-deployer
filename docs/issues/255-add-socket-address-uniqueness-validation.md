# Add Socket Address Uniqueness Validation for Tracker Configuration

**Issue**: #255
**Parent Epic**: N/A (Independent task)
**Related**:

- [docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md](../external-issues/tracker/udp-tcp-port-sharing-allowed.md)
- Test configuration: `envs/bug-test-actual-conflict.json`

- [docs/contributing/error-handling.md](../contributing/error-handling.md)

## Overview

Add validation to prevent invalid tracker configurations where multiple services attempt to bind to the same socket address (IP + Port + Protocol). The deployer currently accepts configurations that cause runtime failures when services cannot bind to already-occupied ports.

**Problem**: Users can create environment configurations with port conflicts that pass validation during `create environment` but fail at runtime during `run`, causing tracker container restart loops with unclear error messages.

**Solution**: Implement socket address uniqueness validation at configuration load time with clear, actionable error messages.

## Goals

- [ ] Prevent same-protocol port conflicts (e.g., two HTTP trackers on same IP:Port)
- [ ] Prevent TCP protocol conflicts (e.g., HTTP tracker + API on same port)
- [ ] Allow cross-protocol port sharing (e.g., UDP + HTTP on same port) with clear understanding
- [ ] Provide clear, actionable error messages when validation fails
- [ ] Fail fast during `create environment` rather than at runtime

## 🏗️ Architecture Requirements

**DDD Layer**: Domain
**Module Path**: `src/domain/tracker/`
**Pattern**: Value Object with validation logic

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Validation logic belongs in domain layer as business rule
- [ ] Error types defined in domain layer
- [ ] Use value objects for bind addresses with built-in validation

### Architectural Constraints

- [ ] Validation must occur in domain layer (business rule)
- [ ] Configuration DTOs in application layer convert to domain types
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Clear, actionable error messages with fix instructions

### Anti-Patterns to Avoid

- ❌ Validation logic in presentation or infrastructure layers
- ❌ Generic "invalid configuration" errors without context
- ❌ Runtime errors that could be caught at config load time
- ❌ Cryptic error messages without actionable guidance

## Specifications

### Socket Address Uniqueness Rules

A **socket address** is uniquely identified by: **IP + Port + Protocol**

```text
Socket Address = (IP Address, Port Number, Protocol Type)

```

### Validation Rules

#### Rule 1: Same Protocol + Same IP:Port = INVALID

Two services using the **same protocol** CANNOT bind to the same IP:Port combination.

**Invalid Configurations:**

```toml
# INVALID: Two UDP trackers on same IP:Port
[[udp_trackers]]
bind_address = "0.0.0.0:7070"

[[udp_trackers]]
bind_address = "0.0.0.0:7070"  # ❌ Conflict: Same protocol (UDP), same socket

```

```toml
# INVALID: Two HTTP trackers on same IP:Port
[[http_trackers]]
bind_address = "0.0.0.0:7070"

[[http_trackers]]
bind_address = "0.0.0.0:7070"  # ❌ Conflict: Same protocol (TCP), same socket

```

```toml
# INVALID: HTTP tracker + API (both TCP)
[[http_trackers]]
bind_address = "0.0.0.0:7070"

[http_api]
bind_address = "0.0.0.0:7070"  # ❌ Conflict: Both use TCP, same socket

```

**Expected Error:**

```text
Configuration validation failed: Socket address conflict detected

Conflict: Multiple services attempting to bind to the same address

- HTTP Tracker: 0.0.0.0:7070 (TCP)
- REST API: 0.0.0.0:7070 (TCP)

Why this fails:
Two services using the same protocol (TCP) cannot bind to the same
IP address and port number. The second service will fail with
"Address already in use" error.

Fix:
Assign different port numbers to each service:

[[http_trackers]]
bind_address = "0.0.0.0:7070"

[http_api]
bind_address = "0.0.0.0:1212"  # Use a different port

Related: docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md

```

#### Rule 2: Different Protocols = VALID (but warn user)

Services using **different protocols** (UDP vs TCP) CAN share the same port number.

```toml
# VALID: UDP and HTTP on same port (different protocols)
[[udp_trackers]]
bind_address = "0.0.0.0:7070"  # UDP protocol

[[http_trackers]]
bind_address = "0.0.0.0:7070"  # TCP protocol - VALID but unusual

```

**Recommendation**: While technically valid, this configuration may be confusing. Consider showing a warning:

```text
⚠️  Warning: Port sharing detected across different protocols

Configuration:

- UDP Tracker: 0.0.0.0:7070 (UDP protocol)
- HTTP Tracker: 0.0.0.0:7070 (TCP protocol)

This configuration is valid because UDP and TCP use separate port
spaces in the operating system. However, it may be confusing for
documentation and firewall management.

Consider: Use different ports for clarity

- UDP Tracker: 0.0.0.0:6969
- HTTP Tracker: 0.0.0.0:7070

To suppress this warning: Set TORRUST_TD_ALLOW_CROSS_PROTOCOL_PORT_SHARING=true

Related: docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md

```

#### Rule 3: Same Port + Different IPs = VALID

Services can bind to the same port if they use **different IP addresses**.

```toml
# VALID: Same port, different IP addresses
[[http_trackers]]
bind_address = "192.168.1.10:7070"

[[http_trackers]]
bind_address = "192.168.1.20:7070"  # ✅ Different IP address

```

### Protocol Mapping

**TCP Protocol Services:**

- HTTP Trackers (`[[http_trackers]]`)
- REST API (`[http_api]`)

- Health Check API (internal, port 1313)

**UDP Protocol Services:**

- UDP Trackers (`[[udp_trackers]]`)

### Test Cases

#### Test Case 1: Duplicate UDP Tracker Ports

```json
{
  "tracker": {
    "udp_trackers": [
      { "bind_address": "0.0.0.0:7070" },
      { "bind_address": "0.0.0.0:7070" }
    ]
  }
}
```

**Expected**: ❌ Validation Error - "Duplicate socket address for UDP trackers"

#### Test Case 2: HTTP Tracker + API Conflict

```json
{
  "tracker": {
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "bind_address": "0.0.0.0:7070"
    }
  }
}
```

**Expected**: ❌ Validation Error - "TCP port conflict between HTTP tracker and API"
**Actual Bug**: Currently accepted, fails at runtime with tracker restart loop

#### Test Case 3: UDP + HTTP Same Port

```json
{
  "tracker": {
    "udp_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }]
  }
}
```

**Expected**: ⚠️ Warning (or accept with env var) - "Cross-protocol port sharing detected"
**Actual**: Works correctly (documented in `docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md`)

#### Test Case 4: Different Ports

```json
{
  "tracker": {
    "udp_trackers": [{ "bind_address": "0.0.0.0:6969" }],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "bind_address": "0.0.0.0:1212"
    }
  }
}
```

**Expected**: ✅ Pass validation - All unique socket addresses

#### Test Case 5: Same Port, Different IPs

```json
{
  "tracker": {
    "http_trackers": [
      { "bind_address": "192.168.1.10:7070" },
      { "bind_address": "192.168.1.20:7070" }
    ]
  }
}
```

**Expected**: ✅ Pass validation - Different IP addresses

### Error Message Format

Follow the project's error handling conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md)):

1. **What went wrong**: Clear description of the conflict
2. **Why it fails**: Technical explanation
3. **How to fix**: Specific, actionable instructions
4. **Related docs**: Links to relevant documentation

## Implementation Plan

**Development Approach**: Follow Test-Driven Development (TDD) - write tests first, then implement minimum code to pass tests.

### Phase 1: Domain Model Enhancement (2-3 hours)

- [ ] Task 1.1: Create `Protocol` enum in `src/domain/tracker/protocol.rs`

  - **Test first**: Write unit tests for Protocol enum (FromStr, Display, equality)
  - **Then implement**:

  ```rust
  pub enum Protocol {
      Udp,
      Tcp,
  }

  // Error type placed WITH the Protocol enum
  pub enum ProtocolParseError {
      UnknownProtocol(String),
  }
  ```

- [ ] Task 1.2: Create `BindingAddress` value object in `src/domain/tracker/binding_address.rs`

  - **Test first**: Write unit tests for parsing, validation, and equality
  - **Then implement**:

  ```rust
  use std::net::SocketAddr;

  pub struct BindingAddress {
      socket: SocketAddr,  // Use standard library type
      protocol: Protocol,
  }

  // Error type placed WITH the BindingAddress
  pub enum BindingAddressError {
      InvalidSocketAddr(String),
      InvalidProtocol(ProtocolParseError),
  }
  ```

### Phase 2: Validation Logic (3-4 hours)

- [ ] Task 2.1: Add validation to `TrackerConfig` in `src/domain/tracker/config.rs`

  - **Test first**: Write unit tests for all validation scenarios:
    - Same protocol conflicts (UDP-UDP, HTTP-HTTP, HTTP-API)
    - Cross-protocol sharing (UDP-HTTP) - should pass
    - Different IPs same port - should pass
    - All unique addresses - should pass
  - **Then implement** validation method directly in `TrackerConfig`:

  ```rust
  // Error placed WITH TrackerConfig
  pub enum TrackerConfigError {
      DuplicateSocketAddress {
          address: SocketAddr,
          protocol: Protocol,
          services: Vec<String>,
      },
  }

  impl TrackerConfig {
      pub fn validate(&self) -> Result<(), TrackerConfigError> {
          // Collect all binding addresses with service names
          // Group by (SocketAddr, Protocol)
          // Check for duplicates
          // Return detailed error if conflicts found
      }
  }
  ```

- [ ] Task 2.2: Add validation call in `EnvironmentCreationConfig::try_from()`
  - **Test first**: Write integration test with invalid config
  - **Then implement**: Call `tracker_config.validate()` during DTO conversion
  - Return validation errors before creating environment

### Phase 3: Error Messages (1-2 hours)

- [ ] Task 3.1: Implement error formatting

  - **Test first**: Write tests verifying error message format
  - **Then implement** `Display` for all error types:
    - `TrackerConfigError`: What/Why/How format
    - `BindingAddressError`: Parse/validation errors
    - `ProtocolParseError`: Protocol parsing errors
  - Include specific conflicting addresses
  - List all affected services
  - Provide fix examples

- [ ] Task 3.2: Add context to error propagation
  - **Test first**: Integration test verifying error context
  - **Then implement**: Ensure errors bubble up with full context
  - Include trace IDs for debugging

### Phase 4: Documentation (1 hour)

- [ ] Task 5.1: Update user documentation

  - Add validation rules to `docs/user-guide/configuration.md`
  - Document error messages and fixes
  - Add examples of valid/invalid configurations

- [ ] Task 5.2: Update JSON schema
  - Consider adding validation hints in schema comments
  - Update `schemas/environment-config.json` if needed

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] Same-protocol port conflicts are rejected during `create environment`
- [ ] Error messages follow What/Why/How format with actionable fixes
- [ ] Cross-protocol port sharing either warns or requires explicit env var
- [ ] Different IP addresses with same port are allowed
- [ ] All test cases pass (see Specifications section)

**Code Quality**:

- [ ] Validation logic in domain layer (not presentation/infrastructure)
- [ ] Error types placed with the types that produce them
- [ ] Value objects used for type safety (BindingAddress using std::net::SocketAddr, Protocol)
- [ ] Comprehensive unit tests for all validation rules (TDD approach)
- [ ] Integration tests verify early failure during `create environment`
- [ ] Unit tests cover all test cases from Specifications section

**User Experience**:

- [ ] Errors appear immediately during `create environment`
- [ ] Error messages clearly identify conflicting services
- [ ] Fix instructions are specific and actionable
- [ ] Related documentation linked in error output
- [ ] No cryptic "port already in use" errors at runtime

**Documentation**:

- [ ] Validation rules documented in user guide
- [ ] Error message examples in documentation
- [ ] Cross-reference to `docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md`

- [ ] ADR created if architectural decisions made

## Related Documentation

- [docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md](../external-issues/tracker/udp-tcp-port-sharing-allowed.md) - Tracker behavior with UDP/TCP port sharing
- [docs/codebase-architecture.md](../codebase-architecture.md) - DDD layer structure
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error message conventions

- [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md) - Where to place validation logic
- [docs/user-guide/configuration.md](../user-guide/configuration.md) - User-facing configuration guide

## Notes

### Why This Bug Matters

**Current Impact:**

- Users create configurations that appear valid
- Environment creation succeeds
- All deployment steps complete (provision, configure, release, run)
- Tracker container enters restart loop at runtime
- Error appears in container logs, not in deployer output
- Difficult to diagnose - requires SSH + docker logs inspection

**After Fix:**

- Immediate failure during `create environment`
- Clear error message with conflicting services
- Specific fix instructions
- No wasted time deploying invalid configuration

### Design Decisions

**Why Domain Layer:**
Socket address uniqueness is a **business rule** - multiple services cannot bind to the same address within the same protocol. This belongs in the domain layer, not as infrastructure validation.

**Why Value Objects:**
Using `SocketAddress` and `Protocol` value objects provides type safety and encapsulates parsing/validation logic, preventing invalid states from being constructed.

**Why Warn for Cross-Protocol:**
UDP and TCP CAN share port numbers legally, but it's unusual and potentially confusing. A warning educates users without preventing valid configurations.

### Testing Evidence

**Bug Reproduction:**

1. Created test config: `envs/bug-test-actual-conflict.json`
2. Configuration: HTTP tracker + API both on `0.0.0.0:7070`
3. `create environment`: ✅ Succeeded (BUG - should fail here)
4. `provision`: ✅ Succeeded
5. `configure`: ✅ Succeeded
6. `release`: ✅ Succeeded
7. `run`: ✅ Succeeded (but tracker enters restart loop)
8. `docker compose ps`: Shows `Restarting (101)`
9. `docker logs tracker`: Shows "Address already in use" error

**Root Cause:**
Deployer has no validation for socket address uniqueness, allowing invalid configurations to pass through all stages until runtime failure.

### Future Enhancements

Consider adding:

- Environment variable `TORRUST_TD_ALLOW_CROSS_PROTOCOL_PORT_SHARING=true` to suppress warning
- JSON schema validation hints (though this requires schema language support)
- Configuration linter command: `torrust-tracker-deployer lint <env-file>`
- Validation mode flags: `--strict` (reject all port sharing) vs `--permissive` (allow cross-protocol)
