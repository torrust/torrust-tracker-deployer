---
name: write-unit-test
description: Guide for writing unit tests following project conventions including behavior-driven naming (it_should_*), AAA pattern, MockClock for time testing, TempDir for isolation, and parameterized tests with rstest. Use when adding tests for domain entities, value objects, utilities, or command logic. Triggers on "write unit test", "add test", "test coverage", "unit testing", or "add unit tests".
metadata:
  author: torrust
  version: "1.0"
---

# Writing Unit Tests

This skill guides you through writing unit tests that follow project conventions and quality standards.

## Why This Matters

**Unit tests are first-class citizens** - They should be as clean, maintainable, and well-structured as production code.

**Key Principles**:

- ✅ **Behavior-driven naming** - Test names document what the code does
- ✅ **AAA Pattern** - Clear structure: Arrange → Act → Assert
- ✅ **Deterministic** - Same input always produces same output
- ✅ **Isolated** - Tests don't depend on each other or external state
- ✅ **Fast** - Unit tests run in milliseconds

## Quick Decision Tree

```text
What are you testing?
├── Domain entity/value object?
│   └── → Phase 1: Simple unit test with naming conventions
├── New test with repeated setup code?
│   └── → Phase 2: Check existing tests for duplicate patterns
├── Duplicate setup code across tests?
│   └── → Phase 3: Extract helper functions and avoid coupling
├── Time-dependent code?
│   └── → Phase 4: Use MockClock for deterministic time
├── File operations?
│   └── → Phase 5: Use TempDir for isolation
├── Multiple input/output combinations?
│   └── → Phase 6: Use parameterized tests (rstest)
└── Command or handler?
    └── → See write-integration-test skill instead
```

## Phase 1: Basic Unit Test with Proper Naming

**Goal**: Write a simple, well-named test following AAA pattern.

### Step 1: Identify What You're Testing

**Questions to answer**:

- What behavior am I validating?
- When does this behavior happen? (condition/scenario)
- What is the expected outcome?

**Example**:

```rust
// Testing: EnvironmentName validation
// Behavior: Reject names with uppercase letters
// Condition: When name contains uppercase
// Outcome: Return ValidationError
```

### Step 2: Write Test with Behavior-Driven Naming

**Format**: `it_should_{expected_behavior}_when_{condition}`

**Pattern Rules**:

- ✅ **ALWAYS** use `it_should_` prefix
- ❌ **NEVER** use `test_` prefix
- ✅ Use `when_` or `given_` for conditions
- ✅ Be specific and descriptive

**Example**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_error_when_name_contains_uppercase_letters() {
        // Arrange: Set up test data
        let invalid_name = "MyEnvironment".to_string();

        // Act: Execute the behavior
        let result = EnvironmentName::new(invalid_name);

        // Assert: Verify the outcome
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidCharacters));
    }

    #[test]
    fn it_should_create_valid_name_when_using_lowercase_and_hyphens() {
        // Arrange
        let valid_name = "my-environment".to_string();

        // Act
        let result = EnvironmentName::new(valid_name.clone());

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "my-environment");
    }
}
```

### Step 3: Apply AAA Pattern

**Structure every test with three clear sections**:

```rust
#[test]
fn it_should_{behavior}_when_{condition}() {
    // Arrange: Set up test data and preconditions
    let input = setup_test_data();
    let expected = calculate_expected_result();

    // Act: Execute the behavior being tested
    let actual = function_under_test(input);

    // Assert: Verify the expected outcome
    assert_eq!(actual, expected);
}
```

**Benefits**:

- **Clarity** - Each section has a clear purpose
- **Debugging** - Easy to identify which phase is failing
- **Maintenance** - Simple to modify specific parts

**Test Phase 1**:

```bash
# Run specific test
cargo test it_should_return_error_when_name_contains_uppercase

# Run all tests in module
cargo test environment_name::tests

# Run with output
cargo test -- --nocapture
```

**Expected Results**:

```text
test domain::environment_name::tests::it_should_return_error_when_name_contains_uppercase_letters ... ok
test domain::environment_name::tests::it_should_create_valid_name_when_using_lowercase_and_hyphens ... ok
```

**Commit**: `test: add unit tests for EnvironmentName validation`

### Step 1: Inject Clock Dependency

**Production code**:

```rust
use crate::shared::clock::Clock;
use std::sync::Arc;

pub struct EventRecorder {
    clock: Arc<dyn Clock>,
}

impl EventRecorder {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self { clock }
    }

    pub fn record_event(&self, name: &str) -> Event {
        Event {
            name: name.to_string(),
            timestamp: self.clock.now(), // Injectable time
        }
    }
}
```

### Step 2: Use MockClock in Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockClock;
    use chrono::{TimeZone, Utc};
    use std::sync::Arc;

    #[test]
    fn it_should_record_event_with_fixed_timestamp_when_clock_is_mocked() {
        // Arrange: Set up mock clock with fixed time
        let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = Arc::new(MockClock::new(fixed_time));
        let recorder = EventRecorder::new(clock);

        // Act: Record event
        let event = recorder.record_event("test-event");

        // Assert: Verify exact timestamp
        assert_eq!(event.name, "test-event");
        assert_eq!(event.timestamp, fixed_time);
    }

    #[test]
    fn it_should_track_time_progression_when_clock_advances() {
        // Arrange: Set up mock clock
        let start_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = Arc::new(MockClock::new(start_time));
        let recorder = EventRecorder::new(clock.clone());

        // Act: Record first event
        let event1 = recorder.record_event("first");

        // Simulate 5 minutes passing
        clock.advance_secs(300);

        // Record second event
        let event2 = recorder.record_event("second");

        // Assert: Verify time difference
        let expected_second_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 5, 0).unwrap();
        assert_eq!(event1.timestamp, start_time);
        assert_eq!(event2.timestamp, expected_second_time);
    }
}
```

**Key MockClock Methods**:

- `MockClock::new(timestamp)` - Create clock with fixed time
- `clock.advance_secs(seconds)` - Move time forward
- `clock.now()` - Get current time

**Benefits**:

- ✅ Deterministic tests - same result every time
- ✅ Fast execution - no actual time delays
- ✅ Edge case testing - easily test timeouts, expirations

**Documentation**: [docs/contributing/testing/unit-testing/mock-clock.md](../../../docs/contributing/testing/unit-testing/mock-clock.md)

**Commit**: `test: add time-dependent tests using MockClock`

## Phase 3: Avoiding Duplicate Test Code

**When to use**: When you notice repeated setup code across multiple tests.

**Why**: DRY principle applies to tests - duplicate code makes tests harder to maintain and increases the risk of inconsistencies.

### Step 1: Identify Duplicate Code Patterns

**Watch for these code smells**:

- ❌ **Repeated Arrange sections** - Same setup code copy-pasted across tests
- ❌ **Coupled helpers** - Helper functions that internally call other helpers with hardcoded values
- ❌ **Magic values** - Hardcoded test data scattered throughout tests
- ❌ **Complex setup** - More than 5-10 lines of boilerplate in Arrange section

**Example of duplicate code** (BAD):

```rust
#[test]
fn it_should_convert_environment_to_dto() {
    // Arrange - DUPLICATED SETUP
    let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
    let ssh_username = Username::new("deployer".to_string()).unwrap();
    let ssh_credentials = SshCredentials::new(
        PathBuf::from("./keys/test_rsa"),
        PathBuf::from("./keys/test_rsa.pub"),
        ssh_username,
    );
    let provider_config = ProviderConfig::Lxd(LxdConfig {
        profile_name: ProfileName::new("lxd-test".to_string()).unwrap(),
    });
    let created_at = Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap();
    let env = Environment::new(env_name, provider_config, ssh_credentials, 22, created_at);
    // ... test logic
}

#[test]
fn it_should_handle_missing_ip() {
    // Arrange - SAME DUPLICATED SETUP AGAIN
    let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
    let ssh_username = Username::new("deployer".to_string()).unwrap();
    let ssh_credentials = SshCredentials::new(
        PathBuf::from("./keys/test_rsa"),
        PathBuf::from("./keys/test_rsa.pub"),
        ssh_username,
    );
    // ... copies continue
}
```

### Step 2: Extract Helper Functions

**Rule of thumb**: If you copy code 2+ times, extract it.

**Pattern**: Create focused helper functions for different aspects of setup:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures and helpers section

    fn create_test_ssh_credentials() -> SshCredentials {
        let ssh_username = Username::new("deployer".to_string()).unwrap();
        SshCredentials::new(
            PathBuf::from("./keys/test_rsa"),
            PathBuf::from("./keys/test_rsa.pub"),
            ssh_username,
        )
    }

    fn create_test_provider_config() -> ProviderConfig {
        ProviderConfig::Lxd(LxdConfig {
            profile_name: ProfileName::new("lxd-test-env".to_string()).unwrap(),
        })
    }

    fn create_test_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap()
    }

    fn create_test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 140, 190, 39))
    }

    // Higher-level builder that composes the parts
    fn create_configured_environment_with_ip(ip: IpAddr) -> Environment<Configured> {
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_credentials = create_test_ssh_credentials();
        let provider_config = create_test_provider_config();
        let created_at = create_test_timestamp();

        Environment::new(env_name, provider_config, ssh_credentials, 22, created_at)
            .start_provisioning()
            .provisioned(ip, ProvisionMethod::Provisioned)
            .start_configuring()
            .configured()
    }

    // Tests section

    #[test]
    fn it_should_convert_configured_environment_to_dto() {
        // Arrange - Now concise and clear!
        let test_ip = create_test_ip();
        let env = create_configured_environment_with_ip(test_ip);

        // Act
        let dto = ConfigureDetailsData::from(&env);

        // Assert
        assert_eq!(dto.instance_ip, Some(test_ip));
    }
}
```

### Step 3: Avoid Coupling Between Helpers

**Problem**: Helpers that call each other with hardcoded values create hidden dependencies.

**Anti-pattern - Coupled helpers** (BAD):

```rust
// ❌ BAD: create_expected_dto() internally calls create_test_ip()
fn create_expected_dto() -> ConfigureDetailsData {
    ConfigureDetailsData {
        instance_ip: Some(create_test_ip()),  // Hardcoded dependency
        // ...
    }
}

#[test]
fn test_conversion() {
    let env = create_configured_environment_with_ip(create_test_ip());
    let expected = create_expected_dto();  // Uses different IP internally!
    assert_eq!(ConfigureDetailsData::from(&env), expected);
}
```

**Fixed - Decoupled helpers** (GOOD):

```rust
// ✅ GOOD: Accept parameter to stay flexible and explicit
fn create_expected_dto(ip: IpAddr) -> ConfigureDetailsData {
    ConfigureDetailsData {
        instance_ip: Some(ip),  // Use provided IP
        // ...
    }
}

#[test]
fn test_conversion() {
    // Arrange: Single source of truth
    let test_ip = create_test_ip();
    let env = create_configured_environment_with_ip(test_ip);
    let expected = create_expected_dto(test_ip);  // Same IP, explicit

    // Act & Assert
    assert_eq!(ConfigureDetailsData::from(&env), expected);
}
```

**Benefits**:

- ✅ **No hidden dependencies** - All inputs are explicit
- ✅ **Single source of truth** - Test data defined once
- ✅ **Easy to vary** - Can test different IPs without changing helpers
- ✅ **Clear intent** - Obvious that both use the same value

### Step 4: Use Derives to Simplify Assertions

**Add `PartialEq` to DTOs** to enable single-line assertions:

```rust
// In production code
#[derive(Debug, Clone, PartialEq, Serialize)]  // Add PartialEq
pub struct ConfigureDetailsData {
    pub environment_name: String,
    pub instance_name: String,
    // ...
}
```

**Before** - Field-by-field assertions:

```rust
// ❌ Verbose: 6 separate assertions
assert_eq!(dto.environment_name, "test-env");
assert_eq!(dto.instance_name, "torrust-tracker-vm-test-env");
assert_eq!(dto.provider, "lxd");
assert_eq!(dto.state, "Configured");
assert_eq!(dto.instance_ip, Some(test_ip));
assert_eq!(dto.created_at, expected_created_at);
```

**After** - Single assertion:

```rust
// ✅ Concise: One assertion, better error messages
let expected = create_expected_dto(test_ip);
assert_eq!(dto, expected);
```

### Step 5: Organize Test Modules

**Pattern**: Separate helpers from tests with clear sections:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // Imports...

    // ========================================
    // Test fixtures and helpers
    // ========================================

    fn create_test_ssh_credentials() -> SshCredentials { /* ... */ }
    fn create_test_provider_config() -> ProviderConfig { /* ... */ }
    fn create_configured_environment_with_ip(ip: IpAddr) -> Environment<Configured> { /* ... */ }

    // ========================================
    // Tests
    // ========================================

    #[test]
    fn it_should_convert_configured_environment_to_dto() { /* ... */ }

    #[test]
    fn it_should_handle_none_instance_ip() { /* ... */ }
}
```

**Benefits**:

- ✅ **Clear separation** - Helpers vs actual tests
- ✅ **Easy navigation** - Find what you need quickly
- ✅ **Reusability** - Helper functions available to all tests in module

**See**: PR [#373](https://github.com/torrust/torrust-tracker-deployer/pull/373) for complete example of refactoring duplicate test code.

**Commit**: `refactor: extract duplicate test code and decouple test setup`

## Phase 4: Time-Dependent Tests with MockClock

**When to use**: Testing code that uses `Utc::now()` or time-based logic.

**Why**: Direct use of `Utc::now()` makes tests non-deterministic.

**When to use**: Testing code that creates files, directories, or modifies filesystem.

**Why**: Tests should never interfere with production data or leave artifacts.

### Step 1: Use TempDir for Isolation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_environment_directories_when_initializing() {
        // Arrange: Create temporary directory (auto-cleaned)
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();

        // Act: Create environment in temp directory
        let environment = Environment::new_in_dir(env_name, temp_path);

        // Assert: Verify directories exist in temp location
        assert!(environment.data_dir().exists());
        assert!(environment.build_dir().exists());
        assert!(environment.data_dir().starts_with(temp_path));

        // TempDir automatically cleans up when dropped
    }

    #[test]
    fn it_should_write_config_file_when_saving_environment() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let environment = create_test_environment();

        // Act
        environment.save_to_file(&config_path).unwrap();

        // Assert
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-env"));

        // TempDir cleanup happens automatically
    }
}
```

**Anti-Pattern - DON'T DO THIS**:

```rust
#[test]
fn bad_test_creates_real_directories() {
    // ❌ BAD: Creates ./data/test and ./build/test
    let env = Environment::new("test".to_string());
    // These directories persist after test!
}
```

**Benefits**:

- ✅ Test isolation - no interference between tests
- ✅ No cleanup code needed - TempDir handles it
- ✅ No pollution of working directory

**Documentation**: [docs/contributing/testing/unit-testing/temp-directories.md](../../../docs/contributing/testing/unit-testing/temp-directories.md)

**Commit**: `test: add filesystem tests using TempDir`

## Phase 6: Parameterized Tests with rstest

**When to use**: Testing same behavior with multiple input/output combinations.

**Why**: Better isolation, clearer output, easier debugging than loops in test body.

### Step 1: Add rstest Dependency

```toml
[dev-dependencies]
rstest = "0.23"
```

### Step 2: Write Parameterized Test

**Before (Loop in Test Body - DON'T DO THIS)**:

```rust
#[test]
fn test_various_inputs() {
    // ❌ BAD: If one case fails, you don't know which one
    let cases = vec![
        ("valid-name", true),
        ("invalid_name", false),
        ("UPPERCASE", false),
    ];

    for (input, should_succeed) in cases {
        let result = validate(input);
        assert_eq!(result.is_ok(), should_succeed);
    }
}
```

**After (Parameterized with rstest - GOOD)**:

```rust
use rstest::rstest;

#[rstest]
#[case("valid-name", true)]
#[case("invalid_name", false)]
#[case("UPPERCASE", false)]
#[case("my-env-123", true)]
fn it_should_validate_environment_name_format(
    #[case] input: &str,
    #[case] expected_valid: bool,
) {
    // Arrange
    let name_str = input.to_string();

    // Act
    let result = EnvironmentName::new(name_str);

    // Assert
    assert_eq!(result.is_ok(), expected_valid);
}
```

**Test Output**:

```text
test it_should_validate_environment_name_format::case_1 ... ok
test it_should_validate_environment_name_format::case_2 ... ok
test it_should_validate_environment_name_format::case_3 ... ok
test it_should_validate_environment_name_format::case_4 ... ok
```

### Step 3: Advanced Parameterization

**Testing multiple conditions**:

```rust
#[rstest]
#[case("e2e-config", "data/e2e-config", "build/e2e-config")]
#[case("production", "data/production", "build/production")]
#[case("dev-test", "data/dev-test", "build/dev-test")]
fn it_should_create_correct_paths_for_different_environments(
    #[case] env_name: &str,
    #[case] expected_data_path: &str,
    #[case] expected_build_path: &str,
) {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let name = EnvironmentName::new(env_name.to_string()).unwrap();

    // Act
    let environment = Environment::new_in_dir(name, temp_dir.path());

    // Assert
    assert!(environment.data_dir().to_str().unwrap().ends_with(expected_data_path));
    assert!(environment.build_dir().to_str().unwrap().ends_with(expected_build_path));
}
```

**When to use**:

- ✅ Testing edge cases with different values
- ✅ Validating configuration variations
- ✅ Verifying data transformation with various inputs
- ✅ Testing boundary conditions

**When NOT to use**:

- ❌ Each case tests fundamentally different behavior (use separate tests)
- ❌ Test logic differs significantly between cases
- ❌ Only one or two cases (just write separate tests)

**Documentation**: [docs/contributing/testing/unit-testing/parameterized-tests.md](../../../docs/contributing/testing/unit-testing/parameterized-tests.md)

**Commit**: `test: add parameterized tests for input validation`

## Phase 7: Verify and Fix

### Step 1: Run Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test environment_name_tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test it_should_validate_environment_name
```

### Step 2: Check Coverage (Optional)

```bash
# Quick coverage check
cargo cov-check

# Generate HTML coverage report
cargo cov-html
# Open target/llvm-cov/html/index.html
```

### Step 3: Verify Test Quality

**Checklist**:

- [ ] Test name follows `it_should_*_when_*` pattern (NO `test_` prefix)
- [ ] AAA pattern is clearly marked with comments
- [ ] Test uses MockClock for time dependencies
- [ ] Test uses TempDir for file operations
- [ ] Parameterized tests use rstest (not loops)
- [ ] Test output is clean (no emoji/progress messages)
- [ ] Test is isolated (doesn't depend on other tests)
- [ ] Test is deterministic (same result every time)

### Step 4: Common Issues and Fixes

#### Issue: Test Output Shows User Messages

**Problem**:

```text
test output:
⏳ Processing...
✅ Complete!
```

**Fix**: Use silent verbosity in test setup

```rust
let context = TestContext::new(); // Uses VerbosityLevel::Silent by default
let output = TestUserOutput::wrapped_silent();
```

**Documentation**: [docs/contributing/testing/quality/clean-output.md](../../../docs/contributing/testing/quality/clean-output.md)

#### Issue: Flaky Time-Dependent Tests

**Problem**: Tests pass sometimes, fail other times

**Fix**: Replace `Utc::now()` with `MockClock`

```rust
// ❌ Before: Non-deterministic
let timestamp = Utc::now();

// ✅ After: Deterministic
let clock = Arc::new(MockClock::new(fixed_time));
let timestamp = clock.now();
```

#### Issue: Tests Leave Artifacts

**Problem**: Directories like `./data/test` persist after tests

**Fix**: Use `TempDir` instead of real directories

```rust
// ❌ Before: Pollutes workspace
let env = Environment::new("test".to_string());

// ✅ After: Isolated and auto-cleaned
let temp_dir = TempDir::new().unwrap();
let env = Environment::new_in_dir("test".to_string(), temp_dir.path());
```

#### Issue: Can't Identify Which Parameterized Case Failed

**Problem**: Test output only shows test name, not which `#[case]` failed

**Fix**: Each case runs as a separate test with rstest

```rust
// ✅ Good: Clear case identification
#[rstest]
#[case("input1", expected1)]
#[case("input2", expected2)]
fn test_name(#[case] input: Type, #[case] expected: Type) {
    // Each case runs independently
}
```

**Commit**: `test: fix test isolation and determinism issues`

## Examples from Codebase

### Example 1: Simple Domain Entity Test

**File**: `src/domain/environment_name/tests.rs`

```rust
#[test]
fn it_should_create_valid_name_when_using_lowercase_with_hyphens() {
    // Arrange
    let valid_name = "my-environment".to_string();

    // Act
    let result = EnvironmentName::new(valid_name);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn it_should_return_error_when_name_contains_uppercase() {
    // Arrange
    let invalid_name = "MyEnvironment".to_string();

    // Act
    let result = EnvironmentName::new(invalid_name);

    // Assert
    assert!(matches!(result.unwrap_err(), ValidationError::InvalidCharacters));
}
```

### Example 2: Test with MockClock

**File**: `src/shared/clock/tests.rs`

```rust
#[test]
fn it_should_return_fixed_time_when_mock_clock_is_set() {
    // Arrange
    let fixed_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let clock = MockClock::new(fixed_time);

    // Act
    let current_time = clock.now();

    // Assert
    assert_eq!(current_time, fixed_time);
}
```

### Example 3: Test with TempDir

**File**: `src/domain/environment/tests.rs`

```rust
#[test]
fn it_should_create_data_directory_when_environment_is_initialized() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let name = EnvironmentName::new("test".to_string()).unwrap();

    // Act
    let env = Environment::new_in_dir(name, temp_dir.path());

    // Assert
    assert!(env.data_dir().exists());
    assert!(env.data_dir().starts_with(temp_dir.path()));
}
```

### Example 4: Parameterized Test

**File**: `src/domain/ip_address/tests.rs`

```rust
#[rstest]
#[case("192.168.1.1", true)]
#[case("invalid.ip", false)]
#[case("256.1.1.1", false)]
#[case("::1", true)]
fn it_should_validate_ip_address_format(
    #[case] input: &str,
    #[case] expected_valid: bool,
) {
    // Arrange & Act
    let result = IpAddress::from_str(input);

    // Assert
    assert_eq!(result.is_ok(), expected_valid);
}
```

## Tips & Best Practices

### Naming Tests

- **DO**: `it_should_return_error_when_input_is_invalid`
- **DON'T**: `test_validation`, `test_error_case`, `invalid_input_test`

### Test Organization

- Group related tests in the same `#[cfg(test)] mod tests` block
- Order tests: success cases first, then error cases
- Use descriptive module names: `domain::environment_name::tests`

### AAA Pattern

- **Always** add comments marking Arrange/Act/Assert sections
- Keep each section focused and minimal
- If Arrange is complex, extract to helper function

### MockClock

- Use for **any** code that uses time
- Don't mix real `Utc::now()` with `MockClock`
- Test time progression with `advance_secs()`

### TempDir

- Use for **all** filesystem operations
- Never use hardcoded paths like `./data/test`
- TempDir cleans up automatically - no manual cleanup needed

### Parameterized Tests

- Use rstest when testing 3+ similar cases
- Each `#[case]` should test the **same behavior** with different inputs
- If behavior differs, write separate tests

### Test Independence

- Tests should not depend on execution order
- Each test should set up its own data
- Use TempDir to avoid shared state

## Quick Reference

**Test Naming Pattern**:

```rust
it_should_{expected_behavior}_when_{condition}
it_should_{expected_behavior}_given_{state}
```

**AAA Structure**:

```rust
// Arrange: Setup
// Act: Execute
// Assert: Verify
```

**Import MockClock**:

```rust
use crate::testing::MockClock;
```

**Import TempDir**:

```rust
use tempfile::TempDir;
```

**Import rstest**:

```rust
use rstest::rstest;
```

## Related Documentation

- **Unit Testing Overview**: [docs/contributing/testing/unit-testing/README.md](../../../docs/contributing/testing/unit-testing/README.md)
- **Naming Conventions**: [docs/contributing/testing/unit-testing/naming-conventions.md](../../../docs/contributing/testing/unit-testing/naming-conventions.md)
- **AAA Pattern**: [docs/contributing/testing/unit-testing/aaa-pattern.md](../../../docs/contributing/testing/unit-testing/aaa-pattern.md)
- **MockClock Guide**: [docs/contributing/testing/unit-testing/mock-clock.md](../../../docs/contributing/testing/unit-testing/mock-clock.md)
- **TempDir Guide**: [docs/contributing/testing/unit-testing/temp-directories.md](../../../docs/contributing/testing/unit-testing/temp-directories.md)
- **Parameterized Tests**: [docs/contributing/testing/unit-testing/parameterized-tests.md](../../../docs/contributing/testing/unit-testing/parameterized-tests.md)
- **Test Output Standards**: [docs/contributing/testing/quality/clean-output.md](../../../docs/contributing/testing/quality/clean-output.md)
- **Coverage Guide**: [docs/contributing/testing/quality/coverage.md](../../../docs/contributing/testing/quality/coverage.md)
- **Testing Principles**: [docs/contributing/testing/principles.md](../../../docs/contributing/testing/principles.md)
- **Quick Reference**: [docs/contributing/testing/reference/quick-reference.md](../../../docs/contributing/testing/reference/quick-reference.md)
- **Troubleshooting**: [docs/contributing/testing/reference/troubleshooting.md](../../../docs/contributing/testing/reference/troubleshooting.md)

## Next Steps After Writing Tests

1. **Run pre-commit checks**: `./scripts/pre-commit.sh`
2. **Check coverage**: `cargo cov-check` (optional)
3. **Commit changes**: `test: add unit tests for [component]`
4. **Consider integration tests**: If testing commands, see `write-integration-test` skill
