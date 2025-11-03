# Simplify Test Infrastructure

**Issue**: [#123](https://github.com/torrust/torrust-tracker-deployer/issues/123)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**: [Refactoring Plan - Proposal #1](../refactors/plans/user-output-architecture-improvements.md#proposal-1-simplify-test-infrastructure)

## Overview

Simplify the test infrastructure for `UserOutput` by replacing the complex `Arc<Mutex<Vec<u8>>>` and custom `SharedWriter` implementation with a simpler, more maintainable approach. This makes test code easier to understand for contributors and reduces cognitive load when writing tests.

**Current Problem**: Test helper uses overly complex synchronization primitives (`Arc`, `Mutex`) with a custom `SharedWriter` implementation that is harder to understand than necessary.

**Proposed Solution**: Create a simpler `TestUserOutput` wrapper that provides direct access to captured output without complex shared ownership patterns.

## Goals

- [ ] Simplify test helper infrastructure
- [ ] Reduce cognitive load for test writers
- [ ] Make test code more maintainable
- [ ] Use standard library types where possible
- [ ] Maintain all existing test functionality

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (Test Support Module)
**Module Path**: `src/presentation/user_output.rs` (test module)
**Pattern**: Test Helper Struct

### Module Structure Requirements

- [ ] Keep test infrastructure in `#[cfg(test)]` module within `user_output.rs`
- [ ] Create `TestUserOutput` helper struct for test scenarios
- [ ] Maintain separation between production code and test helpers
- [ ] Follow module organization conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] **Test-only code**: All changes confined to `#[cfg(test)]` modules
- [ ] **No breaking changes**: Production `UserOutput` API unchanged
- [ ] **Standard library preference**: Use `Vec<u8>` and standard types over complex wrappers
- [ ] **Simplicity over cleverness**: Favor readable code over minimal boilerplate

### Anti-Patterns to Avoid

- ❌ **Over-engineering test infrastructure** - Don't add unnecessary abstractions
- ❌ **Leaking test code to production** - Keep test helpers in test modules only
- ❌ **Complex synchronization primitives** - Avoid `Arc<Mutex<>>` unless truly needed

## Specifications

### Current Implementation

The existing test infrastructure uses complex shared ownership:

```rust
fn create_test_user_output(
    verbosity: VerbosityLevel,
) -> (
    UserOutput,
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
) {
    let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
    let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

    let stdout_writer = Box::new(SharedWriter(Arc::clone(&stdout_buffer)));
    let stderr_writer = Box::new(SharedWriter(Arc::clone(&stderr_buffer)));

    let output = UserOutput::with_writers(verbosity, stdout_writer, stderr_writer);

    (output, stdout_buffer, stderr_buffer)
}

struct SharedWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}
```

**Problems**:

- Requires understanding of `Arc`, `Mutex`, and custom `Write` implementations
- More complex than necessary for single-threaded test scenarios
- Harder for new contributors to understand and modify

### Proposed Implementation

Create a simpler `TestUserOutput` wrapper:

```rust
#[cfg(test)]
mod test_support {
    use super::*;

    /// Test wrapper that provides access to captured output
    ///
    /// This simplifies testing by avoiding complex shared ownership patterns.
    /// Output is captured in plain `Vec<u8>` buffers that can be easily
    /// inspected with `stdout()` and `stderr()` methods.
    pub struct TestUserOutput {
        output: UserOutput,
        stdout_buffer: Rc<RefCell<Vec<u8>>>,
        stderr_buffer: Rc<RefCell<Vec<u8>>>,
    }

    impl TestUserOutput {
        /// Create a new test output with the given verbosity level
        pub fn new(verbosity: VerbosityLevel) -> Self {
            let stdout_buffer = Rc::new(RefCell::new(Vec::new()));
            let stderr_buffer = Rc::new(RefCell::new(Vec::new()));

            let output = UserOutput::with_writers(
                verbosity,
                Box::new(TestWriter(Rc::clone(&stdout_buffer))),
                Box::new(TestWriter(Rc::clone(&stderr_buffer))),
            );

            Self {
                output,
                stdout_buffer,
                stderr_buffer,
            }
        }

        /// Get captured stdout as a string
        pub fn stdout(&self) -> String {
            String::from_utf8_lossy(&self.stdout_buffer.borrow()).to_string()
        }

        /// Get captured stderr as a string
        pub fn stderr(&self) -> String {
            String::from_utf8_lossy(&self.stderr_buffer.borrow()).to_string()
        }

        /// Get mutable reference to the wrapped UserOutput
        pub fn output(&mut self) -> &mut UserOutput {
            &mut self.output
        }

        /// Clear all captured output
        pub fn clear(&mut self) {
            self.stdout_buffer.borrow_mut().clear();
            self.stderr_buffer.borrow_mut().clear();
        }
    }

    /// Simple writer that captures output to a shared buffer
    struct TestWriter(Rc<RefCell<Vec<u8>>>);

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
```

**Benefits**:

- Uses `Rc<RefCell<>>` instead of `Arc<Mutex<>>` (simpler for single-threaded tests)
- Clear, self-documenting API: `test_output.stdout()`, `test_output.stderr()`
- Internal implementation details hidden from test writers
- Easier to understand and modify

### Usage Example

Before (complex):

```rust
#[test]
fn test_progress_message() {
    let (mut output, _stdout, stderr) = create_test_user_output(VerbosityLevel::Normal);
    output.progress("Loading data");
    let captured = String::from_utf8_lossy(&stderr.lock().unwrap()).to_string();
    assert!(captured.contains("⏳ Loading data"));
}
```

After (simple):

```rust
#[test]
fn test_progress_message() {
    let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);
    test_output.output().progress("Loading data");
    assert!(test_output.stderr().contains("⏳ Loading data"));
}
```

## Implementation Plan

### Phase 1: Create New Test Infrastructure (1-2 hours)

- [ ] Create `test_support` module in `#[cfg(test)]` section
- [ ] Implement `TestUserOutput` struct with fields: `output`, `stdout_buffer`, `stderr_buffer`
- [ ] Implement `TestUserOutput::new()` constructor
- [ ] Implement accessor methods: `stdout()`, `stderr()`, `output()`, `clear()`
- [ ] Implement `TestWriter` struct with `Write` trait
- [ ] Add basic documentation for test infrastructure

### Phase 2: Update Existing Tests (2-3 hours)

- [ ] Identify all tests using `create_test_user_output()`
- [ ] Update tests to use `TestUserOutput::new()` instead
- [ ] Update assertions to use `test_output.stdout()` and `test_output.stderr()`
- [ ] Simplify test code where possible with new cleaner API
- [ ] Verify all tests pass with new infrastructure

### Phase 3: Remove Old Infrastructure (30 minutes)

- [ ] Remove `SharedWriter` struct
- [ ] Remove `create_test_user_output()` function
- [ ] Verify no remaining references to old test helpers
- [ ] Run linter and fix any warnings

### Phase 4: Documentation and Verification (30 minutes)

- [ ] Add module documentation explaining test infrastructure
- [ ] Add usage examples in documentation
- [ ] Run full test suite: `cargo test`
- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`
- [ ] Review code for any remaining complex patterns

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Infrastructure Checks**:

- [ ] `TestUserOutput` struct exists in test module
- [ ] `TestUserOutput::new()` creates test instance with specified verbosity
- [ ] `stdout()` and `stderr()` methods return captured output as strings
- [ ] `output()` method provides mutable access to wrapped `UserOutput`
- [ ] `clear()` method resets captured output
- [ ] Old `SharedWriter` and `create_test_user_output()` removed

**Test Migration Checks**:

- [ ] All tests using old infrastructure updated to new helpers
- [ ] No remaining references to `SharedWriter`
- [ ] No remaining calls to `create_test_user_output()`
- [ ] All tests pass with new infrastructure
- [ ] Test code is simpler and more readable

**Code Quality Checks**:

- [ ] Test infrastructure is well-documented
- [ ] Usage examples provided in documentation
- [ ] No unnecessary complexity in test helpers
- [ ] Standard library types used where possible (`Rc<RefCell<>>` instead of `Arc<Mutex<>>`)

## Related Documentation

- [Development Principles](../development-principles.md) - Testability and maintainability principles
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Testing Conventions](../contributing/testing/) - Testing best practices
- [Refactoring Plan](../refactors/plans/user-output-architecture-improvements.md) - Complete refactoring plan

## Notes

- This is purely a test infrastructure improvement - no changes to production code
- All behavioral changes are internal to tests
- Simplification reduces maintenance burden and makes tests easier to write
- Using `Rc<RefCell<>>` is appropriate for single-threaded test scenarios
- The wrapper pattern hides implementation details from test writers
