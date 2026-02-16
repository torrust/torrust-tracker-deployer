# Idempotency Testing

The `DestroyCommand` and similar operations require special testing considerations to ensure they can be safely executed multiple times.

## Verifying Idempotency

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

## Cleanup Testing

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

## Error Recovery Testing

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
