# Update UserInputs to use ProviderConfig

**Issue**: #207
**Parent Epic**: #205 - Epic: Add Hetzner Provider Support
**Dependencies**: #206 - Add Provider enum and ProviderConfig types

## Overview

Refactor the domain `UserInputs` struct to use the new `ProviderConfig` type from the domain layer, moving provider-specific fields (like `profile_name`) into their respective provider configs while keeping global fields (like `instance_name`) at the top level.

## Goals

- [ ] Add `provider_config` field to `UserInputs`
- [ ] Move `profile_name` from global field to `LxdConfig`
- [ ] Keep `instance_name` as global (all providers need a VM name)
- [ ] Update all code that accesses `profile_name` to go through provider config
- [ ] Maintain backward compatibility awareness (clear error messages for old format)

## 🏗️ Architecture Requirements

**DDD Layer**: Domain
**Module Path**: `src/domain/environment/user_inputs.rs`
**Pattern**: Entity / Value Object modification

### Architectural Constraints

- [ ] Domain layer must not depend on application or infrastructure layers
- [ ] `UserInputs` uses `ProviderConfig` from domain layer (see #206)
- [ ] Error handling follows project conventions

### DDD Layer Compliance

**Task #206 establishes the layer separation**:

- **Domain Layer** (`src/domain/provider/`): `Provider`, `ProviderConfig`, `LxdConfig`, `HetznerConfig`

  - These use validated domain types (e.g., `ProfileName`)
  - `UserInputs` can use these directly (both in domain layer)

- **Application Layer** (`src/application/.../config/`): `ProviderSection`, `LxdProviderSection`, `HetznerProviderSection`
  - These use raw primitives for JSON parsing
  - Convert to domain types via `to_provider_config()`

Since both `UserInputs` and `ProviderConfig` are in the domain layer, there's no DDD violation.

## Specifications

### Current UserInputs Structure

```rust
// Current: src/domain/environment/user_inputs.rs
pub struct UserInputs {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,    // Global - keep here
    pub profile_name: ProfileName,      // LXD-specific - move to LxdConfig
    pub ssh_credentials: SshCredentials,
    pub ssh_port: u16,
}
```

### Proposed UserInputs Structure

```rust
// Proposed: src/domain/environment/user_inputs.rs
use crate::domain::provider::{LxdConfig, ProviderConfig};

pub struct UserInputs {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,      // Global - all providers need a VM name
    pub provider_config: ProviderConfig,  // Provider-specific configuration (domain type)
    pub ssh_credentials: SshCredentials,
    pub ssh_port: u16,
}

impl UserInputs {
    /// Returns the provider name as used in directory paths
    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        self.provider_config.provider_name()
    }

    /// Returns the LXD profile name if this is an LXD environment
    ///
    /// # Panics
    /// Panics if called on a non-LXD environment
    #[must_use]
    pub fn lxd_profile_name(&self) -> &ProfileName {
        match &self.provider_config {
            ProviderConfig::Lxd(config) => &config.profile_name,
            _ => panic!("lxd_profile_name() called on non-LXD environment"),
        }
    }

    /// Returns the LXD config if this is an LXD environment
    #[must_use]
    pub fn lxd_config(&self) -> Option<&LxdConfig> {
        match &self.provider_config {
            ProviderConfig::Lxd(config) => Some(config),
            _ => None,
        }
    }
}
```

### ProviderConfig is in Domain Layer (from #206)

Task #206 establishes `ProviderConfig` in the domain layer:

```rust
// src/domain/provider/config.rs (from #206)
use crate::domain::profile_name::ProfileName;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderConfig {
    #[serde(rename = "lxd")]
    Lxd(LxdConfig),
    #[serde(rename = "hetzner")]
    Hetzner(HetznerConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxdConfig {
    pub profile_name: ProfileName,  // Validated domain type
}
```

### Files to Update

| File                                                                                | Changes                                                          |
| ----------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| `src/domain/environment/user_inputs.rs`                                             | Add `provider_config`, remove `profile_name`, add helper methods |
| `src/domain/environment/mod.rs`                                                     | Update any methods that use `profile_name`                       |
| `src/domain/environment/context.rs`                                                 | Update `tofu_build_dir()` if it uses profile_name                |
| `src/infrastructure/external_tools/tofu/template/renderer/mod.rs`                   | Update to get profile_name from provider config                  |
| `src/infrastructure/external_tools/tofu/template/wrappers/lxd/variables/context.rs` | Update variable context                                          |
| Tests                                                                               | Update all tests that construct `UserInputs`                     |

## Implementation Plan

### Phase 1: Update UserInputs Structure (estimated: 1-2 hours)

- [ ] Task 1.1: Add `provider_config: ProviderConfig` field to `UserInputs`
- [ ] Task 1.2: Remove `profile_name` field from `UserInputs`
- [ ] Task 1.3: Add helper methods (`provider_name()`, `lxd_profile_name()`, `lxd_config()`)
- [ ] Task 1.4: Update `UserInputs::new()` constructor to accept `ProviderConfig`

### Phase 2: Update Dependent Code (estimated: 1-2 hours)

- [ ] Task 2.1: Find all usages of `user_inputs.profile_name`
- [ ] Task 2.2: Update each usage to use `user_inputs.lxd_profile_name()` or pattern match
- [ ] Task 2.3: Update TofuTemplateRenderer to get profile from provider config
- [ ] Task 2.4: Update any other infrastructure code that accesses profile_name

### Phase 3: Update Tests (estimated: 1 hour)

- [ ] Task 3.1: Update unit tests for `UserInputs`
- [ ] Task 3.2: Update integration tests that construct `UserInputs`
- [ ] Task 3.3: Ensure all tests compile and pass

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `UserInputs` has `provider_config: ProviderConfig` field
- [ ] `UserInputs` no longer has `profile_name` field at top level
- [ ] `profile_name` is accessible via `LxdConfig` in provider config
- [ ] Helper methods exist for common access patterns
- [ ] All code that previously accessed `profile_name` is updated
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] Rustdoc documentation updated

## Related Documentation

- [Feature Specification](../features/hetzner-provider-support/specification.md)
- [Technical Analysis](../features/hetzner-provider-support/analysis.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)

## Notes

- This task depends on #206 being completed first
- The change from `user_inputs.profile_name` to `user_inputs.lxd_profile_name()` is a breaking change for internal code
- Consider whether to panic or return Option for provider-specific accessors

---

**Created**: December 1, 2025
**Estimated Effort**: 3-4 hours
