# Add Provider Enum and ProviderConfig Types

**Issue**: #206
**Parent Epic**: #205 - Epic: Add Hetzner Provider Support
**Related**: [docs/features/hetzner-provider-support/](../features/hetzner-provider-support/)

## Overview

Add the foundational types for multi-provider support following proper DDD layer separation:

1. **Domain Layer**: `Provider` enum and `ProviderConfig` tagged enum with validated domain types
2. **Application Layer**: `ProviderSection` and provider-specific config types for JSON deserialization

This is the first step in Phase 1 (Make LXD Explicit) of the Hetzner Provider Support feature.

## Goals

- [ ] Create domain types with validated fields for business logic
- [ ] Create application config types with raw primitives for JSON parsing
- [ ] Implement conversion from application types to domain types with validation
- [ ] Support LXD as the initial provider with its specific configuration
- [ ] Design for future Hetzner support (but don't implement HetznerConfig yet)
- [ ] Maintain backward compatibility awareness (clear migration path)

## 🏗️ Architecture Requirements

This task spans two DDD layers with clear separation of concerns:

### Domain Layer: `src/domain/provider/`

**Pattern**: Value Object / Enum

Contains the core business types:

- `Provider` enum - represents which infrastructure provider is used
- `ProviderConfig` tagged enum - provider-specific configuration with **validated domain types**
- `LxdConfig` - LXD configuration with `ProfileName` (domain type)
- `HetznerConfig` - Hetzner configuration with validated fields

The domain types belong here because:

- They're core business concepts used throughout the codebase
- They contain validated fields (e.g., `ProfileName` instead of `String`)
- They represent the semantic meaning of provider configuration
- They have no external dependencies

### Application Layer: `src/application/command_handlers/create/config/`

**Pattern**: Configuration Section (like `EnvironmentSection`, `SshCredentialsConfig`)

Contains config types for external data deserialization:

- `ProviderSection` tagged enum - mirrors domain structure but uses **raw primitives**
- `LxdProviderSection` - uses `String` for profile_name
- `HetznerProviderSection` - uses `String` for all fields
- Conversion methods to validate and create domain types

These types belong here because:

- They're used for deserializing external configuration (JSON files)
- They use raw primitives (`String`) without validation
- They follow the same pattern as `SshCredentialsConfig` and `EnvironmentSection`
- They provide `to_*()` methods for converting to domain types

### Key DDD Pattern

```text
User JSON File → ProviderSection (Application) → ProviderConfig (Domain)
                 (raw String fields)              (validated ProfileName, etc.)
```

This separation ensures:

- Domain layer has no parsing concerns
- Application layer handles all JSON-to-domain conversion
- Validation errors are clear and actionable

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Domain types must not depend on infrastructure or application layers
- [ ] Application config types use raw primitives; domain types use validated types
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Both application config types and domain types should be serializable/deserializable with Serde

### Anti-Patterns to Avoid

- ❌ Putting validated domain types (like `ProfileName`) in application config types
- ❌ Putting raw primitives (like `String` for profile_name) in domain config types
- ❌ Domain layer depending on application layer
- ❌ Adding infrastructure concerns (file paths, API clients) to domain types
- ❌ Creating a generic provider abstraction layer
- ❌ Adding Hetzner-specific code in this task (that's a separate task)

## Specifications

### Domain Layer: `src/domain/provider/`

Create a new module for provider domain types:

```text
src/domain/provider/
├── mod.rs           # Module exports
├── provider.rs      # Provider enum
└── config.rs        # ProviderConfig, LxdConfig, HetznerConfig (domain types)
```

### Application Layer: `src/application/command_handlers/create/config/`

Add provider configuration types alongside existing config types:

```text
src/application/command_handlers/create/config/
├── mod.rs                    # Existing - add new exports
├── environment_config.rs     # Existing
├── ssh_credentials_config.rs # Existing
├── errors.rs                 # Existing - add new error variants
└── provider_section.rs       # NEW - ProviderSection and provider configs
```

This structure:

- Keeps domain concepts (`Provider`, `ProviderConfig`, `LxdConfig`) in the domain layer
- Places application config types (`ProviderSection`, `LxdProviderSection`) with other config types
- Follows the existing pattern in the config module (like `SshCredentialsConfig`)
- Application types convert to domain types with validation

### Provider Enum (Domain Layer)

```rust
// src/domain/provider/provider.rs

use serde::{Deserialize, Serialize};

/// Supported infrastructure providers
///
/// This enum represents the available infrastructure providers for deploying
/// Torrust Tracker environments. It's a domain concept used throughout the
/// codebase to determine provider-specific behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// LXD - Local development and testing
    Lxd,
    /// Hetzner Cloud - Production deployments
    Hetzner,
}

impl Provider {
    /// Returns the provider name as used in directory paths
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lxd => "lxd",
            Self::Hetzner => "hetzner",
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

### ProviderConfig (Domain Layer)

```rust
// src/domain/provider/config.rs

//! Provider Configuration Domain Types
//!
//! This module contains domain types for provider-specific configuration.
//! These types use validated domain types (like `ProfileName`) and represent
//! the semantic meaning of provider configuration.
//!
//! For config types used in JSON deserialization, see
//! `application::command_handlers::create::config::provider_section`.

use serde::{Deserialize, Serialize};
use crate::domain::profile_name::ProfileName;
use super::provider::Provider;

/// Provider-specific configuration (Domain Type)
///
/// Each variant contains the configuration fields specific to that provider
/// using **validated domain types** (e.g., `ProfileName` instead of `String`).
///
/// This is a tagged enum that serializes/deserializes based on the `"provider"` field.
///
/// # Note on Layer Placement
///
/// This is a **domain type** with validated fields. For JSON deserialization,
/// use `ProviderSection` in the application layer, then convert to this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderConfig {
    /// LXD provider configuration
    #[serde(rename = "lxd")]
    Lxd(LxdConfig),

    /// Hetzner provider configuration
    #[serde(rename = "hetzner")]
    Hetzner(HetznerConfig),
}

impl ProviderConfig {
    /// Returns the provider type
    #[must_use]
    pub fn provider(&self) -> Provider {
        match self {
            Self::Lxd(_) => Provider::Lxd,
            Self::Hetzner(_) => Provider::Hetzner,
        }
    }

    /// Returns the provider name as used in directory paths
    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        self.provider().as_str()
    }
}

/// LXD-specific configuration (Domain Type)
///
/// LXD is used for local development and testing. It provides fast VM creation
/// with no cloud costs, making it ideal for E2E tests and CI environments.
///
/// Uses validated domain types (e.g., `ProfileName`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxdConfig {
    /// LXD profile name for the instance (validated domain type)
    pub profile_name: ProfileName,
}

/// Hetzner-specific configuration (Domain Type)
///
/// Hetzner is used for production deployments. It provides cost-effective
/// cloud infrastructure with good European presence.
///
/// Note: This struct is defined for enum completeness but will be
/// fully implemented in Phase 2 (Add Hetzner Provider task).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HetznerConfig {
    /// Hetzner API token
    /// Note: Future improvement could use a validated `ApiToken` type
    pub api_token: String,
    /// Hetzner server type (e.g., "cx22", "cx32", "cpx11")
    /// Note: Future improvement could use a validated `ServerType` type
    pub server_type: String,
    /// Hetzner datacenter location (e.g., "fsn1", "nbg1", "hel1")
    /// Note: Future improvement could use a validated `Location` type
    pub location: String,
}
```

### Provider Module Exports (Domain Layer)

```rust
// src/domain/provider/mod.rs

//! Infrastructure provider types
//!
//! This module defines the `Provider` enum and provider-specific configuration
//! domain types. These are core business concepts used throughout the codebase.
//!
//! # Layer Separation
//!
//! - **Domain types** (this module): `Provider`, `ProviderConfig`, `LxdConfig`, `HetznerConfig`
//!   - Use validated domain types (e.g., `ProfileName`)
//!   - Represent semantic meaning of configuration
//!
//! - **Application config types** (`application::command_handlers::create::config::provider_section`):
//!   - `ProviderSection`, `LxdProviderSection`, `HetznerProviderSection`
//!   - Use raw primitives (e.g., `String`)
//!   - Handle JSON deserialization and conversion to domain types

mod provider;
mod config;

pub use provider::Provider;
pub use config::{HetznerConfig, LxdConfig, ProviderConfig};
```

### ProviderSection (Application Layer)

````rust
// src/application/command_handlers/create/config/provider_section.rs

//! Provider Configuration Types
//!
//! This module contains configuration types for provider-specific configuration.
//! These types are used for deserializing external configuration (JSON files) and
//! contain **raw primitives** (e.g., `String`).
//!
//! After deserialization, use `to_provider_config()` to convert to domain types
//! with validation.
//!
//! # Layer Separation
//!
//! - **These config types** (this module): Raw primitives for JSON parsing
//! - **Domain types** (`domain::provider`): Validated types for business logic

use serde::{Deserialize, Serialize};
use crate::domain::provider::{HetznerConfig, LxdConfig, ProviderConfig, Provider};
use crate::domain::ProfileName;
use super::errors::CreateConfigError;

/// Provider-specific configuration section
///
/// Each variant contains the configuration fields specific to that provider
/// using **raw primitives** (`String`) for JSON deserialization.
///
/// This is a tagged enum that deserializes based on the `"provider"` field in JSON.
///
/// # Conversion
///
/// Use `to_provider_config()` to validate and convert to domain types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderSection {
    /// LXD provider configuration
    #[serde(rename = "lxd")]
    Lxd(LxdProviderSection),

    /// Hetzner provider configuration
    #[serde(rename = "hetzner")]
    Hetzner(HetznerProviderSection),
}

impl ProviderSection {
    /// Returns the provider type (no validation needed)
    #[must_use]
    pub fn provider(&self) -> Provider {
        match self {
            Self::Lxd(_) => Provider::Lxd,
            Self::Hetzner(_) => Provider::Hetzner,
        }
    }

    /// Converts the config to a validated domain `ProviderConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError` if validation fails:
    /// - `InvalidProfileName` - if the LXD profile name is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::ProviderSection;
    ///
    /// let section = ProviderSection::Lxd(LxdProviderSection {
    ///     profile_name: "torrust-profile-dev".to_string(),
    /// });
    ///
    /// let config = section.to_provider_config()?;
    /// assert_eq!(config.provider_name(), "lxd");
    /// ```
    pub fn to_provider_config(self) -> Result<ProviderConfig, CreateConfigError> {
        match self {
            Self::Lxd(lxd) => {
                let profile_name = ProfileName::new(lxd.profile_name)?;
                Ok(ProviderConfig::Lxd(LxdConfig { profile_name }))
            }
            Self::Hetzner(hetzner) => {
                // Note: Future improvement could add validation for these fields
                Ok(ProviderConfig::Hetzner(HetznerConfig {
                    api_token: hetzner.api_token,
                    server_type: hetzner.server_type,
                    location: hetzner.location,
                }))
            }
        }
    }
}

/// LXD-specific configuration section
///
/// Uses raw `String` for JSON deserialization. Convert to domain `LxdConfig`
/// via `ProviderSection::to_provider_config()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxdProviderSection {
    /// LXD profile name (raw string - validated on conversion)
    pub profile_name: String,
}

/// Hetzner-specific configuration section
///
/// Uses raw `String` fields for JSON deserialization. Convert to domain
/// `HetznerConfig` via `ProviderSection::to_provider_config()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HetznerProviderSection {
    /// Hetzner API token (raw string)
    pub api_token: String,
    /// Hetzner server type (e.g., "cx22", "cx32", "cpx11")
    pub server_type: String,
    /// Hetzner datacenter location (e.g., "fsn1", "nbg1", "hel1")
    pub location: String,
}
````

### Update Config Module Exports

```rust
// Update src/application/command_handlers/create/config/mod.rs

// Add to existing module declarations:
pub mod provider_section;

// Add to existing re-exports:
pub use provider_section::{HetznerProviderSection, LxdProviderSection, ProviderSection};
```

### Update Domain Module

Update `src/domain/mod.rs` to export the new provider module:

```rust
// Add to src/domain/mod.rs
pub mod provider;

// Add to re-exports
pub use provider::{HetznerConfig, LxdConfig, Provider, ProviderConfig};
```

### Update Error Types

Add new error variant to `CreateConfigError`:

```rust
// Add to src/application/command_handlers/create/config/errors.rs

/// Invalid profile name provided
#[error("Invalid profile name: {0}")]
InvalidProfileName(#[from] crate::domain::profile_name::ProfileNameError),
```

## Implementation Plan

### Phase 1: Create Domain Provider Types (estimated: 45 minutes)

- [ ] Task 1.1: Create `src/domain/provider/` directory
- [ ] Task 1.2: Implement `Provider` enum in `provider.rs`
- [ ] Task 1.3: Implement `ProviderConfig`, `LxdConfig`, `HetznerConfig` in `config.rs`
- [ ] Task 1.4: Create `mod.rs` with exports
- [ ] Task 1.5: Update `src/domain/mod.rs` to include provider module and re-exports

### Phase 2: Create Application Layer Config Types (estimated: 45 minutes)

- [ ] Task 2.1: Create `provider_section.rs` in `src/application/command_handlers/create/config/`
- [ ] Task 2.2: Implement `ProviderSection` tagged enum
- [ ] Task 2.3: Implement `LxdProviderSection` struct
- [ ] Task 2.4: Implement `HetznerProviderSection` struct
- [ ] Task 2.5: Implement `to_provider_config()` conversion method with validation
- [ ] Task 2.6: Update config `mod.rs` with new exports
- [ ] Task 2.7: Add `InvalidProfileName` error variant to `errors.rs`

### Phase 3: Add Unit Tests (estimated: 1.5 hours)

- [ ] Task 3.1: Test `Provider` enum serialization/deserialization
- [ ] Task 3.2: Test domain `ProviderConfig` with validated types
- [ ] Task 3.3: Test `ProviderSection` JSON parsing (LXD variant)
- [ ] Task 3.4: Test `ProviderSection` JSON parsing (Hetzner variant)
- [ ] Task 3.5: Test `to_provider_config()` successful conversion
- [ ] Task 3.6: Test `to_provider_config()` validation errors (invalid profile name)
- [ ] Task 3.7: Test `provider_name()` method returns correct strings

### Phase 4: Documentation (estimated: 30 minutes)

- [ ] Task 4.1: Add rustdoc comments to all public types and methods
- [ ] Task 4.2: Add module-level documentation explaining the layer separation

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

**Domain Layer** (`src/domain/provider/`):

- [ ] `Provider` enum exists with `Lxd` and `Hetzner` variants
- [ ] `ProviderConfig` tagged enum exists with `Lxd(LxdConfig)` and `Hetzner(HetznerConfig)` variants
- [ ] `LxdConfig` contains `profile_name: ProfileName` (validated domain type)
- [ ] `HetznerConfig` contains `api_token`, `server_type`, `location` fields (String for now)
- [ ] All domain types implement `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**Application Layer** (`src/application/command_handlers/create/config/`):

- [ ] `ProviderSection` tagged enum exists with raw primitive fields
- [ ] `LxdProviderSection` contains `profile_name: String` (raw primitive)
- [ ] `HetznerProviderSection` contains `api_token: String`, `server_type: String`, `location: String`
- [ ] `ProviderSection::to_provider_config()` converts to domain types with validation
- [ ] `InvalidProfileName` error variant added to `CreateConfigError`
- [ ] All config types implement `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

**JSON Deserialization**:

- [ ] `ProviderSection` correctly deserializes JSON with `"provider": "lxd"` or `"provider": "hetzner"`
- [ ] Invalid profile names in JSON cause clear `InvalidProfileName` errors on conversion

**General**:

- [ ] `provider_name()` returns `"lxd"` or `"hetzner"` as appropriate
- [ ] Unit tests cover serialization, deserialization, conversion, and error cases
- [ ] Rustdoc documentation is complete and follows project conventions
- [ ] Module documentation explains layer separation (domain vs application config types)
- [ ] No compiler warnings
- [ ] Code follows module organization conventions

## Example JSON Formats

These JSON examples show how `ProviderSection` is deserialized:

### LXD Provider Config

```json
{
  "provider": "lxd",
  "profile_name": "torrust-profile-my-env"
}
```

### Hetzner Provider Config

```json
{
  "provider": "hetzner",
  "api_token": "your-hetzner-api-token-here",
  "server_type": "cx22",
  "location": "nbg1"
}
```

## Related Documentation

- [Feature Specification](../features/hetzner-provider-support/specification.md)
- [Technical Analysis](../features/hetzner-provider-support/analysis.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Module Organization](../contributing/module-organization.md)
- [Error Handling Guidelines](../contributing/error-handling.md)

## Notes

- This task creates types in **both** domain and application layers following proper DDD separation.
- Domain types (`ProviderConfig`, `LxdConfig`) use validated types like `ProfileName`.
- Application config types (`ProviderSection`, `LxdProviderSection`) use raw primitives for JSON parsing.
- Conversion happens via `ProviderSection::to_provider_config()` with validation.
- The `HetznerConfig` struct is defined for enum completeness but won't be used until Phase 2.
- Future work may add validated types for Hetzner fields (e.g., `ServerType`, `Location`).

---

**Created**: December 1, 2025
**Updated**: December 1, 2025 (Revised for proper DDD layer separation)
**Estimated Effort**: 3-4 hours
