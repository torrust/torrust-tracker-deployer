# Add Domain Support for UDP Trackers

**Issue**: [#279](https://github.com/torrust/torrust-tracker-deployer/issues/279)
**Parent Epic**: None (standalone improvement)
**Related**:

- HTTP tracker configuration: `src/application/command_handlers/create/config/tracker/http_tracker_section.rs`
- UDP tracker configuration: `src/application/command_handlers/create/config/tracker/udp_tracker_section.rs`
- Domain tracker config: `src/domain/tracker/config/udp.rs`
- JSON Schema: `schemas/environment-config.json`
- Live Demo: `udp://tracker.torrust-demo.com:6969/announce`

## Overview

Add optional `domain` field support for UDP trackers in the environment configuration. Currently, UDP trackers only accept a `bind_address` without a domain, while HTTP trackers support both `bind_address` and `domain` fields. This is not a technical limitation - UDP trackers can perfectly work with domains, as demonstrated by the Torrust Live Demo (`udp://tracker.torrust-demo.com:6969/announce`).

Adding domain support for UDP trackers enables:

1. **Documentation consistency**: The announce URL can use a domain instead of an IP
2. **External communication**: When showing users the tracker URL, a domain is more user-friendly
3. **Future integrations**: Domain-based configuration for monitoring, documentation generation, etc.

## Goals

- [ ] Add optional `domain` field to UDP tracker configuration
- [ ] Update application DTO (`UdpTrackerSection`) to parse domain
- [ ] Update domain model (`UdpTrackerConfig`) to store domain
- [ ] Update JSON schema (regenerate with `cargo run -- create schema`)
- [ ] Add validation for domain format
- [ ] Update `show` command to display domain-based URLs when configured
- [ ] Update manual test environment to use domain for UDP tracker

## 🏗️ Architecture Requirements

**DDD Layer**: Application (DTO) + Domain (config model)

**Module Paths**:

- `src/application/command_handlers/create/config/tracker/udp_tracker_section.rs` - DTO parsing
- `src/domain/tracker/config/udp.rs` - Domain model
- `schemas/environment-config.json` - JSON schema

**Pattern**: Value Object (using existing `DomainName` type from `src/shared/`)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Reuse existing `DomainName` value object from `src/shared/`
- [ ] Mirror the pattern used by `HttpTrackerSection` for domain handling
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Domain field is optional (backward compatible)
- [ ] Domain validation uses existing `DomainName::new()` validator
- [ ] No TLS proxy support for UDP (UDP doesn't support TLS like HTTP does)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Adding `use_tls_proxy` field for UDP (TLS not applicable to UDP protocol)
- ❌ Making domain required (would break existing configurations)
- ❌ Duplicating domain validation logic (reuse existing `DomainName`)
- ❌ Creating a new domain type instead of reusing `DomainName`

## Specifications

### Current UDP Tracker Configuration

**Current structure** (from `envs/manual-https-test.json`):

```json
"udp_trackers": [
    {
        "bind_address": "0.0.0.0:6969"
    }
]
```

### Proposed UDP Tracker Configuration

**New structure** (with optional domain):

```json
"udp_trackers": [
    {
        "bind_address": "0.0.0.0:6969",
        "domain": "udp.tracker.local"
    }
]
```

### Application Layer Changes

**File**: `src/application/command_handlers/create/config/tracker/udp_tracker_section.rs`

Update `UdpTrackerSection` struct:

```rust
use crate::shared::DomainName;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct UdpTrackerSection {
    pub bind_address: String,

    /// Domain name for the UDP tracker (optional)
    ///
    /// When present, this domain can be used in announce URLs instead of the IP.
    /// Example: `udp://tracker.example.com:6969/announce`
    ///
    /// Note: Unlike HTTP trackers, UDP does not support TLS, so there is no
    /// `use_tls_proxy` field for UDP trackers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

impl UdpTrackerSection {
    pub fn to_udp_tracker_config(&self) -> Result<UdpTrackerConfig, CreateConfigError> {
        // ... existing bind_address validation ...

        // Convert domain to domain type with validation (if present)
        let domain = match &self.domain {
            Some(domain_str) => {
                let domain = DomainName::new(domain_str)
                    .map_err(|e| CreateConfigError::InvalidDomain {
                        domain: domain_str.clone(),
                        reason: e.to_string(),
                    })?;
                Some(domain)
            }
            None => None,
        };

        Ok(UdpTrackerConfig { bind_address, domain })
    }
}
```

### Domain Layer Changes

**File**: `src/domain/tracker/config/udp.rs`

Update `UdpTrackerConfig` struct:

```rust
use crate::shared::DomainName;

/// UDP tracker bind configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UdpTrackerConfig {
    /// Bind address (e.g., "0.0.0.0:6868")
    #[serde(
        serialize_with = "crate::domain::tracker::config::serialize_socket_addr",
        deserialize_with = "crate::domain::tracker::config::deserialize_socket_addr"
    )]
    pub bind_address: SocketAddr,

    /// Domain name for announce URLs (optional)
    ///
    /// When present, this domain can be used when communicating the tracker's
    /// announce URL to users, e.g., `udp://tracker.example.com:6969/announce`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<DomainName>,
}
```

### JSON Schema Update

**File**: `schemas/environment-config.json`

The schema is auto-generated from the Rust types using `schemars`. To update:

```bash
cargo run --bin torrust-tracker-deployer -- create schema > schemas/environment-config.json
```

The generated schema will automatically include the new `domain` field from `UdpTrackerSection`.

### Show Command Update

**File**: `src/application/command_handlers/show/info/tracker.rs`

Update the UDP tracker URL generation to use domain when available:

```rust
let udp_trackers = tracker_config
    .udp_trackers
    .iter()
    .map(|udp| {
        // Use domain if configured, otherwise fall back to IP
        let host = udp.domain
            .as_ref()
            .map(|d| d.as_str().to_string())
            .unwrap_or_else(|| instance_ip.to_string());
        format!("udp://{}:{}/announce", host, udp.bind_address.port())
    })
    .collect();
```

#### Current Output (Before)

When running `show` command with a UDP tracker configured without domain:

```text
Tracker Services:
  UDP Trackers:
    - udp://10.140.190.124:6969/announce
  HTTP Trackers (HTTPS via Caddy):
    - https://http1.tracker.local/announce
    - https://http2.tracker.local/announce
```

Note: HTTP trackers with domains show the domain, but UDP trackers always show the IP.

#### Expected Output (After)

When running `show` command with a UDP tracker configured **with** domain:

```text
Tracker Services:
  UDP Trackers:
    - udp://udp.tracker.local:6969/announce
  HTTP Trackers (HTTPS via Caddy):
    - https://http1.tracker.local/announce
    - https://http2.tracker.local/announce
```

When running `show` command with a UDP tracker configured **without** domain (backward compatible):

```text
Tracker Services:
  UDP Trackers:
    - udp://10.140.190.124:6969/announce
```

### Manual Test Environment Update

**File**: `envs/manual-https-test.json`

Update to demonstrate the new domain field:

```json
"udp_trackers": [
    {
        "bind_address": "0.0.0.0:6969",
        "domain": "udp.tracker.local"
    }
]
```

### Test Command Analysis

**No changes required for test command.**

The `test` command only validates HTTP endpoints (HTTP API and HTTP trackers) for health checks:

```text
⏳ [3/3] Testing infrastructure...
⏳   ✓ Infrastructure tests passed (took 45ms)
✅ Infrastructure validation completed successfully for 'manual-https-test'
```

UDP testing is not included because:

1. UDP protocol doesn't have a standard health check mechanism like HTTP's `/health_check`
2. Testing UDP would require sending/receiving tracker protocol messages
3. This is out of scope for the smoke test purpose of the `test` command

## Implementation Plan

### Phase 1: Domain Layer Update (15 min)

- [ ] Task 1.1: Add `domain: Option<DomainName>` field to `UdpTrackerConfig` in `src/domain/tracker/config/udp.rs`
- [ ] Task 1.2: Update serialization tests in domain layer
- [ ] Task 1.3: Add unit test for domain field serialization/deserialization

### Phase 2: Application Layer Update (20 min)

- [ ] Task 2.1: Add `domain: Option<String>` field to `UdpTrackerSection` in `src/application/command_handlers/create/config/tracker/udp_tracker_section.rs`
- [ ] Task 2.2: Update `to_udp_tracker_config()` to parse and validate domain
- [ ] Task 2.3: Add unit tests for domain validation (valid domain, invalid domain, missing domain)
- [ ] Task 2.4: Update any places where `UdpTrackerConfig` is constructed

### Phase 3: Show Command Update (15 min)

- [ ] Task 3.1: Update `ServiceInfo::from_tracker_config()` in `src/application/command_handlers/show/info/tracker.rs`
- [ ] Task 3.2: Use domain for UDP tracker URL when configured, fall back to IP otherwise
- [ ] Task 3.3: Add unit tests for domain-based UDP tracker URLs

### Phase 4: Schema Regeneration (5 min)

- [ ] Task 4.1: Regenerate JSON schema: `cargo run -- create schema > schemas/environment-config.json`
- [ ] Task 4.2: Verify the schema includes the new domain field

### Phase 5: Manual Test Update (5 min)

- [ ] Task 5.1: Update `envs/manual-https-test.json` to include domain for UDP tracker
- [ ] Task 5.2: Verify configuration loads correctly

### Phase 6: Verification (15 min)

- [ ] Task 6.1: Run `./scripts/pre-commit.sh` - all checks must pass
- [ ] Task 6.2: Run manual E2E test with updated configuration
- [ ] Task 6.3: Verify `show` command displays domain-based UDP tracker URL
- [ ] Task 6.4: Verify UDP tracker works with domain in configuration

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `UdpTrackerSection` has optional `domain` field with proper documentation
- [ ] `UdpTrackerConfig` (domain) has optional `domain` field
- [ ] Domain validation uses existing `DomainName` type (no duplication)
- [ ] Invalid domains produce clear error messages
- [ ] Missing domain (None) is handled correctly (backward compatible)
- [ ] JSON schema regenerated and includes the new field
- [ ] `show` command displays domain-based URL when domain is configured
- [ ] `show` command displays IP-based URL when domain is not configured (backward compatible)
- [ ] At least one manual test environment demonstrates the domain field
- [ ] Unit tests cover: valid domain, invalid domain, missing domain scenarios
- [ ] Unit tests cover: show command with and without domain
- [ ] No `use_tls_proxy` field added (UDP doesn't support TLS)

## Related Documentation

- [HTTP tracker section](../src/application/command_handlers/create/config/tracker/http_tracker_section.rs) - Reference implementation with domain
- [DomainName value object](../src/shared/) - Existing domain validation
- [Error handling guide](../docs/contributing/error-handling.md)
- [DDD layer placement](../docs/contributing/ddd-layer-placement.md)
- [Torrust Live Demo](https://tracker.torrust-demo.com) - Real-world UDP tracker with domain

## Notes

- **Backward Compatibility**: This change is fully backward compatible. Existing configurations without the `domain` field will continue to work.
- **No TLS for UDP**: Unlike HTTP trackers, UDP trackers do not support TLS encryption, so there is no `use_tls_proxy` field. This is a fundamental protocol limitation.
- **Use Case**: The domain is primarily for documentation/communication purposes (e.g., telling users the announce URL) rather than for runtime behavior changes.
- **Test Command**: The `test` command only validates HTTP endpoints (HTTP API and HTTP trackers) for health checks. UDP testing is not included because UDP doesn't have a standard health check mechanism. Therefore, the `test` command does **not** need changes for this feature.
- **Schema Regeneration**: The JSON schema is auto-generated from Rust types. After updating the Rust structs, run `cargo run -- create schema > schemas/environment-config.json` to regenerate.
