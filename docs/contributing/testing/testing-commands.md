# Testing Commands

## Command Test Patterns

Commands in the application layer require comprehensive testing at multiple levels:

### Unit Tests with Test Builders

Commands should provide test builders for simplified unit testing:

```rust
use torrust_tracker_deployer::application::commands::destroy::tests::DestroyCommandTestBuilder;

#[test]
fn it_should_create_destroy_command_with_all_dependencies() {
    let (command, _temp_dir) = DestroyCommandTestBuilder::new().build();

    // Verify the command was created
    assert_eq!(Arc::strong_count(&command.opentofu_client), 1);
}
```

**Benefits of Test Builders**:

- Manages `TempDir` lifecycle automatically
- Provides sensible defaults for all dependencies
- Allows selective customization of dependencies
- Returns only the command and necessary test artifacts

### Mock Strategies for Commands

For testing error scenarios and edge cases, use mocks:

```rust
use mockall::predicate::*;

#[test]
fn it_should_handle_infrastructure_failure_gracefully() {
    // Create mock dependencies
    let mut mock_client = MockOpenTofuClient::new();
    mock_client.expect_destroy()
        .times(1)
        .returning(|| Err(OpenTofuError::DestroyFailed));

    let mock_repo = Arc::new(MockEnvironmentRepository::new());

    // Create command with mocks
    let command = DestroyCommand::new(Arc::new(mock_client), mock_repo);

    // Execute and verify error handling
    let result = command.execute(test_environment);
    assert!(matches!(result, Err(DestroyCommandError::OpenTofu(_))));
}
```

**When to Use Mocks**:

- Testing error handling paths
- Validating retry logic
- Simulating external service failures
- Testing timeout scenarios

### Integration Tests with E2E Infrastructure

Commands should be tested with real infrastructure in E2E tests:

```rust
#[test]
fn it_should_destroy_real_infrastructure() {
    // Create real dependencies
    let temp_dir = TempDir::new().unwrap();
    let opentofu_client = Arc::new(OpenTofuClient::new(temp_dir.path()));
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(temp_dir.path().to_path_buf());

    // Create command with real dependencies
    let command = DestroyCommand::new(opentofu_client, repository);

    // Execute against real infrastructure
    let destroyed = command.execute(provisioned_environment).unwrap();

    // Verify cleanup
    assert!(!destroyed.data_dir().exists());
    assert!(!destroyed.build_dir().exists());
}
```

**Integration Test Guidelines**:

- Use real external tools (OpenTofu, Ansible) when possible
- Validate actual state changes in infrastructure
- Ensure proper cleanup even on test failure
- Test complete workflows end-to-end

## Testing Destroy Command

The `DestroyCommand` requires special testing considerations:

### Idempotency Testing

Verify the command can be run multiple times safely:

```rust
#[test]
fn it_should_be_idempotent() {
    let (command, _temp_dir) = DestroyCommandTestBuilder::new().build();
    let environment = create_test_environment();

    // First destroy
    let result1 = command.execute(environment.clone());
    assert!(result1.is_ok());

    // Second destroy (should succeed even if already destroyed)
    let result2 = command.execute(environment);
    assert!(result2.is_ok());
}
```

### Cleanup Testing

Verify state files are removed after destruction:

```rust
#[test]
fn it_should_clean_up_state_files() {
    let temp_dir = TempDir::new().unwrap();
    let environment = Environment::new_in_dir(
        EnvironmentName::new("test".to_string()).unwrap(),
        temp_dir.path(),
    );

    // Create some state files
    std::fs::create_dir_all(environment.data_dir()).unwrap();
    std::fs::create_dir_all(environment.build_dir()).unwrap();

    // Execute destroy
    let (command, _) = DestroyCommandTestBuilder::new().build();
    command.execute(environment).unwrap();

    // Verify directories are removed
    assert!(!temp_dir.path().join("data/test").exists());
    assert!(!temp_dir.path().join("build/test").exists());
}
```

### Error Recovery Testing

Test partial failure scenarios:

```rust
#[test]
fn it_should_handle_partial_cleanup_failure() {
    // Create environment with read-only directories
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().join("data/test");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Make directory read-only (simulates permission error)
    let metadata = std::fs::metadata(&data_dir).unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&data_dir, permissions).unwrap();

    // Execute destroy (should fail on cleanup)
    let (command, _) = DestroyCommandTestBuilder::new().build();
    let result = command.execute(environment);

    // Verify appropriate error
    assert!(matches!(result, Err(DestroyCommandError::StateCleanupFailed { .. })));
}
```

## E2E Test Integration

Commands should be integrated into E2E test suites:

### Provision and Destroy E2E Tests

The `e2e-provision-and-destroy-tests` binary tests the complete infrastructure lifecycle:

```rust
// From src/bin/e2e_provision_and_destroy_tests.rs

// Provision infrastructure
let provisioned_env = run_provision_command(&context).await?;

// Validate provisioning
validate_infrastructure(&provisioned_env).await?;

// Destroy infrastructure using DestroyCommand
if let Err(e) = run_destroy_command(&context).await {
    error!("DestroyCommand failed: {}, falling back to manual cleanup", e);
    cleanup_test_infrastructure(&context).await?;
}
```

**E2E Test Strategy**:

- Test complete workflows with real infrastructure
- Use fallback cleanup for CI reliability
- Validate state transitions at each step
- Ensure cleanup regardless of test outcome

For detailed E2E testing information, see [`docs/e2e-testing.md`](../../e2e-testing.md).
