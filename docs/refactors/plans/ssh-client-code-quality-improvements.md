# SSH Client Code Quality Improvements

## 📋 Overview

This refactoring plan addresses code quality, maintainability, readability, testability, and configurability issues in the SSH client implementation at `src/shared/ssh/client.rs`. The current implementation has several hardcoded values that limit flexibility for testing and production tuning, along with opportunities to improve test quality and code organization.

**Target Files:**

- `src/shared/ssh/client.rs`
- `src/shared/ssh/config.rs`
- Test modules within the files

**Scope:**

- Extract magic numbers into new `SshConnectionConfig` type
- Group connection-related configuration (timeout, retry behavior) separately from credentials
- Improve constructor flexibility for testing and production scenarios
- Enhance test code quality with proper assertions and resource management
- Make useful private methods public for advanced use cases
- Improve error context and actionability
- Better align with module organization conventions

**Design Decision**: Based on feedback, connection behavior configuration (timeouts, retries) will be grouped in a new `SshConnectionConfig` type within `SshConfig`, keeping connection concerns separate from authentication credentials.

## 📊 Progress Tracking

**Total Active Proposals**: 8
**Total Postponed**: 0
**Total Discarded**: 2
**Completed**: 6
**In Progress**: 0
**Not Started**: 1

### Phase Summary

- **Phase 0 - Configuration Magic Numbers (High Impact, Low Effort)**: ✅ **3/3 completed (100%)** 🎉
- **Phase 1 - Test Quality Improvements (High Impact, Low Effort)**: ✅ **1/1 completed (100%)** 🎉 _(1 discarded)_
- **Phase 2 - Code Organization and Duplication (Medium Impact, Medium Effort)**: ✅ **2/2 completed (100%)** 🎉 _(1 discarded)_
- **Phase 3 - Advanced Improvements (Medium Impact, Medium Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

- **Proposal #1.1**: Add Assertions to SSH Warning Detection Test - Testing tracing output is too complex due to global state issues
- **Proposal #2.2**: Extract Common SSH Execution Logic - Current implementation is already well-designed with clean delegation

### Postponed Proposals

None yet.

## 🎯 Key Problems Identified

### 1. Magic Numbers Limit Flexibility

**Connection Timeout** - Hardcoded `ConnectTimeout=5` appears in:

- Line 86: SSH connection options in `build_ssh_args()`
- Line 253: `test_connectivity()` method

**Retry Configuration** - Hardcoded in `wait_for_connectivity()`:

- `max_attempts = 30` (line 269)
- `timeout_seconds = 60` (line 270) - not actually used as timeout
- `Duration::from_secs(2)` (line 292) - retry sleep interval
- `(attempt + 1) % 5 == 0` (line 283) - logging frequency

**Issues:**

- Users cannot tune these values for different network conditions
- Testing requires full 60-second waits even for unit tests
- No way to make timeouts more aggressive for fast networks or more lenient for slow networks

### 2. Test Code Quality

**Weak Test Assertions:**

- `it_should_detect_ssh_warnings_in_stderr` has no assertions (line 344+)
- Tests only verify methods don't panic, not that they behave correctly
- No verification of SSH argument construction

**Resource Management:**

- Tests use hardcoded paths like `/path/to/key` that don't exist
- No use of temporary directories for test isolation

**Limited Coverage:**

- No tests with `CommandExecutor` mocks to verify actual SSH command construction
- No tests for retry logic behavior
- No tests for timeout behavior

### 3. Code Organization

**Module Organization Violations:**

- Private helper methods (`build_ssh_args`, `process_ssh_warnings`) appear between public methods
- Per [module-organization.md](../contributing/module-organization.md), private helpers should come after public API

### 4. Code Duplication

**Similar Method Patterns:**

- `execute()` and `check_command()` follow nearly identical patterns
- `execute_with_options()` and `check_command_with_options()` duplicate SSH execution logic
- Only difference is error handling: `execute` returns errors, `check_command` converts to bool

### 5. Error Context

**Limited Actionability:**

- SSH connection failures could provide more specific guidance
- Timeout errors don't suggest what values to try
- Authentication errors could be more descriptive

## 🚀 Refactoring Phases

---

## Phase 0: Configuration Magic Numbers (Highest Priority)

Extract hardcoded timeout and retry values to enable flexible configuration for both production use and testing.

### Proposal #0.1: Create SshConnectionConfig Type with Connection Timeout

**Status**: ✅ **Completed** (Oct 14, 2025)  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0 (Foundation for testing improvements)

**Design Decision**: Following feedback, connection configuration will be grouped in a new `SshConnectionConfig` type that contains all connection behavior parameters (timeout, retries, logging frequency). This keeps `SshConfig` focused on credentials and target address, while connection behavior is properly encapsulated.

#### Problem

The SSH connection timeout is hardcoded to 5 seconds in two places:

```rust
// In build_ssh_args() - line 86
"-o".to_string(),
"ConnectTimeout=5".to_string(),

// In test_connectivity() - line 253
self.check_command_with_options("echo 'SSH connected'", &["ConnectTimeout=5"])
```

**TODO comment** already exists acknowledging this issue:

```rust
// TODO: Make this configurable via SshConfig constructor
```

**Consequences:**

- Users cannot tune connection timeout for different network scenarios
- Tests must wait the full 5 seconds even for fast local connections
- No way to make timeouts more aggressive for fast networks or lenient timeouts for slow networks
- Connection behavior configuration is scattered rather than grouped

#### Proposed Solution

Create a new `SshConnectionConfig` type to group all connection behavior parameters:

```rust
// In src/shared/ssh/config.rs - NEW CONSTANTS

/// Default SSH connection timeout in seconds
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u32 = 5;

/// Default maximum number of connection retry attempts
pub const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 30;

/// Default retry interval in seconds
pub const DEFAULT_RETRY_INTERVAL_SECS: u64 = 2;

/// Default retry log frequency (log every N attempts)
pub const DEFAULT_RETRY_LOG_FREQUENCY: u32 = 5;

// In src/shared/ssh/config.rs - NEW TYPE

/// SSH connection behavior configuration
///
/// Groups all connection-related parameters (timeouts, retries, logging)
/// separately from authentication credentials and target address.
#[derive(Clone, Debug)]
pub struct SshConnectionConfig {
    /// SSH connection timeout in seconds
    pub connect_timeout_secs: u32,
    /// Maximum number of connection retry attempts
    pub max_retry_attempts: u32,
    /// Seconds to wait between retry attempts
    pub retry_interval_secs: u64,
    /// Log progress every N retry attempts
    pub retry_log_frequency: u32,
}

impl SshConnectionConfig {
    /// Create a new connection configuration with custom values
    pub fn new(
        connect_timeout_secs: u32,
        max_retry_attempts: u32,
        retry_interval_secs: u64,
        retry_log_frequency: u32,
    ) -> Self {
        Self {
            connect_timeout_secs,
            max_retry_attempts,
            retry_interval_secs,
            retry_log_frequency,
        }
    }

    /// Total wait time in seconds (max_retry_attempts × retry_interval_secs)
    pub fn total_timeout_secs(&self) -> u64 {
        self.max_retry_attempts as u64 * self.retry_interval_secs
    }
}

impl Default for SshConnectionConfig {
    /// Default connection configuration (production settings)
    ///
    /// Uses constants defined at module level:
    /// - Connection timeout: DEFAULT_CONNECT_TIMEOUT_SECS (5 seconds)
    /// - Max retry attempts: DEFAULT_MAX_RETRY_ATTEMPTS (30)
    /// - Retry interval: DEFAULT_RETRY_INTERVAL_SECS (2 seconds)
    /// - Retry log frequency: DEFAULT_RETRY_LOG_FREQUENCY (every 5 attempts)
    /// - Total wait time: 30 × 2 = 60 seconds
    fn default() -> Self {
        Self {
            connect_timeout_secs: DEFAULT_CONNECT_TIMEOUT_SECS,
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
            retry_interval_secs: DEFAULT_RETRY_INTERVAL_SECS,
            retry_log_frequency: DEFAULT_RETRY_LOG_FREQUENCY,
        }
    }
}

// In src/shared/ssh/config.rs - UPDATE EXISTING TYPE

pub struct SshConfig {
    pub credentials: SshCredentials,
    pub socket_addr: SocketAddr,
    pub connection_config: SshConnectionConfig,  // NEW FIELD
}

impl SshConfig {
    /// Creates SSH config with default connection settings
    pub fn new(credentials: SshCredentials, ssh_socket_addr: SocketAddr) -> Self {
        Self {
            credentials,
            socket_addr: ssh_socket_addr,
            connection_config: SshConnectionConfig::default(),
        }
    }

    /// Creates SSH config with custom connection settings
    pub fn with_connection_config(
        credentials: SshCredentials,
        ssh_socket_addr: SocketAddr,
        connection_config: SshConnectionConfig,
    ) -> Self {
        Self {
            credentials,
            socket_addr: ssh_socket_addr,
            connection_config,
        }
    }

    /// Creates SSH config with default port and connection settings
    pub fn with_default_port(credentials: SshCredentials, host_ip: IpAddr) -> Self {
        let socket_addr = SocketAddr::new(host_ip, DEFAULT_SSH_PORT);
        Self::new(credentials, socket_addr)
    }

    // Existing accessors...
    pub fn connection_timeout_secs(&self) -> u32 {
        self.connection_config.connect_timeout_secs
    }
}

// In src/shared/ssh/client.rs - UPDATE CLIENT

pub struct SshClient {
    ssh_config: SshConfig,
    command_executor: CommandExecutor,
    // Connection config now comes from ssh_config.connection_config
}

impl SshClient {
    /// Creates a new `SshClient` with configuration from SshConfig
    #[must_use]
    pub fn new(ssh_config: SshConfig) -> Self {
        Self {
            ssh_config,
            command_executor: CommandExecutor::new(),
        }
    }

    fn build_ssh_args(&self, remote_command: &str, additional_options: &[&str]) -> Vec<String> {
        let mut args = vec![
            "-i".to_string(),
            self.ssh_config.ssh_priv_key_path().to_string_lossy().to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
            "-o".to_string(),
            // Get timeout from connection config
            format!("ConnectTimeout={}", self.ssh_config.connection_config.connect_timeout_secs),
            "-p".to_string(),
            self.ssh_config.ssh_port().to_string(),
        ];
        // ... rest unchanged
    }

    pub fn test_connectivity(&self) -> Result<bool, CommandError> {
        // No additional options needed - config has timeout
        self.check_command("echo 'SSH connected'")
    }
}
```

**Usage Examples:**

```rust
// Default configuration (production) - uses DEFAULT_* constants
let config = SshConfig::with_default_port(credentials, host_ip);
let client = SshClient::new(config);

// Custom configuration for testing (faster timeouts)
let connection_config = SshConnectionConfig::new(
    1,  // 1 second connect timeout
    5,  // 5 retry attempts
    1,  // 1 second retry interval
    2,  // log every 2 attempts
);
let config = SshConfig::with_connection_config(credentials, socket_addr, connection_config);
let client = SshClient::new(config);

// Custom configuration for slow networks
let connection_config = SshConnectionConfig::new(
    30,  // 30 second connect timeout
    20,  // 20 retry attempts
    6,   // 6 second retry interval
    3,   // log every 3 attempts
);
let config = SshConfig::with_connection_config(credentials, socket_addr, connection_config);
let client = SshClient::new(config);
```

#### Rationale

- **Separation of Concerns**: Connection behavior is grouped separately from credentials and target address
- **Type Safety**: `SshConnectionConfig` encapsulates all connection parameters in one place
- **No Magic Numbers**: Default values come from constants, not hardcoded literals
- **Flexibility**: Users can tune timeout for their network conditions via config
- **Testing**: Tests can create custom configs with shorter timeouts for quick execution
- **Backward Compatible**: Default implementation maintains current 5-second behavior
- **Extensibility**: Easy to add more connection parameters without changing SshClient API
- **Configuration as Data**: Connection settings can be serialized, loaded from files, etc.

**Alternatives Considered:**

1. **Separate fields in SshClient**: Would scatter connection config across client and SshConfig
2. **Builder Pattern**: More complex, overkill for grouped configuration
3. **Environment Variable**: Less explicit, harder to test, not type-safe
4. **Preset methods (fast/slow)**: Deferred until usage patterns emerge showing repeated custom configurations

**Decision**: Following feedback, use a dedicated `SshConnectionConfig` type within `SshConfig` to group all connection behavior parameters. Default values come from module-level constants.

#### Benefits

- ✅ Removes hardcoded magic numbers
- ✅ Groups related connection configuration in dedicated type
- ✅ Default values come from named constants, not magic numbers
- ✅ Enables faster test execution with custom configurations
- ✅ Allows production tuning for different network conditions
- ✅ Resolves existing TODO comment
- ✅ Maintains backward compatibility with existing code
- ✅ Configuration can be serialized/deserialized for storage
- ✅ Easy to extend with additional connection parameters
- ✅ Clean separation: credentials vs connection behavior
- ✅ Constants can be reused across the codebase

#### Implementation Checklist

- [ ] Add constants in `src/shared/ssh/config.rs`: `DEFAULT_CONNECT_TIMEOUT_SECS`, `DEFAULT_MAX_RETRY_ATTEMPTS`, `DEFAULT_RETRY_INTERVAL_SECS`, `DEFAULT_RETRY_LOG_FREQUENCY`
- [ ] Create `SshConnectionConfig` struct in `src/shared/ssh/config.rs`
- [ ] Add fields: `connect_timeout_secs`, `max_retry_attempts`, `retry_interval_secs`, `retry_log_frequency`
- [ ] Implement `new()` constructor
- [ ] Implement `Default` trait using the constants
- [ ] Add `total_timeout_secs()` helper method
- [ ] Add `connection_config: SshConnectionConfig` field to `SshConfig`
- [ ] Update `SshConfig::new()` to use `SshConnectionConfig::default()`
- [ ] Add `SshConfig::with_connection_config()` constructor
- [ ] Update `SshConfig` to expose connection config accessors
- [ ] Update `SshClient::build_ssh_args()` to use `self.ssh_config.connection_config.connect_timeout_secs`
- [ ] Remove hardcoded timeout from `test_connectivity()`
- [ ] Update all existing `SshConfig` construction to ensure compatibility
- [ ] Add tests for `SshConnectionConfig::default()` and `new()`
- [ ] Add tests verifying timeout is properly applied
- [ ] Add tests verifying constants have expected values
- [ ] Remove TODO comment
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues
- [ ] Update SSH configuration documentation with examples

#### Testing Strategy

```rust
#[test]
fn it_should_use_default_timeout_of_5_seconds() {
    let config = create_test_ssh_config();
    let client = SshClient::new(config);

    let args = client.build_ssh_args("echo test", &[]);
    assert!(args.contains(&"ConnectTimeout=5".to_string()));
}

#[test]
fn it_should_use_custom_timeout_when_specified() {
    let config = create_test_ssh_config();
    let client = SshClient::with_timeout(config, 10);

    let args = client.build_ssh_args("echo test", &[]);
    assert!(args.contains(&"ConnectTimeout=10".to_string()));
}
```

---

### Proposal #0.2: Use SshConnectionConfig for Retry Behavior

**Status**: ✅ **Completed** (Oct 14, 2025)  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: Proposal #0.1

**Note**: This proposal is actually already covered by Proposal #0.1, as `SshConnectionConfig` includes both timeout and retry configuration. This section documents the retry-specific aspects.

#### Problem

Retry behavior in `wait_for_connectivity()` is hardcoded with magic numbers:

```rust
pub async fn wait_for_connectivity(&self) -> Result<(), SshError> {
    let max_attempts = 30;        // line 269
    let timeout_seconds = 60;     // line 270 - NOT ACTUALLY USED AS TIMEOUT!
    let mut attempt = 0;

    while attempt < max_attempts {
        // ...
        tokio::time::sleep(Duration::from_secs(2)).await;  // line 292
        attempt += 1;
    }
    // Total wait time: 30 attempts × 2 seconds = 60 seconds
}
```

**Issues:**

- `timeout_seconds` variable is misleading - it's not used as a timeout
- Actual timeout = `max_attempts × sleep_duration` (30 × 2 = 60 seconds)
- Cannot reduce wait time for tests (60 seconds is slow for test suites)
- Cannot adjust for different deployment scenarios (some VMs boot faster)

#### Proposed Solution

The `SshConnectionConfig` created in Proposal #0.1 already includes retry configuration. Update `wait_for_connectivity()` to use it:

```rust
impl SshClient {
    pub async fn wait_for_connectivity(&self) -> Result<(), SshError> {
        let conn_config = &self.ssh_config.connection_config;

        info!(
            operation = "ssh_connectivity",
            host_ip = %self.ssh_config.host_ip(),
            max_attempts = conn_config.max_retry_attempts,
            retry_interval_secs = conn_config.retry_interval_secs,
            total_timeout_secs = conn_config.total_timeout_secs(),
            "Waiting for SSH connectivity"
        );

        let mut attempt = 0;

        while attempt < conn_config.max_retry_attempts {
            match self.test_connectivity() {
                Ok(true) => {
                    info!(
                        operation = "ssh_connectivity",
                        host_ip = %self.ssh_config.host_ip(),
                        status = "success",
                        attempts_used = attempt + 1,
                        "SSH connectivity established"
                    );
                    return Ok(());
                }
                Ok(false) => {
                    // Progress logging based on configured frequency
                    if (attempt + 1) % conn_config.retry_log_frequency == 0 {
                        info!(
                            operation = "ssh_connectivity",
                            host_ip = %self.ssh_config.host_ip(),
                            attempt = attempt + 1,
                            max_attempts = conn_config.max_retry_attempts,
                            "Still waiting for SSH connectivity"
                        );
                    }

                    tokio::time::sleep(Duration::from_secs(conn_config.retry_interval_secs)).await;
                    attempt += 1;
                }
                Err(e) => {
                    return Err(SshError::CommandFailed { source: e });
                }
            }
        }

        Err(SshError::ConnectivityTimeout {
            host_ip: self.ssh_config.host_ip().to_string(),
            attempts: conn_config.max_retry_attempts,
            timeout_seconds: conn_config.total_timeout_secs(),
        })
    }
}
```

#### Rationale

- **Explicit Total Timeout**: Calculated as `attempts × interval` and logged clearly
- **Test Flexibility**: Tests can use `with_config(config, 1, 5, 1)` for 5-second total wait
- **Production Tuning**: Different scenarios can optimize wait times
- **Backward Compatible**: Default constructor maintains current behavior (30 × 2 = 60s)
- **Progressive Disclosure**: Three constructors for different needs

**Alternatives Considered:**

1. **Builder Pattern**: More complex API, overkill for this use case
2. **Configuration Struct**: Adds another type, not worth the complexity
3. **Only Total Timeout**: Less flexible, can't control granularity of attempts

#### Benefits

- ✅ Removes all retry magic numbers
- ✅ Makes total timeout calculation explicit and visible
- ✅ Enables fast test execution (5-10 second waits vs 60 seconds)
- ✅ Allows production optimization for different VM boot times
- ✅ Improves observability with clear timeout logging
- ✅ Fixes misleading `timeout_seconds` variable

#### Implementation Checklist

- [ ] Add `max_retry_attempts: u32` field to `SshClient`
- [ ] Add `retry_interval_secs: u64` field to `SshClient`
- [ ] Update `new()` to call `with_config(ssh_config, 5, 30, 2)`
- [ ] Update `with_timeout()` to call `with_config(ssh_config, connect_timeout_secs, 30, 2)`
- [ ] Add `with_config()` constructor with comprehensive documentation
- [ ] Update `wait_for_connectivity()` to use new fields
- [ ] Add calculated total timeout to initial info log
- [ ] Update `SshError::ConnectivityTimeout` to use calculated timeout
- [ ] Update existing usage sites if needed
- [ ] Add tests for different retry configurations
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues
- [ ] Update documentation with retry configuration examples

#### Testing Strategy

```rust
#[tokio::test]
async fn it_should_use_default_retry_configuration() {
    let config = create_test_ssh_config();
    let client = SshClient::new(config);

    // Test would need mock CommandExecutor to verify behavior
    // Verify: 30 attempts × 2 seconds = 60 second total timeout
}

#[tokio::test]
async fn it_should_use_custom_retry_configuration() {
    let config = create_test_ssh_config();
    let client = SshClient::with_config(config, 1, 5, 1);

    // Test with fast timeout: 5 attempts × 1 second = 5 second total
    // Much faster for test execution!
}
```

---

### Proposal #0.3: Configurable Log Frequency

**Status**: ✅ **Completed** (Oct 14, 2025) - Implemented as part of Proposals #0.1 and #0.2  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P1

> **Design Decision**: User chose **Option B** - Include `retry_log_frequency` as a field in `SshConnectionConfig` for maximum flexibility.
>
> **Note**: This proposal was completed as part of the unified `SshConnectionConfig` design in Proposals #0.1 and #0.2.

#### Problem

Logging frequency in `wait_for_connectivity()` uses a magic number:

```rust
if (attempt + 1) % 5 == 0 {  // line 283
    info!("Still waiting for SSH connectivity");
}
```

**Issues:**

- Magic number `5` has no clear meaning
- Cannot adjust logging verbosity without code changes
- Not discoverable or configurable

#### Proposed Solution

The `retry_log_frequency` field is included in `SshConnectionConfig` (from Proposal #0.1):

```rust
pub struct SshConnectionConfig {
    pub connect_timeout_secs: u32,
    pub max_retry_attempts: u32,
    pub retry_interval_secs: u64,
    pub retry_log_frequency: u32,  // Log progress every N attempts
}

impl Default for SshConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout_secs: DEFAULT_CONNECT_TIMEOUT_SECS,  // 5
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,      // 30
            retry_interval_secs: DEFAULT_RETRY_INTERVAL_SECS,    // 2
            retry_log_frequency: DEFAULT_RETRY_LOG_FREQUENCY,    // 5
        }
    }
}
```

Usage in `wait_for_connectivity()`:

```rust
if (attempt + 1) % conn_config.retry_log_frequency == 0 {
    info!("Still waiting for SSH connectivity");
}
```

#### Rationale

**Why Option B was chosen:**

- Provides maximum flexibility for different deployment scenarios
- Fast deployments can log more frequently (create custom config with frequency=2)
- Production deployments can reduce log noise (create custom config with frequency=10)
- Testing can disable progress logging (set to very high number)
- Consistent with grouping all connection behavior in one config type
- Default value comes from constant, not magic number

#### Benefits

- ✅ Documents the logging frequency purpose
- ✅ Fully configurable per deployment scenario
- ✅ Default value uses constant, not magic number
- ✅ Enables reducing log noise in production
- ✅ Consistent with unified configuration design

#### Implementation Checklist

- [ ] Already included in `SshConnectionConfig` implementation (Proposal #0.1)
- [ ] Ensure `DEFAULT_RETRY_LOG_FREQUENCY` constant is defined
- [ ] Update `wait_for_connectivity()` to use `conn_config.retry_log_frequency`
- [ ] Verify logging behavior in tests

#### Testing Strategy

No specific tests needed - this is a simple constant extraction.

---

## Phase 1: Test Quality Improvements (High Priority)

Improve test code to be more maintainable, readable, and actually verify behavior.

### Proposal #1.1: Add Assertions to SSH Warning Detection Test

**Status**: ❌ **DISCARDED** (Oct 14, 2025)  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0

**Reason for Discarding**: Testing tracing output is very hard because tracing uses global state. Experience from other projects shows this approach doesn't work well. The current test already verifies the important behavior: that the method correctly recognizes warning lines in SSH command output without panicking. Adding assertions for tracing output would add complexity and fragility without sufficient benefit.

#### Problem

The test `it_should_detect_ssh_warnings_in_stderr` has no assertions:

```rust
#[test]
fn it_should_detect_ssh_warnings_in_stderr() {
    let ssh_client = create_test_ssh_client();

    let stderr_with_warning = "Warning: Permanently added...";
    ssh_client.process_ssh_warnings(stderr_with_warning);

    let stderr_without_warning = "Some other output";
    ssh_client.process_ssh_warnings(stderr_without_warning);

    ssh_client.process_ssh_warnings("");

    // ❌ NO ASSERTIONS - Test doesn't verify anything!
}
```

**Issues:**

- Test provides false confidence - only checks that method doesn't panic
- Doesn't verify that warnings are actually logged
- Doesn't verify that non-warnings are not logged
- Against testing conventions: tests should verify behavior

#### Proposed Solution

Use `tracing-subscriber` test utilities to capture and verify log output:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    #[test]
    fn it_should_log_ssh_warnings_at_warn_level() {
        // Arrange: Set up tracing capture
        let (writer, rx) = tracing_appender::non_blocking(std::io::sink());
        let filter = tracing_subscriber::EnvFilter::new("warn");
        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_test_writer()
            .with_max_level(tracing::Level::WARN)
            .finish();

        let _guard = tracing::subscriber::set_default(subscriber);

        let ssh_client = create_test_ssh_client();
        let stderr_with_warning =
            "Warning: Permanently added '10.140.190.14' (ED25519) to the list of known hosts.";

        // Act: Process warning
        ssh_client.process_ssh_warnings(stderr_with_warning);

        // Assert: Verify warning was logged
        // Note: Actual implementation depends on tracing test utilities
        // This is a simplified example
    }

    #[test]
    fn it_should_not_log_non_warning_stderr_output() {
        let ssh_client = create_test_ssh_client();
        let stderr_without_warning = "Some other output";

        // Should not panic, should not log at warn level
        ssh_client.process_ssh_warnings(stderr_without_warning);

        // Verify no warnings logged (implementation depends on tracing capture)
    }

    #[test]
    fn it_should_handle_empty_stderr_gracefully() {
        let ssh_client = create_test_ssh_client();

        // Should not panic
        ssh_client.process_ssh_warnings("");

        // Should not log anything
    }
}
```

#### Rationale

- **Verify Behavior**: Tests should check that warnings are actually logged
- **Tracing Integration**: Use proper tracing test utilities for log verification
- **Test Clarity**: Each test case verifies specific behavior
- **Follow Conventions**: Aligns with [testing.md](../contributing/testing.md) - tests should assert behavior

**Alternative**: Mock the tracing layer for more controlled testing.

#### Benefits

- ✅ Tests actually verify behavior
- ✅ Provides confidence that warning detection works
- ✅ Follows project testing conventions
- ✅ Makes test intent clear

#### Implementation Checklist

- [ ] Research tracing test utilities for log capture
- [ ] Add test dependencies if needed
- [ ] Rewrite test with proper assertions
- [ ] Split into three separate test cases
- [ ] Verify tests fail when code is broken
- [ ] Verify tests pass with correct implementation
- [ ] Document tracing test pattern for future use

#### Testing Strategy

Tests should verify:

1. Warnings starting with "Warning:" are logged at WARN level
2. Non-warning output is not logged
3. Empty stderr is handled gracefully

---

### Proposal #1.2: Improve Test Resource Management with Temporary Directories

**Status**: ✅ **Completed** (Oct 14, 2025)  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P1

#### Problem

Tests use hardcoded paths that don't exist:

```rust
#[test]
fn it_should_create_ssh_client_with_valid_parameters() {
    let credentials = SshCredentials::new(
        PathBuf::from("/path/to/key"),      // ❌ Doesn't exist
        PathBuf::from("/path/to/key.pub"),  // ❌ Doesn't exist
        Username::new("testuser").unwrap(),
    );
    // ...
}
```

**Issues:**

- Violates testing principle of using real, isolated resources
- Against [testing.md](../contributing/testing.md) resource management guidelines
- Tests are fragile - fail if paths matter
- Not following project conventions (see `TempDir` usage in other tests)

#### Proposed Solution

Use `tempfile::TempDir` for proper resource management:

```rust
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create test SSH credentials with temporary key files
    fn create_test_ssh_credentials() -> (TempDir, SshCredentials) {
        let temp_dir = TempDir::new()
            .expect("Failed to create temp directory for SSH key test files");

        let priv_key_path = temp_dir.path().join("test_key");
        let pub_key_path = temp_dir.path().join("test_key.pub");

        // Create actual (empty) key files for realism
        fs::write(&priv_key_path, "fake private key content")
            .expect("Failed to write test private key");
        fs::write(&pub_key_path, "fake public key content")
            .expect("Failed to write test public key");

        let credentials = SshCredentials::new(
            priv_key_path,
            pub_key_path,
            Username::new("testuser").unwrap(),
        );

        (temp_dir, credentials)  // Return TempDir to keep it alive
    }

    #[test]
    fn it_should_create_ssh_client_with_valid_parameters() {
        // Arrange
        let (_temp_dir, credentials) = create_test_ssh_credentials();
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);

        // Act
        let ssh_client = SshClient::new(ssh_config);

        // Assert
        assert!(ssh_client.ssh_config.ssh_priv_key_path().exists());
        assert!(ssh_client.ssh_config.ssh_pub_key_path().exists());
        assert_eq!(ssh_client.ssh_config.ssh_username(), "testuser");
        assert_eq!(ssh_client.ssh_config.host_ip(), host_ip);

        // TempDir automatically cleans up when dropped
    }
}
```

#### Rationale

- **Isolation**: Each test gets its own temporary directory
- **Realism**: Tests use actual files (even if fake content)
- **Cleanup**: `TempDir` automatically removes files after test
- **Convention**: Follows project pattern used in other tests
- **Robustness**: Tests don't depend on system paths

#### Benefits

- ✅ Follows project testing conventions
- ✅ Tests are isolated and don't interfere with each other
- ✅ Automatic cleanup prevents test artifacts
- ✅ More realistic test scenarios
- ✅ Can actually verify file operations if needed

#### Implementation Checklist

- [ ] Add `create_test_ssh_credentials()` helper function
- [ ] Update all tests to use helper
- [ ] Verify key files are created in temporary locations
- [ ] Add assertions that verify file existence where appropriate
- [ ] Ensure all tests clean up properly
- [ ] Verify all tests pass
- [ ] Run linter

#### Testing Strategy

All existing tests should be updated to use temporary directories. Tests should verify:

- SSH credentials point to real temporary files
- Files are cleaned up after test completion
- Tests remain isolated from each other

---

## Phase 2: Code Organization and Duplication (Medium Priority)

Improve code structure and reduce duplication.

### Proposal #2.1: Reorganize Methods Per Module Organization Guidelines

**Status**: ✅ **Completed** (Oct 14, 2025)  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P2

#### Problem

Private helper methods appear between public methods, violating module organization conventions:

**Current order:**

1. Public: `new()`
2. Public: `ssh_config()`
3. Private: `build_ssh_args()` ❌ Should come later
4. Public: `execute()`
5. Private: `execute_with_options()` ❌ Should come later
6. Private: `process_ssh_warnings()` ❌ Should come later
7. Public: `check_command()`
8. Private: `check_command_with_options()` ❌ Should come later
9. Public: `test_connectivity()`
10. Public: `wait_for_connectivity()`

Per [module-organization.md](../contributing/module-organization.md), correct order should be:

1. All public items first
2. Then all private items

#### Proposed Solution

Reorganize to follow **Public Before Private** principle:

```rust
impl SshClient {
    // ============================================================================
    // PUBLIC API - Constructors
    // ============================================================================

    pub fn new(ssh_config: SshConfig) -> Self { }
    pub fn with_timeout(ssh_config: SshConfig, connect_timeout_secs: u32) -> Self { }
    pub fn with_config(...) -> Self { }

    // ============================================================================
    // PUBLIC API - Accessors
    // ============================================================================

    pub fn ssh_config(&self) -> &SshConfig { }

    // ============================================================================
    // PUBLIC API - Command Execution
    // ============================================================================

    pub fn execute(&self, remote_command: &str) -> Result<String, CommandError> { }
    pub fn check_command(&self, remote_command: &str) -> Result<bool, CommandError> { }

    // ============================================================================
    // PUBLIC API - Connectivity Testing
    // ============================================================================

    pub fn test_connectivity(&self) -> Result<bool, CommandError> { }
    pub async fn wait_for_connectivity(&self) -> Result<(), SshError> { }

    // ============================================================================
    // PRIVATE - Helper Methods
    // ============================================================================

    fn build_ssh_args(&self, remote_command: &str, additional_options: &[&str]) -> Vec<String> { }
    fn execute_with_options(...) -> Result<String, CommandError> { }
    fn check_command_with_options(...) -> Result<bool, CommandError> { }
    fn process_ssh_warnings(&self, stderr: &str) { }
}
```

#### Rationale

- **Readability**: Users see public API first without wading through implementation
- **Maintainability**: Clear separation of concerns
- **Convention**: Follows project module organization standards
- **Documentation**: Public API is immediately visible

#### Benefits

- ✅ Follows project conventions
- ✅ Improves code navigation
- ✅ Makes public API more discoverable
- ✅ Better separation of concerns

#### Implementation Checklist

- [ ] Move all private methods after public methods
- [ ] Add section comments per module organization guide
- [ ] Verify no functional changes
- [ ] Verify all tests pass
- [ ] Run linter

---

### Proposal #2.2: Extract Common SSH Execution Logic

**Status**: ❌ **DISCARDED** (Oct 14, 2025)  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposal #2.1

**Reason for Discarding**: After analysis, the current implementation is well-designed. `execute_with_options` is the core implementation and `check_command_with_options` is a thin, focused wrapper with no actual duplication - just clean delegation. Each method has a single responsibility and the separation of concerns is clear.

#### Problem

`execute_with_options()` and `check_command_with_options()` duplicate SSH execution logic:

```rust
fn execute_with_options(
    &self,
    remote_command: &str,
    additional_options: &[&str],
) -> Result<String, CommandError> {
    let args = self.build_ssh_args(remote_command, additional_options);
    let args_str: Vec<&str> = args.iter().map(std::string::String::as_str).collect();
    let result = self.command_executor.run_command("ssh", &args_str, None)?;
    self.process_ssh_warnings(&result.stderr);
    Ok(result.stdout)  // ⬅️ Returns stdout
}

fn check_command_with_options(
    &self,
    remote_command: &str,
    additional_options: &[&str],
) -> Result<bool, CommandError> {
    match self.execute_with_options(remote_command, additional_options) {
        Ok(_) => Ok(true),         // ⬅️ Converts to bool
        Err(CommandError::ExecutionFailed { .. }) => Ok(false),
        Err(other) => Err(other),
    }
}
```

**Issues:**

- Almost identical logic with different return types
- `check_command_with_options` just wraps `execute_with_options`
- Could be simplified

#### Proposed Solution

Keep the current implementation - it's actually well-designed:

```rust
// execute_with_options: Core implementation, returns stdout
fn execute_with_options(...) -> Result<String, CommandError> {
    // Full implementation
}

// check_command_with_options: Thin wrapper, converts to bool
fn check_command_with_options(...) -> Result<bool, CommandError> {
    match self.execute_with_options(remote_command, additional_options) {
        Ok(_) => Ok(true),
        Err(CommandError::ExecutionFailed { .. }) => Ok(false),
        Err(other) => Err(other),
    }
}
```

#### Rationale

After analysis, the current implementation is actually good:

- `execute_with_options` is the core implementation
- `check_command_with_options` is a thin, focused wrapper
- No actual duplication - just delegation
- Clean separation of concerns
- Each method has a single responsibility

**This proposal should be DISCARDED** - no refactoring needed.

#### Benefits

N/A - keeping current implementation

#### Implementation Checklist

- [ ] Mark proposal as DISCARDED
- [ ] Document reasoning in this plan

---

### Proposal #2.3: Make execute_with_options and check_command_with_options Public

**Status**: ✅ **Completed** (October 14, 2025)  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P3

> **Design Decision**: User chose **Option A** - Make methods public directly with documentation. Simple and straightforward approach.

#### Problem

`execute_with_options()` and `check_command_with_options()` are private, but they provide useful functionality for advanced use cases:

- Passing dynamic SSH options per command
- Adding connection keep-alive for long-running commands
- Customizing SSH behavior without creating new client instances

Current workaround: Modify `SshClient` configuration at construction time.

#### Proposed Solution

Make both methods public with comprehensive documentation:

````rust
impl SshClient {
    /// Execute a command with additional SSH options
    ///
    /// This method allows passing custom SSH options for specific commands,
    /// useful for advanced scenarios like connection keep-alive or custom timeouts.
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to execute on the remote host
    /// * `additional_options` - SSH options (e.g., `["ServerAliveInterval=60"]`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Keep connection alive during long-running command
    /// let output = client.execute_with_options(
    ///     "long_running_task",
    ///     &["ServerAliveInterval=60", "ServerAliveCountMax=3"]
    /// )?;
    ///
    /// // Use custom connection timeout for specific command
    /// let output = client.execute_with_options(
    ///     "quick_check",
    ///     &["ConnectTimeout=2"]
    /// )?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CommandError::ExecutionFailed` if the command exits with non-zero status,
    /// or `CommandError::IoError` if SSH execution fails.
    pub fn execute_with_options(
        &self,
        remote_command: &str,
        additional_options: &[&str],
    ) -> Result<String, CommandError> {
        // ... existing implementation
    }

    /// Check if a command succeeds with additional SSH options
    ///
    /// Similar to `execute_with_options()` but returns `true` if the command succeeds
    /// and `false` if it exits with non-zero status. Useful for conditional checks.
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to test on the remote host
    /// * `additional_options` - SSH options (e.g., `["ConnectTimeout=5"]`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Check if service is running with custom timeout
    /// let is_running = client.check_command_with_options(
    ///     "systemctl is-active my-service",
    ///     &["ConnectTimeout=3"]
    /// )?;
    ///
    /// if is_running {
    ///     println!("Service is active");
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CommandError::IoError` if SSH execution fails.
    /// Does NOT return an error for non-zero exit codes (returns `false` instead).
    pub fn check_command_with_options(
        &self,
        remote_command: &str,
        additional_options: &[&str],
    ) -> Result<bool, CommandError> {
        // ... existing implementation
    }
}
````

#### Rationale

**Why Option A was chosen:**

- **Simplicity**: Directly exposes existing functionality without new abstractions
- **Discoverability**: Public methods appear in IDE autocomplete and documentation
- **Flexibility**: Users can pass any SSH options supported by the SSH command
- **No overhead**: No additional types or builder pattern complexity
- **Minimal API surface**: Just two well-documented methods
- **Consistency**: Matches the simple, direct API style of other methods

Option B (builder pattern) would add unnecessary complexity for a simple use case.

#### Benefits

- ✅ Enables advanced SSH configuration per command
- ✅ No need to create new `SshClient` instances for one-off options
- ✅ Useful for connection keep-alive in long-running commands
- ✅ Simple, direct API without extra abstractions
- ✅ Well-documented with practical examples
- ✅ Maintains backward compatibility (only adds public methods)

#### Implementation Checklist

- [ ] Change visibility of `execute_with_options` from private to `pub`
- [ ] Change visibility of `check_command_with_options` from private to `pub`
- [ ] Add comprehensive doc comments with examples (as shown above)
- [ ] Add rustdoc examples that compile (using `doc-test` comments)
- [ ] Write integration tests covering public API usage
- [ ] Verify all existing tests still pass
- [ ] Run linters (rustfmt, clippy)
- [ ] Update module documentation to mention advanced options capability

---

## Phase 3: Advanced Improvements (Medium Priority)

### Proposal #3.1: Improve Error Context and Actionability

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P2

#### Problem

SSH errors could provide more actionable guidance following the tiered help pattern from [error-handling.md](../contributing/error-handling.md).

Current error:

```rust
Err(SshError::ConnectivityTimeout {
    host_ip: "192.168.1.100",
    attempts: 30,
    timeout_seconds: 60,
})
// Display: "Failed to establish SSH connectivity to 192.168.1.100 after 30 attempts (60 seconds)"
```

**Issues:**

- Doesn't explain why connection failed
- Doesn't suggest troubleshooting steps
- Not actionable per project principles

#### Proposed Solution

Enhance `SshError` with tiered help system:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshError {
    /// Failed to establish SSH connectivity within timeout period
    ///
    /// This typically means the SSH service is not yet available or the
    /// instance is still booting. Use `.help()` for detailed troubleshooting.
    #[error("Failed to establish SSH connectivity to {host_ip} after {attempts} attempts ({timeout_seconds}s total)
Tip: Check if instance is fully booted and SSH service is running")]
    ConnectivityTimeout {
        host_ip: String,
        attempts: u32,
        timeout_seconds: u64,
    },

    /// SSH command execution failed
    #[error("SSH command execution failed")]
    CommandFailed {
        #[source]
        source: CommandError,
    },
}

impl SshError {
    /// Get detailed troubleshooting guidance for this error
    pub fn help(&self) -> &'static str {
        match self {
            Self::ConnectivityTimeout { .. } => {
                "SSH Connectivity Timeout - Detailed Troubleshooting:

1. Verify the instance is running:
   - Check VM/container status in your provider
   - Ensure instance has finished booting (may take 30-60s)

2. Check SSH service status:
   - For LXD: lxc exec <instance> -- systemctl status ssh
   - For cloud instances: Check console logs

3. Verify network connectivity:
   - Ping the IP address: ping <host_ip>
   - Check firewall rules allow port 22
   - Verify no network issues between hosts

4. Check SSH configuration:
   - Ensure SSH service is enabled on boot
   - Verify sshd_config allows key authentication
   - Check SSH key permissions (should be 600 or 400)

5. Try manual connection:
   - ssh -i <key_path> -o ConnectTimeout=5 -o StrictHostKeyChecking=no <user>@<host_ip>
   - Check for specific error messages

6. Increase timeout if needed:
   - Slow networks may need more time
   - Use SshClient::with_config() to increase max_retry_attempts or retry_interval_secs

For more information, see the SSH troubleshooting guide."
            }

            Self::CommandFailed { .. } => {
                "SSH Command Failed - Detailed Troubleshooting:

1. Check the underlying command error for specific details
2. Verify SSH authentication is working
3. Ensure remote command is valid on the target system
4. Check for permission issues on remote host

For more information, see the command execution guide."
            }
        }
    }
}
```

Usage in application code:

```rust
match ssh_client.wait_for_connectivity().await {
    Ok(()) => { /* success */ }
    Err(e) => {
        error!("SSH connectivity failed: {}", e);

        if verbose {
            eprintln!("\nTroubleshooting:\n{}", e.help());
        } else {
            eprintln!("\nRun with --verbose for detailed troubleshooting");
        }

        return Err(e);
    }
}
```

#### Rationale

- **Actionability**: Follows project principle of actionable errors
- **Tiered Help**: Uses approved pattern from [actionable-error-messages.md](../decisions/actionable-error-messages.md)
- **User-Friendly**: Provides clear next steps without overwhelming default output
- **Traceability**: Maintains error context for debugging

#### Benefits

- ✅ Errors become actionable with clear troubleshooting steps
- ✅ Follows project error handling conventions
- ✅ Improves user experience significantly
- ✅ Balances brevity with detailed help on demand
- ✅ Platform-aware guidance (Unix/Linux commands)

#### Implementation Checklist

- [ ] Update `SshError` enum with brief tips in `#[error]` attributes
- [ ] Add `.help()` method with detailed troubleshooting
- [ ] Update error display messages to be more specific
- [ ] Add platform-specific troubleshooting commands
- [ ] Update application error handling to use `.help()` when verbose
- [ ] Add tests for error messages and help text
- [ ] Update documentation with error handling examples
- [ ] Verify all tests pass
- [ ] Run linter

#### Testing Strategy

```rust
#[test]
fn it_should_provide_actionable_connectivity_timeout_error() {
    let error = SshError::ConnectivityTimeout {
        host_ip: "192.168.1.100".to_string(),
        attempts: 30,
        timeout_seconds: 60,
    };

    let message = error.to_string();
    assert!(message.contains("192.168.1.100"));
    assert!(message.contains("30 attempts"));
    assert!(message.contains("Tip:"));

    let help = error.help();
    assert!(help.contains("Verify the instance is running"));
    assert!(help.contains("Check SSH service status"));
    assert!(help.contains("ssh -i"));
}
```

---

## 📈 Timeline

- **Start Date**: To be determined
- **Estimated Duration**: 2-3 development sessions

  - Phase 0: 1 session (3-4 hours) - Configuration extraction
  - Phase 1: 1 session (2-3 hours) - Test improvements
  - Phase 2: 0.5-1 session (1-2 hours) - Code organization
  - Phase 3: 0.5-1 session (2-3 hours) - Error improvements

## 🔍 Review Process

### Approval Criteria

- [ ] All proposals reviewed by maintainers
- [ ] Technical feasibility validated
- [ ] Aligns with [Development Principles](../development-principles.md)
- [ ] Implementation plan is clear and actionable
- [ ] Prioritization by impact/effort is reasonable

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing
- [ ] All linters passing
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../development-principles.md)
- [Contributing Guidelines](../contributing/README.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Module Organization](../contributing/module-organization.md)
- [ADR: Actionable Error Messages](../decisions/actionable-error-messages.md)

## 💡 Notes

### Key Insights

1. **Magic numbers are a testing anti-pattern**: Hardcoded timeouts make tests slow and inflexible
2. **Constructor flexibility matters**: Different scenarios (production, testing, fast networks, slow networks) need different configurations
3. **Progressive disclosure in APIs**: Provide simple defaults with the ability to customize when needed
4. **Test code quality matters**: Tests without assertions provide false confidence
5. **Actionable errors improve UX**: Following the tiered help pattern significantly improves troubleshooting experience

### Implementation Notes

- Phase 0 is highest priority - enables fast test execution and production tuning
- Proposals #0.1 and #0.2 have the most impact with least effort
- Test improvements (Phase 1) should be done early to establish good patterns
- Error improvements (Phase 3) can be done incrementally as time permits

---

**Created**: October 14, 2025  
**Last Updated**: October 14, 2025  
**Status**: 📋 Planning
