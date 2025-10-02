# Refactoring: File Lock Module (`file_lock.rs`)

**Status:** In Progress  
**Created:** October 2, 2025  
**Module:** `src/infrastructure/persistence/filesystem/file_lock.rs`  
**Branch:** `environment-state-management`

## 📋 Overview

This document tracks the refactoring of the file locking module to improve code cleanliness, maintainability, readability, and testability. The proposals are organized by impact-to-effort ratio, with high-impact, low-effort improvements implemented first.

## 🎯 Goals

- Make the code cleaner and more maintainable
- Improve code readability
- Enhance testability with better test organization
- Align with project development principles (observability, traceability, actionability)
- Follow project testing conventions

## 📊 Progress Tracking

| #   | Proposal                                       | Priority                     | Status         | Commit |
| --- | ---------------------------------------------- | ---------------------------- | -------------- | ------ |
| 1   | Extract magic numbers to named constants       | High Impact, Low Effort      | ⬜ Not Started | -      |
| 2   | Use rstest for parameterized lock path tests   | High Impact, Low Effort      | ⬜ Not Started | -      |
| 3   | Extract test helper for lock file verification | High Impact, Low Effort      | ⬜ Not Started | -      |
| 4   | Improve error context in Drop with tracing     | High Impact, Low Effort      | ⬜ Not Started | -      |
| 5   | Extract lock acquisition retry logic           | Medium Impact, Medium Effort | ⬜ Not Started | -      |
| 6   | Improve test naming and organization           | Medium Impact, Medium Effort | ⬜ Not Started | -      |
| 7   | Add builder pattern for test configuration     | Medium Impact, Medium Effort | ⬜ Not Started | -      |
| 8   | Add type safety for process IDs                | Lower Priority               | ⬜ Not Started | -      |
| 9   | Improve platform-specific code organization    | Lower Priority               | ⬜ Not Started | -      |
| 10  | Add documentation for testing best practices   | Lower Priority               | ⬜ Not Started | -      |

**Legend:**

- ⬜ Not Started
- 🔄 In Progress
- ✅ Completed
- ⏭️ Skipped

---

## 📝 Detailed Proposals

### Proposal 1: Extract Magic Numbers to Named Constants ⭐⭐⭐

**Priority:** High Impact, Low Effort  
**Status:** ⬜ Not Started

#### Current Issue

Magic numbers scattered throughout the code:

- `Duration::from_millis(100)` - retry sleep interval (line 136)
- `Duration::from_millis(200)` - test timeout (line 497)
- Various other durations in tests without context

#### Solution

Extract to named constants at module level:

```rust
// At module level, after imports
const LOCK_RETRY_INTERVAL_MS: u64 = 100;
const LOCK_RETRY_SLEEP: Duration = Duration::from_millis(LOCK_RETRY_INTERVAL_MS);
```

Update usage in `acquire()`:

```rust
// Wait a bit before retrying
std::thread::sleep(LOCK_RETRY_SLEEP);
```

#### Benefits

- Named constants clarify intent
- Easier to adjust timing behavior globally
- Reduces magic numbers scattered in code
- Self-documenting code

#### Testing Strategy

- Run existing tests - no behavior changes
- All tests should pass unchanged

---

### Proposal 2: Use rstest for Parameterized Lock Path Tests ⭐⭐⭐

**Priority:** High Impact, Low Effort  
**Status:** ⬜ Not Started

#### Current Issue

Test uses loop pattern that doesn't provide good isolation or test output:

```rust
#[test]
fn it_should_generate_correct_lock_file_path() {
    let test_cases = vec![
        ("test.json", "test.json.lock"),
        // ... more cases
    ];
    for (input, expected) in test_cases {
        // Test logic
    }
}
```

#### Solution

Use `rstest` for parameterized testing as per project conventions:

```rust
use rstest::rstest;

#[rstest]
#[case("test.json", "test.json.lock")]
#[case("data/state.json", "data/state.json.lock")]
#[case("/abs/path/file.txt", "/abs/path/file.txt.lock")]
#[case("no_extension", "no_extension.lock")]
fn it_should_generate_correct_lock_file_path(
    #[case] input: &str,
    #[case] expected: &str
) {
    let input_path = Path::new(input);
    let lock_path = FileLock::lock_file_path(input_path);
    assert_eq!(lock_path.to_string_lossy(), expected);
}
```

#### Benefits

- Each case runs independently
- Clearer test output showing which case failed
- Better parallel execution
- Aligns with project testing conventions (`docs/contributing/testing.md`)

#### Testing Strategy

- Run tests before and after conversion
- Verify all cases still pass
- Check that individual test cases appear in output

#### Dependencies

- `rstest` crate already in dev-dependencies

---

### Proposal 3: Extract Test Helper for Lock File Verification ⭐⭐⭐

**Priority:** High Impact, Low Effort  
**Status:** ⬜ Not Started

#### Current Issue

Repeated verification pattern across multiple tests:

```rust
let lock_file_path = FileLock::lock_file_path(&file_path);
assert!(lock_file_path.exists());
let pid_content = fs::read_to_string(&lock_file_path).unwrap();
assert_eq!(pid_content, process::id().to_string());
```

This appears in at least 3 tests.

#### Solution

Create test helper module with verification functions:

```rust
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn assert_lock_file_contains_current_pid(file_path: &Path) {
        let lock_file_path = FileLock::lock_file_path(file_path);
        assert!(
            lock_file_path.exists(),
            "Lock file should exist at {:?}",
            lock_file_path
        );

        let pid_content = fs::read_to_string(&lock_file_path)
            .expect("Should be able to read lock file");
        assert_eq!(
            pid_content,
            process::id().to_string(),
            "Lock file should contain current process ID"
        );
    }

    pub fn assert_lock_file_absent(file_path: &Path) {
        let lock_file_path = FileLock::lock_file_path(file_path);
        assert!(
            !lock_file_path.exists(),
            "Lock file should not exist at {:?}",
            lock_file_path
        );
    }
}
```

Update tests to use helpers.

#### Benefits

- DRY principle - single source of verification logic
- Better error messages with context
- Easier to enhance verification logic in one place
- More readable tests

#### Testing Strategy

- Run tests before and after refactoring
- All tests should pass with identical behavior
- Verify improved error messages on intentional failures

---

### Proposal 4: Improve Error Context in Drop with Tracing ⭐⭐

**Priority:** High Impact, Low Effort  
**Status:** ⬜ Not Started

#### Current Issue

Drop implementation silently ignores errors:

```rust
fn drop(&mut self) {
    if self.acquired {
        // Best effort cleanup, ignore errors on drop
        drop(fs::remove_file(&self.lock_file_path));
    }
}
```

This violates the **Observability** principle from `docs/development-principles.md`.

#### Solution

Add tracing for error visibility:

```rust
fn drop(&mut self) {
    if self.acquired {
        // Best effort cleanup, log errors for observability
        if let Err(e) = fs::remove_file(&self.lock_file_path) {
            tracing::warn!(
                path = ?self.lock_file_path,
                error = %e,
                "Failed to remove lock file during drop"
            );
        }
        self.acquired = false;
    }
}
```

#### Benefits

- Aligns with **Observability** principle
- Helps diagnose cleanup issues in production
- Maintains best-effort semantics
- Enables post-mortem debugging

#### Testing Strategy

- Run existing tests - no behavior changes
- Consider adding test with tracing subscriber to verify log output
- All tests should pass unchanged

#### Dependencies

- `tracing` crate (already used in project)

---

### Proposal 5: Extract Lock Acquisition Retry Logic ⭐⭐

**Priority:** Medium Impact, Medium Effort  
**Status:** ⬜ Not Started

#### Current Issue

The `acquire` method has complex retry logic mixed with error handling, making it hard to test and maintain.

#### Solution

Extract retry logic into dedicated types:

```rust
enum AcquireError {
    StaleProcess(u32),
    HeldByLiveProcess(u32),
    IoError(FileLockError),
}

struct LockRetryStrategy {
    start: Instant,
    timeout: Duration,
}

impl LockRetryStrategy {
    fn new(timeout: Duration) -> Self {
        Self {
            start: Instant::now(),
            timeout,
        }
    }

    fn is_expired(&self) -> bool {
        self.start.elapsed() >= self.timeout
    }

    fn wait(&self) {
        std::thread::sleep(LOCK_RETRY_SLEEP);
    }
}
```

Simplify `acquire` method using these types.

#### Benefits

- Clearer separation of concerns
- Easier to test retry logic independently
- More maintainable and extensible
- Simpler acquire method

#### Testing Strategy

- Run all existing tests
- Consider adding unit tests for `LockRetryStrategy`
- All integration tests should pass unchanged

---

### Proposal 6: Improve Test Naming and Organization ⭐⭐

**Priority:** Medium Impact, Medium Effort  
**Status:** ⬜ Not Started

#### Current Issue

Tests are flat and not grouped by functionality, making navigation difficult.

#### Solution

Organize tests into logical modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod basic_operations {
        use super::*;
        // Lock acquisition and release tests
    }

    mod concurrency {
        use super::*;
        // Multi-threaded scenarios
    }

    mod stale_lock_handling {
        use super::*;
        // Dead process cleanup tests
    }

    mod timeout_behavior {
        use super::*;
        // Retry and timeout logic tests
    }

    mod error_handling {
        use super::*;
        // Error message validation tests
    }

    mod lock_file_path_generation {
        use super::*;
        // Lock path tests
    }
}
```

#### Benefits

- Logical grouping makes tests easier to navigate
- Clear test categories
- Easier to identify coverage gaps
- Better IDE navigation

#### Testing Strategy

- Run tests before and after reorganization
- All tests should pass with identical behavior
- Verify test count remains the same

---

### Proposal 7: Add Builder Pattern for Test Configuration ⭐⭐

**Priority:** Medium Impact, Medium Effort  
**Status:** ⬜ Not Started

#### Current Issue

Tests create temp directories and file paths repeatedly with similar patterns.

#### Solution

Create test builder:

```rust
#[cfg(test)]
struct TestLockScenario {
    temp_dir: TempDir,
    file_name: String,
    timeout: Duration,
}

impl TestLockScenario {
    fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
            file_name: "test.json".to_string(),
            timeout: Duration::from_secs(1),
        }
    }

    fn with_file_name(mut self, name: &str) -> Self {
        self.file_name = name.to_string();
        self
    }

    fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn file_path(&self) -> PathBuf {
        self.temp_dir.path().join(&self.file_name)
    }

    fn lock_file_path(&self) -> PathBuf {
        FileLock::lock_file_path(&self.file_path())
    }

    fn acquire_lock(&self) -> Result<FileLock, FileLockError> {
        FileLock::acquire(&self.file_path(), self.timeout)
    }
}
```

#### Benefits

- Reduces boilerplate in tests
- Consistent test setup
- Easier to add new test configurations
- More expressive test code

#### Testing Strategy

- Gradually migrate existing tests to use builder
- Run tests after each migration
- All tests should pass with identical behavior

---

### Proposal 8: Add Type Safety for Process IDs ⭐

**Priority:** Lower Priority  
**Status:** ⬜ Not Started

#### Solution

Create `ProcessId` newtype wrapper for better type safety and encapsulation of process-related operations.

#### Benefits

- Type safety prevents accidental PID misuse
- Encapsulates platform-specific process checking
- Self-documenting code

---

### Proposal 9: Improve Platform-Specific Code Organization ⭐

**Priority:** Lower Priority  
**Status:** ⬜ Not Started

#### Solution

Extract platform-specific code into dedicated module for better organization and testability.

#### Benefits

- Clearer separation of platform concerns
- Easier to add more platform-specific functionality
- Better testability (can mock platform module)

---

### Proposal 10: Add Documentation for Testing Best Practices ⭐

**Priority:** Lower Priority  
**Status:** ⬜ Not Started

#### Solution

Add comprehensive module-level documentation for the test suite explaining organization, principles, and usage.

#### Benefits

- Helps contributors understand test structure
- Documents testing conventions
- Provides examples of test patterns

---

## 📚 Related Documentation

- [Development Principles](../development-principles.md) - Core principles including observability
- [Testing Conventions](../contributing/testing.md) - Project testing standards
- [Error Handling Guide](../contributing/error-handling.md) - Error handling best practices

## 🔗 References

- Original review discussion: Code review session on October 2, 2025
- Project conventions: See `.github/copilot-instructions.md`
