# Command Testing

Commands in the application layer require comprehensive testing at multiple levels: unit tests, integration tests, and E2E tests.

## Integration Tests with E2E Infrastructure

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

## E2E Test Integration

Commands should be integrated into E2E test suites:

### Provision and Destroy E2E Tests

The `e2e-infrastructure-lifecycle-tests` binary tests the complete infrastructure lifecycle:

```rust
// From src/bin/e2e_infrastructure_lifecycle_tests.rs

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

For detailed E2E testing information, see [`docs/e2e-testing/`](../../../e2e-testing/).
