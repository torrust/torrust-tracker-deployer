# Testing Conventions

This document outlines the testing conventions for the Torrust Tracker Deploy project.

## 🧪 Unit Test Naming Style

Unit tests should use descriptive, behavior-driven naming with the `it_should_` prefix instead of the generic `test_` prefix.

### Naming Convention

- **Format**: `it_should_{describe_expected_behavior}`
- **Style**: Use lowercase with underscores, be descriptive and specific
- **Focus**: Describe what the test validates, not just what function it calls

### Examples

#### ✅ Good Test Names

```rust
#[test]
fn it_should_create_ansible_host_with_valid_ipv4() {
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let host = AnsibleHost::new(ip);
    assert_eq!(host.as_ip_addr(), &ip);
}

#[test]
fn it_should_fail_with_invalid_ip_address() {
    let result = AnsibleHost::from_str("invalid.ip.address");
    assert!(result.is_err());
}

#[test]
fn it_should_serialize_to_json() {
    let host = AnsibleHost::from_str("192.168.1.1").unwrap();
    let json = serde_json::to_string(&host).unwrap();
    assert_eq!(json, "\"192.168.1.1\"");
}
```

#### ❌ Avoid These Test Names

```rust
#[test]
fn test_new() { /* ... */ }

#[test]
fn test_from_str() { /* ... */ }

#[test]
fn test_serialization() { /* ... */ }
```

### Benefits

- **Clarity**: Test names clearly describe the expected behavior
- **Documentation**: Tests serve as living documentation of the code's behavior
- **BDD Style**: Follows Behavior-Driven Development naming conventions
- **Maintainability**: Easier to understand test failures and purpose

## 🧹 Resource Management

Tests should be isolated and clean up after themselves to avoid interfering with production data or leaving artifacts behind.

### Key Rules

- **Tests should clean the resources they create** - Use temporary directories or clean up generated files
- **They should not interfere with production data** - Never use real application directories like `./data` or `./build` in tests

### Example

#### ✅ Good: Using Temporary Directories

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

#### ❌ Avoid: Creating Real Directories

```rust
#[test]
fn it_should_create_environment() {
    // Don't do this - creates real directories that persist after tests
    let env_name = EnvironmentName::new("test".to_string()).unwrap();
    let environment = Environment::new(env_name, ssh_credentials);
    // Creates ./data/test and ./build/test directories
}
```

## 🚀 Getting Started

When writing new tests, always use the `it_should_` prefix and describe the specific behavior being validated. This makes the test suite more readable and maintainable for all contributors.
