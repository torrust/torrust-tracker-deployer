# Refactor Test Command Handler to Folder Module

**Issue**: #184
**Parent Epic**: N/A (Technical Refactoring Task)
**Related**:

- `src/application/command_handlers/test.rs` (current implementation)
- `src/application/command_handlers/provision/` (reference structure)
- `src/application/command_handlers/configure/` (reference structure)

## Overview

Refactor the `TestCommandHandler` from a single-file module (`src/application/command_handlers/test.rs`) into a folder-based module structure (`src/application/command_handlers/test/`) to:

1. Improve code organization and maintainability
2. Align with the architectural pattern used by `provision` and `configure` command handlers
3. Update the `execute` method signature to accept `&EnvironmentName` instead of `&Environment<S>` for consistency
4. Separate concerns (handler logic, errors, tests) into dedicated files

This refactoring enhances consistency across all command handlers in the application layer.

## Goals

- [ ] Convert single-file module to folder-based structure
- [ ] Separate handler logic, errors, and tests into dedicated files
- [ ] Update `execute` method to accept `&EnvironmentName` parameter
- [ ] Maintain all existing functionality and test coverage
- [ ] Ensure consistent structure with `provision` and `configure` handlers

## 🏗️ Architecture Requirements

**DDD Layer**: Application
**Module Path**: `src/application/command_handlers/test/`
**Pattern**: Command Handler (Three-Level Pattern: Command → Steps → Actions)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))
- [ ] Match structure of existing command handlers (`provision`, `configure`)

### Architectural Constraints

- [ ] Handler logic remains in application layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities (unit tests for handler, integration tests for workflow)
- [ ] Repository pattern for environment state management
- [ ] Type-state pattern compatibility (accept any environment state, verify at runtime)

### Anti-Patterns to Avoid

- ❌ Mixing test logic with handler implementation
- ❌ Duplicating error types across modules
- ❌ Breaking existing test coverage during refactoring
- ❌ Inconsistent patterns with other command handlers

## Specifications

### Target Module Structure

```text
src/application/command_handlers/test/
├── errors.rs          # TestCommandHandlerError and related error types
├── handler.rs         # TestCommandHandler implementation
├── mod.rs             # Module exports and documentation
└── tests/             # Test module
    ├── builders.rs    # Test helper builders (if needed)
    ├── integration.rs # Integration tests for the handler
    └── mod.rs         # Test module organization
```

### Current Implementation (test.rs)

The current implementation is a single file with:

- `TestCommandHandler` struct (zero-sized type)
- `TestCommandHandlerError` enum with error variants
- `execute` method accepting `&Environment<S>` (generic over state)
- Unit tests in `#[cfg(test)] mod tests`

### Target Implementation

**File: `src/application/command_handlers/test/errors.rs`**

```rust
//! Error types for test command handler

use crate::domain::environment::state::StateTransitionError;
use crate::infrastructure::remote_actions::RemoteActionError;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `TestCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum TestCommandHandlerError {
    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Remote action failed: {0}")]
    RemoteAction(#[from] RemoteActionError),

    #[error("Environment '{environment_name}' does not have an instance IP set. The environment must be provisioned before running tests.")]
    MissingInstanceIp { environment_name: String },

    #[error("State transition error: {0}")]
    StateTransition(#[from] StateTransitionError),
}
```

**File: `src/application/command_handlers/test/handler.rs`**

```rust
//! Test command handler implementation

use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::TestCommandHandlerError;
use crate::adapters::ssh::SshConfig;
use crate::application::steps::{
    ValidateCloudInitCompletionStep, ValidateDockerComposeInstallationStep,
    ValidateDockerInstallationStep,
};
use crate::domain::environment::repository::{EnvironmentRepository, RepositoryError, TypedEnvironmentRepository};
use crate::domain::EnvironmentName;

/// `TestCommandHandler` orchestrates the complete infrastructure testing and validation workflow
///
/// The `TestCommandHandler` validates that an environment is properly set up with all required
/// infrastructure components.
///
/// ## Validation Steps
///
/// 1. Validate cloud-init completion
/// 2. Validate Docker installation
/// 3. Validate Docker Compose installation
pub struct TestCommandHandler {
    repository: TypedEnvironmentRepository,
}

impl TestCommandHandler {
    /// Create a new `TestCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self {
            repository: TypedEnvironmentRepository::new(repository),
        }
    }

    /// Execute the complete testing and validation workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to test
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found
    /// * Environment does not have an instance IP set
    /// * Any validation step fails:
    ///   - Cloud-init completion validation fails
    ///   - Docker installation validation fails
    ///   - Docker Compose installation validation fails
    #[instrument(
        name = "test_command",
        skip_all,
        fields(
            command_type = "test",
            environment = %env_name
        )
    )]
    pub async fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), TestCommandHandlerError> {
        info!(
            command = "test",
            environment = %env_name,
            "Starting complete infrastructure testing workflow"
        );

        // 1. Load the environment from storage
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(|_| TestCommandHandlerError::StateTransition(
                crate::domain::environment::state::StateTransitionError::NotFound
            ))?;

        // 2. Check if environment exists
        let any_env = any_env.ok_or_else(|| {
            TestCommandHandlerError::StateTransition(
                crate::domain::environment::state::StateTransitionError::NotFound
            )
        })?;

        // 3. Extract instance IP (runtime check - works with any state)
        let instance_ip = any_env.instance_ip().ok_or_else(|| {
            TestCommandHandlerError::MissingInstanceIp {
                environment_name: env_name.to_string(),
            }
        })?;

        let ssh_config = SshConfig::with_default_port(
            any_env.ssh_credentials().clone(),
            instance_ip
        );

        ValidateCloudInitCompletionStep::new(ssh_config.clone())
            .execute()
            .await?;

        ValidateDockerInstallationStep::new(ssh_config.clone())
            .execute()
            .await?;

        ValidateDockerComposeInstallationStep::new(ssh_config)
            .execute()
            .await?;

        info!(
            command = "test",
            environment = %env_name,
            instance_ip = ?instance_ip,
            "Infrastructure testing workflow completed successfully"
        );

        Ok(())
    }
}
```

**File: `src/application/command_handlers/test/mod.rs`**

```rust
//! Test Command Module
//!
//! This module implements the delivery-agnostic `TestCommandHandler`
//! for orchestrating infrastructure validation business logic.
//!
//! ## Architecture
//!
//! The `TestCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Loads environment state via `EnvironmentRepository`
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Asynchronous**: Uses async/await for network operations
//! - **Runtime State Validation**: Accepts any environment state, validates at runtime
//! - **Explicit Errors**: All errors implement `.help()` with actionable guidance
//!
//! ## Validation Workflow
//!
//! The command handler orchestrates a multi-step validation workflow:
//!
//! 1. **Validate cloud-init completion** - Ensure system initialization is complete
//! 2. **Validate Docker installation** - Verify Docker is installed and running
//! 3. **Validate Docker Compose installation** - Verify Docker Compose is available
//!
//! ## State Management
//!
//! Unlike `provision` and `configure` handlers, the test handler does not transition
//! environment state. It accepts an environment name, loads the environment from storage,
//! and performs runtime validation checks regardless of the environment's compile-time state.

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use errors::TestCommandHandlerError;
pub use handler::TestCommandHandler;
```

**File: `src/application/command_handlers/test/tests/mod.rs`**

```rust
//! Tests for test command handler

mod integration;

#[cfg(test)]
pub(crate) mod builders {
    // Test builders if needed in the future
}
```

**File: `src/application/command_handlers/test/tests/integration.rs`**

```rust
//! Integration tests for TestCommandHandler

use super::super::*;

#[test]
fn it_should_create_test_command() {
    // Basic structure test - will need repository mock
}

#[test]
fn it_should_have_correct_error_type_conversions() {
    use crate::infrastructure::remote_actions::RemoteActionError;
    use crate::shared::command::CommandError;

    // Test that all error types can convert to TestCommandHandlerError
    let command_error = CommandError::StartupFailed {
        command: "test".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
    };
    let test_error: TestCommandHandlerError = command_error.into();
    drop(test_error);

    let remote_action_error = RemoteActionError::ValidationFailed {
        action_name: "test".to_string(),
        message: "test error".to_string(),
    };
    let test_error: TestCommandHandlerError = remote_action_error.into();
    drop(test_error);
}
```

### Signature Change Rationale

**Current Signature:**

```rust
pub async fn execute<S>(
    &self,
    environment: &Environment<S>,
) -> Result<(), TestCommandHandlerError>
```

**New Signature:**

```rust
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
) -> Result<(), TestCommandHandlerError>
```

**Why this change?**

1. **Consistency**: Aligns with `ProvisionCommandHandler` and `ConfigureCommandHandler` patterns
2. **Repository Integration**: Loads environment from storage like other handlers
3. **Flexibility**: Allows testing environments regardless of compile-time state
4. **State Agnostic**: Runtime validation that IP is set, rather than compile-time state checking

## Implementation Plan

### Phase 1: Create Folder Structure (15 minutes)

- [ ] Create `src/application/command_handlers/test/` directory
- [ ] Create `src/application/command_handlers/test/tests/` directory
- [ ] Create empty files: `errors.rs`, `handler.rs`, `mod.rs`
- [ ] Create empty test files: `tests/mod.rs`, `tests/integration.rs`

### Phase 2: Move Error Types (10 minutes)

- [ ] Copy `TestCommandHandlerError` from `test.rs` to `errors.rs`
- [ ] Add necessary imports and module documentation
- [ ] Add `StateTransitionError` variant for repository errors
- [ ] Verify error conversions still work

### Phase 3: Refactor Handler Implementation (30 minutes)

- [ ] Copy `TestCommandHandler` struct to `handler.rs`
- [ ] Add `repository` field of type `TypedEnvironmentRepository`
- [ ] Update constructor to accept `Arc<dyn EnvironmentRepository>`
- [ ] Refactor `execute` method to accept `&EnvironmentName`
- [ ] Add repository loading logic (similar to `provision` handler)
- [ ] Add runtime validation for instance IP
- [ ] Update logging statements
- [ ] Add module documentation

### Phase 4: Setup Module Exports (10 minutes)

- [ ] Create `mod.rs` with module documentation
- [ ] Add public exports for `errors`, `handler`
- [ ] Add `#[cfg(test)] mod tests;`
- [ ] Re-export main types (`TestCommandHandler`, `TestCommandHandlerError`)

### Phase 5: Migrate Tests (20 minutes)

- [ ] Move existing unit tests to `tests/integration.rs`
- [ ] Update test imports for new module structure
- [ ] Add test module organization in `tests/mod.rs`
- [ ] Verify all tests pass with new structure
- [ ] Add builders module stub (for future test helpers)

### Phase 6: Update Module References (15 minutes)

- [ ] Update `src/application/command_handlers/mod.rs` to reference new module
- [ ] Update presentation layer imports (CLI commands)
- [ ] Update any E2E test imports
- [ ] Remove old `test.rs` file

### Phase 7: Verification (15 minutes)

- [ ] Run `cargo build` to verify compilation
- [ ] Run `cargo test` to verify all tests pass
- [ ] Run `./scripts/pre-commit.sh` to verify all checks pass
- [ ] Check that E2E tests still work correctly
- [ ] Verify error messages and help text are correct

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Structure Criteria**:

- [ ] `src/application/command_handlers/test/` directory exists
- [ ] Contains `errors.rs`, `handler.rs`, `mod.rs`
- [ ] Contains `tests/` subdirectory with `mod.rs`, `integration.rs`
- [ ] Old `src/application/command_handlers/test.rs` file is removed

**Implementation Criteria**:

- [ ] `TestCommandHandler::execute` accepts `&EnvironmentName` parameter
- [ ] Handler includes `repository: TypedEnvironmentRepository` field
- [ ] Handler loads environment from storage using repository
- [ ] Runtime validation for instance IP presence
- [ ] Error types separated into dedicated `errors.rs`
- [ ] All existing tests migrated and passing

**Consistency Criteria**:

- [ ] Module structure matches `provision` and `configure` handlers
- [ ] Handler pattern matches other command handlers
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Module organization follows conventions (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))

**Documentation Criteria**:

- [ ] Module-level documentation in `mod.rs` explains purpose and architecture
- [ ] Handler documentation explains validation workflow
- [ ] Error types have clear documentation
- [ ] Code comments explain design decisions (especially signature change)

**Functionality Criteria**:

- [ ] All existing functionality preserved
- [ ] E2E tests pass without modification
- [ ] CLI commands work correctly with refactored handler
- [ ] Error messages remain actionable and helpful

## Related Documentation

- [docs/codebase-architecture.md](../codebase-architecture.md) - DDD layers and patterns
- [docs/contributing/module-organization.md](../contributing/module-organization.md) - Module structure conventions
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling patterns
- [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md) - Layer placement guidelines
- `src/application/command_handlers/provision/` - Reference implementation for structure
- `src/application/command_handlers/configure/` - Reference implementation for structure

## Notes

### Design Decision: Why Change the Signature?

The current `execute` method accepts `&Environment<S>` (generic over state) which provides compile-time flexibility but creates inconsistency with other command handlers. The new signature accepting `&EnvironmentName`:

1. **Aligns with established patterns**: Both `provision` and `configure` handlers load environments from storage
2. **Maintains flexibility**: Runtime check for instance IP provides same validation guarantees
3. **Improves consistency**: Single pattern across all command handlers simplifies understanding
4. **Enables state persistence**: Repository integration allows tracking of test executions (future enhancement)

### Test Command Special Case

Unlike `provision` and `configure`, the test command does not modify environment state. It's a read-only validation operation. However, using the same signature pattern:

- Improves code consistency
- Simplifies dependency injection
- Enables future enhancements (e.g., tracking test history)
- Makes the codebase more predictable

### Migration Strategy

This refactoring should be done in a single PR to avoid having both old and new structures coexisting. The implementation plan is ordered to minimize breaking changes:

1. Create new structure
2. Migrate code piece by piece
3. Update references
4. Remove old file

This ensures the codebase remains in a working state throughout the refactoring process.
