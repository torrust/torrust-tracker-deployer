# Create Environment Command

**Issue**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34)
**Parent Epic**: #2 - Scaffolding for main app
**Related**: [Roadmap Task 1.5](../roadmap.md), [Environment entity](../codebase-architecture.md)

## Overview

Implement the `torrust-tracker-deployer create` command to create new deployment environments from configuration files. This command initializes environment data, validates configuration, and prepares the environment for provisioning.

**Architectural Integration**: This command integrates with existing infrastructure rather than creating new systems:

- Uses existing `Environment::new(environment_name, ssh_credentials, ssh_port)` for environment creation
  - Note: `ssh_credentials` uses existing `adapters::ssh::SshCredentials` type
  - Configuration layer converts from config format to this domain type
- Leverages existing `EnvironmentName` validation and `EnvironmentRepository` patterns
- Extends existing `TemplateManager` for configuration templates (no duplication)
- Follows existing **synchronous** command handler patterns from `provision.rs` with dependency injection
- Creates `Environment<Created>` state that integrates with existing state transitions
- Repository handles directory creation during `save()` for atomicity

## Goals

- [ ] Add CLI subcommand `create` with mandatory config file
- [ ] Support both JSON and TOML configuration formats
- [ ] Validate configuration against JSON schema
- [ ] Create environment data directory with persistent state
- [ ] Set initial environment state to "created"
- [ ] Environment name comes from configuration file only

**Future Enhancements** (implemented in later subissues):

- [ ] Provide embedded configuration template accessible via the binary (Subissue 7)

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (CLI interface) + Application (command logic)
**Module Path**: `src/presentation/console/subcommands/create/` + `src/application/commands/create/`
**Pattern**: CLI Subcommand + Command Pattern

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] **Domain layer** (`src/domain/config/`) contains pure configuration value objects and business validation rules
- [ ] **Application layer** (`src/application/commands/create/`) contains delivery-agnostic CreateCommand business logic
- [ ] **Infrastructure layer** (`src/infrastructure/templates/`) handles template storage and file system operations
- [ ] **Presentation layer** (`src/presentation/console/subcommands/create/`) handles Figment parsing, CLI arguments, and user feedback
- [ ] Use explicit error enums with `thiserror` in each layer for better error handling and pattern matching
- [ ] **Figment stays in presentation layer** - application layer receives clean domain objects

### Architectural Constraints

- [ ] CLI presentation layer only handles argument parsing and user feedback
- [ ] Configuration validation in application layer
- [ ] Environment state persistence through repository pattern
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Use Clock service for deterministic timestamps in tests

### DDD Layer Organization

**Domain Layer** (`src/domain/config/`):

- `EnvironmentConfig` and `SshCredentialsConfig` value objects (pure domain objects)
- Configuration validation business rules (domain validation logic)
- `ConfigValidationError` enum with thiserror for domain-specific validation errors

**Application Layer** (`src/application/commands/create/`):

- `CreateCommand` with business logic orchestration (receives clean domain objects)
- `CreateCommandError` enum for command-specific errors
- Integration between domain configuration and infrastructure services
- **No knowledge of Figment or specific parsing libraries** (delivery-agnostic)

**Infrastructure Layer** (`src/infrastructure/templates/`):

- Embedded template storage and retrieval
- File system operations for template generation
- JSON schema validation implementation

**Presentation Layer** (`src/presentation/console/subcommands/create/`):

- **Figment integration for configuration file parsing** (delivery mechanism)
- CLI argument parsing and validation
- Conversion from raw file data to domain objects
- User-friendly error message presentation using tiered help system
- Command help documentation and usage examples

### Anti-Patterns to Avoid

- ❌ Configuration validation in presentation layer
- ❌ Direct file system access from presentation layer
- ❌ Business logic in CLI subcommand modules

## Specifications

### CLI Interface

```bash
# Create environment from configuration file (JSON format)
torrust-tracker-deployer create environment --env-file ./config/environment.json

# Create environment with custom working directory
torrust-tracker-deployer --working-dir /path/to/workspace create environment --env-file ./config/environment.json

# Show help for create command and subcommands
torrust-tracker-deployer create --help
torrust-tracker-deployer create environment --help
torrust-tracker-deployer create template --help
```

**Note**: The `--working-dir` flag is available for both production use and testing, allowing users to manage multiple deployment workspaces.

**Note**: TOML support will be added in a separate issue after JSON implementation is complete.

**Future Enhancement**: Template generation functionality will be added in Subissue 7:

```bash
# Generate template configuration file (JSON format) - Future enhancement
torrust-tracker-deployer create template
# Creates: ./environment-template.json in current working directory

# Generate template in specific directory - Future enhancement
torrust-tracker-deployer create template ./config/environment.json
```

### Configuration File Format

#### JSON Format (Initial Implementation)

```json
{
  "environment": {
    "name": "production"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust",
    "port": 22
  }
}
```

**Future Enhancement**: TOML format will be added in a separate issue:

```toml
# TOML format will be supported in a future issue
[environment]
name = "production"

[ssh_credentials]
private_key_path = "~/.ssh/id_rsa"
public_key_path = "~/.ssh/id_rsa.pub"
username = "torrust"  # optional, default: "torrust"
port = 22            # optional, default: 22
```

### Configuration Loading with Figment

The configuration system uses clean DDD layering where **Figment stays in the presentation layer** as a delivery mechanism:

#### Domain Layer: Configuration Integration with Existing Environment System

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::domain::environment::{EnvironmentName, EnvironmentNameError};
use crate::adapters::ssh::SshCredentials; // Use existing SSH credentials from adapters layer
use crate::shared::Username;

// Configuration object that integrates with existing Environment::new() pattern
#[derive(Debug, Deserialize, Serialize)]
pub struct EnvironmentCreationConfig {
    pub environment: EnvironmentSection,
    pub ssh_credentials: SshCredentialsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvironmentSection {
    pub name: String, // Will be validated using existing EnvironmentName::new()
}

// Configuration-layer SSH credentials (distinct from adapters::ssh::SshCredentials)
#[derive(Debug, Deserialize, Serialize)]
pub struct SshCredentialsConfig {
    pub private_key_path: String,
    pub public_key_path: String,
    #[serde(default = "default_ssh_username")]
    pub username: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
}

impl EnvironmentCreationConfig {
    // Convert to existing domain objects for Environment::new()
    pub fn to_environment_params(self) -> Result<(EnvironmentName, SshCredentials, u16), CreateConfigError> {
        // Use existing EnvironmentName validation
        let environment_name = EnvironmentName::new(self.environment.name)
            .map_err(CreateConfigError::InvalidEnvironmentName)?;

        // Convert string username to Username type
        let username = Username::new(self.ssh_credentials.username)
            .map_err(CreateConfigError::InvalidUsername)?;

        // Convert to existing adapters::ssh::SshCredentials domain object
        let ssh_credentials = SshCredentials::new(
            PathBuf::from(self.ssh_credentials.private_key_path),
            PathBuf::from(self.ssh_credentials.public_key_path),
            username,
        );

        Ok((environment_name, ssh_credentials, self.ssh_credentials.port))
    }
}

// Error enum that wraps existing domain errors
#[derive(Debug, Error)]
pub enum CreateConfigError {
    #[error("Invalid environment name")]
    InvalidEnvironmentName(#[source] EnvironmentNameError),

    #[error("Invalid SSH username")]
    InvalidUsername(#[source] UsernameError), // From shared::Username

    #[error("SSH key file not found: {path}")]
    SshKeyFileNotFound { path: PathBuf },

    #[error("SSH key file not readable: {path}")]
    SshKeyFileNotReadable { path: PathBuf },
}

fn default_ssh_username() -> String {
    "torrust".to_string()
}

fn default_ssh_port() -> u16 {
    22
}
```

#### Application Layer: Integration with Existing Patterns

```rust
use crate::domain::environment::{Environment, EnvironmentRepository};
use crate::shared::Clock;
use std::sync::Arc;

// Command follows existing SYNCHRONOUS patterns from provision.rs
pub struct CreateCommand {
    environment_repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateCommand {
    pub fn new(
        environment_repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            environment_repository,
            clock,
        }
    }

    // Follow existing execute() pattern from provision command (SYNCHRONOUS)
    pub fn execute(&self, config: EnvironmentCreationConfig) -> Result<Environment<Created>, CreateCommandError> {
        // Convert config to existing domain objects
        let (environment_name, ssh_credentials, ssh_port) = config
            .to_environment_params()
            .map_err(CreateCommandError::InvalidConfiguration)?;

        // Check if environment already exists
        if self.environment_repository
            .exists(&environment_name)
            .map_err(CreateCommandError::RepositoryError)?
        {
            return Err(CreateCommandError::EnvironmentAlreadyExists {
                name: environment_name.as_str().to_string(),
            });
        }

        // Use existing Environment::new() pattern - no create_from_config() needed
        let environment = Environment::new(environment_name, ssh_credentials, ssh_port);

        // Repository handles directory creation during save() for atomicity
        self.environment_repository
            .save(&environment.into_any())
            .map_err(CreateCommandError::RepositoryError)?;

        Ok(environment)
    }
}

// Error enum following existing command error patterns with .help() methods
#[derive(Debug, Error)]
pub enum CreateCommandError {
    #[error("Configuration validation failed")]
    InvalidConfiguration(#[source] CreateConfigError),

    #[error("Environment '{name}' already exists")]
    EnvironmentAlreadyExists { name: String },

    #[error("Failed to save environment")]
    RepositoryError(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl CreateCommandError {
    /// Get detailed troubleshooting guidance for this error
    pub fn help(&self) -> &'static str {
        match self {
            Self::EnvironmentAlreadyExists { .. } => {
                "Environment Already Exists - Detailed Troubleshooting:

1. List existing environments to verify:
   torrust-tracker-deployer list

2. If you want to use a different name:
   - Update the 'name' field in your configuration file
   - Run the create command again

3. If you want to replace the existing environment:
   - Destroy the existing environment: torrust-tracker-deployer destroy <env-name>
   - Then run create again

4. If you want to work with the existing environment:
   - Use other commands (provision, configure, etc.) with the existing name
   - No need to create it again

For more information, see the documentation on environment management."
            }
            Self::InvalidConfiguration(_) => {
                "Invalid Configuration - Detailed Troubleshooting:

1. Check your configuration file syntax (valid JSON format)
2. Verify all required fields are present:
   - environment.name
   - ssh_credentials.private_key_path
   - ssh_credentials.public_key_path
3. Ensure SSH key files exist and are readable
4. Verify environment name follows naming rules:
   - Alphanumeric characters, hyphens, and underscores
   - Must start with alphanumeric character
5. Check SSH port is valid (1-65535)

For configuration examples, see the documentation or generate a template."
            }
            Self::RepositoryError(_) => {
                "Repository Error - Detailed Troubleshooting:

1. Check file system permissions for data directory
2. Verify sufficient disk space
3. Ensure no other process is accessing the environment
4. Check for file system errors or corruption

If the problem persists, report it with system details."
            }
        }
    }
}
```

#### Presentation Layer: Figment Integration

````rust
use figment::{Figment, providers::{Format, Json, Serialized}};

// Presentation layer - handles file parsing with Figment
pub struct CreateSubcommand {
    create_command: CreateCommand,
}

impl CreateSubcommand {
    pub fn execute(&self, config_path: &PathBuf) -> Result<(), CreateSubcommandError> {
        // Figment parsing logic stays in presentation layer
        let config: EnvironmentCreationConfig = self.load_config_from_file(config_path)?;

        // Pass clean domain object to application layer
        self.create_command.execute(config)
            .map_err(CreateSubcommandError::CommandFailed)?;

        println!("Environment created successfully!");
        Ok(())
    }

    fn load_config_from_file(&self, config_path: &PathBuf) -> Result<EnvironmentCreationConfig, CreateSubcommandError> {
        Figment::new()
            .merge(Serialized::defaults(EnvironmentCreationConfig::default()))
            .merge(Json::file(config_path))
            .extract()
            .map_err(|e| CreateSubcommandError::ConfigParsingFailed {
                path: config_path.clone(),
                source: e,
            })
    }
}

#[derive(Debug, Error)]
pub enum CreateSubcommandError {
    #[error("Failed to parse configuration file: {path}")]
    ConfigParsingFailed {
        path: PathBuf,
        #[source]
        source: figment::Error,
    },

    #[error("Command execution failed")]
    CommandFailed(#[source] CreateCommandError),
}
```

#### Benefits of This Architecture

**✅ Delivery-Agnostic Application Layer**:
- `CreateCommand` can be reused from CLI, REST API, GraphQL, or any other delivery mechanism
- Application logic is completely independent of how configuration is obtained
- Easy to test with simple domain objects

**✅ Clean Separation of Concerns**:
- Domain: Business rules and validation logic
- Application: Use case orchestration
- Presentation: File parsing and user interaction

**✅ Easy to Replace Figment**:
- If we want to switch from Figment to another config library, only presentation layer changes
- Domain and application layers remain unchanged

**✅ Future REST API Ready**:
```rust
// Future REST API endpoint - same application command
#[post("/environments")]
async fn create_environment(config: Json<EnvironmentCreationConfig>) -> Result<Json<Environment>, Error> {
    // Same CreateCommand, different delivery mechanism
    create_command.execute(config.into_inner())
        .map(Json)
        .map_err(Error::from)
}
```

**Unit Tests for Serialization/Deserialization**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn it_should_deserialize_valid_json_configuration() {
        let json = r#"{
            "environment": {"name": "test-env"},
            "ssh_credentials": {
                "private_key_path": "/home/user/.ssh/id_rsa",
                "public_key_path": "/home/user/.ssh/id_rsa.pub"
            }
        }"#;

        let config: EnvironmentCreationConfig = serde_json::from_str(json)
            .expect("Valid JSON should deserialize successfully");

        assert_eq!(config.environment.name, "test-env");
        assert_eq!(config.ssh_credentials.username, "torrust"); // default value
        assert_eq!(config.ssh_credentials.port, 22); // default value
    }

    #[test]
    fn it_should_apply_default_values_for_optional_fields() {
        let json = r#"{
            "environment": {"name": "test"},
            "ssh_credentials": {
                "private_key_path": "/path/to/key",
                "public_key_path": "/path/to/key.pub"
            }
        }"#;

        let config: EnvironmentCreationConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.ssh_credentials.username, "torrust");
        assert_eq!(config.ssh_credentials.port, 22);
    }

    #[test]
    fn it_should_fail_deserialization_with_missing_required_fields() {
        let json = r#"{"environment": {"name": "test"}}"#; // Missing ssh_credentials

        let result: Result<EnvironmentCreationConfig, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_convert_to_existing_domain_objects() {
        let temp_dir = TempDir::new().unwrap();
        let private_key = temp_dir.path().join("id_rsa");
        let public_key = temp_dir.path().join("id_rsa.pub");

        // Create the key files
        fs::write(&private_key, "private_key_content").unwrap();
        fs::write(&public_key, "public_key_content").unwrap();

        let config = EnvironmentCreationConfig {
            environment: EnvironmentSection {
                name: "test-env".to_string(),
            },
            ssh_credentials: SshCredentialsConfig {
                private_key_path: private_key.to_string_lossy().to_string(),
                public_key_path: public_key.to_string_lossy().to_string(),
                username: "torrust".to_string(),
                port: 22,
            },
        };

        let result = config.to_environment_params();
        assert!(result.is_ok());
        let (env_name, ssh_creds, port) = result.unwrap();
        assert_eq!(env_name.as_str(), "test-env");
        assert_eq!(port, 22);
    }

    #[test]
    fn it_should_fail_conversion_with_invalid_environment_name() {
        let config = EnvironmentCreationConfig {
            environment: EnvironmentSection {
                name: "invalid name with spaces".to_string(), // Invalid name
            },
            ssh_credentials: SshCredentialsConfig {
                private_key_path: "/valid/path".to_string(),
                public_key_path: "/valid/path".to_string(),
                username: "torrust".to_string(),
                port: 22,
            },
        };

        let result = config.to_environment_params();
        assert!(matches!(result, Err(CreateConfigError::InvalidEnvironmentName(_))));
    }

    #[test]
    fn it_should_fail_conversion_with_invalid_username() {
        let config = EnvironmentCreationConfig {
            environment: EnvironmentSection {
                name: "valid-name".to_string(),
            },
            ssh_credentials: SshCredentialsConfig {
                private_key_path: "/valid/path".to_string(),
                public_key_path: "/valid/path".to_string(),
                username: "".to_string(), // Invalid empty username
                port: 22,
            },
        };

        let result = config.to_environment_params();
        assert!(matches!(result, Err(CreateConfigError::InvalidUsername(_))));
    }
}
````

**Future Enhancement**: TOML and environment variable support can be easily added in the presentation layer:

```rust
use figment::providers::{Toml, Env};

// Presentation layer - enhanced config loading
fn load_config_from_file(&self, config_path: &PathBuf) -> Result<EnvironmentCreationConfig, CreateSubcommandError> {
    Figment::new()
        .merge(Serialized::defaults(EnvironmentCreationConfig::default()))
        .merge(Json::file(&config_path))        // JSON support
        .merge(Toml::file(&config_path))        // Future: TOML support
        .merge(Env::prefixed("TORRUST_"))       // Future: Environment variables
        .extract()
        .map_err(|e| CreateSubcommandError::ConfigParsingFailed {
            path: config_path.clone(),
            source: e,
        })
}
```

### Configuration Template (Future Enhancement - Subissue 7)

Template generation will be available in a future enhancement:

```bash
# Generate template configuration file (JSON format) - Future enhancement
torrust-tracker-deployer create template
# Creates: ./environment-template.json in current working directory

# Generate template in specific directory - Future enhancement
torrust-tracker-deployer create template ./config/environment.json
```

Template content (JSON):

```json
{
  "environment": {
    "name": "REPLACE_WITH_ENVIRONMENT_NAME"
  },
  "ssh_credentials": {
    "private_key_path": "REPLACE_WITH_SSH_PRIVATE_KEY_PATH",
    "public_key_path": "REPLACE_WITH_SSH_PUBLIC_KEY_PATH",
    "username": "torrust",
    "port": 22
  }
}
```

**Note**: Comments in JSON templates will be provided in accompanying documentation since JSON doesn't support comments natively.

### JSON Schema

```json
{
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
}
```

### Environment State Management

After successful creation:

```bash
# Directory structure created:
./data/{ENV_NAME}/
├── environment.json          # Serialized environment entity (persistent user data)
└── traces/                   # Command execution traces (persistent user data)

./build/{ENV_NAME}/
└── templates/               # Generated templates (ephemeral, created when provisioned)

# Example: ./data/my-environment/environment.json
{
  "Created": {
    "context": {
      "user_inputs": {
        "name": "my-environment",
        "instance_name": "torrust-tracker-vm-my-environment",
        "profile_name": "torrust-profile-my-environment",
        "ssh_credentials": {
          "ssh_priv_key_path": "/home/user/.ssh/id_rsa",
          "ssh_pub_key_path": "/home/user/.ssh/id_rsa.pub",
          "ssh_username": "torrust"
        },
        "ssh_port": 22
      },
      "internal_config": {
        "build_dir": "build/my-environment",
        "data_dir": "data/my-environment"
      },
      "runtime_outputs": {}
    },
    "state": null
  }
}
```

**Important Data Management Distinction**:

- **`./data/{ENV_NAME}/`**: **Persistent user data** - Contains user-created environments and traces. This data belongs to the user and should never be automatically deleted by the application. Users must manually delete environments they no longer need.

- **`./build/{ENV_NAME}/`**: **Ephemeral application data** - Contains generated templates, configurations, and other files that can be rebuilt from the data in `./data/`. The application can safely delete and recreate this data at any time as needed.

**Integration with Existing Environment System**:

The create command integrates with the existing Environment<S> type-state pattern:

- Creates `Environment<Created>` using existing `Environment::new(environment_name, ssh_credentials, ssh_port)`
- Persists via existing `repository.save(&environment.into_any())` pattern
- Directory structure follows existing patterns from e2e tests and provision command
- Environment state JSON follows existing serialization format with type-erased states

## Implementation Plan

This feature will be implemented as separate subissues for incremental delivery:

**Key Architecture Decisions**:

- All commands are **synchronous** (not async) to match existing patterns
- Configuration uses `SshCredentialsConfig` (distinct from `adapters::ssh::SshCredentials`) to avoid naming conflicts
- Conversion from config strings to domain types (e.g., `Username`) happens at the boundary
- Repository handles directory creation during `save()` for atomicity
- No `Environment::create_from_config()` method - use existing `Environment::new()` directly
- All errors implement `.help()` methods following the tiered help system pattern
- `--working-dir` flag is available in production CLI (not just tests)
- JSON Schema validation is optional and can be deferred
- Subissues 6-7 (template generation) are optional enhancements, not required for MVP

### Subissue 1: Configuration Infrastructure Integration

**Issue**: [#35](https://github.com/torrust/torrust-tracker-deployer/issues/35)
**Estimated Time**: 2-3 hours  
**Details**: See [35-subissue-1-core-configuration-infrastructure.md](./35-subissue-1-core-configuration-infrastructure.md)

**Summary**: Create domain-layer configuration value objects (`EnvironmentCreationConfig`, `SshCredentialsConfig`) that convert to existing domain types. Use existing `EnvironmentName::new()` validation and convert to `adapters::ssh::SshCredentials`. Handle `Username` type conversion from string. JSON Schema validation is optional/can be deferred.

### Subissue 2: Create Command Implementation

**Issue**: [#36](https://github.com/torrust/torrust-tracker-deployer/issues/36)
**Depends On**: [#35](https://github.com/torrust/torrust-tracker-deployer/issues/35)
**Estimated Time**: 3-4 hours  
**Details**: See [36-subissue-2-application-layer-command.md](./36-subissue-2-application-layer-command.md)

**Summary**: Create **synchronous** `CreateCommand` following existing command handler patterns from `provision.rs`. Use existing `Environment::new()` directly (no `create_from_config()` method needed). Repository handles all directory creation during `save()` for atomicity. Implement comprehensive error handling with `.help()` methods following the tiered help system pattern. Check for existing environments before creation.

### Subissue 3: CLI Presentation Layer

**Issue**: [#37](https://github.com/torrust/torrust-tracker-deployer/issues/37)
**Depends On**: [#36](https://github.com/torrust/torrust-tracker-deployer/issues/36)
**Estimated Time**: 3-4 hours  
**Details**: See [37-subissue-3-cli-presentation-layer.md](./37-subissue-3-cli-presentation-layer.md)

**Summary**: Create `create` subcommand with Figment integration for configuration parsing, argument handling, conversion to domain objects, and user-friendly error presentation using tiered help system. Add `--working-dir` flag to main CLI for production use (not just tests), allowing users to manage multiple deployment workspaces.

### Subissue 4: E2E Black Box Test for Create Command

**Issue**: [#38](https://github.com/torrust/torrust-tracker-deployer/issues/38)
**Depends On**: [#37](https://github.com/torrust/torrust-tracker-deployer/issues/37)
**Estimated Time**: 3-4 hours  
**Details**: See [38-subissue-4-e2e-black-box-test.md](./38-subissue-4-e2e-black-box-test.md)

**Summary**: Implement a true black-box end-to-end test that runs the production application as an external process. Test complete workflow from configuration file to persisted environment state using temporary directories for isolation. Test with `--working-dir` argument to verify workspace management functionality.

### Subissue 5: Update E2E Full Tests to Use Create Command

**Issue**: [#39](https://github.com/torrust/torrust-tracker-deployer/issues/39)
**Depends On**: [#36](https://github.com/torrust/torrust-tracker-deployer/issues/36)
**Estimated Time**: 1-2 hours  
**Details**: See [39-subissue-5-update-e2e-full-tests.md](./39-subissue-5-update-e2e-full-tests.md)

**Summary**: Update `src/bin/e2e_tests_full.rs` to use the new CreateCommand handler for environment creation instead of direct environment creation. Ensures the comprehensive test suite exercises the complete create command functionality.

### Subissue 6: Template System Integration

**Issue**: [#40](https://github.com/torrust/torrust-tracker-deployer/issues/40)
**Status**: OPTIONAL ENHANCEMENT
**Estimated Time**: 2-3 hours  
**Details**: See [40-subissue-6-template-system-integration.md](./40-subissue-6-template-system-integration.md)

**Summary**: Extend existing `TemplateManager` for configuration templates using existing template infrastructure, rust-embed patterns, and Tera variable syntax. No duplication of existing functionality.

**Dependencies**: This is an optional enhancement that can be implemented after the core create command functionality (Subissues 1-5) is complete. Not required for MVP.

### Subissue 7: Template Generation Support

**Issue**: [#41](https://github.com/torrust/torrust-tracker-deployer/issues/41)
**Depends On**: [#40](https://github.com/torrust/torrust-tracker-deployer/issues/40)
**Status**: OPTIONAL ENHANCEMENT
**Estimated Time**: 1-2 hours  
**Details**: See [41-subissue-7-template-generation-support.md](./41-subissue-7-template-generation-support.md)

**Dependencies**:

- **Requires Subissue 6** (Template System Integration) to be completed first
- Optional enhancement, not required for MVP

**Summary**:

- [ ] Implement `template` subcommand functionality
- [ ] Support JSON template format
- [ ] Template validation and helpful placeholder content

### Subissue 8: Fix Destroy Command: Handle Created State Gracefully

**Issue**: [#50](https://github.com/torrust/torrust-tracker-deployer/issues/50)
**Status**: Not Started
**Estimated Time**: 4-5 hours
**Details**: See [50-subissue-8-fix-destroy-command-created-state-handling.md](./50-subissue-8-fix-destroy-command-created-state-handling.md)

**Summary**: Fix critical bug where destroy command fails for Created state environments

**Dependencies**: None (independent, must be completed before Subissue 9)

### Subissue 9: Fix Destroy Command: Add Working Directory Support

**Issue**: [#51](https://github.com/torrust/torrust-tracker-deployer/issues/51)
**Status**: Not Started
**Estimated Time**: 3-4 hours
**Details**: See [51-subissue-9-fix-destroy-command-working-dir-support.md](./51-subissue-9-fix-destroy-command-working-dir-support.md)

**Summary**: Add --working-dir parameter support to destroy command

**Dependencies**: [#50](https://github.com/torrust/torrust-tracker-deployer/issues/50) must be completed first

### Subissue 10: Document Create Environment Command

**Issue**: [#52](https://github.com/torrust/torrust-tracker-deployer/issues/52)
**Status**: Not Started
**Estimated Time**: 4 hours
**Details**: See [52-subissue-10-document-create-environment-command.md](./52-subissue-10-document-create-environment-command.md)

**Summary**: Add comprehensive user-facing documentation for create environment command

**Dependencies**: Subissues 1-7 (create command implementation)

### Future Subissue: TOML Format Support (Separate Issue)

**Note**: This will be implemented as a separate issue after the JSON implementation is complete.

- [ ] Add TOML parsing support to Figment configuration
- [ ] Add `--format` flag to CLI for template generation
- [ ] Add TOML template embedded in binary
- [ ] Update documentation with TOML examples
- [ ] Add TOML-specific tests

## Future Enhancements

### Configuration Format Extensibility

This implementation uses the **Figment** crate to provide a foundation for future configuration format support. While the initial release supports JSON only, the architecture is designed to easily add:

- **TOML Support**: Future enhancement for human-readable configuration files

  ```bash
  torrust-tracker-deployer create --env-file config.toml
  torrust-tracker-deployer create --generate-template --format toml
  ```

- **Environment Variable Support**: Configuration via environment variables for containerized deployments

  ```bash
  TTD_ENV_NAME=staging TTD_PROVIDER=lxd torrust-tracker-deployer create
  ```

- **Mixed Sources**: Figment's layered configuration system can combine multiple sources with proper precedence

The JSON-first approach enables faster delivery while maintaining architectural flexibility for future needs.

## Acceptance Criteria

- [ ] `torrust-tracker-deployer create --env-file config.json` loads JSON configuration
- [ ] Configuration validation rejects invalid files with clear error messages
- [ ] Environment directory `./data/{ENV_NAME}/` is created with proper structure (persistent user data)
- [ ] Build directory `./build/{ENV_NAME}/` is created as needed (ephemeral application data)
- [ ] Environment state is set to "created" in persistent storage
- [ ] SSH key paths are validated to exist and be readable
- [ ] Command includes proper help documentation
- [ ] All error messages follow project error handling guidelines (actionable and clear)

**Future Enhancement Criteria** (Subissue 7):

- [ ] `torrust-tracker-deployer create --generate-template` generates JSON template in current directory
- [ ] `torrust-tracker-deployer create --generate-template ./config/environment.json` generates JSON template at specified path
- [ ] Configuration templates are embedded in binary (no external template dependencies)

## 🎯 Implementation Summary

This create command implementation leverages existing infrastructure:

### ✅ **Uses Existing Systems**

- **Environment Creation**: `Environment::new(environment_name, ssh_credentials, ssh_port)` pattern
  - Uses existing `adapters::ssh::SshCredentials` type (not creating a new one)
  - Configuration layer converts from `SshCredentialsConfig` to this existing type
- **Validation**: Existing `EnvironmentName::new()` and `EnvironmentNameError` handling
- **Username Conversion**: String to `Username` type conversion at configuration boundary
- **Persistence**: Existing `EnvironmentRepository` trait and `repository.save(&environment.into_any())` pattern
  - Repository handles directory creation during save for atomicity
- **Templates**: Existing `TemplateManager` with rust-embed and Tera rendering (optional enhancement)
- **Testing**: Existing `MockClock`, test builders, and comprehensive testing patterns
- **Error Handling**: Existing thiserror-based error patterns with actionable `.help()` methods

### ✅ **Follows Existing Patterns**

- **Command Structure**: Mirrors `ProvisionCommandHandler` with injected dependencies
- **Synchronous Execution**: All commands are sync (not async) following existing patterns
- **State Management**: Creates `Environment<Created>` that works with existing state transitions
- **Directory Structure**: Follows existing `./data/` (persistent) and `./build/` (ephemeral) separation
  - Repository creates directories atomically during save
- **DDD Architecture**: Clean layer separation with presentation handling Figment parsing
- **Error Help System**: All errors implement `.help()` methods with detailed troubleshooting

### ✅ **No Duplication**

- **No new SSH credentials type** - uses existing `adapters::ssh::SshCredentials`
  - Configuration uses `SshCredentialsConfig` (different name) to avoid conflicts
- **No new template system** - extends existing TemplateManager (optional enhancement)
- **No new validation** - uses existing EnvironmentName and Username validation
- **No new repository** - uses existing EnvironmentRepository trait
- **No new error patterns** - follows existing thiserror and actionable error conventions
- **No create_from_config()** - uses existing `Environment::new()` directly

### ⚙️ **Key Implementation Decisions**

- **Synchronous by design**: Matches existing command patterns
- **Repository owns directories**: Creates them atomically during save
- **Type conversion at boundaries**: Config strings → Domain types (Username, PathBuf)
- **Clean naming**: `SshCredentialsConfig` (config) vs `SshCredentials` (domain)
- **Working directory support**: `--working-dir` flag available in production
- **MVP focused**: Template generation (Subissues 6-7) is optional
- **JSON Schema optional**: Can defer validation to domain layer initially

The result is a create command that feels native to the existing codebase and leverages all the excellent infrastructure already in place.

## Related Documentation

- [Environment entity specification](../codebase-architecture.md#domain-layer)
- [Command pattern implementation](../codebase-architecture.md#application-layer)
- [Error handling guidelines](../contributing/error-handling.md)
- [CLI subcommand structure](../codebase-architecture.md#presentation-layer)
- [E2E test configuration examples](../../src/bin/e2e_tests_full.rs)

## Notes

### Future Enhancements

- **Provider Configuration**: When adding infrastructure providers (Hetzner, AWS, etc.), provider-specific configuration will be stored in separate files (e.g., `./data/{ENV_NAME}/provider.json`) to avoid configuration file complexity
- **Multiple Tracker Support**: The configuration schema supports separate arrays for HTTP and UDP tracker configurations for future multi-tracker deployments. HTTP trackers have a `private` field, while UDP trackers are simpler with only port configuration.
- **Environment Variables**: Figment makes it easy to add environment variable support (e.g., `TORRUST_SSH_CREDENTIALS_USERNAME=custom`) without changing the core configuration structure
- **Configuration Assistant**: Future AI-powered configuration generation tool could make the TOML vs JSON choice less important for users

### Implementation Considerations

- Use Figment crate for configuration loading to enable future environment variable support
- Start with JSON format for implementation simplicity, add TOML as enhancement
- Configuration validation should provide specific field-level error messages with `.help()` methods
- Template generation should include helpful comments explaining each field (optional enhancement)
- All file operations should be testable using temporary directories
- Configuration parsing should auto-detect format based on file extension (.json/.toml)
- Environment name comes exclusively from configuration file to avoid inconsistencies
- Figment allows easy addition of default values through serde defaults and Serialized provider
- **Data vs Build Directories**: Maintain clear separation between persistent user data (`./data/`) and ephemeral application data (`./build/`) that can be safely recreated
- **Synchronous execution**: All commands follow sync patterns (not async) from existing codebase
- **Type naming**: Use `SshCredentialsConfig` for configuration to avoid conflict with `adapters::ssh::SshCredentials`
- **Username conversion**: Convert string username to `Username` type at configuration boundary
- **Repository atomicity**: Repository handles directory creation during save for atomic operations
- **No new constructors**: Use existing `Environment::new()` - no need for `create_from_config()`
- **Working directory flag**: `--working-dir` available in production CLI for workspace management
- **JSON Schema**: Optional validation that can be deferred - domain validation is primary
- **Template system**: Subissues 6-7 are optional enhancements, not required for MVP
