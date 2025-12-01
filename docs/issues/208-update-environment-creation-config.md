# Update EnvironmentCreationConfig

**Issue**: #208
**Parent Epic**: #205 - Epic: Add Hetzner Provider Support
**Dependencies**: #206 - Add Provider enum and ProviderConfig types

## Overview

Update the application layer `EnvironmentCreationConfig` to include provider configuration, allowing users to specify their provider choice in the environment JSON file. The config will use `ProviderSection` for JSON parsing, then convert to domain `ProviderConfig` during validation.

## Goals

- [ ] Add `provider` section to `EnvironmentCreationConfig` using `ProviderSection`
- [ ] Update `to_environment_params()` to return domain `ProviderConfig`
- [ ] Update JSON deserialization to handle provider config
- [ ] Provide clear error messages for missing or invalid provider config

## 🏗️ Architecture Requirements

**DDD Layer**: Application
**Module Path**: `src/application/command_handlers/create/config/environment_config.rs`
**Pattern**: Configuration Type (like `SshCredentialsConfig`, `EnvironmentSection`)

### DDD Layer Compliance

This task follows the layer separation established in #206:

- **Application config types**: `ProviderSection` uses raw primitives (`String`) for JSON parsing
- **Domain Types**: `ProviderConfig` uses validated types (`ProfileName`)
- **Conversion**: `ProviderSection::to_provider_config()` validates and converts

### Architectural Constraints

- [ ] Application layer config types use raw primitives for external input
- [ ] Conversion to domain types happens with validation
- [ ] Error handling follows project conventions
- [ ] JSON format should be user-friendly and self-documenting

## Specifications

### Current EnvironmentCreationConfig Structure

```rust
// Current: src/application/command_handlers/create/config/environment_config.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentCreationConfig {
    pub environment: EnvironmentSection,
    pub ssh_credentials: SshCredentialsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSection {
    pub name: String,
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
}
```

### Proposed EnvironmentCreationConfig Structure

```rust
// Proposed: src/application/command_handlers/create/config/environment_config.rs
use super::provider_section::ProviderSection;
use crate::domain::provider::ProviderConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentCreationConfig {
    pub environment: EnvironmentSection,
    pub ssh_credentials: SshCredentialsConfig,
    pub provider: ProviderSection,  // NEW: Uses ProviderSection for JSON parsing
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSection {
    pub name: String,
    pub instance_name: Option<String>,  // NEW: Optional, auto-generated if not provided
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
}
```

Note: `ProviderSection` is used in the struct (for JSON parsing), but `to_environment_params()` returns domain `ProviderConfig` after validation.

### Current JSON Format

```json
{
  "environment": {
    "name": "my-env",
    "ssh_port": 22
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub"
  }
}
```

### Proposed JSON Format

```json
{
  "environment": {
    "name": "my-env",
    "instance_name": "torrust-tracker-vm-my-env",
    "ssh_port": 22
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-my-env"
  }
}
```

Or for Hetzner:

```json
{
  "environment": {
    "name": "production",
    "instance_name": "torrust-tracker-demo",
    "ssh_port": 22
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub"
  },
  "provider": {
    "provider": "hetzner",
    "api_token": "your-hetzner-api-token",
    "server_type": "cx22",
    "location": "nbg1"
  }
}
```

### Update to_environment_params Method

```rust
impl EnvironmentCreationConfig {
    /// Converts the configuration to domain parameters
    ///
    /// This method:
    /// 1. Validates all string fields and converts to domain types
    /// 2. Converts `ProviderSection` to domain `ProviderConfig` with validation
    /// 3. Returns validated domain objects ready for `Environment::new()`
    ///
    /// Returns (EnvironmentName, InstanceName, ProviderConfig, SshCredentials, ssh_port)
    pub fn to_environment_params(
        self,
    ) -> Result<(EnvironmentName, InstanceName, ProviderConfig, SshCredentials, u16), CreateConfigError> {
        let name = EnvironmentName::new(&self.environment.name)
            .map_err(|e| CreateConfigError::InvalidEnvironmentName {
                name: self.environment.name.clone(),
                reason: e.to_string(),
            })?;

        // Instance name: use provided or auto-generate from environment name
        let instance_name = match &self.environment.instance_name {
            Some(name_str) => InstanceName::new(name_str.clone())
                .map_err(|e| CreateConfigError::InvalidInstanceName {
                    name: name_str.clone(),
                    reason: e.to_string(),
                })?,
            None => InstanceName::generate_from_env_name(&name),
        };

        // Convert DTO to domain type (validates profile_name, etc.)
        let provider_config = self.provider.to_provider_config()?;

        let ssh_credentials = self.ssh_credentials.to_ssh_credentials()?;

        Ok((
            name,
            instance_name,
            provider_config,
            ssh_credentials,
            self.environment.ssh_port,
        ))
    }
}
```

Note the key pattern:

- `self.provider` is `ProviderSection` (raw primitives for JSON parsing)
- `provider_config` is domain `ProviderConfig` (validated domain types)
- `to_provider_config()` does the validation and conversion

### Error Handling

Add new error variants for provider configuration errors:

```rust
// In src/application/command_handlers/create/config/errors.rs

#[derive(Debug, Error)]
pub enum CreateConfigError {
    // ... existing variants ...

    #[error("Missing provider configuration")]
    MissingProviderConfig,

    #[error("Invalid provider configuration: {reason}")]
    InvalidProviderConfig { reason: String },

    #[error("Invalid instance name '{name}': {reason}")]
    InvalidInstanceName { name: String, reason: String },
}
```

## Implementation Plan

### Phase 1: Update EnvironmentCreationConfig (estimated: 1 hour)

- [ ] Task 1.1: Add `provider: ProviderSection` field to `EnvironmentCreationConfig`
- [ ] Task 1.2: Add `instance_name: Option<String>` field to `EnvironmentSection`
- [ ] Task 1.3: Update `to_environment_params()` to convert DTO to domain `ProviderConfig`
- [ ] Task 1.4: Add instance name auto-generation logic
- [ ] Task 1.5: Update `template()` method to include provider section

### Phase 2: Update Error Handling (estimated: 30 minutes)

- [ ] Task 2.1: Add new error variants for provider config errors (if not already in #206)
- [ ] Task 2.2: Implement `.help()` method for new errors
- [ ] Task 2.3: Add clear error messages for JSON deserialization failures

### Phase 3: Update Tests (estimated: 1 hour)

- [ ] Task 3.1: Update existing unit tests for `EnvironmentCreationConfig`
- [ ] Task 3.2: Add tests for LXD provider config parsing and conversion
- [ ] Task 3.3: Add tests for Hetzner provider config parsing and conversion
- [ ] Task 3.4: Add tests for invalid profile name validation error
- [ ] Task 3.5: Add tests for instance name auto-generation

### Phase 4: Update Documentation (estimated: 30 minutes)

- [ ] Task 4.1: Update rustdoc for `EnvironmentCreationConfig`
- [ ] Task 4.2: Update example JSON in module documentation
- [ ] Task 4.3: Update `generate_template_file()` to produce new format

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `EnvironmentCreationConfig` has `provider: ProviderSection` field
- [ ] JSON with `"provider": { "provider": "lxd", ... }` deserializes correctly
- [ ] JSON with `"provider": { "provider": "hetzner", ... }` deserializes correctly
- [ ] Missing `provider` field produces clear error message
- [ ] `to_environment_params()` returns domain `ProviderConfig` (not DTO)
- [ ] Invalid profile name in JSON produces clear validation error on conversion
- [ ] Instance name is auto-generated if not provided
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] Rustdoc documentation updated
- [ ] `generate_template_file()` produces valid new format with provider section

## Related Documentation

- [Feature Specification](../features/hetzner-provider-support/specification.md)
- [Technical Analysis](../features/hetzner-provider-support/analysis.md)
- [Error Handling Guidelines](../contributing/error-handling.md)

## Notes

- This task can be done in parallel with Task 2 (Update UserInputs) since both depend only on Task 1
- The JSON format change is a breaking change for existing environment files
- Consider whether to support both old and new formats during migration (probably not needed - tool is not in production)

---

**Created**: December 1, 2025
**Estimated Effort**: 3 hours
