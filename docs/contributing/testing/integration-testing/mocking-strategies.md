# Mocking Strategies for Commands

For testing error scenarios and edge cases, use mocks:

## Example

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

## When to Use Mocks

- Testing error handling paths
- Validating retry logic
- Simulating external service failures
- Testing timeout scenarios
