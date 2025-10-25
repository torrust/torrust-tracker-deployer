# Core Configuration Infrastructure

**Issue**: [#35](https://github.com/torrust/torrust-tracker-deployer/issues/35)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Related**: [Roadmap Task 1.5](../roadmap.md), [Domain Layer Architecture](../codebase-architecture.md#domain-layer)

## Overview

Implement the core configuration infrastructure in the domain layer with pure configuration value objects, business validation rules, and explicit error handling. This forms the foundation for configuration management across all delivery mechanisms.

**Key Architecture Points**:

- Configuration uses `SshCredentialsConfig` (distinct name from `adapters::ssh::SshCredentials`)
- Converts configuration strings to domain types (`Username`, `PathBuf`)
- JSON Schema validation is **optional** and can be deferred to focus on domain validation first
- All errors implement `.help()` methods following the tiered help system pattern

## Goals

- [ ] **Optional**: Implement JSON Schema validation as specified in parent epic
  - [ ] Can be deferred - domain validation is primary
  - [ ] Embed JSON Schema from parent epic for configuration validation
  - [ ] Use `jsonschema` crate for standardized validation
  - [ ] Provide clear validation error messages with actionable guidance
  - [ ] Support JSON configuration file format (TOML support will be added in separate issue)
- [ ] Create configuration value objects that convert to existing domain types
  - [ ] Use existing `EnvironmentName::new()` validation from `src/domain/environment/name.rs`
  - [ ] Create `EnvironmentCreationConfig` value object with `to_environment_params()` method
  - [ ] Create `SshCredentialsConfig` (config layer) distinct from `adapters::ssh::SshCredentials` (domain)
  - [ ] Convert string username to `shared::Username` type at configuration boundary
  - [ ] Convert string paths to `PathBuf` for domain objects
- [ ] Configuration validation
  - [ ] Primary validation via domain layer (EnvironmentName, Username, file existence)
  - [ ] Optional: Secondary validation via JSON Schema for standardization
  - [ ] Follow existing error handling patterns with thiserror and `.help()` methods
- [ ] Add unit tests that integrate with existing patterns
  - [ ] Test conversion to existing `Environment::new(environment_name, ssh_credentials, ssh_port)` params
  - [ ] Test `Username` type conversion from string
  - [ ] Test that converted params work with existing `repository.save(&environment.into_any())` pattern
  - [ ] Verify compatibility with existing Environment<Created> state transitions
  - [ ] Test invalid username and environment name handling

**Estimated Time**: 2-3 hours

## 🏗️ Architecture Requirements

**DDD Layer**: Domain Layer (`src/domain/config/`)
**Pattern**: Value Objects + Domain Validation + Explicit Error Enums
**Dependencies**: None (pure domain objects)

### Module Structure

```text
src/domain/config/
├── mod.rs                    # Module exports and documentation
├── environment_config.rs     # EnvironmentConfig value object
├── ssh_credentials.rs        # SshCredentials value object
├── schema.rs                 # Embedded JSON Schema and validation
├── validation.rs             # Domain-specific validation (file existence, permissions)
└── errors.rs                 # ConfigValidationError enum
```

### Dependencies

Add to `Cargo.toml`:

````toml
[dependencies]
jsonschema = "0.21"  # JSON Schema validation
serde_json = "1.0"   # JSON Schema handling
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
```## Specifications

### Domain Value Objects

#### EnvironmentConfig Value Object

```rust
// src/domain/config/environment_config.rs
use serde::{Deserialize, Serialize};
use super::ssh_credentials::SshCredentials;
use super::errors::ConfigValidationError;

/// Configuration for creating a deployment environment
///
/// This is a pure domain object that represents the business concept
/// of environment configuration without any infrastructure concerns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub environment: EnvironmentSettings,
    pub ssh_credentials: SshCredentials,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentSettings {
    pub name: String,
}

impl EnvironmentConfig {
    /// Create a new environment configuration
    pub fn new(name: String, ssh_credentials: SshCredentials) -> Self {
        Self {
            environment: EnvironmentSettings { name },
            ssh_credentials,
        }
    }

    /// Validate the entire configuration according to business rules
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // Primary validation via JSON Schema
        super::schema::validate_against_schema(self)?;

        // Secondary validation for domain-specific concerns (file existence, permissions)
        self.ssh_credentials.validate_file_access()?;

        Ok(())
    }

    /// Get the environment name
    pub fn environment_name(&self) -> &str {
        &self.environment.name
    }
}

impl EnvironmentSettings {
    /// Basic validation (JSON Schema handles most validation)
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // JSON Schema handles format validation, this is for any additional business rules
        Ok(())
    }
}
````

#### SshCredentials Value Object

```rust
// src/domain/config/ssh_credentials.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::errors::ConfigValidationError;

/// SSH credentials configuration for remote access
///
/// Contains all necessary information for SSH connectivity
/// with sensible defaults for common scenarios.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SshCredentials {
    pub private_key_path: String,
    pub public_key_path: String,
    #[serde(default = "default_ssh_username")]
    pub username: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
}

impl SshCredentials {
    /// Create new SSH credentials
    pub fn new(
        private_key_path: String,
        public_key_path: String,
        username: Option<String>,
        port: Option<u16>,
    ) -> Self {
        Self {
            private_key_path,
            public_key_path,
            username: username.unwrap_or_else(default_ssh_username),
            port: port.unwrap_or_else(default_ssh_port),
        }
    }

    /// Validate SSH credentials for file access and permissions
    /// (JSON Schema handles format validation)
    pub fn validate_file_access(&self) -> Result<(), ConfigValidationError> {
        super::validation::validate_ssh_key_files_exist(&self.private_key_path, &self.public_key_path)?;
        Ok(())
    }

    /// Get the private key path as PathBuf
    pub fn private_key_path(&self) -> PathBuf {
        PathBuf::from(&self.private_key_path)
    }

    /// Get the public key path as PathBuf
    pub fn public_key_path(&self) -> PathBuf {
        PathBuf::from(&self.public_key_path)
    }
}

fn default_ssh_username() -> String {
    "torrust".to_string()
}

fn default_ssh_port() -> u16 {
    22
}
```

### JSON Schema Validation

```rust
// src/domain/config/schema.rs
use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;
use super::environment_config::EnvironmentConfig;
use super::errors::ConfigValidationError;

/// Embedded JSON Schema for environment configuration validation
///
/// This schema is embedded from the parent epic specification and provides
/// standardized validation for environment configuration files.
const ENVIRONMENT_CONFIG_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Environment Configuration",
  "type": "object",
  "required": ["environment", "ssh_credentials"],
  "properties": {
    "environment": {
      "type": "object",
      "required": ["name"],
      "properties": {
        "name": {
          "type": "string",
          "pattern": "^[a-zA-Z0-9][a-zA-Z0-9-_]*$",
          "description": "Environment name (alphanumeric, hyphens, underscores)"
        }
      }
    },
    "ssh_credentials": {
      "type": "object",
      "required": ["private_key_path", "public_key_path"],
      "properties": {
        "private_key_path": {
          "type": "string",
          "description": "Path to SSH private key file"
        },
        "public_key_path": {
          "type": "string",
          "description": "Path to SSH public key file"
        },
        "username": {
          "type": "string",
          "default": "torrust",
          "description": "SSH username"
        },
        "port": {
          "type": "integer",
          "minimum": 1,
          "maximum": 65535,
          "default": 22,
          "description": "SSH port number"
        }
      }
    }
  }
}"#;

/// Validate configuration against the embedded JSON Schema
pub fn validate_against_schema(config: &EnvironmentConfig) -> Result<(), ConfigValidationError> {
    // Parse the schema
    let schema_value: Value = serde_json::from_str(ENVIRONMENT_CONFIG_SCHEMA)
        .map_err(|e| ConfigValidationError::SchemaParseError {
            source: e
        })?;

    let schema = JSONSchema::compile(&schema_value)
        .map_err(|e| ConfigValidationError::SchemaCompileError {
            message: e.to_string()
        })?;

    // Convert config to JSON for validation
    let config_value = serde_json::to_value(config)
        .map_err(|e| ConfigValidationError::ConfigSerializationError {
            source: e
        })?;

    // Validate against schema
    if let Err(errors) = schema.validate(&config_value) {
        let validation_errors: Vec<String> = errors
            .map(|error| format_schema_error(error))
            .collect();

        return Err(ConfigValidationError::SchemaValidationFailed {
            errors: validation_errors,
        });
    }

    Ok(())
}

/// Format JSON Schema validation error into human-readable message
fn format_schema_error(error: ValidationError) -> String {
    match error.keyword.as_str() {
        "required" => {
            format!("Missing required field: {}", error.instance_path)
        }
        "pattern" => {
            format!("Field '{}' has invalid format: {}", error.instance_path, error.instance)
        }
        "type" => {
            format!("Field '{}' has wrong type: expected {}, got {}",
                error.instance_path,
                error.schema_path,
                error.instance
            )
        }
        "minimum" | "maximum" => {
            format!("Field '{}' value {} is out of range", error.instance_path, error.instance)
        }
        _ => {
            format!("Validation error at '{}': {}", error.instance_path, error)
        }
    }
}

/// Validate raw JSON content against schema before deserialization
pub fn validate_raw_json_config(content: &str) -> Result<(), ConfigValidationError> {
    let config_value: Value = serde_json::from_str(content)
        .map_err(|e| ConfigValidationError::JsonParseError { source: e })?;

    // Parse and compile schema
    let schema_value: Value = serde_json::from_str(ENVIRONMENT_CONFIG_SCHEMA)
        .map_err(|e| ConfigValidationError::SchemaParseError { source: e })?;

    let schema = JSONSchema::compile(&schema_value)
        .map_err(|e| ConfigValidationError::SchemaCompileError { message: e.to_string() })?;

    // Validate
    if let Err(errors) = schema.validate(&config_value) {
    if let Err(errors) = schema.validate(&config_value) {
        let validation_errors: Vec<String> = errors
            .map(|error| format_schema_error(error))
            .collect();

        return Err(ConfigValidationError::SchemaValidationFailed {
            errors: validation_errors,
        });
    }

    Ok(())
}

```

### Domain Validation Functions

```rust
// src/domain/config/validation.rs
```

### Domain Validation Functions

```rust
// src/domain/config/validation.rs
use std::path::Path;
use super::errors::ConfigValidationError;

/// Validate that SSH key files exist and are accessible
/// (This complements JSON Schema validation with file system checks)
pub fn validate_ssh_key_files_exist(private_key: &str, public_key: &str) -> Result<(), ConfigValidationError> {
    let private_path = expand_tilde_path(private_key);
    let public_path = expand_tilde_path(public_key);

    if !private_path.exists() {
        return Err(ConfigValidationError::SshPrivateKeyNotFound {
            path: private_path,
        });
    }

    if !public_path.exists() {
        return Err(ConfigValidationError::SshPublicKeyNotFound {
            path: public_path,
        });
    }

    // Validate file permissions (private key should be readable by owner only)
    validate_private_key_permissions(&private_path)?;

    Ok(())
}

/// Expand tilde (~) in file paths to user home directory
fn expand_tilde_path(path: &str) -> std::path::PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            let mut expanded = std::path::PathBuf::from(home);
            expanded.push(&path[2..]);
            return expanded;
        }
    }
    std::path::PathBuf::from(path)
}

/// Validate private key file permissions (Unix systems only)
#[cfg(unix)]
fn validate_private_key_permissions(path: &Path) -> Result<(), ConfigValidationError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)
        .map_err(|source| ConfigValidationError::SshKeyPermissionCheck {
            path: path.to_path_buf(),
            source,
        })?;

    let mode = metadata.permissions().mode();

    // Check if file is readable by others (should not be for private keys)
    if mode & 0o077 != 0 {
        return Err(ConfigValidationError::SshPrivateKeyInsecurePermissions {
            path: path.to_path_buf(),
            current_mode: format!("{:o}", mode),
        });
    }

    Ok(())
}

/// No-op validation for non-Unix systems
#[cfg(not(unix))]
fn validate_private_key_permissions(_path: &Path) -> Result<(), ConfigValidationError> {
    Ok(())
}
    Ok(())
}
```

### Domain Error Types

```rust
// src/domain/config/errors.rs
use thiserror::Error;
use std::path::PathBuf;

/// Domain validation errors for configuration
///
/// These errors represent business rule violations and validation failures
/// in the configuration domain. They provide structured context and
/// actionable error messages following the project's error handling guidelines.
#[derive(Debug, Error)]
pub enum ConfigValidationError {
    // JSON Schema validation errors
    #[error("Configuration validation failed: {errors:?}")]
    SchemaValidationFailed { errors: Vec<String> },

    #[error("Failed to parse JSON Schema")]
    SchemaParseError {
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to compile JSON Schema: {message}")]
    SchemaCompileError { message: String },

    #[error("Failed to serialize configuration for validation")]
    ConfigSerializationError {
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to parse JSON configuration")]
    JsonParseError {
        #[source]
        source: serde_json::Error,
    },

    // File system validation errors
    #[error("SSH private key not found: {path}")]
    SshPrivateKeyNotFound { path: PathBuf },

    #[error("SSH public key not found: {path}")]
    SshPublicKeyNotFound { path: PathBuf },

    #[error("SSH private key has insecure permissions: {path} (mode: {current_mode})")]
    SshPrivateKeyInsecurePermissions {
        path: PathBuf,
        current_mode: String,
    },

    #[error("Failed to check SSH key permissions: {path}")]
    SshKeyPermissionCheck {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl ConfigValidationError {
    /// Get detailed troubleshooting guidance for this error
    pub fn help(&self) -> &'static str {
        match self {
            Self::SchemaValidationFailed { .. } => {
                "Configuration Schema Validation Failed - Detailed Troubleshooting:

1. Check the configuration file format matches the JSON schema requirements
2. Verify all required fields are present: environment.name, ssh_credentials.private_key_path, ssh_credentials.public_key_path
3. Ensure field values match the expected format:
   - environment.name: alphanumeric characters, hyphens, underscores only
   - ssh_credentials.port: number between 1-65535
   - ssh_credentials.username: alphanumeric characters, hyphens, underscores only
4. Use the --generate-template option to create a valid configuration template

For more information, see the configuration documentation."
            }

            Self::JsonParseError { .. } => {
                "JSON Parse Error - Detailed Troubleshooting:

1. Check JSON syntax: proper quotes, commas, brackets
2. Validate JSON format using: jq . your-config.json
3. Common issues: trailing commas, missing quotes, unescaped characters
4. Use a JSON validator or editor with syntax highlighting

For more information, see the JSON format documentation."
            }

            Self::SshPrivateKeyNotFound { .. } => {
                "SSH Private Key Not Found - Detailed Troubleshooting:

1. Check if the private key file exists at the specified path
2. Verify the path is correct and accessible
3. Generate a new SSH key pair if needed:
   ssh-keygen -t rsa -b 4096 -f ~/.ssh/id_rsa
4. Update the configuration file with the correct path

For more information, see the SSH setup documentation."
            }

            Self::SshPublicKeyNotFound { .. } => {
                "SSH Public Key Not Found - Detailed Troubleshooting:

1. Check if the public key file exists at the specified path
2. Verify the path is correct and accessible
3. If you have the private key, generate the public key:
   ssh-keygen -y -f ~/.ssh/id_rsa > ~/.ssh/id_rsa.pub
4. Update the configuration file with the correct path

For more information, see the SSH setup documentation."
            }

            Self::SshPrivateKeyInsecurePermissions { .. } => {
                "SSH Private Key Insecure Permissions - Detailed Troubleshooting:

1. Fix the private key permissions:
   chmod 600 ~/.ssh/id_rsa
2. Ensure only the owner can read the private key file
3. This is a security requirement for SSH private keys

For more information, see the SSH security documentation."
            }

            Self::SshKeyPermissionCheck { .. } => {
                "SSH Key Permission Check Failed - Detailed Troubleshooting:

1. Check if the file exists and is accessible
2. Verify you have permission to read the file metadata
3. Check filesystem and disk space issues

If the problem persists, report it with system details."
            }

            Self::SchemaParseError { .. } | Self::SchemaCompileError { .. } => {
                "Internal Schema Error - Detailed Troubleshooting:

This indicates a problem with the embedded JSON schema configuration.
This should not normally occur and indicates a bug in the application.

Please report this issue with full error details."
            }

            Self::ConfigSerializationError { .. } => {
                "Configuration Serialization Error - Detailed Troubleshooting:

1. Check that the configuration values are valid
2. Ensure no circular references or invalid data structures
3. This may indicate a problem with the configuration parsing

If the problem persists, report it with configuration details."
            }
        }
    }
            }
        }
    }
}
```

## Unit Tests

### JSON Schema Validation Tests

```rust
// src/domain/config/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn it_should_validate_valid_json_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let private_key = temp_dir.path().join("id_rsa");
        let public_key = temp_dir.path().join("id_rsa.pub");

        // Create test SSH key files
        fs::write(&private_key, "dummy private key").unwrap();
        fs::write(&public_key, "dummy public key").unwrap();

        let config = EnvironmentConfig::new(
            "test-env".to_string(),
            SshCredentials::new(
                private_key.to_string_lossy().to_string(),
                public_key.to_string_lossy().to_string(),
                Some("torrust".to_string()),
                Some(22),
            ),
        );

        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_reject_invalid_environment_name() {
        let config_json = r#"{
            "environment": {
                "name": "_invalid-start"
            },
            "ssh_credentials": {
                "private_key_path": "/path/to/key",
                "public_key_path": "/path/to/key.pub",
                "username": "torrust",
                "port": 22
            }
        }"#;

        let result = schema::validate_raw_config(config_json, schema::ConfigFormat::Json);
        assert!(result.is_err());

        if let Err(ConfigValidationError::SchemaValidationFailed { errors }) = result {
            assert!(!errors.is_empty());
            assert!(errors[0].contains("pattern"));
        } else {
            panic!("Expected SchemaValidationFailed error");
        }
    }

    #[test]
    fn it_should_reject_missing_required_fields() {
        let config_json = r#"{
            "environment": {
                "name": "test"
            }
        }"#;

        let result = schema::validate_raw_config(config_json, schema::ConfigFormat::Json);
        assert!(result.is_err());

        if let Err(ConfigValidationError::SchemaValidationFailed { errors }) = result {
            assert!(!errors.is_empty());
            assert!(errors[0].contains("required") || errors[0].contains("ssh_credentials"));
        } else {
            panic!("Expected SchemaValidationFailed error");
        }
    }

    #[test]
    fn it_should_reject_invalid_ssh_port() {
        let config_json = r#"{
            "environment": {
                "name": "test"
            },
            "ssh_credentials": {
                "private_key_path": "/path/to/key",
                "public_key_path": "/path/to/key.pub",
                "username": "torrust",
                "port": 0
            }
        }"#;

        let result = schema::validate_raw_json_config(config_json);
        assert!(result.is_err());

        if let Err(ConfigValidationError::SchemaValidationFailed { errors }) = result {
            assert!(!errors.is_empty());
            assert!(errors[0].contains("minimum") || errors[0].contains("out of range"));
        } else {
            panic!("Expected SchemaValidationFailed error");
        }
    }    #[test]
    fn it_should_apply_default_values() {
        let temp_dir = TempDir::new().unwrap();
        let private_key = temp_dir.path().join("id_rsa");
        let public_key = temp_dir.path().join("id_rsa.pub");

        // Create test SSH key files
        fs::write(&private_key, "dummy private key").unwrap();
        fs::write(&public_key, "dummy public key").unwrap();

        let config = EnvironmentConfig::new(
            "test-env".to_string(),
            SshCredentials::new(
                private_key.to_string_lossy().to_string(),
                public_key.to_string_lossy().to_string(),
                None, // Should default to "torrust"
                None, // Should default to 22
            ),
        );

        assert_eq!(config.ssh_credentials.username, "torrust");
        assert_eq!(config.ssh_credentials.port, 22);
    }

    #[test]
    fn it_should_fail_validation_for_missing_ssh_files() {
        let config = EnvironmentConfig::new(
            "test-env".to_string(),
            SshCredentials::new(
                "/nonexistent/private/key".to_string(),
                "/nonexistent/public/key.pub".to_string(),
                Some("torrust".to_string()),
                Some(22),
            ),
        );

        let result = config.validate();
        assert!(result.is_err());

        if let Err(ConfigValidationError::SshPrivateKeyNotFound { .. }) = result {
            // Expected error
        } else {
            panic!("Expected SshPrivateKeyNotFound error, got: {:?}", result);
        }
    }

    #[test]
    fn it_should_integrate_with_existing_environment_creation() {
        let temp_dir = TempDir::new().unwrap();
        let private_key = temp_dir.path().join("id_rsa");
        let public_key = temp_dir.path().join("id_rsa.pub");

        // Create test SSH key files with proper permissions
        fs::write(&private_key, "dummy private key").unwrap();
        fs::write(&public_key, "dummy public key").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&private_key).unwrap().permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&private_key, perms).unwrap();
        }

        let config = EnvironmentConfig::new(
            "test-env".to_string(),
            SshCredentials::new(
                private_key.to_string_lossy().to_string(),
                public_key.to_string_lossy().to_string(),
                Some("torrust".to_string()),
                Some(22),
            ),
        );

        // Validate configuration
        config.validate().unwrap();

        // Test integration with existing Environment::new() pattern
        // This would be the actual integration point with the domain layer
        let environment_name = crate::domain::environment::EnvironmentName::new(config.environment_name().to_string()).unwrap();
        let ssh_credentials = crate::domain::ssh::SshCredentials::new(
            config.ssh_credentials.private_key_path(),
            config.ssh_credentials.public_key_path(),
            config.ssh_credentials.username.clone(),
        );

        // This should integrate seamlessly with existing Environment::new()
        let _environment = crate::domain::environment::Environment::new(
            environment_name,
            ssh_credentials,
            config.ssh_credentials.port,
        );
    }
}
3. Must be a valid system username on the target server
4. Examples: 'torrust', 'admin', 'deploy-user'

For more information, see the SSH user configuration."
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Core Value Objects (2 hours)

- [ ] Create `src/domain/config/mod.rs` with module documentation
- [ ] Implement `EnvironmentConfig` value object with serde support
- [ ] Implement `SshCredentials` value object with defaults
- [ ] Add comprehensive documentation for all public APIs

### Phase 2: Domain Validation (2 hours)

- [ ] Implement validation functions in `src/domain/config/validation.rs`
- [ ] Add environment name validation with business rules
- [ ] Add SSH key path validation with file existence checks
- [ ] Add SSH port and username validation
- [ ] Implement tilde expansion for SSH key paths
- [ ] Add Unix permission checks for private keys

### Phase 3: Error Handling (1 hour)

- [ ] Create `ConfigValidationError` enum with thiserror
- [ ] Add structured error context for all validation scenarios
- [ ] Implement tiered help system with detailed troubleshooting
- [ ] Ensure all errors follow project error handling guidelines

### Phase 4: Comprehensive Testing (1-2 hours)

- [ ] Unit tests for `EnvironmentConfig` serialization/deserialization
- [ ] Unit tests for `SshCredentials` with default values
- [ ] Parameterized tests for environment name validation
- [ ] Tests for SSH key file validation with temporary files
- [ ] Tests for SSH port and username validation
- [ ] Tests for tilde expansion functionality
- [ ] Error scenario tests for all validation functions

## Acceptance Criteria

- [ ] `EnvironmentConfig` value object with complete serde support
- [ ] `SshCredentials` value object with sensible defaults
- [ ] Environment name validation with comprehensive business rules
- [ ] SSH key file validation with existence and permission checks
- [ ] Tilde expansion support for SSH key paths
- [ ] Explicit error enums with structured context and actionable messages
- [ ] Tiered help system for all validation errors
- [ ] 100% test coverage for all validation logic
- [ ] No external dependencies (pure domain objects)
- [ ] All code follows project conventions and passes linting

## Testing Strategy

### Unit Tests Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use rstest::rstest;

    mod environment_config {
        use super::*;

        #[test]
        fn it_should_serialize_to_json() {
            let config = create_valid_config();
            let json = serde_json::to_string(&config).unwrap();
            assert!(json.contains("production"));
        }

        #[test]
        fn it_should_deserialize_from_json() {
            let json = r#"{
                "environment": {"name": "test"},
                "ssh_credentials": {
                    "private_key_path": "/path/to/key",
                    "public_key_path": "/path/to/key.pub"
                }
            }"#;

            let config: EnvironmentConfig = serde_json::from_str(json).unwrap();
            assert_eq!(config.environment_name(), "test");
            assert_eq!(config.ssh_credentials.username, "torrust");
        }
    }

    mod ssh_credentials {
        use super::*;

        #[test]
        fn it_should_apply_default_username() {
            let creds = SshCredentials::new(
                "/key".to_string(),
                "/key.pub".to_string(),
                None,
                None,
            );
            assert_eq!(creds.username, "torrust");
        }

        #[test]
        fn it_should_apply_default_port() {
            let creds = SshCredentials::new(
                "/key".to_string(),
                "/key.pub".to_string(),
                None,
                None,
            );
            assert_eq!(creds.port, 22);
        }
    }

    mod validation {
        use super::*;

        #[rstest]
        #[case("production")]
        #[case("staging-1")]
        #[case("test_env")]
        #[case("dev2024")]
        fn it_should_accept_valid_environment_names(#[case] name: &str) {
            let result = validate_environment_name(name);
            assert!(result.is_ok());
        }

        #[rstest]
        #[case("", "Environment name cannot be empty")]
        #[case("-prod", "cannot start with hyphen")]
        #[case("_test", "cannot start with underscore")]
        #[case("prod env", "must contain only alphanumeric")]
        #[case("prod@env", "must contain only alphanumeric")]
        fn it_should_reject_invalid_environment_names(#[case] name: &str, #[case] expected_reason: &str) {
            let result = validate_environment_name(name);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.to_string().contains(expected_reason));
        }

        #[test]
        fn it_should_validate_existing_ssh_keys() {
            let temp_dir = TempDir::new().unwrap();
            let private_key = temp_dir.path().join("id_rsa");
            let public_key = temp_dir.path().join("id_rsa.pub");

            fs::write(&private_key, "private_key_content").unwrap();
            fs::write(&public_key, "public_key_content").unwrap();

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&private_key).unwrap().permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&private_key, perms).unwrap();
            }

            let result = validate_ssh_key_paths(
                &private_key.to_string_lossy(),
                &public_key.to_string_lossy(),
            );
            assert!(result.is_ok());
        }
    }
}
```

## Related Documentation

- [Domain Layer Architecture](../codebase-architecture.md#domain-layer)
- [Error Handling Guidelines](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Module Organization](../contributing/module-organization.md)

## Notes

- This subissue establishes the foundation for all configuration handling
- The domain objects are pure and can be reused across any delivery mechanism
- Comprehensive validation ensures data integrity at the domain level
- Error handling follows project conventions with actionable messages and tiered help
