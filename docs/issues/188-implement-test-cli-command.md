# Implement CLI Command: `torrust-tracker-deployer test`

**Issue**: [#188](https://github.com/torrust/torrust-tracker-deployer/issues/188)
**Parent Epic**: [#2 - Scaffolding for main app](https://github.com/torrust/torrust-tracker-deployer/issues/2)
**Related**:

- [Application Layer Test Command Handler](https://github.com/torrust/torrust-tracker-deployer/blob/main/src/application/command_handlers/test/handler.rs)
- [Provision CLI Command Implementation](https://github.com/torrust/torrust-tracker-deployer/blob/main/src/presentation/controllers/provision/)
- [ADR: Test Command as Smoke Test](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/decisions/test-command-as-smoke-test.md)

## Overview

Implement the presentation layer (CLI interface) for the `test` command, enabling users to verify deployment infrastructure through the command-line interface. The application layer `TestCommandHandler` is already implemented and performs validation checks for cloud-init completion, Docker installation, and Docker Compose installation.

This task focuses exclusively on creating the user-facing CLI subcommand that calls the existing business logic, following the same architectural pattern used in the `provision` command controller.

## Goals

- [ ] Create presentation layer controller for `test` CLI subcommand
- [ ] Implement error handling with actionable help messages
- [ ] Integrate with existing `TestCommandHandler` business logic
- [ ] Follow DDD patterns consistent with other CLI commands (provision, configure, destroy)
- [ ] Provide clear user feedback during validation workflow
- [ ] Enable end-to-end deployment workflow verification via CLI

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/controllers/test/`
**Pattern**: CLI Subcommand Controller (orchestrates Application layer command handler)

### Module Structure Requirements

- [ ] Follow presentation layer conventions from `src/presentation/controllers/provision/`
- [ ] Separate concerns into submodules: `mod.rs`, `handler.rs`, `errors.rs`, `tests/`
- [ ] Follow DDD dependency rules: Presentation → Application → Domain
- [ ] Use `ExecutionContext` pattern for dependency injection
- [ ] Integrate with shared `UserOutput` system for consistent formatting

### Architectural Constraints

- [ ] **No business logic** in presentation layer - delegate all validation to `TestCommandHandler`
- [ ] Error types must implement `.help()` methods with actionable troubleshooting guidance
- [ ] Follow error handling conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Use `ProgressReporter` for user feedback during multi-step validation
- [ ] Support both `ExecutionContext` pattern and direct dependency injection (like provision)

### Anti-Patterns to Avoid

- ❌ Duplicating validation logic from `TestCommandHandler`
- ❌ Direct infrastructure access (SSH, Docker) - use application layer
- ❌ Generic error messages without context or actionable help
- ❌ Mixing CLI argument parsing with business logic

## Specifications

### Module Structure

Create the following structure matching the `provision` controller pattern:

```text
src/presentation/controllers/test/
├── mod.rs          # Module documentation and re-exports
├── handler.rs      # Main controller implementation
├── errors.rs       # Presentation-layer error types
└── tests/          # Unit tests for controller logic
    └── mod.rs
```

### Error Types (`errors.rs`)

Define presentation-layer errors with `.help()` methods:

```rust
#[derive(Debug, Error)]
pub enum TestSubcommandError {
    #[error("Invalid environment name '{name}': {source}")]
    InvalidEnvironmentName {
        name: String,
        #[source]
        source: EnvironmentNameError,
    },

    #[error("Environment '{name}' not found in data directory '{data_dir}'")]
    EnvironmentNotFound {
        name: String,
        data_dir: String,
    },

    #[error("Environment '{name}' does not have instance IP set")]
    MissingInstanceIp {
        name: String,
    },

    #[error("Validation failed for environment '{name}': {source}")]
    ValidationFailed {
        name: String,
        #[source]
        source: Box<TestCommandHandlerError>,
    },

    #[error("Failed to report progress: {source}")]
    ProgressReportingFailed {
        #[source]
        source: ProgressReporterError,
    },
}

impl TestSubcommandError {
    pub fn help(&self) -> String {
        match self {
            Self::InvalidEnvironmentName { .. } => {
                "Environment names must be 1-63 characters, start with letter/digit, \
                 contain only letters/digits/hyphens.\n\
                 Example: my-env-123"
            }
            Self::EnvironmentNotFound { data_dir, .. } => {
                format!(
                    "Check if environment exists:\n  \
                     ls -la {}/\n\n\
                     Create environment first:\n  \
                     torrust-tracker-deployer create <environment-name>",
                    data_dir
                )
            }
            Self::MissingInstanceIp { name } => {
                format!(
                    "Environment '{}' must be provisioned before testing.\n\n\
                     Provision infrastructure:\n  \
                     torrust-tracker-deployer provision {}",
                    name, name
                )
            }
            Self::ValidationFailed { .. } => {
                "Check logs for validation failure details.\n\
                 Re-run with verbose logging:\n  \
                 torrust-tracker-deployer test <environment-name> --log-output file-and-stderr"
            }
            Self::ProgressReportingFailed { .. } => {
                "Critical internal error - please report with full logs:\n  \
                 torrust-tracker-deployer test <environment-name> --log-output file-and-stderr"
            }
        }
        .to_string()
    }
}
```

### Controller Implementation (`handler.rs`)

Implement two API levels following the provision pattern:

**High-level API** (ExecutionContext pattern):

```rust
/// Handle test command using `ExecutionContext` pattern
pub async fn handle(
    environment_name: &str,
    working_dir: &std::path::Path,
    context: &ExecutionContext,
) -> Result<(), TestSubcommandError> {
    handle_test_command(
        environment_name,
        working_dir,
        context.repository_factory(),
        &context.user_output(),
    )
    .await
}
```

**Direct dependency injection** (for testing):

```rust
/// Handle the test command with direct dependencies
pub async fn handle_test_command(
    environment_name: &str,
    working_dir: &std::path::Path,
    repository_factory: Arc<RepositoryFactory>,
    user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
) -> Result<(), TestSubcommandError> {
    let mut controller = TestCommandController::new(
        working_dir.to_path_buf(),
        repository_factory,
        user_output.clone(),
    );

    controller.execute(environment_name).await
}
```

**Constants**:

```rust
/// Number of main steps in the test workflow
const TEST_WORKFLOW_STEPS: usize = 4;
```

**Controller structure**:

```rust
struct TestCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}

impl TestCommandController {
    fn new(
        working_dir: PathBuf,
        repository_factory: Arc<RepositoryFactory>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let data_dir = working_dir.join("data");
        let repository = repository_factory.create(data_dir);
        let progress = ProgressReporter::new(user_output.clone(), TEST_WORKFLOW_STEPS);

        Self {
            repository,
            user_output,
            progress,
        }
    }

    async fn execute(&mut self, environment_name: &str) -> Result<(), TestSubcommandError> {
        // 1. Validate environment name
        let env_name = self.validate_environment_name(environment_name)?;

        // 2. Create command handler
        let handler = self.create_command_handler()?;

        // 3. Execute validation workflow via application layer
        self.test_infrastructure(&handler, &env_name).await?;

        // 4. Complete workflow
        self.complete_workflow(environment_name)?;

        Ok(())
    }

    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, TestSubcommandError> {
        self.progress.start_step("Validating environment")?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            TestSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    fn create_command_handler(
        &mut self,
    ) -> Result<TestCommandHandler, TestSubcommandError> {
        self.progress.start_step("Creating command handler")?;

        let handler = TestCommandHandler::new(self.repository.clone());
        self.progress.complete_step(None)?;

        Ok(handler)
    }

    async fn test_infrastructure(
        &mut self,
        handler: &TestCommandHandler,
        env_name: &EnvironmentName,
    ) -> Result<(), TestSubcommandError> {
        self.progress.start_step("Testing infrastructure")?;

        handler.execute(env_name).await.map_err(|source| {
            TestSubcommandError::ValidationFailed {
                name: env_name.to_string(),
                source: Box::new(source),
            }
        })?;

        self.progress.complete_step(Some("Infrastructure tests passed"))?;

        Ok(())
    }

    fn complete_workflow(
        &mut self,
        environment_name: &str,
    ) -> Result<(), TestSubcommandError> {
        self.progress.complete_workflow()?;

        self.user_output
            .lock()
            .borrow_mut()
            .success(&format!(
                "✓ Infrastructure validation completed successfully for '{environment_name}'"
            ));

        Ok(())
    }
}
```

### Integration with CLI (`main.rs`)

Add `test` subcommand to the CLI:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Verify deployment infrastructure
    Test {
        /// Name of the environment to test
        environment_name: String,
    },
}

// In command dispatch:
Commands::Test { environment_name } => {
    controllers::test::handle(&environment_name, &working_dir, &context)
        .await
        .map_err(|e| {
            eprintln!("Error: {e}");
            eprintln!("\n{}", e.help());
            std::process::exit(1);
        })?;
}
```

### User Experience

**Success case**:

```bash
$ torrust-tracker-deployer test my-env
[1/3] Validating cloud-init completion... ✓
[2/3] Validating Docker installation... ✓
[3/3] Validating Docker Compose installation... ✓
✓ Infrastructure validation completed successfully
```

**Error case - environment not provisioned**:

```bash
$ torrust-tracker-deployer test my-env
Error: Environment 'my-env' does not have instance IP set

Environment 'my-env' must be provisioned before testing.

Provision infrastructure:
  torrust-tracker-deployer provision my-env
```

**Error case - validation failure**:

```bash
$ torrust-tracker-deployer test my-env
[1/3] Validating cloud-init completion... ✓
[2/3] Validating Docker installation... ✗
Error: Docker is not installed on the instance

Check logs for validation failure details.
Re-run with verbose logging:
  torrust-tracker-deployer test my-env --log-output file-and-stderr
```

## Implementation Plan

### Phase 1: Module Structure and Error Types (1-2 hours)

- [ ] Create `src/presentation/controllers/test/` directory
- [ ] Create `mod.rs` with module documentation following provision pattern
- [ ] Implement `errors.rs` with all error variants and `.help()` methods
- [ ] Add comprehensive error context and actionable troubleshooting
- [ ] Create `tests/mod.rs` skeleton

### Phase 2: Controller Implementation (2-3 hours)

- [ ] Implement `TestCommandController` struct in `handler.rs`
- [ ] Implement `validate_environment_name` method with progress reporting
- [ ] Implement `create_command_handler` method to instantiate `TestCommandHandler`
- [ ] Implement `test_infrastructure` method to delegate to application layer
- [ ] Implement `complete_workflow` method with success message
- [ ] Setup `ProgressReporter` for 4-step workflow (validate env, create handler, test, complete)
- [ ] Add proper error conversion and context propagation
- [ ] Implement both API levels (ExecutionContext + direct injection)

### Phase 3: CLI Integration (1 hour)

- [ ] Add `Test` variant to `Commands` enum in `main.rs`
- [ ] Implement command dispatch for `test` subcommand
- [ ] Add error handling with `.help()` output
- [ ] Update `src/presentation/controllers/mod.rs` to export test controller

### Phase 4: Testing (2-3 hours)

- [ ] Write unit tests for error `.help()` messages
- [ ] Write unit tests for environment name validation
- [ ] Write integration tests for successful validation workflow
- [ ] Write integration tests for error scenarios:
  - Invalid environment name
  - Environment not found
  - Missing instance IP
  - Validation failures (cloud-init, Docker, Docker Compose)
- [ ] Test with real provisioned environment (manual E2E verification)

### Phase 5: Documentation and Manual Testing (2-3 hours)

- [ ] Add rustdoc documentation for all public APIs
- [ ] Update `docs/console-commands.md` with test command usage
- [ ] Manual testing: Complete deployment workflow verification (see Manual Testing Phase below)
- [ ] Verify error messages are clear and actionable
- [ ] Check progress reporting works correctly

### Phase 6: Quality Assurance (30 minutes)

- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`
- [ ] Fix any linting issues
- [ ] Ensure all tests pass
- [ ] Verify documentation builds correctly

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] CLI command `torrust-tracker-deployer test <environment-name>` executes successfully
- [ ] Command validates environment name and provides clear error for invalid names
- [ ] Command checks environment exists and provides actionable error if not found
- [ ] Command verifies instance IP is set and guides user to provision if missing
- [ ] Progress reporter shows 4 workflow steps: validate env, create handler, test infrastructure, complete
- [ ] All validation checks (cloud-init, Docker, Docker Compose) are delegated to `TestCommandHandler`
- [ ] All error types implement `.help()` with specific troubleshooting steps
- [ ] Error messages follow project conventions (context + actionable guidance)
- [ ] Module structure matches provision controller pattern
- [ ] Both API levels implemented (ExecutionContext + direct injection)
- [ ] Integration tests cover success and error scenarios
- [ ] Documentation updated in `docs/console-commands.md`
- [ ] Manual E2E test completes full deployment workflow using only CLI commands

## Manual Testing Phase

After implementation, perform comprehensive manual testing of the full deployment workflow using **only the public CLI interface**:

### Test Scenario: Complete Deployment Workflow

This scenario verifies that all CLI commands work together seamlessly and the `test` command properly validates infrastructure at the appropriate stage.

**Prerequisites**:

- Clean environment (no existing deployment)
- LXD installed and configured
- Working directory: `./data/manual-test-env/`

**Steps**:

1. **Create Environment**

   ```bash
   torrust-tracker-deployer create manual-test-env
   ```

   - Expected: Environment created successfully
   - Verify: `ls -la ./data/manual-test-env/` shows environment directory

2. **Test Before Provision** (should fail gracefully)

   ```bash
   torrust-tracker-deployer test manual-test-env
   ```

   - Expected: Error with message about missing instance IP
   - Expected: Help text guides user to run `provision` command
   - Verify: Error is actionable and clear

3. **Provision Infrastructure**

   ```bash
   torrust-tracker-deployer provision manual-test-env
   ```

   - Expected: VM provisioned successfully
   - Expected: Instance IP assigned
   - Verify: Progress steps complete successfully

4. **Test After Provision, Before Configure** (should fail on Docker checks)

   ```bash
   torrust-tracker-deployer test manual-test-env
   ```

   - Expected: Cloud-init validation passes
   - Expected: Docker validation fails (not yet installed)
   - Expected: Clear error message about Docker not being installed
   - Verify: Progress shows step 1 succeeded, step 2 failed

5. **Configure Infrastructure**

   ```bash
   torrust-tracker-deployer configure manual-test-env
   ```

   - Expected: Configuration applied successfully (Docker and Docker Compose installed)
   - Verify: Progress steps complete successfully

6. **Test After Configure** (should pass all validations)

   ```bash
   torrust-tracker-deployer test manual-test-env
   ```

   - Expected: All 3 validation steps pass:
     - [1/3] Validating cloud-init completion... ✓
     - [2/3] Validating Docker installation... ✓
     - [3/3] Validating Docker Compose installation... ✓
   - Expected: Success message displayed
   - Verify: No errors, all checks green

7. **Test Idempotency** (run test multiple times)

   ```bash
   torrust-tracker-deployer test manual-test-env
   torrust-tracker-deployer test manual-test-env
   ```

   - Expected: Both runs succeed with same results
   - Verify: No state changes, consistent output

8. **Test Invalid Environment Name**

   ```bash
   torrust-tracker-deployer test invalid_name_with_underscores
   ```

   - Expected: Clear validation error
   - Expected: Help text shows valid naming format
   - Verify: Error message is actionable

9. **Test Non-existent Environment**

   ```bash
   torrust-tracker-deployer test does-not-exist
   ```

   - Expected: Environment not found error
   - Expected: Help text suggests checking directory and creating environment
   - Verify: Error message is actionable

10. **Cleanup**

    ```bash
    torrust-tracker-deployer destroy manual-test-env
    ```

    - Expected: Environment destroyed successfully
    - Verify: Infrastructure cleaned up

### Manual Test Acceptance Criteria

- [ ] All 10 test steps execute as expected
- [ ] Error messages are clear and actionable at each failure point
- [ ] Progress reporting shows correct step counts and descriptions
- [ ] Test command integrates seamlessly into deployment workflow
- [ ] Help messages guide users to correct next steps
- [ ] Command is idempotent (can run multiple times safely)
- [ ] No confusing or misleading output

## Related Documentation

- [Codebase Architecture](../codebase-architecture.md) - DDD layer organization
- [Error Handling Guide](../contributing/error-handling.md) - Error conventions and `.help()` patterns
- [Module Organization](../contributing/module-organization.md) - Code organization within modules
- [ADR: Test Command as Smoke Test](../decisions/test-command-as-smoke-test.md) - Design rationale
- [Provision Controller Implementation](../../src/presentation/controllers/provision/) - Reference implementation
- [Test Command Handler](../../src/application/command_handlers/test/) - Business logic being wrapped

## Notes

### Design Considerations

- **Why separate presentation layer?** - Follows DDD principles, separates user interaction from business logic, enables reusability (the same `TestCommandHandler` could be used by REST API, gRPC, etc.)
- **Why two API levels?** - `ExecutionContext` for production use, direct injection for testing flexibility
- **Why ProgressReporter?** - Provides consistent UX across all commands, helps users understand multi-step workflows
- **Why explicit error types?** - Type-safe error handling, enables pattern matching, forces comprehensive error documentation

### Future Enhancements

When the `Running` state is implemented and the deployment workflow is complete, the `test` command will be updated to perform actual smoke tests:

- HTTP health checks to Tracker services
- Basic API request verification
- Metrics endpoint validation

The current infrastructure validation steps are **temporary scaffolding** and will be removed. See the `TestCommandHandler` documentation for details on the target implementation.

### Implementation Notes

- Follow the exact same pattern as `configure` controller for consistency
- The controller **only** orchestrates the workflow - all validation logic is in `TestCommandHandler`
- Controller responsibilities:
  - Validate environment name format
  - Create and invoke the application layer `TestCommandHandler`
  - Report progress to the user (4 steps)
  - Format success/error messages
- The `TestCommandHandler.execute()` method performs all infrastructure validation internally:
  - Cloud-init completion check
  - Docker installation verification
  - Docker Compose installation verification
- Reuse existing `ProgressReporter` infrastructure
- Leverage `ExecutionContext` pattern already established in the codebase
- Pay special attention to error message quality - they are part of the user experience
- Test edge cases thoroughly (invalid names, missing environments, partial deployments)

**Estimated Total Time**: 8-12 hours
