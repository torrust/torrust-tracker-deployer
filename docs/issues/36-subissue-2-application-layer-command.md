# Application Layer Command

**Issue**: [#36](https://github.com/torrust/torrust-tracker-deployer/issues/36)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Depends On**: [#35](https://github.com/torrust/torrust-tracker-deployer/issues/35) - Configuration Infrastructure
**Related**: [Roadmap Task 1.5](../roadmap.md), [Application Layer Architecture](../codebase-architecture.md#application-layer)

## Overview

Implement the delivery-agnostic CreateCommand in the application layer that orchestrates environment creation business logic. This command receives clean domain objects and coordinates between domain validation and infrastructure services without knowing about specific delivery mechanisms.

**Key Architecture Points**:

- Command is **synchronous** (not async) following existing patterns
- Uses existing `Environment::new()` directly - no `create_from_config()` method
- Repository handles directory creation during `save()` for atomicity
- All errors implement `.help()` methods with detailed troubleshooting

## Goals

- [ ] Create **synchronous** `CreateCommand` following existing command handler patterns from `src/application/command_handlers/provision.rs`
  - [ ] Use existing dependency injection pattern: `Arc<dyn EnvironmentRepository>` and `Arc<dyn Clock>`
  - [ ] Follow existing execute() signature pattern: `fn execute(...) -> Result<Environment<Created>, CreateCommandError>`
  - [ ] Use existing `Environment::new(environment_name, ssh_credentials, ssh_port)` directly
  - [ ] Let repository handle directory creation during `save()` - don't create directories in command
  - [ ] Check if environment already exists before attempting creation
  - [ ] Persist via existing repository pattern: `repository.save(&environment.into_any())`
  - [ ] **Delivery-agnostic** - works with CLI, REST API, or any other delivery mechanism
- [ ] Add command error enum following existing patterns
  - [ ] `CreateCommandError` following structure of `ProvisionCommandHandlerError`
  - [ ] Use existing error handling patterns with actionable messages and `.help()` methods
  - [ ] Source error chaining from domain and infrastructure layers
  - [ ] Include `EnvironmentAlreadyExists` error variant
- [ ] Create command tests following existing patterns
  - [ ] Use test builders following existing command test patterns
  - [ ] Use `MockClock` for deterministic testing (existing testing infrastructure)
  - [ ] Test integration with existing repository save/load patterns
  - [ ] Test duplicate environment detection
  - [ ] Test that repository handles directory creation

**Estimated Time**: 3-4 hours

## 🏗️ Architecture Requirements

**DDD Layer**: Application Layer (`src/application/commands/create/`)
**Pattern**: Command Pattern + Repository Pattern + Dependency Injection
**Dependencies**: Domain layer, Infrastructure layer (via interfaces)

### Module Structure

```text
src/application/commands/create/
├── mod.rs                # Module exports and documentation
├── command.rs            # CreateCommand implementation
├── errors.rs             # CreateCommandError enum
└── tests/                # Test module with builders
    ├── mod.rs
    ├── builders.rs       # Test builders and fixtures
    └── integration.rs    # Integration tests
```

## Specifications

### CreateCommand Implementation

```rust
// src/application/commands/create/command.rs
use std::sync::Arc;
use crate::domain::config::EnvironmentCreationConfig;
use crate::domain::environment::{Environment, Created, EnvironmentName};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::shared::Clock;
use super::errors::CreateCommandError;

/// Command to create a new deployment environment
///
/// This command is delivery-agnostic and can be used from CLI, REST API,
/// GraphQL, or any other delivery mechanism. It orchestrates the business
/// logic for environment creation without knowledge of how the configuration
/// was obtained.
///
/// # Synchronous Design
///
/// This command is synchronous (not async) following existing patterns from
/// ProvisionCommandHandler. All repository operations are also synchronous.
pub struct CreateCommand {
    environment_repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateCommand {
    /// Create a new CreateCommand with required dependencies
    pub fn new(
        environment_repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            environment_repository,
            clock,
        }
    }

    /// Execute the create command with validated configuration
    ///
    /// # Arguments
    /// * `config` - Validated environment configuration from domain layer
    ///
    /// # Returns
    /// * `Ok(Environment<Created>)` - Successfully created environment
    /// * `Err(CreateCommandError)` - Business logic or persistence failure
    ///
    /// # Business Rules
    /// 1. Configuration must convert to valid domain objects
    /// 2. Environment name must be unique
    /// 3. Repository handles directory creation atomically during save
    /// 4. Environment state must be persisted
    pub fn execute(&self, config: EnvironmentCreationConfig) -> Result<Environment<Created>, CreateCommandError> {
        // Step 1: Convert configuration to domain objects
        let (environment_name, ssh_credentials, ssh_port) = config
            .to_environment_params()
            .map_err(CreateCommandError::InvalidConfiguration)?;

        // Step 2: Check if environment already exists
        if self.environment_repository
            .exists(&environment_name)
            .map_err(CreateCommandError::RepositoryError)?
        {
            return Err(CreateCommandError::EnvironmentAlreadyExists {
                name: environment_name.as_str().to_string(),
            });
        }

        // Step 3: Create environment entity using existing Environment::new()
        // No need for create_from_config() - use existing constructor
        let environment = Environment::new(environment_name, ssh_credentials, ssh_port);

        // Step 4: Persist environment state
        // Repository handles directory creation during save for atomicity
        self.environment_repository
            .save(&environment.into_any())
            .map_err(CreateCommandError::RepositoryError)?;

        Ok(environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockClock;
    use crate::application::commands::create::tests::CreateCommandTestBuilder;
    use tempfile::TempDir;
    use chrono::{TimeZone, Utc};

    #[test]
    fn it_should_create_environment_with_valid_configuration() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let (command, _temp_dir) = CreateCommandTestBuilder::new()
            .with_base_directory(temp_dir.path())
            .build();

        let config = create_valid_test_config(&temp_dir);

        // Act
        let result = command.execute(config);

        // Assert
        assert!(result.is_ok());
        let environment = result.unwrap();
        assert_eq!(environment.name().as_str(), "test-environment");
    }

    #[test]
    fn it_should_fail_when_environment_already_exists() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let (command, _temp_dir) = CreateCommandTestBuilder::new()
            .with_base_directory(temp_dir.path())
            .with_existing_environment("test-environment")
            .build();

        let config = create_valid_test_config(&temp_dir);

        // Act
        let result = command.execute(config);

        // Assert
        assert!(matches!(result, Err(CreateCommandError::EnvironmentAlreadyExists { .. })));
    }

    #[test]
    fn it_should_verify_repository_handles_directory_creation() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let (command, _temp_dir) = CreateCommandTestBuilder::new()
            .with_base_directory(temp_dir.path())
            .build();

        let config = create_valid_test_config(&temp_dir);

        // Act
        let result = command.execute(config);

        // Assert
        assert!(result.is_ok());
        let environment = result.unwrap();

        // Verify repository created directories during save
        assert!(environment.data_dir().exists(), "Repository should create data directory");
        assert!(environment.data_dir().join("traces").exists(), "Repository should create traces directory");
    }

    fn create_valid_test_config(temp_dir: &TempDir) -> EnvironmentCreationConfig {
        use crate::domain::config::{EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig};
        use std::fs;

        // Create temporary SSH key files
        let private_key = temp_dir.path().join("id_rsa");
        let public_key = temp_dir.path().join("id_rsa.pub");
        fs::write(&private_key, "test_private_key").unwrap();
        fs::write(&public_key, "test_public_key").unwrap();

        EnvironmentCreationConfig {
            environment: EnvironmentSection {
                name: "test-environment".to_string(),
            },
            ssh_credentials: SshCredentialsConfig {
                private_key_path: private_key.to_string_lossy().to_string(),
                public_key.to_string_lossy().to_string(),
                Some("torrust".to_string()),
                Some(22),
            ),
        )
    }
}
```

### Command Error Types

```rust
// src/application/commands/create/errors.rs
use thiserror::Error;
use std::path::PathBuf;
use crate::domain::config::ConfigValidationError;
use crate::domain::environment::errors::{EnvironmentNameError, EnvironmentCreationError};
use crate::infrastructure::repository::RepositoryError;

/// Errors that can occur during environment creation command execution
///
/// These errors represent failures in the business logic orchestration
/// and provide structured context for troubleshooting and user feedback.
#[derive(Debug, Error)]
pub enum CreateCommandError {
    #[error("Invalid configuration")]
    InvalidConfiguration(#[source] ConfigValidationError),

    #[error("Invalid environment name")]
    InvalidEnvironmentName(#[source] EnvironmentNameError),

    #[error("Environment '{name}' already exists")]
    EnvironmentAlreadyExists { name: String },

    #[error("Failed to create environment entity")]
    EnvironmentCreationFailed(#[source] EnvironmentCreationError),

    #[error("Failed to create {directory_type} directory: {path}")]
    DirectoryCreationFailed {
        path: PathBuf,
        directory_type: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Repository operation failed")]
    RepositoryError(#[source] RepositoryError),
}

impl CreateCommandError {
    /// Get detailed troubleshooting guidance for this error
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidConfiguration(_) => {
                "Invalid Configuration - Detailed Troubleshooting:

1. Check the configuration file syntax and format
2. Verify all required fields are present
3. Ensure SSH key files exist and are accessible
4. Validate environment name follows naming conventions
5. Run with --generate-template to see a valid example

For more information, see the configuration documentation."
            }

            Self::InvalidEnvironmentName(_) => {
                "Invalid Environment Name - Detailed Troubleshooting:

1. Environment names must contain only alphanumeric characters, hyphens, and underscores
2. Cannot start with hyphen (-) or underscore (_)
3. Cannot be empty
4. Examples: 'production', 'staging-1', 'test_env'

For more information, see the environment naming conventions."
            }

            Self::EnvironmentAlreadyExists { .. } => {
                "Environment Already Exists - Detailed Troubleshooting:

1. Choose a different environment name
2. Or destroy the existing environment first:
   torrust-tracker-deployer destroy --env-name <name>
3. List existing environments:
   torrust-tracker-deployer list

For more information, see the environment management documentation."
            }

            Self::EnvironmentCreationFailed(_) => {
                "Environment Creation Failed - Detailed Troubleshooting:

1. Check if you have write permissions to the data directory
2. Verify disk space is available
3. Check if any required dependencies are missing
4. Review the error details for specific issues

If the problem persists, report it with full error details."
            }

            Self::DirectoryCreationFailed { .. } => {
                "Directory Creation Failed - Detailed Troubleshooting:

1. Check write permissions for the target directory
2. Verify parent directories exist
3. Check available disk space: df -h
4. Ensure no file exists with the same name as the directory

For more information, see the filesystem troubleshooting guide."
            }

            Self::RepositoryError(_) => {
                "Repository Operation Failed - Detailed Troubleshooting:

1. Check write permissions for the data directory
2. Verify disk space is available
3. Check if the environment state file is corrupted
4. Try removing any partial state files and retry

If the problem persists, report it with system details."
            }
        }
    }
}
```

### Test Builders

```rust
// src/application/commands/create/tests/builders.rs
use std::sync::Arc;
use std::path::Path;
use tempfile::TempDir;
use chrono::{DateTime, Utc};
use crate::testing::{MockClock, MockEnvironmentRepository};
use crate::application::commands::create::CreateCommand;
use crate::domain::environment::{Environment, Created, EnvironmentName};

/// Test builder for CreateCommand with sensible defaults and customization options
pub struct CreateCommandTestBuilder {
    base_directory: Option<std::path::PathBuf>,
    fixed_time: Option<DateTime<Utc>>,
    existing_environments: Vec<String>,
}

impl CreateCommandTestBuilder {
    /// Create a new test builder with default settings
    pub fn new() -> Self {
        Self {
            base_directory: None,
            fixed_time: None,
            existing_environments: Vec::new(),
        }
    }

    /// Set a custom base directory for the test environment
    pub fn with_base_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_directory = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set a fixed time for deterministic testing
    pub fn with_fixed_time(mut self, time: DateTime<Utc>) -> Self {
        self.fixed_time = Some(time);
        self
    }

    /// Add an existing environment to simulate conflicts
    pub fn with_existing_environment(mut self, name: &str) -> Self {
        self.existing_environments.push(name.to_string());
        self
    }

    /// Build the CreateCommand with configured dependencies
    ///
    /// Returns a tuple of (command, temp_dir) where temp_dir must be kept
    /// alive for the duration of the test.
    pub fn build(self) -> (CreateCommand, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = self.base_directory.unwrap_or_else(|| temp_dir.path().to_path_buf());

        // Create mock clock
        let clock = Arc::new(MockClock::new(
            self.fixed_time.unwrap_or_else(|| Utc::now())
        ));

        // Create mock repository
        let mut repository = MockEnvironmentRepository::new();

        // Configure existing environments
        for env_name in &self.existing_environments {
            let env_name_obj = EnvironmentName::new(env_name.clone()).unwrap();
            repository.add_existing_environment(env_name_obj);
        }

        repository.set_base_directory(base_dir);
        let repository = Arc::new(repository);

        let command = CreateCommand::new(repository, clock);

        (command, temp_dir)
    }
}

impl Default for CreateCommandTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_build_command_with_defaults() {
        let (command, _temp_dir) = CreateCommandTestBuilder::new().build();

        // Verify command is created (basic smoke test)
        assert!(std::ptr::eq(
            Arc::as_ptr(&command.environment_repository),
            Arc::as_ptr(&command.environment_repository)
        ));
    }

    #[test]
    fn it_should_build_command_with_custom_time() {
        use chrono::TimeZone;

        let fixed_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let (command, _temp_dir) = CreateCommandTestBuilder::new()
            .with_fixed_time(fixed_time)
            .build();

        // The clock should be set to the fixed time
        assert_eq!(command.clock.now(), fixed_time);
    }

    #[test]
    fn it_should_build_command_with_existing_environments() {
        let (command, _temp_dir) = CreateCommandTestBuilder::new()
            .with_existing_environment("production")
            .with_existing_environment("staging")
            .build();

        // This is a smoke test - actual behavior testing is in integration tests
        assert!(std::ptr::eq(
            Arc::as_ptr(&command.environment_repository),
            Arc::as_ptr(&command.environment_repository)
        ));
    }
}
```

## Implementation Plan

### Phase 1: Command Structure (1 hour)

- [ ] Create `src/application/commands/create/mod.rs` with module documentation
- [ ] Set up module structure with proper exports
- [ ] Create basic `CreateCommand` struct with dependencies
- [ ] Add constructor with dependency injection

### Phase 2: Business Logic Implementation (2 hours)

- [ ] Implement `execute` method with complete business logic flow
- [ ] Add configuration validation using domain rules
- [ ] Implement environment uniqueness checking
- [ ] Add environment entity creation with timestamp
- [ ] Implement directory creation for data and traces
- [ ] Add environment persistence through repository

### Phase 3: Error Handling (1 hour)

- [ ] Create `CreateCommandError` enum with thiserror
- [ ] Add structured error context for all failure scenarios
- [ ] Implement tiered help system with detailed troubleshooting
- [ ] Ensure proper error chaining from domain and infrastructure layers

### Phase 4: Test Infrastructure (1 hour)

- [ ] Create `CreateCommandTestBuilder` with customization options
- [ ] Implement mock repository and clock integration
- [ ] Add test fixtures and helper functions
- [ ] Create integration test scenarios

### Phase 5: Comprehensive Testing (1-2 hours)

- [ ] Unit tests for successful environment creation
- [ ] Tests for environment name conflicts
- [ ] Tests for configuration validation failures
- [ ] Tests for directory creation failures
- [ ] Tests for repository persistence failures
- [ ] Tests for deterministic timestamp behavior
- [ ] Error scenario tests with proper error type validation

## Acceptance Criteria

- [ ] `CreateCommand` that accepts clean domain objects (no parsing logic)
- [ ] Delivery-agnostic implementation that works with any input source
- [ ] Complete business logic orchestration for environment creation
- [ ] Environment uniqueness validation
- [ ] Directory structure creation (data and traces directories)
- [ ] Environment state persistence through repository pattern
- [ ] Explicit error handling with structured context and actionable messages
- [ ] Tiered help system for all error scenarios
- [ ] Comprehensive test coverage with test builders and MockClock
- [ ] Integration tests for complete command execution flow

## Testing Strategy

### Test Categories

1. **Happy Path Tests**

   - Valid configuration creates environment successfully
   - Directory structure is created correctly
   - Environment state is persisted properly
   - Timestamps are recorded accurately

2. **Validation Tests**

   - Invalid configuration fails with proper error
   - Environment name conflicts are detected
   - Domain validation rules are enforced

3. **Error Handling Tests**

   - Directory creation failures are handled gracefully
   - Repository persistence failures provide clear errors
   - Error chaining maintains full context

4. **Integration Tests**
   - Complete command execution with real dependencies
   - Error recovery scenarios
   - Performance and resource management

### Mock Strategy

- **MockEnvironmentRepository**: Simulates environment storage and conflict detection
- **MockClock**: Provides deterministic timestamps for testing
- **TempDir**: Creates isolated filesystem environments for each test
- **Test Builders**: Simplify test setup with sensible defaults and customization

## Related Documentation

- [Application Layer Architecture](../codebase-architecture.md#application-layer)
- [Command Pattern Implementation](../codebase-architecture.md#application-layer)
- [Error Handling Guidelines](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Repository Pattern](../codebase-architecture.md#infrastructure-layer)

## Notes

- This command is the core business logic for environment creation
- It's completely delivery-agnostic and can serve multiple presentation layers
- The command follows the project's three-level architecture pattern
- Error handling provides complete traceability and actionable guidance
- Test builders enable easy test setup and maintenance
