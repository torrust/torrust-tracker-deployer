# File Lock Module Refactoring Plan

**Module**: `src/infrastructure/persistence/filesystem/file_lock.rs`  
**Date Created**: October 2, 2025  
**Status**: 📋 Planning  
**Priority**: Maintainability and Readability

## 📋 Overview

This document outlines a comprehensive refactoring plan for the file lock module to improve code cleanliness, maintainability, readability, and testability. The proposals are organized by impact-to-effort ratio, starting with high-impact, low-effort improvements.

**Key Goals**:

- ✨ Make the code cleaner and more maintainable
- 📖 Improve readability for developers
- 🧪 Enhance testability across the module
- 🎯 Align with project principles: Observability, Traceability, Actionability

**Out of Scope**:

- ❌ Proposal #6 (Make `ProcessId` testable with traits) - Deferred for future consideration
- ❌ Proposal #12 (Performance optimizations with `parking_lot`) - Not a priority currently

---

## 🎯 Progress Tracking

### Summary

| Phase                    | Proposals | Status         | Completion |
| ------------------------ | --------- | -------------- | ---------- |
| **Phase 1: Quick Wins**  | #1-4      | 🚧 In Progress | 3/4        |
| **Phase 2: Testability** | #5, #7    | ⏳ Not Started | 0/2        |
| **Phase 3: Polish**      | #8-10     | ⏳ Not Started | 0/3        |
| **Phase 4: Advanced**    | #11       | ⏳ Not Started | 0/1        |
| **Total**                |           |                | **3/10**   |

### Legend

- ⏳ **Not Started** - Proposal not yet implemented
- 🚧 **In Progress** - Currently being worked on
- ✅ **Completed** - Implemented and committed
- 🔄 **Review** - Implementation done, pending review
- ⏸️ **Blocked** - Cannot proceed due to dependencies
- ❌ **Cancelled** - Decided not to implement

---

## 📦 Phase 1: Quick Wins (1-2 hours)

High-impact improvements with minimal effort. Can be completed in a single PR.

### Proposal #1: Extract Magic Numbers to Named Constants

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

The fake PID `999_999_u32` in tests is a magic number without explanation, making the code less readable and harder to maintain.

#### Current Code

```rust
#[test]
fn it_should_clean_up_stale_lock_with_invalid_pid() {
    // ...
    let fake_pid = 999_999_u32;
    fs::write(&lock_file_path, fake_pid.to_string()).unwrap();
    // ...
}
```

#### Proposed Solution

```rust
// In tests module - add at the top level
/// PID value that is highly unlikely to be a running process
/// Used in tests to simulate stale locks from dead processes
const FAKE_DEAD_PROCESS_PID: u32 = 999_999;

#[test]
fn it_should_clean_up_stale_lock_with_invalid_pid() {
    // ...
    let fake_pid = FAKE_DEAD_PROCESS_PID;
    fs::write(&lock_file_path, fake_pid.to_string()).unwrap();
    // ...
}
```

#### Benefits

- ✅ Self-documenting code - clear intent
- ✅ Easy to update if needed across all tests
- ✅ Consistent test behavior

#### Implementation Checklist

- [x] Add `FAKE_DEAD_PROCESS_PID` constant to test module
- [x] Replace all instances of `999_999_u32` in tests
- [x] Verify all tests pass
- [x] Run linters

---

### Proposal #2: Replace `unwrap()` with Descriptive Error Messages

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

Tests use `.unwrap()` without context, making test failures harder to debug. When a test fails, developers don't get immediate context about what went wrong.

#### Current Code

```rust
#[test]
fn it_should_successfully_acquire_lock() {
    let scenario = TestLockScenario::new();
    let lock = scenario.acquire_lock();
    assert!(lock.is_ok());
    let lock = lock.unwrap(); // No context if this fails
    // ...
}
```

#### Proposed Solution

```rust
#[test]
fn it_should_successfully_acquire_lock() {
    let scenario = TestLockScenario::new();
    let lock = scenario.acquire_lock();
    assert!(lock.is_ok());
    let lock = lock.expect("Failed to acquire lock for basic operations test");
    // ...
}
```

#### Areas to Update

1. **TempDir creation**: `TempDir::new().expect("Failed to create temporary directory for test")`
2. **Lock acquisition in setup**: `.expect("Failed to acquire lock in test setup")`
3. **File operations**: `.expect("Failed to read lock file in test")`
4. **Thread operations**: `.expect("Failed to join test thread")`

#### Benefits

- ✅ Immediate context when tests fail
- ✅ Aligns with project's observability principles
- ✅ No performance cost
- ✅ Better developer experience

#### Implementation Checklist

- [x] Audit all `.unwrap()` calls in test code
- [x] Replace with `.expect()` with descriptive messages
- [x] Document pattern in testing conventions
- [x] Verify all tests pass with new messages

---

### Proposal #3: Add Builder Methods for Common Test Scenarios

**Status**: ✅ Completed  
**Impact**: 🟢🟢 Medium-High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

Many tests repeat the pattern of creating `TempDir`, file paths, and configuring timeouts. This leads to boilerplate and makes tests harder to read.

#### Current Code

```rust
#[test]
fn it_should_clean_up_stale_lock_with_invalid_pid() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_temp_file_path(&temp_dir, "stale.json");
    let lock_file_path = FileLock::lock_file_path(&file_path);

    let fake_pid = 999_999_u32;
    fs::write(&lock_file_path, fake_pid.to_string()).unwrap();

    let lock_result = FileLock::acquire(&file_path, Duration::from_secs(1));
    // ...
}
```

#### Proposed Solution

Add convenience methods to `TestLockScenario`:

```rust
impl TestLockScenario {
    /// Create scenario with short timeout for failure tests (200ms)
    fn for_timeout_test() -> Self {
        Self::new().with_timeout(Duration::from_millis(200))
    }

    /// Create scenario with long timeout for success tests (5 seconds)
    fn for_success_test() -> Self {
        Self::new().with_timeout(Duration::from_secs(5))
    }

    /// Create a stale lock file with a dead process PID
    fn with_stale_lock(&self, fake_pid: u32) -> Result<(), std::io::Error> {
        fs::write(&self.lock_file_path(), fake_pid.to_string())
    }

    /// Create a lock file with invalid content for error testing
    fn with_invalid_lock(&self, content: &str) -> Result<(), std::io::Error> {
        fs::write(&self.lock_file_path(), content)
    }

    /// Create a lock file held by the current process
    fn with_current_process_lock(&self) -> Result<(), std::io::Error> {
        fs::write(&self.lock_file_path(), ProcessId::current().to_string())
    }
}
```

#### Example Usage

```rust
#[test]
fn it_should_clean_up_stale_lock_with_invalid_pid() {
    // Arrange
    let scenario = TestLockScenario::for_success_test()
        .with_file_name("stale.json");
    scenario.with_stale_lock(FAKE_DEAD_PROCESS_PID)
        .expect("Failed to create stale lock file");

    // Act
    let lock_result = scenario.acquire_lock();

    // Assert
    assert!(lock_result.is_ok());
    assert_lock_file_contains_current_pid(&scenario.file_path());
}
```

#### Benefits

- ✅ Less repetitive code
- ✅ More readable test setup
- ✅ Easier to maintain common patterns
- ✅ Self-documenting test scenarios

#### Implementation Checklist

- [x] Add builder methods to `TestLockScenario`
- [x] Refactor existing tests to use new methods
- [x] Add documentation for new methods
- [x] Verify all tests pass
- [x] Update testing conventions doc if needed

---

### Proposal #4: Extract Assertion Helpers into Granular Functions

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium-High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

Current assertion helpers are limited and some checks are repeated across tests. More specific helpers would improve test readability.

#### Current Helpers

```rust
fn assert_lock_file_contains_current_pid(file_path: &Path) { /* ... */ }
fn assert_lock_file_absent(file_path: &Path) { /* ... */ }
```

#### Proposed Additional Helpers

```rust
/// Assert that lock file exists (without checking content)
fn assert_lock_file_exists(file_path: &Path) {
    let lock_file_path = FileLock::lock_file_path(file_path);
    assert!(
        lock_file_path.exists(),
        "Lock file should exist at {lock_file_path:?}"
    );
}

/// Assert that lock file contains specific PID
fn assert_lock_file_contains_pid(file_path: &Path, expected_pid: ProcessId) {
    let lock_file_path = FileLock::lock_file_path(file_path);
    let pid_content = fs::read_to_string(&lock_file_path)
        .expect("Should be able to read lock file");
    assert_eq!(
        pid_content.trim(),
        expected_pid.to_string(),
        "Lock file should contain PID {expected_pid}"
    );
}

/// Assert that lock acquisition failed with timeout error
fn assert_timeout_error(result: Result<FileLock, FileLockError>) {
    assert!(result.is_err(), "Expected timeout error");
    match result.unwrap_err() {
        FileLockError::AcquisitionTimeout { .. } => {},
        other => panic!("Expected AcquisitionTimeout, got: {other:?}"),
    }
}

/// Assert that lock acquisition failed with timeout and check holder PID
fn assert_timeout_error_with_holder(
    result: Result<FileLock, FileLockError>,
    expected_holder: ProcessId,
) {
    assert!(result.is_err(), "Expected timeout error");
    match result.unwrap_err() {
        FileLockError::AcquisitionTimeout { holder_pid, .. } => {
            assert_eq!(
                holder_pid,
                Some(expected_holder),
                "Expected holder PID {expected_holder}"
            );
        }
        other => panic!("Expected AcquisitionTimeout, got: {other:?}"),
    }
}

/// Assert that lock acquisition failed with invalid lock file error
fn assert_invalid_lock_file_error(
    result: Result<FileLock, FileLockError>,
    expected_content: &str,
) {
    assert!(result.is_err(), "Expected invalid lock file error");
    match result.unwrap_err() {
        FileLockError::InvalidLockFile { content, .. } => {
            assert_eq!(content, expected_content);
        }
        other => panic!("Expected InvalidLockFile, got: {other:?}"),
    }
}
```

#### Benefits

- ✅ Tests become more declarative
- ✅ Reusable across different test modules
- ✅ Easier to add new assertions
- ✅ Better error messages when assertions fail

#### Implementation Checklist

- [ ] Add new assertion helper functions
- [ ] Refactor existing tests to use helpers
- [ ] Document assertion helpers
- [ ] Verify all tests pass
- [ ] Consider moving to shared test utilities if needed

---

## 🧪 Phase 2: Testability Enhancements (2-4 hours)

Medium-effort improvements that significantly enhance test quality and maintainability.

### Proposal #5: Introduce `LockAcquisitionState` Enum

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: None

#### Problem

The internal `acquire` logic is complex with multiple retry paths, making it hard to test specific scenarios in isolation. The state transitions during lock acquisition aren't explicitly modeled.

#### Proposed Solution

Extract the acquisition logic into a testable state machine:

```rust
/// Represents the state of lock acquisition process
///
/// This enum makes the lock acquisition state machine explicit and testable.
#[derive(Debug, PartialEq, Eq)]
enum LockAcquisitionState {
    /// Lock acquisition is being attempted
    Attempting,
    /// Found a lock held by a dead process (stale lock)
    FoundStaleLock(ProcessId),
    /// Lock is held by a live process
    Blocked(ProcessId),
    /// Lock was successfully acquired
    Acquired,
    /// Timeout occurred while waiting for lock
    TimedOut(ProcessId),
}

impl FileLock {
    /// Get the current state of lock acquisition (testable helper)
    ///
    /// This is primarily for testing to verify lock states without
    /// actually acquiring locks or waiting for timeouts.
    #[cfg(test)]
    fn check_lock_state(file_path: &Path) -> LockAcquisitionState {
        let lock_path = Self::lock_file_path(file_path);
        let current_pid = ProcessId::current();

        match Self::try_create_lock(&lock_path, current_pid) {
            Ok(()) => {
                // Clean up the lock file we just created for testing
                let _ = fs::remove_file(&lock_path);
                LockAcquisitionState::Acquired
            }
            Err(FileLockError::LockHeldByProcess { pid }) => {
                if pid.is_alive() {
                    LockAcquisitionState::Blocked(pid)
                } else {
                    LockAcquisitionState::FoundStaleLock(pid)
                }
            }
            Err(_) => LockAcquisitionState::Attempting,
        }
    }
}
```

#### Example Tests

```rust
#[test]
fn it_should_detect_acquired_state_when_no_lock_exists() {
    let scenario = TestLockScenario::new();
    let state = FileLock::check_lock_state(&scenario.file_path());
    assert_eq!(state, LockAcquisitionState::Acquired);
}

#[test]
fn it_should_detect_stale_lock_state() {
    let scenario = TestLockScenario::new();
    scenario.with_stale_lock(FAKE_DEAD_PROCESS_PID).unwrap();

    let state = FileLock::check_lock_state(&scenario.file_path());
    assert_eq!(state, LockAcquisitionState::FoundStaleLock(
        ProcessId::from_raw(FAKE_DEAD_PROCESS_PID)
    ));
}

#[test]
fn it_should_detect_blocked_state_when_lock_held() {
    let scenario = TestLockScenario::new();
    let _lock = scenario.acquire_lock().unwrap();

    let state = FileLock::check_lock_state(&scenario.file_path());
    assert_eq!(state, LockAcquisitionState::Blocked(ProcessId::current()));
}
```

#### Benefits

- ✅ Each state can be tested independently
- ✅ Better separation of concerns
- ✅ Easier to add new states in the future
- ✅ More explicit state machine logic
- ✅ Improved debugging capabilities

#### Implementation Checklist

- [ ] Add `LockAcquisitionState` enum
- [ ] Implement `check_lock_state()` test helper
- [ ] Add unit tests for state detection
- [ ] Consider refactoring `acquire()` to use states explicitly
- [ ] Update documentation
- [ ] Verify all tests pass

---

### Proposal #6: Make `ProcessId` Methods More Testable

**Status**: ❌ Cancelled  
**Impact**: 🟢🟢 Medium-High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1 (Deferred)  
**Depends On**: None

**Note**: This proposal has been deferred for future consideration. The current implementation is sufficient for now.

#### Problem

`ProcessId::is_alive()` calls platform-specific code that's hard to mock in tests. This makes tests:

- Dependent on actual process states
- Slower (requires real system calls)
- Less deterministic (race conditions with real processes)

#### Proposed Solution

Introduce a trait for process status checking with test-friendly implementation:

```rust
/// Trait for checking process liveness (mockable in tests)
trait ProcessChecker {
    fn is_alive(&self, pid: ProcessId) -> bool;
}

/// Production implementation using platform-specific checks
struct SystemProcessChecker;

impl ProcessChecker for SystemProcessChecker {
    fn is_alive(&self, pid: ProcessId) -> bool {
        platform::is_process_alive(pid)
    }
}

/// Test implementation with configurable process states
#[cfg(test)]
struct MockProcessChecker {
    alive_pids: std::collections::HashSet<u32>,
}

#[cfg(test)]
impl MockProcessChecker {
    fn new() -> Self {
        Self {
            alive_pids: std::collections::HashSet::new(),
        }
    }

    fn with_alive_pid(mut self, pid: u32) -> Self {
        self.alive_pids.insert(pid);
        self
    }

    fn mark_alive(&mut self, pid: u32) {
        self.alive_pids.insert(pid);
    }

    fn mark_dead(&mut self, pid: u32) {
        self.alive_pids.remove(&pid);
    }
}

#[cfg(test)]
impl ProcessChecker for MockProcessChecker {
    fn is_alive(&self, pid: ProcessId) -> bool {
        self.alive_pids.contains(&pid.as_u32())
    }
}

impl ProcessId {
    /// Check if this process is currently alive (uses system checker)
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.is_alive_with(&SystemProcessChecker)
    }

    /// Check if process is alive with custom checker (primarily for testing)
    #[cfg(test)]
    fn is_alive_with(&self, checker: &dyn ProcessChecker) -> bool {
        checker.is_alive(*self)
    }
}
```

#### Example Tests

```rust
#[test]
fn it_should_detect_live_process_with_mock_checker() {
    let checker = MockProcessChecker::new()
        .with_alive_pid(12345);

    let pid = ProcessId::from_raw(12345);
    assert!(pid.is_alive_with(&checker));
}

#[test]
fn it_should_detect_dead_process_with_mock_checker() {
    let checker = MockProcessChecker::new(); // No alive PIDs

    let pid = ProcessId::from_raw(99999);
    assert!(!pid.is_alive_with(&checker));
}

#[test]
fn it_should_handle_process_state_changes() {
    let mut checker = MockProcessChecker::new();
    let pid = ProcessId::from_raw(54321);

    // Initially dead
    assert!(!pid.is_alive_with(&checker));

    // Mark alive
    checker.mark_alive(54321);
    assert!(pid.is_alive_with(&checker));

    // Mark dead again
    checker.mark_dead(54321);
    assert!(!pid.is_alive_with(&checker));
}
```

#### Benefits

- ✅ Testable without spawning real processes
- ✅ Faster tests (no system calls)
- ✅ More deterministic test behavior
- ✅ Easier to test edge cases
- ✅ Better separation of concerns

#### Implementation Checklist

- [ ] Add `ProcessChecker` trait
- [ ] Implement `SystemProcessChecker`
- [ ] Implement `MockProcessChecker` for tests
- [ ] Update `ProcessId::is_alive()` implementation
- [ ] Add tests for mock checker
- [ ] Refactor existing tests to use mock where appropriate
- [ ] Verify all tests pass
- [ ] Update documentation

---

### Proposal #7: Consolidate Duplicate Test Setup Code

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Low-Medium  
**Priority**: P1  
**Depends On**: Proposal #3 (shares similar goals)

#### Problem

Several tests repeat similar setup patterns for creating temp directories, file paths, and acquiring locks. This violates the DRY principle and makes tests harder to maintain.

#### Proposed Solution

Create a dedicated test fixture module:

```rust
#[cfg(test)]
mod test_fixtures {
    use super::*;

    /// Setup for basic lock tests - returns temp dir and file path
    pub fn basic_lock_setup() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new()
            .expect("Failed to create temporary directory for test");
        let file_path = temp_dir.path().join("test.json");
        (temp_dir, file_path)
    }

    /// Setup for concurrent lock tests - returns temp dir and two cloned paths
    pub fn concurrent_lock_setup(file_name: &str) -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new()
            .expect("Failed to create temporary directory for concurrent test");
        let file_path = temp_dir.path().join(file_name);
        let clone = file_path.clone();
        (temp_dir, file_path, clone)
    }

    /// Create a lock and verify it was acquired successfully
    pub fn acquire_and_verify(file_path: &Path, timeout: Duration) -> FileLock {
        let lock = FileLock::acquire(file_path, timeout)
            .expect("Failed to acquire lock in test setup");
        assert_lock_file_contains_current_pid(file_path);
        lock
    }

    /// Setup for stale lock testing - creates file with dead process PID
    pub fn setup_stale_lock(file_path: &Path, fake_pid: u32) {
        let lock_file_path = FileLock::lock_file_path(file_path);
        fs::write(&lock_file_path, fake_pid.to_string())
            .expect("Failed to create stale lock file for test");
    }

    /// Setup for invalid lock testing - creates file with invalid content
    pub fn setup_invalid_lock(file_path: &Path, content: &str) {
        let lock_file_path = FileLock::lock_file_path(file_path);
        fs::write(&lock_file_path, content)
            .expect("Failed to create invalid lock file for test");
    }
}
```

#### Example Usage

```rust
#[test]
fn it_should_successfully_acquire_lock() {
    // Arrange
    let (_temp_dir, file_path) = test_fixtures::basic_lock_setup();

    // Act
    let lock = FileLock::acquire(&file_path, Duration::from_secs(1));

    // Assert
    assert!(lock.is_ok());
    let lock = lock.expect("Lock should have been acquired");
    assert!(lock.acquired);
    assert_lock_file_contains_current_pid(&file_path);
}

#[test]
fn it_should_clean_up_stale_lock_with_invalid_pid() {
    // Arrange
    let (_temp_dir, file_path) = test_fixtures::basic_lock_setup();
    test_fixtures::setup_stale_lock(&file_path, FAKE_DEAD_PROCESS_PID);

    // Act
    let lock_result = FileLock::acquire(&file_path, Duration::from_secs(1));

    // Assert
    assert!(lock_result.is_ok());
    assert_lock_file_contains_current_pid(&file_path);
}
```

#### Benefits

- ✅ Follows DRY principle
- ✅ Consistent test setup across all tests
- ✅ Easier refactoring when setup needs change
- ✅ More readable test code
- ✅ Reduces boilerplate

#### Implementation Checklist

- [ ] Create `test_fixtures` module
- [ ] Implement setup functions
- [ ] Refactor existing tests to use fixtures
- [ ] Verify all tests pass
- [ ] Document fixture patterns in testing conventions
- [ ] Consider coordination with Proposal #3

---

## 🎨 Phase 3: Polish and Enhancement (4-6 hours)

Higher-effort improvements focusing on error handling, organization, and comprehensive testing.

### Proposal #8: Improve Error Context with Structured Fields

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵🔵 Medium  
**Priority**: P2  
**Depends On**: None

#### Problem

Some errors lack actionable guidance per the project's development principles (Observability, Traceability, Actionability). Users need clear steps to resolve issues when locks fail.

#### Current State

Errors provide basic information but lack actionable guidance:

```rust
#[error("Failed to acquire lock for {path:?} within {timeout:?} (held by process {holder_pid:?})")]
AcquisitionTimeout {
    path: PathBuf,
    holder_pid: Option<ProcessId>,
    timeout: Duration,
},
```

#### Proposed Solution

Enhance error messages with actionable guidance and troubleshooting steps:

```rust
#[derive(Debug, Error)]
pub enum FileLockError {
    #[error("Failed to acquire lock for '{path}' within {timeout:?}

The lock is currently held by process {holder_pid}.

To resolve this issue:
1. Check if process {holder_pid} is still running:
   ps -p {holder_pid}

2. If the process is running and should release the lock:
   - Wait for the process to complete its operation
   - Or increase the timeout duration

3. If the process is stuck or hung:
   - Terminate it: kill {holder_pid}
   - Or force terminate: kill -9 {holder_pid}

4. If the process doesn't exist (stale lock):
   - This should be handled automatically
   - If you see this error repeatedly, report a bug

Current timeout: {timeout:?}
Lock file: {path}.lock")]
    AcquisitionTimeout {
        path: PathBuf,
        holder_pid: ProcessId, // Changed from Option to required
        timeout: Duration,
    },

    #[error("Failed to create lock file at '{path}'

Possible causes and solutions:
1. Insufficient permissions:
   - Check directory permissions: ls -la {parent_dir}
   - Ensure write access: chmod u+w {parent_dir}

2. Parent directory doesn't exist:
   - Create it: mkdir -p {parent_dir}

3. Disk full:
   - Check disk space: df -h
   - Free up space or use a different location

4. File system issue:
   - Check file system health
   - Try a different directory

Original error: {source}
Lock file path: {path}.lock")]
    CreateFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read lock file at '{path}'

This may indicate:
1. File system corruption
2. Permission changes after lock creation
3. Concurrent file deletion

Troubleshooting:
- Check if file exists: ls -la {path}.lock
- Check file permissions: stat {path}.lock
- Check file system status: df -h

If this error persists, the lock may be corrupted.
You can manually remove it: rm {path}.lock

Original error: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid lock file content at '{path}'

Expected: Process ID (numeric value)
Found: '{content}'

This indicates:
1. Manual modification of lock file (not recommended)
2. File system corruption
3. Lock file created by incompatible software

To resolve:
- Remove the invalid lock file: rm {path}.lock
- Let the system recreate it properly
- Ensure no manual edits to .lock files

Lock file path: {path}.lock")]
    InvalidLockFile {
        path: PathBuf,
        content: String
    },

    #[error("Failed to release lock file at '{path}'

This is a cleanup error and may not affect functionality,
but the lock file may persist.

Possible causes:
1. File was already deleted (race condition)
2. Permission changed after lock creation
3. File system issue

Manual cleanup:
- Remove lock file: rm {path}.lock

Original error: {source}")]
    ReleaseFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
```

#### Implementation Notes

**Helper method for parent directory**: Add a method to extract parent directory for error messages:

```rust
impl FileLockError {
    fn parent_dir(path: &Path) -> String {
        path.parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string())
    }
}
```

**Platform-specific commands**: Consider adding platform-specific guidance:

```rust
#[cfg(unix)]
fn get_process_check_command(pid: ProcessId) -> String {
    format!("ps -p {pid}")
}

#[cfg(windows)]
fn get_process_check_command(pid: ProcessId) -> String {
    format!("tasklist /FI \"PID eq {pid}\"")
}
```

#### Benefits

- ✅ Aligns with "Actionability" development principle
- ✅ Better user experience for error scenarios
- ✅ Reduces support burden and debugging time
- ✅ Self-documenting error handling
- ✅ Platform-aware guidance

#### Testing Requirements

Add tests to verify error message content:

```rust
#[test]
fn it_should_include_troubleshooting_steps_in_timeout_error() {
    let error = FileLockError::AcquisitionTimeout {
        path: PathBuf::from("/test/file.json"),
        holder_pid: ProcessId::from_raw(12345),
        timeout: Duration::from_secs(5),
    };

    let message = error.to_string();
    assert!(message.contains("To resolve this issue"));
    assert!(message.contains("ps -p"));
    assert!(message.contains("kill"));
}
```

#### Implementation Checklist

- [ ] Update error variant definitions with enhanced messages
- [ ] Add helper methods for path formatting
- [ ] Add platform-specific command helpers
- [ ] Update error construction sites if needed
- [ ] Add tests for error message content
- [ ] Update documentation
- [ ] Verify all tests pass
- [ ] Run linters

---

### Proposal #9: Extract Platform Module to Separate File

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵🔵 Medium  
**Priority**: P2  
**Depends On**: None

#### Problem

The `platform` module is currently embedded within `file_lock.rs`, mixing platform-specific logic with the main file locking logic. This makes the module harder to navigate and test.

#### Current Structure

```text
src/infrastructure/persistence/filesystem/
├── file_lock.rs (700+ lines including platform module)
└── mod.rs
```

#### Proposed Structure

```text
src/infrastructure/persistence/filesystem/
├── file_lock.rs (main logic)
├── platform.rs (platform-specific process checking)
└── mod.rs
```

#### Proposed Implementation

**File: `src/infrastructure/persistence/filesystem/platform.rs`**

````rust
//! Platform-specific process checking functionality
//!
//! This module provides a unified interface for checking if a process is alive
//! across different operating systems (Unix and Windows).
//!
//! # Platform Support
//!
//! - **Unix/Linux/macOS**: Uses `kill -0` signal to check process existence
//! - **Windows**: Uses `tasklist` command to query running processes
//!
//! # Design
//!
//! The implementation uses command-line tools rather than OS APIs because:
//! - Simple and portable across different Unix flavors
//! - No additional dependencies required
//! - Sufficient for our use case (not performance-critical)

use crate::infrastructure::persistence::filesystem::file_lock::ProcessId;
use std::process::Command;

/// Check if a process with the given PID is currently running
///
/// # Platform Behavior
///
/// ## Unix/Linux/macOS
/// Uses `kill -0` which sends signal 0 to the process.
/// - Returns `true` if process exists and is accessible
/// - Returns `false` if process doesn't exist or permission denied
/// - Signal 0 doesn't actually send a signal, just checks permissions
///
/// ## Windows
/// Uses `tasklist /FI "PID eq {pid}"` to query process status.
/// - Returns `true` if process appears in task list
/// - Returns `false` if process doesn't exist
///
/// # Arguments
///
/// * `pid` - Process ID to check
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::infrastructure::persistence::filesystem::platform;
/// use torrust_tracker_deploy::infrastructure::persistence::filesystem::file_lock::ProcessId;
///
/// let current_pid = ProcessId::current();
/// assert!(platform::is_process_alive(current_pid));
/// ```
#[cfg(unix)]
pub fn is_process_alive(pid: ProcessId) -> bool {
    // On Unix, we can send signal 0 to check if process exists
    // This doesn't actually send a signal, just checks permissions
    match Command::new("kill")
        .arg("-0")
        .arg(pid.as_u32().to_string())
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Check if a process with the given PID is currently running (Windows)
///
/// See main documentation for `is_process_alive`.
#[cfg(windows)]
pub fn is_process_alive(pid: ProcessId) -> bool {
    // On Windows, try to query the process using tasklist
    Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {}", pid.as_u32()))
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .contains(&pid.as_u32().to_string())
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_detect_current_process_as_alive() {
        let current_pid = ProcessId::current();
        assert!(
            is_process_alive(current_pid),
            "Current process should always be detected as alive"
        );
    }

    #[test]
    fn it_should_detect_fake_process_as_dead() {
        // Use a PID that is very unlikely to exist
        let fake_pid = ProcessId::from_raw(999_999);
        assert!(
            !is_process_alive(fake_pid),
            "Fake PID 999999 should not be detected as alive"
        );
    }

    #[test]
    fn it_should_handle_pid_1_correctly() {
        // PID 1 is init/systemd on Unix, System on Windows
        // It should always be alive
        #[cfg(unix)]
        {
            let init_pid = ProcessId::from_raw(1);
            // On Unix, PID 1 is always the init process
            assert!(
                is_process_alive(init_pid),
                "PID 1 (init) should always be alive on Unix"
            );
        }

        // On Windows, PID 1 may or may not exist, so we don't test it
    }
}
````

**File: `src/infrastructure/persistence/filesystem/file_lock.rs`** (updated)

```rust
// Remove the embedded platform module and add import
use super::platform;

// Rest of the file remains the same
```

**File: `src/infrastructure/persistence/filesystem/mod.rs`** (updated)

```rust
pub mod file_lock;
mod platform; // Add this line

// Re-export if needed
pub use file_lock::{FileLock, FileLockError, ProcessId};
```

#### Benefits

- ✅ Better separation of concerns
- ✅ Easier to test platform-specific code in isolation
- ✅ Cleaner main module (reduced line count)
- ✅ More discoverable code organization
- ✅ Easier to add new platform-specific functionality

#### Implementation Checklist

- [ ] Create `platform.rs` file with module documentation
- [ ] Move platform-specific code from `file_lock.rs`
- [ ] Add tests to `platform.rs`
- [ ] Update imports in `file_lock.rs`
- [ ] Update `mod.rs` to include platform module
- [ ] Verify all tests pass
- [ ] Run linters
- [ ] Update documentation references if needed

---

### Proposal #10: Add Multi-Process Integration Tests

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵🔵 Medium-High  
**Priority**: P2  
**Depends On**: None

#### Problem

Current tests mostly simulate concurrency within a single process using threads. While useful, these don't test true inter-process locking scenarios, which are the primary use case for file locks.

#### Current Test Coverage

- ✅ Single process lock acquisition and release
- ✅ Thread-based concurrent access (same process)
- ✅ Stale lock detection
- ❌ **Missing**: True multi-process locking scenarios
- ❌ **Missing**: Lock handoff between processes
- ❌ **Missing**: Process crash scenarios

#### Proposed Solution

Add integration tests in the `tests/` directory that spawn actual child processes:

**File: `tests/file_lock_multiprocess.rs`**

````rust
//! Multi-Process Integration Tests for File Locking
//!
//! These tests spawn actual child processes to verify true inter-process locking
//! behavior. They are more comprehensive than unit tests but slower to run.
//!
//! # Running These Tests
//!
//! ```bash
//! # Run all multi-process tests
//! cargo test --test file_lock_multiprocess
//!
//! # Run with verbose output
//! cargo test --test file_lock_multiprocess -- --nocapture
//! ```
//!
//! # Test Strategy
//!
//! 1. Spawn child processes that hold locks for specific durations
//! 2. Verify parent process cannot acquire while child holds lock
//! 3. Verify lock is released when child exits
//! 4. Test crash scenarios by killing child processes

use std::path::PathBuf;
use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use torrust_tracker_deploy::infrastructure::persistence::filesystem::file_lock::{
    FileLock, FileLockError,
};

/// Helper to spawn a child process that holds a lock
fn spawn_lock_holder(lock_file: &PathBuf, duration_secs: u64) -> Child {
    Command::new(env!("CARGO_BIN_EXE_lock_holder_helper"))
        .arg(lock_file.to_str().expect("Invalid path"))
        .arg(duration_secs.to_string())
        .spawn()
        .expect("Failed to spawn child process")
}

#[test]
fn it_should_prevent_lock_acquisition_across_processes() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("cross_process.json");

    // Spawn child process that holds lock for 2 seconds
    let mut child = spawn_lock_holder(&lock_file, 2);

    // Give child time to acquire lock
    thread::sleep(Duration::from_millis(100));

    // Act: Try to acquire in parent - should fail
    let result = FileLock::acquire(&lock_file, Duration::from_millis(500));

    // Assert
    assert!(
        matches!(result, Err(FileLockError::AcquisitionTimeout { .. })),
        "Should timeout when child process holds lock"
    );

    // Cleanup
    child.wait().expect("Failed to wait for child");
}

#[test]
fn it_should_acquire_lock_after_child_releases() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("handoff.json");

    // Spawn child that holds lock for 1 second
    let mut child = spawn_lock_holder(&lock_file, 1);

    // Give child time to acquire
    thread::sleep(Duration::from_millis(100));

    // Act: Try to acquire with 3 second timeout (child releases after 1 second)
    let result = FileLock::acquire(&lock_file, Duration::from_secs(3));

    // Assert: Should succeed after retries
    assert!(
        result.is_ok(),
        "Should eventually acquire after child releases lock"
    );

    // Cleanup
    child.wait().expect("Failed to wait for child");
}

#[test]
#[cfg(unix)] // Test Unix-specific crash handling
fn it_should_clean_up_stale_lock_after_process_crash() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("crash.json");

    // Spawn child that holds lock
    let mut child = spawn_lock_holder(&lock_file, 10); // Long duration

    // Give child time to acquire
    thread::sleep(Duration::from_millis(100));

    // Act: Kill child process (simulating crash)
    child.kill().expect("Failed to kill child process");
    child.wait().expect("Failed to wait for child");

    // Small delay to ensure OS registers process death
    thread::sleep(Duration::from_millis(50));

    // Try to acquire lock - should succeed by cleaning stale lock
    let result = FileLock::acquire(&lock_file, Duration::from_secs(2));

    // Assert
    assert!(
        result.is_ok(),
        "Should clean up stale lock from crashed process"
    );
}

#[test]
fn it_should_handle_rapid_lock_handoff_between_processes() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("rapid_handoff.json");

    // Act: Spawn multiple children that quickly acquire and release
    let mut children = vec![];
    for _ in 0..5 {
        let mut child = spawn_lock_holder(&lock_file, 1);
        thread::sleep(Duration::from_millis(50));
        children.push(child);
    }

    // Wait for all children to complete
    for child in children.iter_mut() {
        child.wait().expect("Failed to wait for child");
    }

    // Assert: Should be able to acquire after all children finish
    let result = FileLock::acquire(&lock_file, Duration::from_secs(1));
    assert!(
        result.is_ok(),
        "Should acquire lock after rapid handoffs"
    );
}
````

**File: `src/bin/lock_holder_helper.rs`** (helper binary for tests)

````rust
//! Helper binary for multi-process lock testing
//!
//! This binary acquires a lock and holds it for a specified duration,
//! allowing integration tests to verify inter-process locking behavior.
//!
//! # Usage
//!
//! ```bash
//! lock_holder_helper <file_path> <duration_seconds>
//! ```
//!
//! # Example
//!
//! ```bash
//! # Hold lock on test.json for 5 seconds
//! lock_holder_helper ./test.json 5
//! ```

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use torrust_tracker_deploy::infrastructure::persistence::filesystem::file_lock::FileLock;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <duration_seconds>", args[0]);
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);
    let duration_secs: u64 = args[2].parse()
        .expect("Duration must be a number");

    println!("Acquiring lock on {:?}", file_path);
    let _lock = FileLock::acquire(&file_path, Duration::from_secs(10))?;
    println!("Lock acquired, holding for {} seconds", duration_secs);

    thread::sleep(Duration::from_secs(duration_secs));

    println!("Releasing lock");
    Ok(())
}
````

**File: `Cargo.toml`** (add test binary)

```toml
[[bin]]
name = "lock_holder_helper"
path = "src/bin/lock_holder_helper.rs"
# Only build for tests
required-features = [] # Available for tests
```

#### Benefits

- ✅ Tests real-world inter-process scenarios
- ✅ Catches platform-specific issues
- ✅ Verifies stale lock cleanup in crash scenarios
- ✅ Higher confidence in production behavior
- ✅ Documents expected behavior with real processes

#### Implementation Checklist

- [ ] Create helper binary `lock_holder_helper.rs`
- [ ] Create integration test file `tests/file_lock_multiprocess.rs`
- [ ] Add test for basic inter-process blocking
- [ ] Add test for lock handoff between processes
- [ ] Add test for crash/stale lock cleanup
- [ ] Add test for rapid handoff scenarios
- [ ] Update `Cargo.toml` to build helper binary
- [ ] Document how to run these tests
- [ ] Add to CI pipeline if appropriate
- [ ] Verify all tests pass

---

## 🔬 Phase 4: Advanced Observability (Optional, 2-3 hours)

Advanced improvements for production observability and debugging.

### Proposal #11: Add Tracing Spans for Lock Operations

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵🔵 Medium  
**Priority**: P3 (Optional)  
**Depends On**: None

#### Problem

Currently, lock operations have limited observability. When debugging issues in production, it's hard to trace:

- Which locks are being acquired and when
- How long lock acquisition takes
- Why locks are timing out
- The sequence of lock operations across the system

#### Proposed Solution

Add structured tracing throughout the lock lifecycle using the `tracing` crate:

```rust
use tracing::{debug, trace, warn, instrument};

impl FileLock {
    /// Attempt to acquire a lock for the given file path
    #[instrument(
        name = "file_lock_acquire",
        skip(file_path),
        fields(
            file = %file_path.display(),
            timeout_ms = timeout.as_millis(),
            pid = %ProcessId::current(),
        )
    )]
    pub fn acquire(file_path: &Path, timeout: Duration) -> Result<Self, FileLockError> {
        debug!("Attempting to acquire lock");

        let lock_file_path = Self::lock_file_path(file_path);
        let current_pid = ProcessId::current();
        let retry_strategy = LockRetryStrategy::new(timeout);

        trace!(
            lock_file = %lock_file_path.display(),
            "Lock file path determined"
        );

        let mut attempt = 0;
        loop {
            attempt += 1;
            trace!(attempt, "Lock acquisition attempt");

            match Self::try_acquire_once(&lock_file_path, current_pid) {
                AcquireAttemptResult::Success => {
                    debug!(attempts = attempt, "Lock acquired successfully");
                    return Ok(Self {
                        lock_file_path,
                        acquired: true,
                    });
                }
                AcquireAttemptResult::StaleProcess(pid) => {
                    warn!(
                        stale_pid = %pid,
                        attempt,
                        "Detected stale lock, cleaning up"
                    );
                    drop(fs::remove_file(&lock_file_path));
                }
                AcquireAttemptResult::HeldByLiveProcess(pid) => {
                    trace!(
                        holder_pid = %pid,
                        attempt,
                        elapsed_ms = retry_strategy.start.elapsed().as_millis(),
                        "Lock held by live process"
                    );

                    if retry_strategy.is_expired() {
                        warn!(
                            holder_pid = %pid,
                            attempts = attempt,
                            timeout_ms = timeout.as_millis(),
                            "Lock acquisition timed out"
                        );
                        return Err(FileLockError::AcquisitionTimeout {
                            path: lock_file_path,
                            holder_pid: Some(pid),
                            timeout,
                        });
                    }
                    LockRetryStrategy::wait();
                }
                AcquireAttemptResult::Error(e) => {
                    warn!(
                        error = %e,
                        attempts = attempt,
                        "Error during lock acquisition"
                    );
                    return Err(e);
                }
            }
        }
    }

    /// Release the lock by removing the lock file
    #[instrument(
        name = "file_lock_release",
        skip(self),
        fields(lock_file = %self.lock_file_path.display())
    )]
    pub fn release(mut self) -> Result<(), FileLockError> {
        debug!("Releasing lock");

        if self.acquired {
            fs::remove_file(&self.lock_file_path).map_err(|source| {
                warn!(error = %source, "Failed to remove lock file");
                FileLockError::ReleaseFailed {
                    path: self.lock_file_path.clone(),
                    source,
                }
            })?;
            self.acquired = false;
            debug!("Lock released successfully");
        } else {
            trace!("Lock was not acquired, nothing to release");
        }
        Ok(())
    }

    /// Try to create lock file atomically with current process ID
    #[instrument(
        name = "file_lock_try_create",
        skip(lock_path),
        fields(lock_file = %lock_path.display(), pid = %pid)
    )]
    fn try_create_lock(lock_path: &Path, pid: ProcessId) -> Result<(), FileLockError> {
        trace!("Attempting to create lock file");

        use std::fs::OpenOptions;
        use std::io::Write;

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(mut file) => {
                trace!("Lock file created, writing PID");
                write!(file, "{pid}").map_err(|source| {
                    warn!(error = %source, "Failed to write PID to lock file");
                    FileLockError::CreateFailed {
                        path: lock_path.to_path_buf(),
                        source,
                    }
                })?;
                debug!("Lock file created successfully");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                trace!("Lock file already exists, reading holder PID");

                let content =
                    fs::read_to_string(lock_path).map_err(|source| {
                        warn!(error = %source, "Failed to read existing lock file");
                        FileLockError::ReadFailed {
                            path: lock_path.to_path_buf(),
                            source,
                        }
                    })?;

                let holder_pid = content.trim().parse::<ProcessId>().map_err(|_| {
                    warn!(content = %content, "Invalid PID content in lock file");
                    FileLockError::InvalidLockFile {
                        path: lock_path.to_path_buf(),
                        content,
                    }
                })?;

                trace!(holder_pid = %holder_pid, "Lock held by process");
                Err(FileLockError::LockHeldByProcess { pid: holder_pid })
            }
            Err(source) => {
                warn!(error = %source, "Failed to create lock file");
                Err(FileLockError::CreateFailed {
                    path: lock_path.to_path_buf(),
                    source,
                })
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if self.acquired {
            if let Err(e) = fs::remove_file(&self.lock_file_path) {
                warn!(
                    path = ?self.lock_file_path,
                    error = %e,
                    "Failed to remove lock file during drop"
                );
            } else {
                trace!(
                    path = ?self.lock_file_path,
                    "Lock file removed during drop"
                );
            }
            self.acquired = false;
        }
    }
}
```

#### Example Tracing Output

With tracing configured, you would see:

```text
DEBUG file_lock_acquire{file="./data/env/state.json" timeout_ms=5000 pid=12345}: Attempting to acquire lock
TRACE file_lock_acquire{file="./data/env/state.json" timeout_ms=5000 pid=12345}: lock_file="./data/env/state.json.lock" Lock file path determined
TRACE file_lock_acquire{file="./data/env/state.json" timeout_ms=5000 pid=12345}: attempt=1 Lock acquisition attempt
TRACE file_lock_try_create{lock_file="./data/env/state.json.lock" pid=12345}: Attempting to create lock file
TRACE file_lock_try_create{lock_file="./data/env/state.json.lock" pid=12345}: Lock file created, writing PID
DEBUG file_lock_try_create{lock_file="./data/env/state.json.lock" pid=12345}: Lock file created successfully
DEBUG file_lock_acquire{file="./data/env/state.json" timeout_ms=5000 pid=12345}: attempts=1 Lock acquired successfully
```

#### Benefits

- ✅ Enhanced observability in production
- ✅ Easy debugging of lock contention issues
- ✅ Performance analysis (lock wait times)
- ✅ Audit trail of lock operations
- ✅ Integration with existing tracing infrastructure
- ✅ Aligns with "Observability" development principle

#### Testing Tracing

Add tests to verify tracing behavior:

```rust
#[cfg(test)]
mod tracing_tests {
    use super::*;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    #[test]
    fn it_should_emit_trace_events_during_lock_operations() {
        // Setup tracing subscriber for testing
        let (subscriber, handle) = tracing_subscriber::fmt()
            .with_test_writer()
            .finish()
            .with_subscriber();

        let _guard = tracing::subscriber::set_default(subscriber);

        // Perform lock operation
        let scenario = TestLockScenario::new();
        let _lock = scenario.acquire_lock().unwrap();

        // Verify trace events were emitted
        // (Implementation depends on your tracing testing strategy)
    }
}
```

#### Implementation Checklist

- [ ] Add `#[instrument]` attributes to public methods
- [ ] Add strategic `trace!`, `debug!`, `warn!` calls
- [ ] Add tracing context (attempt counts, PIDs, timings)
- [ ] Test tracing output in development
- [ ] Document tracing levels and output
- [ ] Add tracing tests if applicable
- [ ] Update observability documentation
- [ ] Verify all tests pass
- [ ] Run linters

---

## 📅 Implementation Timeline

### Estimated Timeline by Phase

| Phase                        | Duration  | Cumulative |
| ---------------------------- | --------- | ---------- |
| Phase 1: Quick Wins          | 1-2 hours | 1-2 hours  |
| Phase 2: Testability         | 1-2 hours | 2-4 hours  |
| Phase 3: Polish              | 4-6 hours | 6-10 hours |
| Phase 4: Advanced (Optional) | 2-3 hours | 8-13 hours |

**Total Estimated Time**: 8-13 hours (excluding Proposals #6 and #12)

### Recommended Sprint Planning

**Sprint 1** (Week 1):

- Complete Phase 1 (all 4 proposals)
- Begin Phase 2 (Proposals #5 and #7)

**Sprint 2** (Week 2):

- Complete Phase 2 (Proposal #7)
- Begin Phase 3 (Proposal #8)

**Sprint 3** (Week 3):

- Complete Phase 3 (Proposals #9-10)
- Optional: Phase 4 (Proposal #11) if time permits

---

## 🔄 Review and Approval Process

### Before Implementation

- [ ] Review all proposals for technical feasibility
- [ ] Validate alignment with project principles
- [ ] Confirm priority ordering
- [ ] Approve or request modifications
- [ ] Set implementation timeline

### During Implementation

- [ ] Create GitHub issue for tracking (optional)
- [ ] Create feature branch from main
- [ ] Implement proposals in phase order
- [ ] Run all tests after each proposal
- [ ] Run all linters after each proposal
- [ ] Commit after each completed proposal

### After Implementation

- [ ] Final test run across all changes
- [ ] Update this document with completion status
- [ ] Create pull request for review
- [ ] Address review feedback
- [ ] Merge to main branch

---

## 📝 Notes and Decisions

### Decisions Made

- **Date**: October 2, 2025
- **Decision**: Exclude Proposal #12 (Performance optimizations) - Not a priority
- **Rationale**: Focus on maintainability and readability over performance

### Open Questions

- Should we create separate PRs for each phase or one large PR?
- Do we need to update any related documentation beyond this file?
- Should multi-process tests be run in CI or only locally?

### References

- [Project Development Principles](../development-principles.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Commit Process](../contributing/commit-process.md)

---

## 📞 Contact

For questions or discussions about this refactoring plan:

- Open a GitHub issue with label `refactoring`
- Mention this document: `docs/refactors/file-lock-improvements.md`
- Tag relevant maintainers

---

**Last Updated**: October 2, 2025  
**Status**: 📋 Awaiting Review and Approval
