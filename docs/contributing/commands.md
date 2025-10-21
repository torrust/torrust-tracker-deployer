# Command Architecture - Developer Guide

This guide explains the command architecture in the Torrust Tracker Deployer for developers working on the codebase. It covers internal implementation details, patterns, and best practices for developing and maintaining commands in the application layer.

## Overview

Commands are the primary entry points in the application layer that orchestrate complex workflows by composing multiple steps. They follow Domain-Driven Design (DDD) principles and provide a clean interface between the user interface (CLI) and the underlying business logic.

## Architecture Patterns

### DDD Application Layer

Commands reside in the `src/application/commands/` directory and follow DDD patterns:

```text
src/application/
├── commands/           # Application layer commands
│   ├── mod.rs
│   ├── provision.rs    # ProvisionCommand
│   ├── configure.rs    # ConfigureCommand
│   ├── destroy.rs      # DestroyCommand
│   └── test.rs         # TestCommand
└── steps/              # Reusable workflow steps
    ├── mod.rs
    ├── infrastructure/
    └── system/
```

**Key Principles**:

- **Commands orchestrate, Steps execute**: Commands compose steps; they don't implement low-level logic
- **Type-state pattern**: Commands work with typed environment states (`Environment<Created>` → `Environment<Provisioned>`)
- **Dependency injection**: All external dependencies (clients, repositories) are injected
- **Error aggregation**: Commands define comprehensive error types that aggregate all possible failures

### Three-Level Pattern

The codebase follows a three-level abstraction pattern for deployment operations:

```text
1. Commands (Application Layer)
   └─> Orchestrate workflows, manage state transitions
       │
2. Steps (Application Layer)
   └─> Execute specific tasks, coordinate actions
       │
3. Actions (Infrastructure Layer)
   └─> Interact with external tools and systems
```

For detailed information about this pattern, see [`docs/codebase-architecture.md`](../codebase-architecture.md).

## Command Implementation Guide

### DestroyCommand

The `DestroyCommand` orchestrates the complete infrastructure destruction workflow.

#### Architecture

Located in `src/application/commands/destroy.rs`, the command follows established patterns:

- **State Transition**: Any state → `Destroyed`
- **Idempotency**: Safe to run multiple times; succeeds if infrastructure already destroyed
- **Error Handling**: Comprehensive error types with tracing support
- **State Persistence**: Persists destroyed state after successful execution

#### Internal API Usage

##### Basic Usage

```rust
use std::sync::Arc;
use torrust_tracker_deployer::application::commands::destroy::DestroyCommand;
use torrust_tracker_deployer::domain::Environment;

async fn destroy_environment(
    environment: Environment<S>,
    opentofu_client: Arc<OpenTofuClient>,
    repository: Arc<dyn EnvironmentRepository>,
) -> Result<Environment<Destroyed>, DestroyCommandError> {
    let destroy_command = DestroyCommand::new(opentofu_client, repository);
    destroy_command.execute(environment)
}
```

##### Integration with E2E Tests

```rust
use torrust_tracker_deployer::application::commands::destroy::DestroyCommand;

async fn cleanup_test_environment(
    env: Environment<Provisioned>,
    opentofu_client: Arc<OpenTofuClient>,
    repository: Arc<dyn EnvironmentRepository>,
) -> Result<(), DestroyCommandError> {
    let destroy_cmd = DestroyCommand::new(opentofu_client, repository);
    let destroyed_env = destroy_cmd.execute(env)?;
    
    // Environment is now in Destroyed state
    Ok(())
}
```

#### Error Handling

The `DestroyCommand` defines a comprehensive error type:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DestroyCommandError {
    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] RepositoryError),

    #[error("Failed to clean up state files at '{path}': {source}")]
    StateCleanupFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
```

**Error Types Explained**:

- **OpenTofu**: Infrastructure destruction failed (e.g., network issues, resource conflicts)
- **Command**: External command execution failed (e.g., process spawn failure)
- **StatePersistence**: Failed to save the destroyed state to repository
- **StateCleanupFailed**: Failed to remove data/build directories after destruction

#### Error Context and Traceability

The command implements the `Traceable` trait for deep error context:

```rust
impl Traceable for DestroyCommandError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofu(e) => {
                format!("DestroyCommandError: OpenTofu command failed - {e}")
            }
            // ... other variants
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        match self {
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            // ... other variants
        }
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::OpenTofu(_) => ErrorKind::InfrastructureOperation,
            Self::Command(_) => ErrorKind::CommandExecution,
            Self::StatePersistence(_) | Self::StateCleanupFailed { .. } => {
                ErrorKind::StatePersistence
            }
        }
    }
}
```

**For user-facing error handling**, see [`docs/contributing/error-handling.md`](error-handling.md).

#### Testing Strategies

##### Unit Testing with Test Builder

The command provides a test builder for simplified unit testing:

```rust
use torrust_tracker_deployer::application::commands::destroy::tests::DestroyCommandTestBuilder;

#[test]
fn it_should_create_destroy_command_with_all_dependencies() {
    let (command, _temp_dir) = DestroyCommandTestBuilder::new().build();

    // Verify the command was created
    assert_eq!(Arc::strong_count(&command.opentofu_client), 1);
}
```

**Benefits**:

- Manages `TempDir` lifecycle automatically
- Provides sensible defaults for all dependencies
- Allows selective customization of dependencies
- Returns only the command and necessary test artifacts

##### Mock Strategies

For more complex testing scenarios, inject mock dependencies:

```rust
use std::sync::Arc;
use mockall::predicate::*;

#[test]
fn it_should_handle_opentofu_failure_gracefully() {
    // Create mock OpenTofu client that fails
    let mut mock_client = MockOpenTofuClient::new();
    mock_client.expect_destroy()
        .times(1)
        .returning(|| Err(OpenTofuError::DestroyFailed));

    // Create mock repository
    let mock_repo = Arc::new(MockEnvironmentRepository::new());

    // Create command with mocks
    let command = DestroyCommand::new(Arc::new(mock_client), mock_repo);

    // Execute and verify error handling
    let result = command.execute(test_environment);
    assert!(matches!(result, Err(DestroyCommandError::OpenTofu(_))));
}
```

##### Integration Testing

For integration tests with real infrastructure:

```rust
#[test]
fn it_should_destroy_real_infrastructure() {
    // Create real OpenTofu client
    let temp_dir = TempDir::new().unwrap();
    let opentofu_client = Arc::new(OpenTofuClient::new(temp_dir.path()));

    // Create real repository
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(temp_dir.path().to_path_buf());

    // Create command with real dependencies
    let command = DestroyCommand::new(opentofu_client, repository);

    // Execute against real infrastructure
    let destroyed = command.execute(provisioned_environment).unwrap();

    // Verify state
    assert!(destroyed.data_dir().exists() == false);
    assert!(destroyed.build_dir().exists() == false);
}
```

#### Workflow Details

The `DestroyCommand` executes the following workflow:

1. **Infrastructure Destruction**
   - Executes `DestroyInfrastructureStep` (wraps OpenTofu destroy)
   - Idempotent: succeeds even if infrastructure doesn't exist
   - Validates destruction completion

2. **State Transition**
   - Transitions environment to `Destroyed` state
   - Uses type-state pattern for compile-time safety

3. **State Cleanup**
   - Removes data directory (`data/<environment-name>/`)
   - Removes build directory (`build/<environment-name>/`)
   - Only executes after successful infrastructure destruction

4. **State Persistence**
   - Saves destroyed state to repository
   - Enables state recovery and auditing

**Error Recovery**: On failure, cleanup may be partial. Users should manually verify and complete cleanup if necessary.

#### Debugging and Development

##### Logging

The command uses structured logging with tracing:

```rust
#[instrument(
    name = "destroy_command",
    skip_all,
    fields(
        command_type = "destroy",
        environment = %environment.name()
    )
)]
pub fn execute<S>(
    &self,
    environment: Environment<S>,
) -> Result<Environment<Destroyed>, DestroyCommandError> {
    info!(
        command = "destroy",
        environment = %environment.name(),
        "Starting complete infrastructure destruction workflow"
    );
    
    // ... implementation
    
    info!(
        command = "destroy",
        environment = %destroyed.name(),
        "Infrastructure destruction completed successfully"
    );
}
```

**Enable debug logging**:

```bash
RUST_LOG=debug cargo test
RUST_LOG=torrust_tracker_deployer::application::commands::destroy=trace cargo test
```

##### Common Issues

**Issue**: State cleanup fails but infrastructure is destroyed

```text
DestroyCommandError: Failed to clean up state files at 'data/my-env'
```

**Solution**: Manually remove directories:

```bash
rm -rf data/my-env build/my-env
```

**Issue**: OpenTofu destroy hangs or fails

```text
DestroyCommandError: OpenTofu command failed - timeout
```

**Solution**: Check OpenTofu state and manually destroy:

```bash
cd build/tofu/lxd
tofu destroy -auto-approve
```

### ProvisionCommand

The `ProvisionCommand` orchestrates the complete infrastructure provisioning workflow.

#### Architecture

Located in `src/application/commands/provision.rs`, the command handles:

- **State Transition**: `Created` → `Provisioned`
- **Template Rendering**: OpenTofu and Ansible templates with environment-specific data
- **Infrastructure Creation**: VM/container provisioning via OpenTofu
- **Validation**: Cloud-init completion and SSH connectivity checks

#### Internal API Usage

```rust
use torrust_tracker_deployer::application::commands::provision::ProvisionCommand;

async fn provision_new_environment(
    environment: Environment<Created>,
    opentofu_client: Arc<OpenTofuClient>,
    ansible_client: Arc<AnsibleClient>,
    repository: Arc<dyn EnvironmentRepository>,
) -> Result<Environment<Provisioned>, ProvisionCommandError> {
    let provision_command = ProvisionCommand::new(
        opentofu_client,
        ansible_client,
        repository,
    );
    
    provision_command.execute(environment)
}
```

#### Workflow Details

1. **Template Rendering**
   - Renders OpenTofu templates to `build/tofu/lxd/`
   - Prepares infrastructure configuration files

2. **Infrastructure Initialization**
   - Runs `tofu init` to initialize providers

3. **Infrastructure Planning**
   - Runs `tofu plan` to preview changes

4. **Infrastructure Application**
   - Runs `tofu apply` to create infrastructure

5. **Instance Information Retrieval**
   - Gets VM IP address and instance details

6. **Ansible Template Rendering**
   - Renders Ansible playbooks with dynamic VM data

7. **System Readiness Validation**
   - Waits for cloud-init completion
   - Validates SSH connectivity

8. **State Persistence**
   - Saves provisioned state with instance information

### ConfigureCommand

The `ConfigureCommand` orchestrates system configuration and software installation.

#### Architecture

Located in `src/application/commands/configure.rs`, the command handles:

- **State Transition**: `Provisioned` → `Configured`
- **Software Installation**: Docker, Docker Compose, and system packages
- **System Configuration**: Firewall, security updates, monitoring

#### Internal API Usage

```rust
use torrust_tracker_deployer::application::commands::configure::ConfigureCommand;

async fn configure_environment(
    environment: Environment<Provisioned>,
    ansible_client: Arc<AnsibleClient>,
    repository: Arc<dyn EnvironmentRepository>,
) -> Result<Environment<Configured>, ConfigureCommandError> {
    let configure_command = ConfigureCommand::new(ansible_client, repository);
    configure_command.execute(environment)
}
```

## Common Patterns

### Dependency Injection

All commands use constructor injection for dependencies:

```rust
pub struct DestroyCommand {
    opentofu_client: Arc<OpenTofuClient>,
    repository: Arc<dyn EnvironmentRepository>,
}

impl DestroyCommand {
    pub fn new(
        opentofu_client: Arc<OpenTofuClient>,
        repository: Arc<dyn EnvironmentRepository>,
    ) -> Self {
        Self {
            opentofu_client,
            repository,
        }
    }
}
```

**Benefits**:

- Testability: Easy to inject mocks for testing
- Flexibility: Can swap implementations without changing command code
- Clarity: Dependencies are explicit and documented

### Type-State Pattern

Commands work with typed environment states:

```rust
// Command signature shows state transition
pub fn execute<S>(
    &self,
    environment: Environment<S>,
) -> Result<Environment<Destroyed>, DestroyCommandError>

// Usage
let provisioned: Environment<Provisioned> = /* ... */;
let destroyed: Environment<Destroyed> = destroy_cmd.execute(provisioned)?;
```

**Benefits**:

- **Compile-time safety**: Invalid state transitions caught at compile time
- **Type documentation**: Function signatures document state requirements
- **Refactoring safety**: State changes detected by compiler

For details, see the [Command State Return Pattern ADR](../decisions/command-state-return-pattern.md).

### Error Composition

Commands aggregate errors from multiple sources:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("External tool failed: {0}")]
    ExternalTool(#[from] OpenTofuError),
    
    #[error("Infrastructure issue: {0}")]
    Infrastructure(#[from] SshError),
    
    #[error("State management failed: {0}")]
    State(#[from] RepositoryError),
}
```

**Best Practices**:

- Use `#[from]` for automatic conversion where appropriate
- Provide detailed error messages with context
- Implement `Traceable` trait for error chains
- Include actionable information in error messages

For comprehensive error handling guidelines, see [`docs/contributing/error-handling.md`](error-handling.md).

## CI/CD Considerations

### Command Testing in CI

Commands are tested through multiple test suites:

1. **Unit Tests**: Test individual command logic with mocks
2. **Integration Tests**: Test commands with real dependencies in containers
3. **E2E Tests**: Test complete workflows with infrastructure

**CI Configuration**:

```yaml
# .github/workflows/test-commands.yml
- name: Test Commands
  run: |
    cargo test --lib application::commands
    cargo run --bin e2e-provision-tests
    cargo run --bin e2e-config-tests
```

### Performance Considerations

- **Provision Tests**: ~30 seconds (LXD VM creation)
- **Configuration Tests**: ~2-3 seconds (Docker containers)
- **Destroy Tests**: ~10 seconds (infrastructure cleanup)

For CI optimizations, see [`docs/e2e-testing.md`](../e2e-testing.md).

## Related Documentation

- [Error Handling Guide](error-handling.md) - Error handling patterns and user-facing errors
- [Testing Conventions](testing.md) - Unit test patterns and best practices
- [E2E Testing Guide](../e2e-testing.md) - End-to-end testing with commands
- [Codebase Architecture](../codebase-architecture.md) - Three-level pattern and DDD layers
- [Command State Return Pattern ADR](../decisions/command-state-return-pattern.md) - Type-state pattern decision
- [Actionable Error Messages ADR](../decisions/actionable-error-messages.md) - Error message design

## Contributing

When adding or modifying commands:

1. **Follow established patterns**: Use dependency injection, type-state, and error composition
2. **Write comprehensive tests**: Unit, integration, and E2E tests
3. **Document internal APIs**: Add code examples for developers
4. **Update this guide**: Add new commands and patterns to this document
5. **Consider error scenarios**: Implement proper error handling with tracing
6. **Add logging**: Use structured logging with tracing for observability

For detailed contribution guidelines, see [`docs/contributing/README.md`](README.md).
