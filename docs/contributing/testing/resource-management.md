# Resource Management

Tests should be isolated and clean up after themselves to avoid interfering with production data or leaving artifacts behind.

## Key Rules

- **Tests should clean the resources they create** - Use temporary directories or clean up generated files
- **They should not interfere with production data** - Never use real application directories like `./data` or `./build` in tests

## Example

### ✅ Good: Using Temporary Directories

```rust
use tempfile::TempDir;

#[test]
fn it_should_create_environment_with_auto_generated_paths() {
    // Use temporary directory to avoid creating real directories
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let env_name = EnvironmentName::new("test-example".to_string()).unwrap();
    let ssh_credentials = create_test_ssh_credentials(&temp_path);
    let environment = Environment::new(env_name, ssh_credentials);

    assert!(environment.data_dir().starts_with(temp_path));
    assert!(environment.build_dir().starts_with(temp_path));

    // TempDir automatically cleans up when dropped
}
```

### ❌ Avoid: Creating Real Directories

```rust
#[test]
fn it_should_create_environment() {
    // Don't do this - creates real directories that persist after tests
    let env_name = EnvironmentName::new("test".to_string()).unwrap();
    let environment = Environment::new(env_name, ssh_credentials);
    // Creates ./data/test and ./build/test directories
}
```
