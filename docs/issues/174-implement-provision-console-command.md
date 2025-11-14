# Implement Provision Console Command

**Issue**: #174
**Parent Epic**: #2 - Scaffolding for main app
**Related**:

- Roadmap task 1.6: [docs/roadmap.md](../roadmap.md)
- Application layer: `ProvisionCommandHandler` (already implemented)
- DDD architecture: [docs/codebase-architecture.md](../codebase-architecture.md)
- Module organization: [docs/contributing/module-organization.md](../contributing/module-organization.md)
- Error handling: [docs/contributing/error-handling.md](../contributing/error-handling.md)

## Overview

Implement the presentation layer for the `provision` console command to enable users to provision VM infrastructure from the CLI. The application layer `ProvisionCommandHandler` is already implemented - this task focuses on creating the console interface (CLI command, controller, router integration, and user interaction).

## Goals

- [ ] Add `provision` subcommand to CLI command definitions
- [ ] Create presentation layer controller following the destroy controller pattern
- [ ] Integrate controller into the dispatch router
- [ ] Provide clear user feedback during provisioning workflow
- [ ] Handle errors with actionable messages

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/`
**Pattern**: CLI Subcommand + Controller (single command pattern)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Use `ExecutionContext` pattern for dependency injection
- [ ] Controller follows destroy controller reference pattern
- [ ] Error handling uses explicit enum errors (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Module organization follows project conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] **No business logic in presentation layer** - delegate to `ProvisionCommandHandler`
- [ ] **Single command pattern** - No subcommands, direct execution like destroy
- [ ] **Dependencies flow toward application layer** - Controller depends on `ProvisionCommandHandler`
- [ ] **User output through `UserOutput` trait** - Consistent output formatting
- [ ] **Error messages are actionable** - Include help methods with troubleshooting steps

### Anti-Patterns to Avoid

- ❌ Implementing provisioning logic in controller (already in `ProvisionCommandHandler`)
- ❌ Direct infrastructure access from presentation layer
- ❌ Using `anyhow` for command-specific errors
- ❌ Mixing routing and execution logic

## Specifications

### CLI Command Definition

Add the `Provision` command to `src/presentation/input/cli/commands.rs`:

```rust
/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...
    
    /// Provision a new deployment environment infrastructure
    ///
    /// This command provisions the virtual machine infrastructure for a deployment
    /// environment that was previously created. It will:
    /// - Render and apply OpenTofu templates
    /// - Create LXD VM instances
    /// - Configure networking
    /// - Wait for SSH connectivity
    /// - Wait for cloud-init completion
    ///
    /// The environment must be in "Created" state (use 'create environment' first).
    Provision {
        /// Name of the environment to provision
        ///
        /// The environment name must match an existing environment that was
        /// previously created and is in "Created" state.
        environment: String,
    },
}
```

### Controller Structure

Create `src/presentation/controllers/provision/` following the destroy controller pattern:

```text
src/presentation/controllers/provision/
├── handler.rs      # Main command handler function
├── errors.rs       # ProvisionSubcommandError with help methods
├── tests/          # Command-specific tests
│   └── mod.rs      # Test module organization
└── mod.rs          # Module exports
```

### Controller Handler API

**File**: `src/presentation/controllers/provision/handler.rs`

```rust
//! Provision Command Handler
//!
//! This module handles the provision command execution at the presentation layer,
//! including environment validation, repository initialization, and user interaction.

use std::sync::Arc;
use std::path::Path;

use crate::application::command_handlers::ProvisionCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::Provisioned;
use crate::domain::environment::Environment;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::ProvisionSubcommandError;

/// Number of main steps in the provision workflow
const PROVISION_WORKFLOW_STEPS: usize = 9;

/// Handle provision command using `ExecutionContext` pattern
///
/// # Arguments
///
/// * `environment_name` - Name of the environment to provision
/// * `working_dir` - Working directory path for operations
/// * `context` - Execution context providing access to services
///
/// # Returns
///
/// * `Ok(Environment<Provisioned>)` - Environment provisioned successfully
/// * `Err(ProvisionSubcommandError)` - Provision operation failed
///
/// # Errors
///
/// Returns `ProvisionSubcommandError` when:
/// * Environment name is invalid or contains special characters
/// * Working directory is not accessible or doesn't exist
/// * Environment is not found or not in "Created" state
/// * Infrastructure provisioning fails (OpenTofu/LXD errors)
/// * SSH connectivity cannot be established
/// * Cloud-init does not complete successfully
#[allow(clippy::result_large_err)]
pub fn handle(
    environment_name: &str,
    working_dir: &Path,
    context: &crate::presentation::dispatch::context::ExecutionContext,
) -> Result<Environment<Provisioned>, ProvisionSubcommandError> {
    // Implementation follows destroy controller pattern
    // 1. Validate environment name
    // 2. Initialize repository
    // 3. Load environment (must be in Created state)
    // 4. Create progress reporter
    // 5. Execute ProvisionCommandHandler
    // 6. Report success
}
```

### Error Type

**File**: `src/presentation/controllers/provision/errors.rs`

```rust
//! Provision Subcommand Error Types
//!
//! This module defines errors specific to the provision console subcommand.

use std::fmt;
use crate::application::command_handlers::provision::errors::ProvisionCommandHandlerError;
use crate::domain::environment::name::InvalidEnvironmentName;
use crate::domain::environment::repository::RepositoryError;

/// Error type for provision subcommand execution
#[derive(Debug)]
pub enum ProvisionSubcommandError {
    /// Environment name validation failed
    InvalidName(InvalidEnvironmentName),
    
    /// Repository initialization or access failed
    Repository(RepositoryError),
    
    /// Environment not found or invalid state
    EnvironmentNotFound(String),
    
    /// Environment is not in Created state
    InvalidState {
        environment: String,
        current_state: String,
        required_state: String,
    },
    
    /// Provision command handler execution failed
    Handler(ProvisionCommandHandlerError),
    
    /// Progress reporter encountered poisoned mutex
    ProgressReporterPoisoned,
}

impl ProvisionSubcommandError {
    /// Get actionable help message for troubleshooting
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::InvalidName(e) => format!(
                "Environment name is invalid: {}\n\
                 \n\
                 Valid names must:\n\
                 - Start with a letter or underscore\n\
                 - Contain only letters, numbers, hyphens, and underscores\n\
                 - Be between 1 and 63 characters\n\
                 \n\
                 Examples: 'prod-01', 'staging_env', 'test-environment'",
                e
            ),
            Self::Repository(e) => format!(
                "Failed to access environment repository: {}\n\
                 \n\
                 Troubleshooting:\n\
                 1. Verify the working directory exists and is writable\n\
                 2. Check file system permissions\n\
                 3. Ensure sufficient disk space\n\
                 4. Check for file locks by other processes",
                e
            ),
            Self::EnvironmentNotFound(name) => format!(
                "Environment '{}' not found\n\
                 \n\
                 The environment must be created before provisioning.\n\
                 \n\
                 Steps:\n\
                 1. Create environment configuration: torrust-tracker-deployer create template\n\
                 2. Edit the generated template with your settings\n\
                 3. Create the environment: torrust-tracker-deployer create environment -f <config-file>\n\
                 4. Then provision: torrust-tracker-deployer provision {}",
                name, name
            ),
            Self::InvalidState { environment, current_state, required_state } => format!(
                "Environment '{}' is in '{}' state, but '{}' state is required\n\
                 \n\
                 The provision command requires an environment in 'Created' state.\n\
                 \n\
                 Current state: {}\n\
                 \n\
                 Troubleshooting:\n\
                 - If already provisioned: Environment is ready to use\n\
                 - If provisioning failed: Review error logs and retry after fixing issues\n\
                 - If in wrong state: Create a new environment or use appropriate command",
                environment, current_state, required_state, current_state
            ),
            Self::Handler(e) => format!(
                "Provision command failed: {}\n\
                 \n\
                 See error details above for specific failure information.\n\
                 Check logs for detailed trace information.",
                e
            ),
            Self::ProgressReporterPoisoned => {
                "Progress reporter encountered poisoned mutex\n\
                 \n\
                 This is an internal error. Please report this issue with:\n\
                 - Command that was running\n\
                 - Log file contents\n\
                 - Steps to reproduce"
                    .to_string()
            }
        }
    }
}

// Implement Display, Error, and From traits following destroy pattern
```

### Router Integration

**File**: `src/presentation/dispatch/router.rs`

Add the new command to the router's match statement:

```rust
pub fn route_command(
    command: Commands,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::route_command(action, working_dir, context)?;
            Ok(())
        }
        Commands::Destroy { environment } => {
            destroy::handle(&environment, working_dir, context)?;
            Ok(())
        }
        Commands::Provision { environment } => {
            provision::handle(&environment, working_dir, context)?;
            Ok(())
        }
    }
}
```

### User Feedback

The controller should provide clear progress updates during the provisioning workflow:

```text
Starting provision for environment 'my-env'...

[Step 1/9] Rendering OpenTofu templates...
[Step 2/9] Initializing OpenTofu...
[Step 3/9] Validating infrastructure configuration...
[Step 4/9] Planning infrastructure...
[Step 5/9] Applying infrastructure...
[Step 6/9] Getting instance information...
[Step 7/9] Rendering Ansible templates...
[Step 8/9] Waiting for SSH connectivity...
[Step 9/9] Waiting for cloud-init completion...

✓ Environment 'my-env' provisioned successfully
  Instance IP: 10.x.x.x
  State: Provisioned
```

## Implementation Plan

### Phase 1: CLI Command Definition (30 minutes)

- [ ] Add `Provision` variant to `Commands` enum in `src/presentation/input/cli/commands.rs`
- [ ] Add comprehensive documentation comments
- [ ] Test CLI parsing with `cargo build`

### Phase 2: Controller Structure (1 hour)

- [ ] Create `src/presentation/controllers/provision/` directory
- [ ] Create `mod.rs` with module exports
- [ ] Create `errors.rs` with `ProvisionSubcommandError` enum
- [ ] Implement `help()` method for all error variants
- [ ] Implement `Display` and `Error` traits
- [ ] Implement `From` conversions for error types

### Phase 3: Controller Handler (2 hours)

- [ ] Create `handler.rs` with main `handle()` function
- [ ] Implement environment name validation
- [ ] Implement repository initialization
- [ ] Load environment from repository (validate Created state)
- [ ] Create progress reporter with 9 steps
- [ ] Call `ProvisionCommandHandler::execute()`
- [ ] Handle success and error cases
- [ ] Add comprehensive documentation

### Phase 4: Router Integration (30 minutes)

- [ ] Add `provision` module to `src/presentation/controllers/mod.rs`
- [ ] Update `src/presentation/dispatch/router.rs` with provision route
- [ ] Update `CommandError` if needed for provision errors
- [ ] Test routing with minimal integration test

### Phase 5: Testing (1.5 hours)

- [ ] Create `src/presentation/controllers/provision/tests/mod.rs`
- [ ] Add unit tests for error help messages
- [ ] Add unit tests for handler with mock services
- [ ] Add integration test for successful provision
- [ ] Add integration test for error cases (invalid state, not found)
- [ ] Test CLI parsing and routing

### Phase 6: Documentation and Review (30 minutes)

- [ ] Update module documentation
- [ ] Verify all error messages are actionable
- [ ] Check code follows module organization conventions
- [ ] Verify import style (short names, no long paths)
- [ ] Run linters and fix any issues

### Phase 7: Manual End-to-End Testing (45 minutes)

Perform complete workflow testing to verify the provision command integrates correctly with create and destroy commands:

- [ ] **Step 1**: Create temporary test directory
  - [ ] Create a clean temporary directory for testing
  - [ ] Navigate to the test directory

- [ ] **Step 2**: Run create command
  - [ ] Generate template: `torrust-tracker-deployer create template`
  - [ ] Edit template with valid test configuration
  - [ ] Create environment: `torrust-tracker-deployer create environment -f environment-template.json`
  - [ ] Verify command completes successfully
  - [ ] Verify environment state file created in `data/` directory
  - [ ] Verify state shows `Created` status
  - [ ] Verify all required fields are present in state file

- [ ] **Step 3**: Run provision command (NEW - being tested)
  - [ ] Provision infrastructure: `torrust-tracker-deployer provision <environment-name>`
  - [ ] Verify all 9 progress steps are displayed
  - [ ] Verify command completes successfully
  - [ ] Verify success message includes instance IP address
  - [ ] Verify environment state file updated in `data/` directory
  - [ ] Verify state shows `Provisioned` status
  - [ ] Verify instance IP is saved in state file
  - [ ] Verify `build/` directory contains generated templates

- [ ] **Step 4**: Run destroy command
  - [ ] Destroy environment: `torrust-tracker-deployer destroy <environment-name>`
  - [ ] Verify command completes successfully
  - [ ] Verify environment state file updated to `Destroyed` status
  - [ ] Verify infrastructure is actually torn down (LXD instance removed)

- [ ] **Step 5**: Validate state consistency
  - [ ] Verify state transitions are correct: Created → Provisioned → Destroyed
  - [ ] Verify no orphaned files in `data/` or `build/` directories
  - [ ] Verify log files contain complete trace information

- [ ] **Step 6**: Test error scenarios
  - [ ] Try provisioning non-existent environment (should fail with clear error)
  - [ ] Try provisioning already provisioned environment (should fail with state error)
  - [ ] Verify all error messages are actionable and helpful

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**CLI Integration**:

- [ ] `torrust-tracker-deployer provision <environment>` command is available
- [ ] `torrust-tracker-deployer provision --help` shows clear documentation
- [ ] CLI parsing correctly extracts environment name parameter

**Controller Implementation**:

- [ ] Controller follows destroy controller reference pattern
- [ ] Handler function uses `ExecutionContext` pattern
- [ ] Error type provides actionable help messages for all variants
- [ ] Progress reporter shows all 9 provisioning steps
- [ ] Success message includes instance IP address

**Router Integration**:

- [ ] Provision command is registered in dispatch router
- [ ] Router correctly routes to provision controller
- [ ] Error handling flows correctly through layers

**Error Handling**:

- [ ] Invalid environment name shows validation rules
- [ ] Environment not found suggests create workflow
- [ ] Invalid state error explains current state and required state
- [ ] All error messages follow actionability principles
- [ ] Errors include context for traceability

**Testing**:

- [ ] Unit tests cover error help messages
- [ ] Unit tests cover handler logic with mocks
- [ ] Integration tests verify successful provision workflow
- [ ] Integration tests verify error cases
- [ ] All tests pass locally

**Code Quality**:

- [ ] Follows DDD layer placement guidelines
- [ ] Respects dependency flow rules
- [ ] Uses appropriate module organization
- [ ] Import style uses short names
- [ ] Documentation is comprehensive
- [ ] No clippy warnings
- [ ] Code is formatted with rustfmt

**User Experience**:

- [ ] Progress updates are clear and informative
- [ ] Success message includes all relevant information
- [ ] Error messages guide users to solutions
- [ ] Command integrates seamlessly with existing CLI

## Related Documentation

- [Roadmap](../roadmap.md) - Task 1.6
- [Codebase Architecture](../codebase-architecture.md) - DDD layers and patterns
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Error Handling](../contributing/error-handling.md) - Error handling principles
- [Destroy Controller](../../src/presentation/controllers/destroy/) - Reference implementation
- [Application Layer Handler](../../src/application/command_handlers/provision/handler.rs) - Business logic

## Notes

### Why This Task is Important

The `ProvisionCommandHandler` in the application layer contains all the business logic for provisioning infrastructure, but users cannot access it from the CLI. This task creates the user interface layer that makes the functionality available and user-friendly.

### Design Decisions

1. **Single Command Pattern**: Following destroy controller pattern since provision is a single command without subcommands

2. **State Validation**: Must validate environment is in "Created" state before provisioning

3. **Progress Reporting**: Provision has 9 steps (more than destroy's 3), requires clear progress feedback

4. **Error Messages**: Focus on guiding users through the create → provision → configure workflow

5. **No Business Logic**: All provisioning logic stays in `ProvisionCommandHandler` - controller only handles presentation concerns

### Estimated Time

Total: ~6 hours for complete implementation, testing, and documentation
