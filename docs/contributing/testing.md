# Testing Conventions

This document outlines the testing conventions for the Torrust Tracker Deployer project.

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

## 🎯 Principles

Test code should be held to the same quality standards as production code. Tests are not second-class citizens in the codebase.

### Core Principles

- **Maintainability**: Tests should be easy to update when requirements change
- **Readability**: Tests should be clear and understandable at first glance
- **Reliability**: Tests should be deterministic and not flaky
- **Isolation**: Each test should be independent and not affect other tests
- **Documentation**: Tests serve as living documentation of the system's behavior

Just like production code, tests should follow:

- **DRY (Don't Repeat Yourself)**: Extract common setup logic into helpers and builders
- **Single Responsibility**: Each test should verify one behavior
- **Clear Intent**: Test names and structure should make the purpose obvious
- **Clean Code**: Apply the same refactoring and quality standards as production code

Remember: **If the test code is hard to read or maintain, it will become a burden rather than an asset.**

## ✅ Good Practices

### AAA Pattern (Arrange-Act-Assert)

All tests should follow the AAA pattern, also known as Given-When-Then:

- **Arrange (Given)**: Set up the test data and preconditions
- **Act (When)**: Execute the behavior being tested
- **Assert (Then)**: Verify the expected outcome

This pattern makes tests:

- Easy to read and understand
- Clear about what is being tested
- Simple to maintain and modify

#### Example

```rust
#[test]
fn it_should_create_ansible_host_with_valid_ipv4() {
    // Arrange: Set up test data
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    // Act: Execute the behavior
    let host = AnsibleHost::new(ip);

    // Assert: Verify the outcome
    assert_eq!(host.as_ip_addr(), &ip);
}
```

#### Benefits

- **Clarity**: Each section has a clear purpose
- **Structure**: Consistent test organization across the codebase
- **Debugging**: Easy to identify which phase is failing
- **Maintenance**: Simple to modify specific parts of the test

### Parameterized Tests Over Loops

When testing the same behavior with different inputs and expected outputs, prefer parameterized tests over loops in the test body.

**Why?** Parameterized tests provide:

- **Better Test Isolation**: Each parameter combination runs as a separate test case
- **Clearer Test Output**: Individual test cases show up separately in test results
- **Parallel Execution**: Test framework can run each case in parallel
- **Easier Debugging**: When a test fails, you know exactly which parameter combination caused it
- **Better IDE Support**: Modern IDEs can run individual parameterized test cases

**How?** Use the `rstest` crate for parameterized testing.

#### ❌ Avoid: Loop in Test Body

```rust
#[test]
fn it_should_create_state_file_in_environment_specific_subdirectory() {
    let test_cases = vec![
        ("e2e-config", "e2e-config/state.json"),
        ("e2e-full", "e2e-full/state.json"),
        ("e2e-provision", "e2e-provision/state.json"),
    ];

    for (env_name, expected_path) in test_cases {
        // Test logic here...
        // If one case fails, you don't know which one without debugging
    }
}
```

**Problem**: If the second iteration fails, the test output only shows the test name, not which specific case failed.

#### ✅ Good: Parameterized Test with rstest

```rust
use rstest::rstest;

#[rstest]
#[case("e2e-config", "e2e-config/state.json")]
#[case("e2e-full", "e2e-full/state.json")]
#[case("e2e-provision", "e2e-provision/state.json")]
fn it_should_create_state_file_in_environment_specific_subdirectory(
    #[case] env_name: &str,
    #[case] expected_path: &str,
) {
    // Test logic here...
    // Each case runs as a separate test with clear identification
}
```

**Benefits**: Test output shows individual cases:

- `it_should_create_state_file_in_environment_specific_subdirectory::case_1` ✅
- `it_should_create_state_file_in_environment_specific_subdirectory::case_2` ✅
- `it_should_create_state_file_in_environment_specific_subdirectory::case_3` ✅

#### When to Use Parameterized Tests

Use parameterized tests when:

- ✅ Testing the same behavior with multiple input/output combinations
- ✅ Validating edge cases with different values
- ✅ Testing configuration variations
- ✅ Verifying data transformation with various inputs

Don't use parameterized tests when:

- ❌ Each case tests fundamentally different behavior (use separate tests)
- ❌ The test logic differs significantly between cases
- ❌ You only have one or two cases (just write separate tests)

#### Setup

Add `rstest` to your `Cargo.toml`:

```toml
[dev-dependencies]
rstest = "0.23"
```

Then import it in your test module:

```rust
#[cfg(test)]
mod tests {
    use rstest::rstest;
    // ... other imports
}
```

### Using the Clock Service for Deterministic Time Tests

Time is treated as an infrastructure concern. Always use the `Clock` trait instead of calling `Utc::now()` directly to make time-dependent code testable and deterministic.

#### Why Use Clock Service?

Direct use of `Utc::now()` makes tests:

- **Non-deterministic**: Tests produce different results on each run
- **Hard to test**: Cannot control time progression or test specific timestamps
- **Difficult to debug**: Time-related issues are hard to reproduce

The Clock service solves these problems by:

- **Controlling time in tests**: Set specific timestamps for predictable behavior
- **Making tests deterministic**: Same test always produces same result
- **Testing time-dependent logic**: Simulate time progression without actual delays
- **Enabling edge case testing**: Test timeouts, expiration, and time-based conditions

#### Production Code

In production code, inject the `Clock` dependency:

```rust
use crate::shared::Clock;
use chrono::{DateTime, Utc};

pub struct EventRecorder {
    clock: Arc<dyn Clock>,
}

impl EventRecorder {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self { clock }
    }

    pub fn record_event(&self) -> DateTime<Utc> {
        // Use clock.now() instead of Utc::now()
        let timestamp = self.clock.now();
        println!("Event recorded at: {}", timestamp);
        timestamp
    }
}
```

#### Test Code

In tests, use `MockClock` for full control over time:

```rust
use crate::testing::MockClock;
use chrono::{TimeZone, Utc};
use std::sync::Arc;

#[test]
fn it_should_record_event_with_specific_timestamp() {
    // Arrange: Set up mock clock with fixed time
    let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let clock = Arc::new(MockClock::new(fixed_time));
    let recorder = EventRecorder::new(clock.clone());

    // Act: Record event
    let recorded_time = recorder.record_event();

    // Assert: Verify exact timestamp
    assert_eq!(recorded_time, fixed_time);
}

#[test]
fn it_should_handle_time_progression() {
    // Arrange: Set up mock clock
    let start_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let clock = Arc::new(MockClock::new(start_time));
    let recorder = EventRecorder::new(clock.clone());

    // Act: Record first event
    let first_event = recorder.record_event();

    // Simulate 5 minutes passing
    clock.advance_secs(300);

    // Record second event
    let second_event = recorder.record_event();

    // Assert: Verify time difference
    let expected_second = Utc.with_ymd_and_hms(2025, 10, 7, 12, 5, 0).unwrap();
    assert_eq!(first_event, start_time);
    assert_eq!(second_event, expected_second);
}
```

#### Key Benefits

- **Deterministic Tests**: Tests always produce the same results
- **Fast Execution**: No need for actual time delays with `sleep()`
- **Edge Case Testing**: Easily test timeouts, expirations, and time boundaries
- **Improved Debugging**: Failures are reproducible with exact timestamps
- **Better Test Coverage**: Can test time-dependent scenarios that would be impractical otherwise

#### When to Use

Use `MockClock` when testing:

- Timestamp generation and recording
- Timeout and expiration logic
- Time-based retries and backoff strategies
- Duration calculations and measurements
- Time-series data processing
- Scheduled operations and cron-like behavior

## 🧪 Testing Commands

### Command Test Patterns

Commands in the application layer require comprehensive testing at multiple levels:

#### Unit Tests with Test Builders

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

#### Mock Strategies for Commands

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

#### Integration Tests with E2E Infrastructure

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

### Testing Destroy Command

The `DestroyCommand` requires special testing considerations:

#### Idempotency Testing

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

#### Cleanup Testing

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

#### Error Recovery Testing

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

### E2E Test Integration

Commands should be integrated into E2E test suites:

#### Provision and Destroy E2E Tests

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

For detailed E2E testing information, see [`docs/e2e-testing.md`](../e2e-testing.md).

## 🚀 Getting Started

When writing new tests:

- Always use the `it_should_` prefix and describe the specific behavior being validated
- Use `MockClock` for any time-dependent tests instead of `Utc::now()`
- Follow the AAA pattern for clear test structure
- Ensure tests are isolated and don't interfere with each other
- Use test builders for command testing to simplify setup
- Test commands at multiple levels: unit, integration, and E2E

## 🔄 Pre-commit Integration Testing

The project includes integration tests that validate all components of the pre-commit script to ensure they work correctly in any environment (including GitHub Copilot's environment).

### How It Works

**By default, `cargo test` runs expensive integration tests** that validate:

- **Dependency check**: `cargo-machete` for unused dependencies
- **Linting**: `cargo run --bin linter all` for code quality
- **Documentation**: `cargo doc` for documentation builds
- **E2E tests**: `cargo run --bin e2e-config-tests` and `cargo run --bin e2e-provision-and-destroy-tests` for end-to-end validation

These tests ensure that when someone runs `./scripts/pre-commit.sh`, all the tools and dependencies are available and working.

### Skipping Expensive Tests During Development

If you need faster test cycles during development, you can skip the expensive integration tests:

```bash
# Skip expensive pre-commit integration tests
SKIP_EXPENSIVE_TESTS=1 cargo test
```

**Default Behavior**: Expensive tests **run by default** when `SKIP_EXPENSIVE_TESTS` is not set. This ensures AI assistants like GitHub Copilot always validate pre-commit requirements.

**When to skip**:

- ✅ Rapid development cycles where you're running tests frequently
- ✅ Working on isolated code that doesn't affect pre-commit tools
- ✅ CI environments that run pre-commit checks separately

**When NOT to skip**:

- ❌ Before creating a PR (let the full tests run at least once)
- ❌ When modifying anything that could affect linting, dependencies, or documentation
- ❌ When testing in a new environment or after dependency changes

### Why This Approach

This integration testing strategy helps with:

- **✅ Environment validation**: Catches missing tools or configuration issues early
- **✅ Copilot compatibility**: Ensures GitHub Copilot's environment has all necessary dependencies
- **✅ Fast feedback**: Developers see pre-commit issues during normal test cycles
- **✅ Flexible development**: Can be disabled when needed for faster iteration

### Running Tests

```bash
# Default: Run all tests including expensive pre-commit validation
cargo test

# Fast development: Skip expensive tests
SKIP_EXPENSIVE_TESTS=1 cargo test

# Explicitly run only AI precommit enforcement tests
cargo test ai_precommit_enforcement
```

This makes the test suite more readable, maintainable, and reliable for all contributors.

## 🤖 AI Assistant Integration

The project includes a dedicated test file `tests/ai_precommit_enforcement.rs` that ensures AI assistants (like GitHub Copilot) run all necessary pre-commit checks before committing code.

### Purpose

AI assistants often work in remote environments where they don't have access to local Git hooks or pre-commit scripts. These integration tests force the AI to validate all quality checks during the normal test execution.
