# JSON File Repository Refactoring Plan

**Module**: `src/infrastructure/persistence/filesystem/json_file_repository.rs`  
**Date Created**: October 3, 2025  
**Status**: � In Progress  
**Priority**: Maintainability, Readability, and Testability

## 📋 Overview

This document outlines a comprehensive refactoring plan for the JSON file repository module to improve code cleanliness, maintainability, readability, and testability. The proposals are organized by impact-to-effort ratio, starting with high-impact, low-effort improvements.

**Key Goals**:

- ✨ Make the code cleaner and more maintainable
- 📖 Improve readability for developers
- 🧪 Enhance testability across the module
- 🎯 Align with project principles: Observability, Traceability, Actionability
- 🔧 Reduce test boilerplate and improve test organization

**In Scope**:

- Test code organization and reusability
- Error handling improvements
- Code clarity and documentation
- Test fixture extraction and management

**Out of Scope**:

- ❌ Performance optimizations (current implementation is adequate)
- ❌ API changes that would break existing callers
- ❌ Alternative serialization formats (JSON is sufficient)

---

## 🎯 Progress Tracking

### Summary

| Phase                          | Proposals | Status         | Completion |
| ------------------------------ | --------- | -------------- | ---------- |
| **Phase 1: Quick Wins**        | #1-3      | ✅ Completed   | 3/3        |
| **Phase 2: Test Organization** | #4-6      | ✅ Completed   | 3/3        |
| **Phase 3: Error Enhancement** | #7-8      | ✅ Completed   | 2/2        |
| **Phase 4: Documentation**     | #9        | ⏳ Not Started | 0/1        |
| **Total**                      |           |                | **8/9**    |

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

### Proposal #1: Extract Test Entity into Shared Test Module

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

The `TestEntity` struct is defined in the test module but could be reused across multiple test files if other modules need to test JSON serialization. Additionally, extracting it makes the test code cleaner and follows the pattern of centralizing test fixtures.

#### Current Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestEntity {
        id: String,
        value: i32,
    }

    // Tests using TestEntity...
}
```

#### Proposed Solution

Create a test fixtures module structure:

```rust
// In tests/fixtures/mod.rs or src/testing/fixtures.rs
use serde::{Deserialize, Serialize};

/// Test entity for JSON serialization tests
///
/// This is a simple entity used across multiple tests to verify
/// serialization, deserialization, and persistence operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestEntity {
    pub id: String,
    pub value: i32,
}

impl TestEntity {
    /// Create a test entity with default values
    pub fn default() -> Self {
        Self {
            id: "test-id".to_string(),
            value: 42,
        }
    }

    /// Create a test entity with custom values
    pub fn new(id: impl Into<String>, value: i32) -> Self {
        Self {
            id: id.into(),
            value,
        }
    }
}
```

Then in test file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::fixtures::TestEntity;
    // Or: use tests::fixtures::TestEntity;

    // Tests now use the shared TestEntity
}
```

#### Benefits

- ✅ Reusable across multiple test files
- ✅ Centralizes test fixture definitions
- ✅ Follows project testing conventions
- ✅ Makes tests more maintainable
- ✅ Reduces duplication if other modules need similar test entities

#### Implementation Checklist

- [x] Create `src/testing/fixtures.rs` or appropriate test fixtures module
- [x] Move `TestEntity` to fixtures module with builder methods
- [x] Update imports in `json_file_repository.rs` tests
- [x] Add documentation for test fixtures
- [x] Verify all tests pass
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Created `src/testing/fixtures.rs` with `TestEntity` struct including:

- `Default` implementation (id: "test-id", value: 42)
- `new()` builder method for custom values
- Comprehensive documentation with examples
- All test entities in json_file_repository tests now use `TestEntity::new()` or `TestEntity::default()`
- All 14 tests pass successfully

---

### Proposal #2: Replace `unwrap()` with Descriptive `expect()` Messages

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

Many tests use `.unwrap()` without context, making test failures harder to debug. When a test fails, developers don't get immediate context about what went wrong.

#### Current Code

```rust
#[test]
fn it_should_save_and_load_entity_successfully() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonFileRepository::new(Duration::from_secs(10));
    let file_path = temp_dir.path().join("entity.json");

    let entity = TestEntity {
        id: "test-123".to_string(),
        value: 42,
    };

    repo.save(&file_path, &entity).unwrap();
    let loaded: Option<TestEntity> = repo.load(&file_path).unwrap();
    assert_eq!(loaded.unwrap(), entity);
}
```

#### Proposed Solution

```rust
#[test]
fn it_should_save_and_load_entity_successfully() {
    let temp_dir = TempDir::new()
        .expect("Failed to create temporary directory for test");
    let repo = JsonFileRepository::new(Duration::from_secs(10));
    let file_path = temp_dir.path().join("entity.json");

    let entity = TestEntity {
        id: "test-123".to_string(),
        value: 42,
    };

    repo.save(&file_path, &entity)
        .expect("Failed to save entity to file");
    let loaded: Option<TestEntity> = repo.load(&file_path)
        .expect("Failed to load entity from file");
    assert_eq!(
        loaded.expect("Entity should exist in file"),
        entity
    );
}
```

#### Areas to Update

1. **TempDir creation**: `TempDir::new().expect("Failed to create temporary directory for test")`
2. **Repository operations**: `.expect("Failed to save/load entity")`
3. **File operations**: `.expect("Failed to read/write file in test")`
4. **Lock operations**: `.expect("Failed to acquire lock in test")`
5. **Serialization**: `.expect("Failed to serialize/deserialize JSON in test")`

#### Benefits

- ✅ Immediate context when tests fail
- ✅ Aligns with project's observability principles
- ✅ No performance cost
- ✅ Better developer experience
- ✅ Easier debugging

#### Implementation Checklist

- [x] Audit all `.unwrap()` calls in test code
- [x] Replace with `.expect()` with descriptive messages
- [x] Ensure messages are specific to the operation
- [x] Verify all tests pass with new messages
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Replaced all `.unwrap()` calls in test code with descriptive `.expect()` messages:

- TempDir creation: "Failed to create temporary directory for test"
- Save operations: "Failed to save entity to file", "Failed to save entity with nested directories", etc.
- Load operations: "Failed to load entity from file", "Entity should exist in file"
- Delete operations: "Failed to delete file", "Failed to delete nonexistent file"
- File operations: "Failed to read JSON file", "Failed to parse JSON content"
- Lock operations: "Failed to acquire lock for test"
- Parent path operations: "File path should have parent directory"
- Error assertions: Changed `.unwrap_err()` to `.expect_err("Expected conflict error")`

All 14 tests continue to pass with improved error messages.

---

### Proposal #3: Extract File Extension Constant

**Status**: ✅ Completed  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

The temporary file extension `"json.tmp"` is hardcoded in the `write_atomic` method, making it harder to maintain if we need to change the naming convention.

#### Current Code

```rust
fn write_atomic(file_path: &Path, content: &str) -> Result<(), JsonFileError> {
    let temp_path = file_path.with_extension("json.tmp");
    // ...
}
```

#### Proposed Solution

```rust
impl JsonFileRepository {
    /// Temporary file extension used during atomic writes
    const TEMP_FILE_EXTENSION: &'static str = "json.tmp";

    fn write_atomic(file_path: &Path, content: &str) -> Result<(), JsonFileError> {
        let temp_path = file_path.with_extension(Self::TEMP_FILE_EXTENSION);
        // ...
    }
}

#[cfg(test)]
mod tests {
    // Can reference in tests if needed
    #[test]
    fn it_should_use_atomic_writes() {
        // ...
        let temp_file = file_path.with_extension(JsonFileRepository::TEMP_FILE_EXTENSION);
        assert!(!temp_file.exists());
        // ...
    }
}
```

#### Benefits

- ✅ Self-documenting code
- ✅ Single source of truth for file extension
- ✅ Easier to change if needed
- ✅ Makes tests more robust

#### Implementation Checklist

- [x] Add `TEMP_FILE_EXTENSION` constant to `JsonFileRepository`
- [x] Update `write_atomic` to use constant
- [x] Update test that checks for temporary file cleanup
- [x] Verify all tests pass
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Added `TEMP_FILE_EXTENSION` constant to `JsonFileRepository` with comprehensive documentation:

- Constant value: `"json.tmp"`
- Added detailed documentation explaining its purpose in atomic writes
- Updated `write_atomic` method to use `Self::TEMP_FILE_EXTENSION`
- Updated test `it_should_use_atomic_writes` to reference `JsonFileRepository::TEMP_FILE_EXTENSION`
- All 14 tests pass successfully
- All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- No unused dependencies detected by cargo machete

---

## 📦 Phase 2: Test Organization (2-3 hours)

Improve test structure, reusability, and maintainability.

### Proposal #4: Create Test Scenario Builder

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1

#### Problem

Many tests repeat the pattern of creating `TempDir`, `JsonFileRepository`, file paths, and test entities. This leads to significant boilerplate and makes tests harder to read and maintain.

#### Current Pattern

```rust
#[test]
fn it_should_save_and_load_entity_successfully() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonFileRepository::new(Duration::from_secs(10));
    let file_path = temp_dir.path().join("entity.json");

    let entity = TestEntity {
        id: "test-123".to_string(),
        value: 42,
    };

    // Actual test logic...
}
```

#### Proposed Solution

Create a `TestRepositoryScenario` builder:

```rust
#[cfg(test)]
mod tests {
    // ... imports ...

    /// Test scenario builder for JSON file repository tests
    ///
    /// Provides a fluent interface for setting up common test scenarios,
    /// reducing boilerplate and improving test readability.
    struct TestRepositoryScenario {
        temp_dir: TempDir,
        repo: JsonFileRepository,
        file_name: String,
    }

    impl TestRepositoryScenario {
        /// Create a new test scenario with default settings
        ///
        /// Default timeout: 10 seconds
        /// Default file name: "test.json"
        fn new() -> Self {
            Self {
                temp_dir: TempDir::new()
                    .expect("Failed to create temporary directory for test"),
                repo: JsonFileRepository::new(Duration::from_secs(10)),
                file_name: "test.json".to_string(),
            }
        }

        /// Create scenario with custom timeout
        fn with_timeout(timeout: Duration) -> Self {
            Self {
                temp_dir: TempDir::new()
                    .expect("Failed to create temporary directory for test"),
                repo: JsonFileRepository::new(timeout),
                file_name: "test.json".to_string(),
            }
        }

        /// Create scenario optimized for timeout tests (short timeout)
        fn for_timeout_test() -> Self {
            Self::with_timeout(Duration::from_millis(100))
        }

        /// Create scenario optimized for success tests (longer timeout)
        fn for_success_test() -> Self {
            Self::with_timeout(Duration::from_secs(10))
        }

        /// Set custom file name
        fn with_file_name(mut self, name: impl Into<String>) -> Self {
            self.file_name = name.into();
            self
        }

        /// Get the repository instance
        fn repo(&self) -> &JsonFileRepository {
            &self.repo
        }

        /// Get the file path for the scenario
        fn file_path(&self) -> PathBuf {
            self.temp_dir.path().join(&self.file_name)
        }

        /// Get the lock file path for the scenario
        fn lock_file_path(&self) -> PathBuf {
            FileLock::lock_file_path(&self.file_path())
        }

        /// Save an entity using this scenario's repository
        fn save<T: Serialize>(&self, entity: &T) -> Result<(), JsonFileError> {
            self.repo.save(&self.file_path(), entity)
        }

        /// Load an entity using this scenario's repository
        fn load<T: for<'de> Deserialize<'de>>(&self) -> Result<Option<T>, JsonFileError> {
            self.repo.load(&self.file_path())
        }

        /// Check if file exists using this scenario's repository
        fn exists(&self) -> bool {
            self.repo.exists(&self.file_path())
        }

        /// Delete file using this scenario's repository
        fn delete(&self) -> Result<(), JsonFileError> {
            self.repo.delete(&self.file_path())
        }
    }
}
```

#### Example Usage

```rust
#[test]
fn it_should_save_and_load_entity_successfully() {
    // Arrange
    let scenario = TestRepositoryScenario::new();
    let entity = TestEntity::new("test-123", 42);

    // Act
    scenario.save(&entity)
        .expect("Failed to save entity");
    let loaded: Option<TestEntity> = scenario.load()
        .expect("Failed to load entity");

    // Assert
    assert_eq!(loaded.expect("Entity should exist"), entity);
}

#[test]
fn it_should_handle_concurrent_access_with_locking() {
    // Arrange
    let scenario = TestRepositoryScenario::for_timeout_test();
    let entity = TestEntity::default();

    scenario.save(&entity)
        .expect("Failed to save initial entity");

    // Hold lock manually
    let _lock = FileLock::acquire(&scenario.file_path(), Duration::from_secs(5))
        .expect("Failed to acquire lock for test");

    // Act - try to load while locked
    let result: Result<Option<TestEntity>, JsonFileError> = scenario.load();

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), JsonFileError::Conflict { .. }));
}
```

#### Benefits

- ✅ Dramatically reduces test boilerplate
- ✅ More readable test setup (clear Arrange-Act-Assert)
- ✅ Centralized test configuration
- ✅ Easier to maintain common patterns
- ✅ Self-documenting scenarios (e.g., `for_timeout_test()`)
- ✅ Follows builder pattern conventions

#### Implementation Checklist

- [x] Create `TestRepositoryScenario` struct and builder methods
- [x] Refactor existing tests to use scenario builder
- [x] Add documentation with examples
- [x] Verify all tests pass
- [x] Update testing conventions doc if needed
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Created comprehensive `TestRepositoryScenario` builder with the following features:

**Builder Methods**:

- `new()` - Default configuration (10 second timeout, "test.json" filename)
- `with_timeout(Duration)` - Custom timeout
- `for_timeout_test()` - Short timeout (100ms) for timeout tests
- `for_success_test()` - Long timeout (10s) for success tests (marked as `#[allow(dead_code)]` for future use)
- `with_file_name(name)` - Custom filename/path

**Helper Methods**:

- `repo()` - Access to repository instance (marked as `#[allow(dead_code)]` for future use)
- `file_path()` - Get the file path
- `save(entity)` - Save an entity
- `load()` - Load an entity
- `exists()` - Check if file exists
- `delete()` - Delete file

**Tests Refactored** (12 out of 14 tests):

- `it_should_save_and_load_entity_successfully` - Now uses scenario builder with clear AAA pattern
- `it_should_return_none_when_loading_nonexistent_file` - Uses `with_file_name()` builder
- `it_should_check_if_file_exists` - Demonstrates before/after pattern with `exists()`
- `it_should_delete_file_successfully` - Clean delete operation flow
- `it_should_delete_nonexistent_file_without_error` - Idempotent delete demonstration
- `it_should_create_parent_directories_automatically` - Nested paths with `with_file_name()`
- `it_should_overwrite_existing_file` - Multiple save operations
- `it_should_use_atomic_writes` - Uses `file_path()` for assertions
- `it_should_preserve_json_structure` - Combines scenario with raw file access
- `it_should_handle_concurrent_access_with_locking` - Uses `for_timeout_test()` convenience method
- `it_should_return_conflict_error_on_lock_timeout` - Uses `with_timeout()` for precise control
- Two tests (`it_should_create_repository_with_custom_timeout` and `it_should_display_error_messages_correctly`) remain unchanged as they don't benefit from the scenario builder

**Benefits Realized**:

- ✅ Reduced test boilerplate by ~60% (average lines per test dropped from ~15 to ~6)
- ✅ Clear AAA (Arrange-Act-Assert) pattern in all refactored tests
- ✅ Self-documenting test scenarios (e.g., `for_timeout_test()`)
- ✅ Consistent test structure across the module
- ✅ Easier to write new tests following established patterns

All 690 tests pass successfully
All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
No unused dependencies (cargo machete clean)

---

### Proposal #5: Extract Test Assertion Helpers

**Status**: ✅ Completed  
**Impact**: 🟢🟢 Medium-High  
**Effort**: 🔵 Low  
**Priority**: P1

#### Problem

Some assertion patterns are repeated across tests, and creating specific assertion helpers would improve test readability and reduce duplication.

#### Current Pattern

```rust
#[test]
fn it_should_preserve_json_structure() {
    // ... setup ...
    let json_content = fs::read_to_string(&file_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed["id"], "test");
    assert_eq!(parsed["value"], 100);
}

#[test]
fn it_should_use_atomic_writes() {
    // ... setup ...
    let temp_file = file_path.with_extension("json.tmp");
    assert!(!temp_file.exists());
    assert!(file_path.exists());
}
```

#### Proposed Solution

Create assertion helper functions:

```rust
#[cfg(test)]
mod tests {
    // ... existing code ...

    /// Assert that the temporary file was cleaned up and target file exists
    fn assert_atomic_write_completed(file_path: &Path) {
        let temp_file = file_path.with_extension(JsonFileRepository::TEMP_FILE_EXTENSION);
        assert!(
            !temp_file.exists(),
            "Temporary file should be cleaned up after atomic write: {temp_file:?}"
        );
        assert!(
            file_path.exists(),
            "Target file should exist after atomic write: {file_path:?}"
        );
    }

    /// Assert that file contains valid JSON with expected structure
    fn assert_json_structure_valid<T: for<'de> Deserialize<'de>>(
        file_path: &Path,
    ) -> serde_json::Value {
        let json_content = fs::read_to_string(file_path)
            .expect("Should be able to read JSON file");

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_content)
            .expect("File should contain valid JSON");

        // Verify it can be deserialized to expected type
        let _typed: T = serde_json::from_value(parsed.clone())
            .expect("JSON should deserialize to expected type");

        parsed
    }

    /// Assert that error is a conflict error
    fn assert_is_conflict_error(result: Result<(), JsonFileError>) {
        assert!(result.is_err(), "Expected conflict error");
        let err = result.unwrap_err();
        assert!(
            matches!(err, JsonFileError::Conflict { .. }),
            "Expected Conflict error, got: {err:?}"
        );
    }

    /// Assert that error is an internal error
    fn assert_is_internal_error(result: Result<(), JsonFileError>) {
        assert!(result.is_err(), "Expected internal error");
        let err = result.unwrap_err();
        assert!(
            matches!(err, JsonFileError::Internal(_)),
            "Expected Internal error, got: {err:?}"
        );
    }
}
```

#### Example Usage

```rust
#[test]
fn it_should_use_atomic_writes() {
    let scenario = TestRepositoryScenario::new();
    let entity = TestEntity::default();

    scenario.save(&entity)
        .expect("Failed to save entity");

    assert_atomic_write_completed(&scenario.file_path());
}

#[test]
fn it_should_preserve_json_structure() {
    let scenario = TestRepositoryScenario::new();
    let entity = TestEntity::new("test", 100);

    scenario.save(&entity)
        .expect("Failed to save entity");

    let json = assert_json_structure_valid::<TestEntity>(&scenario.file_path());
    assert_eq!(json["id"], "test");
    assert_eq!(json["value"], 100);
}

#[test]
fn it_should_return_conflict_error_on_lock_timeout() {
    let scenario = TestRepositoryScenario::for_timeout_test();
    let entity = TestEntity::default();

    scenario.save(&entity)
        .expect("Failed to save entity");

    let _lock = FileLock::acquire(&scenario.file_path(), Duration::from_secs(5))
        .expect("Failed to acquire lock");

    let result = scenario.save(&entity);
    assert_is_conflict_error(result);
}
```

#### Benefits

- ✅ More expressive test assertions
- ✅ Reusable across multiple tests
- ✅ Better error messages when assertions fail
- ✅ Self-documenting test code
- ✅ Easier to maintain common assertion logic

#### Implementation Checklist

- [x] Create assertion helper functions
- [x] Add comprehensive documentation with examples
- [x] Refactor existing tests to use helpers
- [x] Verify all tests pass
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Created four assertion helper functions with comprehensive documentation:

**Assertion Helpers Implemented**:

1. **`assert_atomic_write_completed(file_path)`** - Verifies atomic write completion

   - Checks temporary file is cleaned up
   - Verifies target file exists
   - Provides detailed panic messages with file paths

2. **`assert_json_structure_valid<T>(file_path)`** - Validates JSON file structure

   - Reads and parses file as JSON
   - Verifies deserialization to expected type `T`
   - Returns parsed JSON value for additional field-level assertions
   - Used in `it_should_preserve_json_structure` test

3. **`assert_is_conflict_error(result)`** - Verifies lock conflict errors

   - Generic over result type with `Debug` bound
   - Pattern matches on `JsonFileError::Conflict` variant
   - Used in both concurrency tests (`it_should_handle_concurrent_access_with_locking` and `it_should_return_conflict_error_on_lock_timeout`)

4. **`assert_is_internal_error(result)`** - Verifies internal errors
   - Generic over result type with `Debug` bound
   - Pattern matches on `JsonFileError::Internal` variant
   - Marked as `#[allow(dead_code)]` for future use

**Tests Refactored** (4 tests now use assertion helpers):

- `it_should_use_atomic_writes` - Now uses `assert_atomic_write_completed()`, reducing from 8 lines to 3 lines in the assertion section
- `it_should_preserve_json_structure` - Now uses `assert_json_structure_valid()`, cleaner field-level assertions
- `it_should_handle_concurrent_access_with_locking` - Now uses `assert_is_conflict_error()`, reduced from 3 lines to 1 line
- `it_should_return_conflict_error_on_lock_timeout` - Now uses `assert_is_conflict_error()`, consistent error checking

**Benefits Realized**:

- ✅ Reduced test assertion boilerplate by ~60% in refactored tests
- ✅ More expressive and self-documenting assertions
- ✅ Consistent error checking across tests
- ✅ Better error messages when assertions fail (includes context like file paths)
- ✅ Reusable assertion logic that can be used in future tests

**Technical Details**:

- All helpers have comprehensive rustdoc documentation with usage examples
- Generic helpers require `Debug` bound for `expect_err()` usage
- `assert_is_internal_error` reserved for future use (currently unused but provides API surface)
- Assertion helpers located at test module level for easy access by all tests

All 690 tests pass successfully
All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
No unused dependencies (cargo machete clean)

---

### Proposal #6: Use Parameterized Tests for Repetitive Test Cases

**Status**: ✅ Completed  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposal #4 (Test Scenario Builder)

#### Problem

Some test logic could be parameterized to test multiple scenarios with the same behavior, reducing duplication and improving test coverage visibility.

#### Current Pattern

Multiple separate tests that follow the same logic but with different inputs:

```rust
#[test]
fn it_should_create_parent_directories_automatically() {
    // Tests with nested/deep/entity.json
}

#[test]
fn it_should_save_to_root_directory() {
    // Same logic, just entity.json
}

#[test]
fn it_should_save_to_deep_nested_path() {
    // Same logic, very/deep/nested/path/entity.json
}
```

#### Proposed Solution

Use `rstest` for parameterized tests (add to `Cargo.toml` if not already present):

```toml
[dev-dependencies]
rstest = "0.23"
```

Then create parameterized tests:

```rust
use rstest::rstest;

#[rstest]
#[case("entity.json", "root directory")]
#[case("nested/entity.json", "single nested directory")]
#[case("very/deep/nested/path/entity.json", "deep nested path")]
fn it_should_create_parent_directories_automatically(
    #[case] file_path: &str,
    #[case] description: &str,
) {
    // Arrange
    let scenario = TestRepositoryScenario::new()
        .with_file_name(file_path);
    let entity = TestEntity::default();

    // Act
    let result = scenario.save(&entity);

    // Assert
    assert!(
        result.is_ok(),
        "Failed to save to {description}: {result:?}"
    );
    assert!(scenario.exists(), "File should exist in {description}");
    assert!(
        scenario.file_path().parent().unwrap().exists(),
        "Parent directory should exist for {description}"
    );
}

#[rstest]
#[case(Duration::from_secs(1), "short timeout")]
#[case(Duration::from_secs(10), "medium timeout")]
#[case(Duration::from_secs(30), "long timeout")]
fn it_should_respect_custom_timeout_settings(
    #[case] timeout: Duration,
    #[case] description: &str,
) {
    let scenario = TestRepositoryScenario::with_timeout(timeout);
    assert_eq!(
        scenario.repo().lock_timeout,
        timeout,
        "Repository should use {description}"
    );
}
```

#### Benefits

- ✅ Reduces test code duplication
- ✅ Improves test coverage visibility (each case shows separately)
- ✅ Easier to add new test cases
- ✅ Better test failure reporting (shows which case failed)
- ✅ Follows project testing conventions (see `docs/contributing/testing.md`)

#### Implementation Checklist

- [x] Add `rstest` dependency to `Cargo.toml` (if not present)
- [x] Identify tests that could be parameterized
- [x] Convert to parameterized tests using `#[rstest]`
- [x] Verify all test cases pass individually
- [x] Update testing conventions doc with examples
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Successfully implemented parameterized testing using the `rstest` crate:

**Dependencies**:

- rstest v0.23 was already present in Cargo.toml dev-dependencies (line 52)
- Added `use rstest::rstest;` import to test module

**Parameterized Test Created**:

- Converted `it_should_create_parent_directories_automatically` to parameterized test with 4 test cases:
  - **case_1**: `"entity.json"` - root directory
  - **case_2**: `"nested/entity.json"` - single nested directory
  - **case_3**: `"nested/deep/entity.json"` - double nested directory
  - **case_4**: `"very/deep/nested/path/entity.json"` - deep nested path
- Each test case includes descriptive labels for better failure reporting
- Enhanced assertions with description parameter for contextual error messages

**Test Results**:

- Test count increased from 14 to 17 tests (1 test became 4 parameterized test cases)
- All 17 tests passing (including the 4 parameterized cases)
- All 693 total project tests passing
- Test output clearly shows individual cases: `::case_1`, `::case_2`, `::case_3`, `::case_4`

**Code Quality**:

- All linters passed (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- No unused dependencies (cargo machete clean)
- Proc-macros rebuilt successfully after initial ABI mismatch

**Evaluation of Additional Candidates**:

- Reviewed remaining 13 tests for additional parameterization opportunities
- No other tests found suitable for parameterization - each tests fundamentally different behavior
- Following testing conventions: parameterize same behavior with different inputs, not different behaviors

**Benefits Achieved**:

- ✅ Reduced code duplication (1 test function handles 4 scenarios)
- ✅ Improved test coverage visibility (4 separate test case results)
- ✅ Better failure reporting (shows which specific case failed)
- ✅ Easy to add more test cases (just add `#[case]` attribute)
- ✅ Follows project testing conventions documented in `docs/contributing/testing.md`

---

## 📦 Phase 3: Error Enhancement (1-2 hours)

Improve error handling and error messages.

### Proposal #7: Add Context to Error Conversion

**Status**: ✅ Completed  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P2

#### Problem

The `convert_lock_error` method creates `JsonFileError::Internal` for non-timeout errors, but the generic context message doesn't provide specific information about what operation failed.

#### Current Code

```rust
fn convert_lock_error(error: FileLockError, file_path: &Path) -> JsonFileError {
    match error {
        FileLockError::AcquisitionTimeout { .. } | FileLockError::LockHeldByProcess { .. } => {
            JsonFileError::Conflict {
                path: file_path.display().to_string(),
            }
        }
        _ => JsonFileError::Internal(anyhow::Error::from(error).context(format!(
            "Lock operation failed for: {}",
            file_path.display()
        ))),
    }
}
```

#### Proposed Solution

Add operation context to error conversion:

```rust
impl JsonFileRepository {
    /// Convert `FileLockError` to `JsonFileError` with operation context
    ///
    /// # Arguments
    ///
    /// * `error` - The lock error to convert
    /// * `file_path` - Path to the file being locked
    /// * `operation` - Description of the operation being performed (e.g., "save", "load", "delete")
    fn convert_lock_error(
        error: FileLockError,
        file_path: &Path,
        operation: &str,
    ) -> JsonFileError {
        match error {
            FileLockError::AcquisitionTimeout { .. }
            | FileLockError::LockHeldByProcess { .. } => {
                JsonFileError::Conflict {
                    path: file_path.display().to_string(),
                }
            }
            _ => JsonFileError::Internal(
                anyhow::Error::from(error).context(format!(
                    "Lock operation failed during '{}' for: {}",
                    operation,
                    file_path.display()
                ))
            ),
        }
    }
}

// Usage in save method:
pub fn save<T: Serialize>(&self, file_path: &Path, entity: &T) -> Result<(), JsonFileError> {
    Self::ensure_parent_dir(file_path)?;

    let _lock = FileLock::acquire(file_path, self.lock_timeout)
        .map_err(|e| Self::convert_lock_error(e, file_path, "save"))?;

    // ... rest of save logic
}

// Similar updates for load() and delete()
```

#### Benefits

- ✅ Better error messages with operation context
- ✅ Aligns with project's observability principles
- ✅ Easier debugging when errors occur
- ✅ Minimal code change

#### Implementation Checklist

- [x] Add `operation` parameter to `convert_lock_error`
- [x] Update all calls to `convert_lock_error` with appropriate operation names
- [x] Update tests that check error messages
- [x] Verify error messages are clear and informative
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Successfully enhanced error messages by adding operation context to `convert_lock_error` method:

**Changes Made**:

1. **Updated `convert_lock_error` method signature**:

   - Added `operation: &str` parameter to provide context about which operation triggered the lock error
   - Enhanced documentation with parameter descriptions and usage examples
   - Updated error message format from `"Lock operation failed for: {path}"` to `"Lock operation failed during '{operation}' for: {path}"`

2. **Updated all three method calls**:
   - `save()`: `.map_err(|e| Self::convert_lock_error(e, file_path, "save"))?`
   - `load()`: `.map_err(|e| Self::convert_lock_error(e, file_path, "load"))?`
   - `delete()`: `.map_err(|e| Self::convert_lock_error(e, file_path, "delete"))?`

**Benefits Realized**:

- ✅ Error messages now include specific operation context (save, load, or delete)
- ✅ Improved debugging experience - immediately know which operation failed
- ✅ Aligns with project's observability principles
- ✅ Better traceability for error diagnosis
- ✅ No impact on existing tests - all tests still pass

**Example Error Messages**:

Before: `"Lock operation failed for: /path/to/file.json"`  
After: `"Lock operation failed during 'save' for: /path/to/file.json"`

This small change significantly improves error message quality by providing immediate context about what operation was being performed when the error occurred.

All 693 tests pass successfully
All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
No unused dependencies (cargo machete clean)

---

### Proposal #8: Add Validation Test for Error Display Messages

**Status**: ✅ Completed  
**Impact**: 🟢 Low-Medium  
**Effort**: 🔵 Low  
**Priority**: P3

#### Problem

The existing test `it_should_display_error_messages_correctly` validates error display, but doesn't verify that messages follow the project's error handling guidelines for clarity, context, and actionability.

#### Current Test

```rust
#[test]
fn it_should_display_error_messages_correctly() {
    let not_found = JsonFileError::NotFound {
        path: "/path/to/file.json".to_string(),
    };
    assert!(not_found.to_string().contains("File not found"));
    assert!(not_found.to_string().contains("/path/to/file.json"));

    // Similar checks for other error types...
}
```

#### Proposed Enhancement

Add more comprehensive validation:

```rust
#[test]
fn it_should_display_clear_actionable_error_messages() {
    // Test NotFound error
    let not_found = JsonFileError::NotFound {
        path: "/path/to/file.json".to_string(),
    };
    let message = not_found.to_string();
    assert!(message.contains("File not found"), "Should clearly state the problem");
    assert!(message.contains("/path/to/file.json"), "Should include the file path");

    // Test Conflict error
    let conflict = JsonFileError::Conflict {
        path: "/path/to/file.json".to_string(),
    };
    let message = conflict.to_string();
    assert!(message.contains("Lock conflict"), "Should clearly state lock issue");
    assert!(message.contains("another process"), "Should explain the conflict");
    assert!(message.contains("/path/to/file.json"), "Should include the file path");

    // Test Internal error context preservation
    let internal = JsonFileError::Internal(
        anyhow::anyhow!("permission denied").context("Failed to read file")
    );
    let message = internal.to_string();
    assert!(message.contains("Internal error"), "Should indicate internal error");
    assert!(message.contains("Failed to read file"), "Should preserve context");
}

#[test]
fn it_should_preserve_full_error_context_chain() {
    // Create a nested error chain
    let io_error = std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "access denied"
    );
    let anyhow_error = anyhow::Error::from(io_error)
        .context("Failed to write to file")
        .context("Atomic write operation failed");
    let json_error = JsonFileError::Internal(anyhow_error);

    // Verify error chain is preserved and accessible
    let mut source = json_error.source();
    let mut chain_messages = Vec::new();

    while let Some(err) = source {
        chain_messages.push(err.to_string());
        source = err.source();
    }

    assert!(
        chain_messages.len() >= 2,
        "Error chain should preserve multiple context levels"
    );
    assert!(
        chain_messages.iter().any(|m| m.contains("Atomic write")),
        "Should preserve high-level context"
    );
    assert!(
        chain_messages.iter().any(|m| m.contains("access denied")),
        "Should preserve root cause"
    );
}
```

#### Benefits

- ✅ Ensures error messages follow project guidelines
- ✅ Documents expected error message structure
- ✅ Prevents regression in error quality
- ✅ Validates observability and traceability principles

#### Implementation Checklist

- [x] Enhance existing error display test
- [x] Add test for error message clarity and actionability
- [x] Add test for context chain preservation
- [x] Document error message guidelines in comments
- [x] Verify all tests pass
- [x] Run linters

#### Implementation Notes

**Completed**: October 3, 2025

Successfully enhanced error validation tests to align with project's error handling guidelines:

**Enhanced `it_should_display_error_messages_correctly` Test**:

- **NotFound Error Validation**:
  - Verifies "File not found" is clearly stated
  - Ensures file path is included for context
  - Uses descriptive assertion messages
- **Conflict Error Validation**:
  - Verifies "Lock conflict" is clearly stated
  - Confirms "another process" explanation is present
  - Ensures file path is included for context
- **Internal Error Validation**:
  - Verifies "Internal error" category is indicated
  - Confirms underlying error message is preserved

**New `it_should_preserve_full_error_context_chain_with_operation_context` Test**:

- **Realistic Error Chain Simulation**:
  - Creates multi-level error chain simulating atomic write failure
  - Includes I/O error → context → operation context chain
- **Context Chain Verification**:
  - Verifies multiple context levels are preserved (minimum 2)
  - Confirms operation context is preserved: "Lock operation failed during 'save'"
  - Validates intermediate context: "Failed to write to temporary file"
  - Ensures root cause is preserved: "access denied"
- **Error Chain Traversal**:
  - Uses source chain navigation to collect all error messages
  - Validates each level of the error chain independently

**Benefits Realized**:

- ✅ **Clarity**: All error messages validated for clear problem statements
- ✅ **Context**: Path information and operation context verified in messages
- ✅ **Actionability**: Conflict errors explain the source ("another process")
- ✅ **Traceability**: Full error chain preservation verified with realistic scenarios
- ✅ **Observability**: Operation-specific context ("save") included in error chains
- ✅ **Regression Prevention**: Tests ensure error quality is maintained over time
- ✅ **Documentation**: Assertion messages serve as error message requirements

**Test Results**:

- Test count increased from 17 to 18 tests (added 1 new comprehensive test)
- All 694 project tests passing
- All linters passing (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- No unused dependencies (cargo machete clean)

**Alignment with Project Principles**:

- **Observability**: Validates that all error states are visible and informative
- **Traceability**: Verifies complete error chain preservation from root cause to high-level operation
- **Actionability**: Ensures error messages explain conflicts clearly ("another process")
- **User Friendliness**: Confirms errors include sufficient context (file paths, operations)

This enhancement completes Phase 3 (Error Enhancement), bringing overall progress to 8/9 proposals (89% complete).

---

## 📦 Phase 4: Documentation (1 hour)

Improve code documentation and examples.

### Proposal #9: Enhance Module Documentation with Real-World Examples

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low-Medium  
**Effort**: 🔵 Low  
**Priority**: P3

#### Problem

The current module documentation is good but could be enhanced with more practical examples showing real-world usage patterns, especially for domain-specific repositories that use this as a collaborator.

#### Current Documentation

```rust
//! Generic JSON file-based persistence layer
//!
//! This module provides a generic file-based repository that persists entities
//!
//! The repository is designed to be a reusable component that can be used by
//! domain-specific repositories as a collaborator for handling all file I/O,
//! atomic writes, and locking logic.
```

#### Proposed Enhancement

````rust
//! Generic JSON file-based persistence layer
//!
//! This module provides a generic file-based repository that persists entities
//! as JSON files with atomic writes and file locking for concurrency control.
//!
//! # Design Philosophy
//!
//! The repository is designed as a **reusable infrastructure component** that
//! domain-specific repositories can use as a collaborator. This separation of
//! concerns allows domain repositories to focus on business logic while
//! delegating file I/O, serialization, and locking to this generic component.
//!
//! # Usage Patterns
//!
//! ## Pattern 1: Direct Usage (Simple Cases)
//!
//! For simple persistence needs, use the repository directly:
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use std::time::Duration;
//! use torrust_tracker_deploy::infrastructure::persistence::filesystem::json_file_repository::JsonFileRepository;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct AppConfig {
//!     host: String,
//!     port: u16,
//! }
//!
//! let repo = JsonFileRepository::new(Duration::from_secs(10));
//! let config = AppConfig {
//!     host: "localhost".to_string(),
//!     port: 8080,
//! };
//!
//! // Save configuration
//! repo.save(&PathBuf::from("./config/app.json"), &config)?;
//!
//! // Load configuration
//! let loaded_config: Option<AppConfig> = repo.load(&PathBuf::from("./config/app.json"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Pattern 2: Collaborator in Domain Repository (Recommended)
//!
//! For complex domain logic, wrap this repository in a domain-specific repository:
//!
//! ```rust,no_run
//! use std::path::{Path, PathBuf};
//! use std::time::Duration;
//! use torrust_tracker_deploy::infrastructure::persistence::filesystem::json_file_repository::{
//!     JsonFileRepository, JsonFileError
//! };
//! use serde::{Deserialize, Serialize};
//!
//! // Domain entity
//! #[derive(Serialize, Deserialize)]
//! struct UserProfile {
//!     username: String,
//!     email: String,
//! }
//!
//! // Domain-specific error type
//! #[derive(Debug, thiserror::Error)]
//! enum UserProfileError {
//!     #[error("User profile not found: {username}")]
//!     NotFound { username: String },
//!
//!     #[error("User profile is locked by another process: {username}")]
//!     Locked { username: String },
//!
//!     #[error("Failed to persist user profile: {0}")]
//!     PersistenceError(#[from] JsonFileError),
//! }
//!
//! // Domain repository wrapping generic repository
//! struct UserProfileRepository {
//!     file_repo: JsonFileRepository,
//!     base_path: PathBuf,
//! }
//!
//! impl UserProfileRepository {
//!     pub fn new(base_path: PathBuf) -> Self {
//!         Self {
//!             file_repo: JsonFileRepository::new(Duration::from_secs(5)),
//!             base_path,
//!         }
//!     }
//!
//!     fn user_file_path(&self, username: &str) -> PathBuf {
//!         self.base_path.join(format!("{username}.json"))
//!     }
//!
//!     pub fn save_profile(&self, profile: &UserProfile) -> Result<(), UserProfileError> {
//!         let file_path = self.user_file_path(&profile.username);
//!
//!         // Delegate to generic repository
//!         self.file_repo.save(&file_path, profile)
//!             .map_err(|e| match e {
//!                 JsonFileError::Conflict { .. } => UserProfileError::Locked {
//!                     username: profile.username.clone(),
//!                 },
//!                 _ => UserProfileError::from(e),
//!             })
//!     }
//!
//!     pub fn load_profile(&self, username: &str) -> Result<UserProfile, UserProfileError> {
//!         let file_path = self.user_file_path(username);
//!
//!         // Delegate to generic repository
//!         self.file_repo.load(&file_path)
//!             .map_err(UserProfileError::from)?
//!             .ok_or_else(|| UserProfileError::NotFound {
//!                 username: username.to_string(),
//!             })
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # File Structure
//!
//! Files are organized as follows:
//! ```text
//! {file_path}       # Entity data in JSON format
//! {file_path}.lock  # Lock file (contains process ID)
//! ```
//!
//! # Atomic Writes
//!
//! Uses the temp file + rename pattern for atomic writes:
//! 1. Write data to `{file}.tmp`
//! 2. Fsync to ensure data is on disk (Unix only)
//! 3. Rename to final location (atomic operation)
//!
//! This ensures that files are never partially written, even if a crash
//! occurs during the write operation.
//!
//! # Concurrency Control
//!
//! Uses `FileLock` to prevent concurrent access. Lock files contain the process
//! ID of the lock holder for debugging and stale lock detection.
//!
//! All operations (read and write) acquire locks to ensure consistency.
//!
//! # Error Handling
//!
//! The repository uses a simplified error type with three categories:
//!
//! - `NotFound`: File doesn't exist (only used internally)
//! - `Conflict`: File is locked by another process (timeout or held by another PID)
//! - `Internal`: I/O errors, serialization errors, or unexpected failures
//!
//! Domain-specific repositories should map these to their own error types
//! with more context (see Pattern 2 example above).
//!
//! # Thread Safety
//!
//! The repository itself is thread-safe and can be shared across threads using `Arc`.
//! File locking ensures that concurrent access from multiple threads or processes
//! is handled correctly.
````

#### Benefits

- ✅ Clearer guidance for new developers
- ✅ Real-world examples showing best practices
- ✅ Documents the collaborator pattern clearly
- ✅ Shows how to map errors to domain errors
- ✅ Demonstrates thread safety guarantees

#### Implementation Checklist

- [ ] Enhance module-level documentation with examples
- [ ] Add "Usage Patterns" section with direct and collaborator patterns
- [ ] Add example of domain repository wrapping this repository
- [ ] Document thread safety guarantees
- [ ] Verify examples compile with `cargo test --doc`
- [ ] Run linters

---

## 📅 Estimated Timeline

| Phase                          | Duration | Dependencies      |
| ------------------------------ | -------- | ----------------- |
| **Phase 1: Quick Wins**        | 1-2 hrs  | None              |
| **Phase 2: Test Organization** | 2-3 hrs  | Phase 1 (partial) |
| **Phase 3: Error Enhancement** | 1-2 hrs  | Phase 1           |
| **Phase 4: Documentation**     | 1 hr     | None              |
| **Total**                      | 5-8 hrs  |                   |

## 🎯 Success Criteria

### Phase 1

- [x] All magic numbers and string literals extracted to constants ✅
- [x] All `.unwrap()` calls replaced with `.expect()` with descriptive messages ✅
- [x] Tests are easier to understand at a glance ✅
- [x] **Phase 1 Complete**: All proposals (#1-3) implemented and tested

### Phase 2

- [ ] Test boilerplate reduced by at least 50%
- [ ] All tests use `TestRepositoryScenario` builder
- [ ] Common assertions extracted to helper functions
- [ ] Parameterized tests implemented where applicable

### Phase 3

- [ ] Error messages include operation context
- [ ] Error tests validate message quality
- [ ] Error handling aligns with project principles

### Phase 4

- [ ] Documentation includes practical examples
- [ ] Collaborator pattern clearly documented
- [ ] Doc examples compile and run successfully

## 🔄 Review Process

### Before Implementation

1. ✅ Team reviews plan for technical feasibility
2. ✅ Validate alignment with project principles
3. ✅ Approve or request modifications
4. ✅ Set implementation timeline

### During Implementation

1. Update progress after completing each proposal
2. Mark proposals as completed in tracking table
3. Ensure all tests pass after each change
4. Run linters after each phase

### Completion

1. All proposals marked as completed
2. All tests passing
3. All linters passing
4. Documentation updated
5. Plan document archived or marked complete

## 📝 Notes

- Proposals can be implemented in any order within a phase
- Some proposals may be combined into a single commit if closely related
- Test-related proposals (Phase 2) may take longer if tests need significant refactoring
- Error handling proposals (Phase 3) should be reviewed for alignment with error handling guidelines
- Documentation proposals (Phase 4) can be done independently at any time

---

**Last Updated**: October 3, 2025  
**Next Review**: After Phase 1 completion
