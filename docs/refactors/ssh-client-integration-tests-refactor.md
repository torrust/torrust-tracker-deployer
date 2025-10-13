# SSH Client Integration Tests Refactor

## 📋 Overview

This refactoring plan addresses code quality, maintainability, and testability issues in the SSH client integration tests. The current `tests/ssh_client_integration.rs` file (462 lines) contains significant code duplication, mixed concerns, and inconsistent patterns that make it difficult to maintain and extend.

**Target Files:**

- `tests/ssh_client_integration.rs`
- `src/testing/fixtures/` (new)
- `src/testing/helpers/` (new)

**Scope:**

- Extract common test setup patterns into reusable builders and fixtures
- Split large monolithic test file into focused modules
- Implement consistent AAA (Arrange-Act-Assert) pattern
- Add parameterized testing with rstest
- Reduce code duplication by 70% while maintaining full test coverage

## 📊 Progress Tracking

**Total Proposals**: 5
**Completed**: 1
**In Progress**: 0
**Not Started**: 4

### Phase Summary

- **Phase 1 - Foundation (Day 1)**: 📋 1/2 completed (50%)
- **Phase 2 - Structure (Day 2)**: 📋 0/2 completed (0%)
- **Phase 3 - Enhancement (Day 3)**: 📋 0/1 completed (0%)

## 🎯 Key Problems Identified

### 1. Code Duplication (Critical)

**SSH Setup Boilerplate**: Every test function contains nearly identical SSH credential and configuration setup:

```rust
// Repeated 6+ times across test functions
let ssh_credentials = SshCredentials::new(
    PathBuf::from("/nonexistent/key"),
    PathBuf::from("/nonexistent/key.pub"),
    Username::new("testuser").unwrap(),
);
let ssh_config = SshConfig::new(ssh_credentials, SocketAddr::new(host_ip, port));
let ssh_client = SshClient::new(ssh_config);
```

**Timeout Assertions**: Complex timeout logic duplicated with slight variations:

```rust
// Pattern repeated 4+ times
assert!(
    elapsed.as_secs() <= 10,
    "SSH timeout should complete within 10 seconds, took {elapsed:?}"
);
```

### 2. Single Responsibility Violation

**Mixed Concerns**: Single file handles multiple distinct responsibilities:

- Mock server connectivity testing
- Real Docker server testing
- Command execution testing
- Configuration validation testing
- Timeout/error handling testing

### 3. Test Structure Issues

**Inconsistent AAA Pattern**: Tests mix arrangement, action, and assertion without clear separation
**Manual Retry Logic**: Complex retry patterns duplicated across multiple tests
**Hard-coded Values**: Magic numbers (port 2222, timeouts 10s) scattered throughout

### 4. Limited Parameterization

**Similar Tests as Separate Functions**: Tests with different parameters written as individual functions instead of using parameterized testing (recommended in project's testing conventions)

## 🚀 Refactoring Phases

---

## Phase 1: Foundation (Day 1)

Extract reusable components following the project's DRY principle and testing conventions.

### Proposal #1: Extract Test Builders and Fixtures

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢🟢🟢 Very High  
**Effort**: 🔵🔵 Low  
**Priority**: P0 (Foundation for all other improvements)

#### Problem

Massive code duplication in SSH credential and configuration setup across all test functions. Every test manually creates the same SSH components with slight variations.

#### Solution

Create reusable test builders and fixtures in `src/testing/fixtures/ssh.rs`:

```rust
pub struct SshTestBuilder {
    username: String,
    host_ip: IpAddr,
    port: u16,
    use_real_keys: bool,
}

impl SshTestBuilder {
    pub fn new() -> Self {
        Self {
            username: TEST_USERNAME.to_string(),
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 22,
            use_real_keys: false,
        }
    }

    pub fn with_mock_container(container: &MockSshServerContainer) -> Self {
        Self::new()
            .with_host_ip(container.host_ip())
            .with_port(container.ssh_port())
            .with_username(&container.test_username())
    }

    pub fn with_real_container(container: &RealSshServerContainer) -> Self {
        Self::new()
            .with_host_ip(container.host_ip())
            .with_port(container.ssh_port())
            .with_username(&container.test_username())
            .with_real_keys()
    }

    pub fn with_unreachable_host() -> Self {
        Self::new()
            .with_host_ip(UNREACHABLE_IP.parse().unwrap())
    }

    pub fn build_client(self) -> SshClient {
        let credentials = self.build_credentials();
        let config = SshConfig::new(
            credentials,
            SocketAddr::new(self.host_ip, self.port),
        );
        SshClient::new(config)
    }
}

// Constants following testing conventions
pub const UNREACHABLE_IP: &str = "192.0.2.1"; // RFC 5737 TEST-NET-1
pub const TEST_USERNAME: &str = "testuser";
pub const REAL_SSH_PRIVATE_KEY: &str = "fixtures/testing_rsa";
pub const REAL_SSH_PUBLIC_KEY: &str = "fixtures/testing_rsa.pub";
```

#### Benefits

- Eliminates 80% of boilerplate code across all test functions
- Single place to modify SSH test setup
- Consistent test configuration following project conventions
- Fluent builder API improves test readability

#### Implementation Checklist

- [x] ~~Create `src/testing/fixtures/ssh.rs`~~ **Modified**: Added `SshTestBuilder` directly to `tests/ssh_client_integration.rs` to avoid premature abstraction
- [x] Implement `SshTestBuilder` with fluent API
- [x] Add constants for common test values
- [x] Add builder methods for each container type
- [x] ~~Update `src/testing/mod.rs` to export new fixtures~~ **Not needed**: Builder is local to test file
- [x] Verify tests still pass with existing functionality

---

### Proposal #2: Extract Timeout and Connectivity Helpers

**Status**: ✅ COMPLETED  
**Impact**: 🟢🟢🟢🟢 High  
**Effort**: 🔵🔵 Low  
**Priority**: P0 (Foundation for cleaner test assertions)

#### Problem

Complex timeout assertions and connectivity retry logic duplicated across multiple tests with inconsistent error messages and timeout values.

#### Solution

Create focused helper functions in `src/testing/helpers/ssh.rs`:

```rust
#[derive(Debug, Clone)]
pub struct ConnectivityTestResult {
    pub succeeded: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

/// Test connectivity with retry logic for CI environments
pub async fn test_connectivity_with_retry(
    client: &SshClient,
    max_attempts: u32,
    retry_delay: Duration
) -> ConnectivityTestResult {
    let start_time = Instant::now();
    let mut last_error = None;

    for attempt in 0..max_attempts {
        match client.test_connectivity() {
            Ok(true) => {
                return ConnectivityTestResult::new(true, start_time.elapsed(), None);
            }
            Ok(false) => {
                if attempt < max_attempts - 1 {
                    tokio::time::sleep(retry_delay).await;
                }
            }
            Err(e) => {
                last_error = Some(e.to_string());
                break;
            }
        }
    }

    ConnectivityTestResult::new(false, start_time.elapsed(), last_error)
}

/// Assert timeout duration is within expected range
pub fn assert_timeout_duration(duration: Duration, expected_range: Range<u64>) {
    assert!(
        duration.as_secs() >= expected_range.start && duration.as_secs() < expected_range.end,
        "Timeout should be in range {:?}s, was: {:?}", expected_range, duration
    );
}

/// Assert connectivity fails quickly (for unreachable hosts and mock servers)
pub async fn assert_connectivity_fails_quickly(client: &SshClient, max_seconds: u64) {
    let start_time = Instant::now();
    let result = client.test_connectivity();
    let duration = start_time.elapsed();

    assert!(
        result.is_err() || !result.unwrap(),
        "Expected connectivity to fail for unreachable/mock server"
    );

    assert_timeout_duration(duration, 1..max_seconds + 1);
}

/// Assert connectivity succeeds eventually (for real servers)
pub async fn assert_connectivity_succeeds_eventually(client: &SshClient, max_attempts: u32) {
    let result = test_connectivity_with_retry(client, max_attempts, Duration::from_secs(2)).await;

    assert!(
        result.succeeded,
        "Expected connectivity to succeed within {} attempts, failed after {:?}: {:?}",
        max_attempts, result.duration, result.error
    );
}
```

#### Benefits

- Eliminates duplicate retry and timeout logic
- Clearer test intent without implementation details
- Easier to modify timeout behavior across all tests
- Consistent error handling patterns

#### Implementation Checklist

- [ ] Create `src/testing/helpers/ssh.rs`
- [ ] Implement `ConnectivityTestResult` struct
- [ ] Add `test_connectivity_with_retry` function
- [ ] Add timeout assertion helpers
- [ ] Add convenience assertion functions
- [ ] Update `src/testing/mod.rs` to export new helpers
- [ ] Verify helpers work with existing test patterns

---

## Phase 2: Structure (Day 2)

Improve test organization and consistency following project conventions.

### Proposal #3: Apply AAA Pattern Consistently

**Status**: 📋 Not Started  
**Impact**: 🟢🟢🟢 Medium  
**Effort**: 🔵🔵 Low  
**Priority**: P1 (Required by project testing conventions)

#### Problem

Tests mix arrangement, action, and assertion without clear structure, violating the project's testing conventions that require consistent AAA (Arrange-Act-Assert) pattern.

#### Solution

Apply AAA pattern consistently with clear commenting and separation:

**Before (Mixed Structure)**:

```rust
#[tokio::test]
async fn it_should_establish_ssh_connectivity_with_mock_server() {
    let ssh_container = match MockSshServerContainer::start() {
        Ok(container) => container,
        Err(e) => {
            println!("Skipping SSH integration test - Docker not available: {e}");
            return;
        }
    };
    let ssh_credentials = SshCredentials::new(
        PathBuf::from("/nonexistent/key"),
        PathBuf::from("/nonexistent/key.pub"),
        Username::new(ssh_container.test_username()).unwrap(),
    );
    let ssh_config = SshConfig::new(
        ssh_credentials,
        SocketAddr::new(ssh_container.host_ip(), ssh_container.ssh_port()),
    );
    let ssh_client = SshClient::new(ssh_config);
    let start_time = std::time::Instant::now();
    let connectivity_result = ssh_client.test_connectivity();
    let elapsed = start_time.elapsed();
    // Complex assertion logic mixed with other code...
}
```

**After (Clear AAA Structure)**:

```rust
#[tokio::test]
async fn it_should_establish_ssh_connectivity_with_mock_server() {
    // Arrange: Set up mock container and SSH client
    let container = MockSshServerContainer::start()
        .expect("Mock container should always start");
    let client = SshTestBuilder::new()
        .with_mock_container(&container)
        .build_client();

    // Act: Test connectivity and measure duration
    let start_time = Instant::now();
    let result = client.test_connectivity();
    let duration = start_time.elapsed();

    // Assert: Verify expected failure and quick timeout
    assert!(
        result.is_err() || !result.unwrap(),
        "Mock server should not establish real SSH connectivity"
    );
    assert_timeout_duration(duration, 1..10);
}
```

#### Benefits

- Improved test readability following project conventions
- Easier to understand test purpose and identify failures
- Consistent pattern across all tests
- Clear separation of concerns within each test

#### Implementation Checklist

- [ ] Review all test functions in `ssh_client_integration.rs`
- [ ] Add clear AAA comments to each test
- [ ] Separate arrangement, action, and assertion sections
- [ ] Use extracted builders and helpers from Phase 1
- [ ] Verify all tests maintain their original behavior
- [ ] Update any complex tests to follow AAA structure

---

### Proposal #4: Split Into Focused Test Modules

**Status**: 📋 Not Started  
**Impact**: 🟢🟢🟢🟢 High  
**Effort**: 🔵🔵🔵 Medium  
**Priority**: P1 (Addresses single responsibility violation)

#### Problem

Single large file (462 lines) mixing multiple concerns violates the single responsibility principle and project's module organization conventions.

#### Solution

Split into focused test modules following project's module organization principles:

```text
tests/
├── ssh_client/
│   ├── mod.rs                    // Common setup, re-exports, shared fixtures
│   ├── connectivity_tests.rs     // Pure connectivity testing (mock/real/timeout)
│   ├── command_execution_tests.rs // Remote command execution tests
│   └── configuration_tests.rs    // Configuration validation tests
└── ssh_client_integration.rs     // Backwards compatibility (re-exports modules)
```

**Module Structure**:

```rust
// tests/ssh_client/mod.rs
pub mod connectivity_tests;
pub mod command_execution_tests;
pub mod configuration_tests;

// Re-export common testing utilities
pub use crate::testing::fixtures::ssh::{SshTestBuilder, UNREACHABLE_IP, TEST_USERNAME};
pub use crate::testing::helpers::ssh::{
    test_connectivity_with_retry, assert_timeout_duration,
    assert_connectivity_fails_quickly, assert_connectivity_succeeds_eventually
};

// tests/ssh_client_integration.rs (backwards compatibility)
mod ssh_client {
    pub mod connectivity_tests;
    pub mod command_execution_tests;
    pub mod configuration_tests;
}
```

#### Benefits

- Single responsibility per test file (aligns with project conventions)
- Easier to locate specific test types
- Faster test execution (can run specific test categories)
- Better parallel execution
- Follows project's module organization principles

#### Implementation Checklist

- [ ] Create `tests/ssh_client/` directory
- [ ] Create `tests/ssh_client/mod.rs` with common exports
- [ ] Move connectivity tests to `connectivity_tests.rs`
- [ ] Move command execution tests to `command_execution_tests.rs`
- [ ] Move configuration tests to `configuration_tests.rs`
- [ ] Update `ssh_client_integration.rs` for backwards compatibility
- [ ] Verify all tests can be run individually and together
- [ ] Update any test discovery patterns if needed

---

## Phase 3: Enhancement (Day 3)

Add advanced testing patterns following project conventions.

### Proposal #5: Implement Parameterized Tests with rstest

**Status**: 📋 Not Started  
**Impact**: 🟢🟢🟢 Medium  
**Effort**: 🔵🔵🔵 Medium  
**Priority**: P2 (Enhancement following project testing conventions)

#### Problem

Similar tests with different parameters written as separate functions, violating the DRY principle. The project's testing conventions recommend using `rstest` for parameterized testing.

#### Solution

Use `rstest` crate for parameterized testing following project conventions:

**Before (Separate Functions)**:

```rust
#[tokio::test]
async fn it_should_handle_connectivity_timeouts() {
    // Test with unreachable IP...
}

#[tokio::test]
async fn it_should_establish_ssh_connectivity_with_mock_server() {
    // Test with mock server...
}

#[tokio::test]
async fn it_should_connect_to_real_ssh_server_and_test_connectivity() {
    // Test with real server...
}
```

**After (Parameterized)**:

```rust
use rstest::rstest;

#[derive(Debug, Clone)]
enum ServerType {
    Mock,
    Unreachable,
    Real,
}

#[rstest]
#[case::mock_server(ServerType::Mock, false, 1..10)] // Should fail quickly
#[case::unreachable_host(ServerType::Unreachable, false, 4..10)] // Should timeout ~5s
#[tokio::test]
async fn it_should_handle_connectivity_scenarios(
    #[case] server_type: ServerType,
    #[case] should_succeed: bool,
    #[case] timeout_range: Range<u64>,
) {
    // Arrange
    let client = match server_type {
        ServerType::Mock => {
            let container = MockSshServerContainer::start()
                .expect("Mock container should always start");
            SshTestBuilder::new().with_mock_container(&container).build_client()
        },
        ServerType::Unreachable => {
            SshTestBuilder::new().with_unreachable_host().build_client()
        },
        ServerType::Real => {
            let container = match RealSshServerContainer::start().await {
                Ok(c) => c,
                Err(_) => return, // Skip if Docker not available
            };
            SshTestBuilder::new().with_real_container(&container).build_client()
        }
    };

    // Act & Assert
    if should_succeed {
        assert_connectivity_succeeds_eventually(&client, 10).await;
    } else {
        assert_connectivity_fails_quickly(&client, timeout_range.end).await;
    }
}

#[rstest]
#[case::echo_command("echo 'hello'", "hello")]
#[case::whoami_command("whoami", "testuser")]
#[case::pwd_command("pwd", "/home/testuser")]
#[tokio::test]
async fn it_should_execute_commands_correctly(
    #[case] command: &str,
    #[case] expected_output: &str,
) {
    // Arrange
    let container = match RealSshServerContainer::start().await {
        Ok(c) => c,
        Err(_) => return, // Skip if Docker not available
    };
    let client = SshTestBuilder::new().with_real_container(&container).build_client();

    // Act
    client.wait_for_connectivity().await.expect("Should connect");
    let output = client.execute(command).expect("Command should execute");

    // Assert
    assert_eq!(output.trim(), expected_output);
}
```

#### Benefits

- Reduces code duplication for similar test scenarios
- Better test coverage with less code
- Clearer identification of failing test cases in CI output
- Follows project's preference for parameterized tests over loops

#### Implementation Checklist

- [ ] Add `rstest = "0.23"` to `Cargo.toml` dev-dependencies
- [ ] Identify similar test functions that can be parameterized
- [ ] Create `ServerType` enum for connectivity tests
- [ ] Convert connectivity tests to parameterized format
- [ ] Convert command execution tests to parameterized format
- [ ] Verify parameterized tests provide same coverage
- [ ] Update test documentation to reflect new structure

---

## 🎯 Expected Outcomes

After implementing this refactoring plan:

- **Maintainability**: 70% reduction in code duplication
- **Readability**: Clear separation of concerns and consistent AAA pattern
- **Testability**: Better test isolation and faster execution
- **Sustainability**: Easier to add new tests following established patterns
- **Developer Experience**: More intuitive test writing following project conventions

## 📊 Success Metrics

- [ ] Reduce file size by 50% through extraction of common patterns (from 462 lines)
- [ ] Achieve 100% consistent AAA pattern across all tests
- [ ] Enable parameterized testing for timeout/connectivity scenarios
- [ ] Zero code duplication in SSH setup and configuration
- [ ] Maintain all existing test coverage while improving structure
- [ ] Pass all existing linting and testing requirements

## ⏰ Implementation Timeline

**Total Estimated Time**: 3 days

- **Day 1 (Phase 1)**: 6-8 hours

  - Extract test builders and fixtures (3-4 hours)
  - Extract timeout and connectivity helpers (3-4 hours)

- **Day 2 (Phase 2)**: 6-8 hours

  - Apply AAA pattern consistently (2-3 hours)
  - Split into focused test modules (4-5 hours)

- **Day 3 (Phase 3)**: 4-6 hours
  - Implement parameterized tests with rstest (4-6 hours)

## 🔍 Review Process

### Approval Criteria

- [ ] All proposals align with project's development principles
- [ ] Timeline is realistic based on team capacity
- [ ] Implementation order prioritizes high-impact, low-effort improvements
- [ ] Plan follows project's testing conventions
- [ ] Success metrics are measurable and achievable

### Implementation Validation

- [ ] All existing tests continue to pass
- [ ] Code coverage is maintained or improved
- [ ] Linting passes without errors
- [ ] No performance regressions in test execution
- [ ] New test patterns are documented and follow project conventions

## 🔗 Related Documentation

- [Testing Conventions](../contributing/testing.md) - Project testing standards and AAA pattern requirements
- [Development Principles](../development-principles.md) - Core principles including DRY and maintainability
- [Module Organization](../contributing/module-organization.md) - Guidelines for organizing code and test modules
