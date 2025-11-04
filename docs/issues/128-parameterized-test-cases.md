# Parameterized Test Cases for User Output

**Issue**: [#128](https://github.com/torrust/torrust-tracker-deployer/issues/128)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- Refactoring Plan: [docs/refactors/plans/user-output-architecture-improvements.md](../refactors/plans/user-output-architecture-improvements.md)
- Proposal #1 (Dependency): Simplify Test Infrastructure ([#123](https://github.com/torrust/torrust-tracker-deployer/issues/123))

## Overview

This task refactors the `UserOutput` test suite to use parameterized tests with `rstest`, eliminating significant code duplication. Currently, many tests are nearly identical, differing only in the method called, expected symbol, verbosity level, and output channel. By using parameterized tests, we can express the same behavior matrix with much less code while maintaining clarity and improving maintainability.

**Current State**: The codebase already has simplified test infrastructure (Proposal #1, Issue #123) with `TestUserOutput` helper in the `test_support` module. This helper provides:

- `TestUserOutput::new(verbosity)` - creates test instance with captured buffers
- `TestUserOutput::with_theme(verbosity, theme)` - creates test instance with custom theme
- `.stdout()` and `.stderr()` methods - easy access to captured output
- `.output` field - direct access to `UserOutput` for calling methods

This proposal builds on this foundation to replace duplicate tests with parameterized versions.

## Goals

- [ ] Add `rstest` dependency for parameterized testing
- [ ] Create parameterized tests for channel routing (stdout vs stderr)
- [ ] Create parameterized tests for verbosity level behavior
- [ ] Create parameterized tests for message formatting
- [ ] Remove duplicate test methods
- [ ] Maintain or improve test coverage
- [ ] Ensure all tests pass with parameterized approach
- [ ] Document the test matrix for future maintainers

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (Tests)
**Module Path**: `src/presentation/user_output.rs` (test module)
**Pattern**: Parameterized unit tests with rstest

### Module Structure Requirements

- [ ] Follow testing conventions (see [docs/contributing/testing/](../contributing/testing/))
- [ ] Use appropriate module organization for tests (see [docs/contributing/module-organization.md](../contributing/module-organization.md))
- [ ] Keep test code clean and maintainable

### Architectural Constraints

- [ ] Tests must be clear and self-documenting
- [ ] Test cases should act as behavior specification
- [ ] Parameterized tests should not sacrifice clarity
- [ ] Maintain independence between test cases

### Anti-Patterns to Avoid

- ❌ Overly complex test parameterization that reduces clarity
- ❌ Hidden dependencies between test cases
- ❌ Parameterization for the sake of parameterization (only where it reduces duplication)
- ❌ Loss of test coverage during refactoring

## Specifications

### Current Problem: Test Code Duplication

Many tests are nearly identical, creating maintenance burden:

```rust
#[test]
fn it_should_write_progress_messages_to_stderr() {
    let (mut output, stdout_buf, stderr_buf) = create_test_user_output(VerbosityLevel::Normal);
    output.progress("Testing progress message");
    let stderr_content = String::from_utf8(stderr_buf.lock().unwrap().clone()).unwrap();
    assert_eq!(stderr_content, "⏳ Testing progress message\n");
}

#[test]
fn it_should_write_success_messages_to_stderr() {
    let (mut output, stdout_buf, stderr_buf) = create_test_user_output(VerbosityLevel::Normal);
    output.success("Testing success message");
    let stderr_content = String::from_utf8(stderr_buf.lock().unwrap().clone()).unwrap();
    assert_eq!(stderr_content, "✅ Testing success message\n");
}

#[test]
fn it_should_write_warning_messages_to_stderr() {
    let (mut output, stdout_buf, stderr_buf) = create_test_user_output(VerbosityLevel::Normal);
    output.warn("Testing warning message");
    let stderr_content = String::from_utf8(stderr_buf.lock().unwrap().clone()).unwrap();
    assert_eq!(stderr_content, "⚠️ Testing warning message\n");
}
```

### Solution: Parameterized Tests with rstest

#### Add rstest Dependency

Add to `Cargo.toml` dev-dependencies section:

```toml
[dev-dependencies]
rstest = "0.18"
```

#### Parameterized Channel Routing Tests

Test that each message type goes to the correct output channel:

````rust
use rstest::rstest;

```rust
use super::test_support::TestUserOutput;

#[rstest]
#[case("progress", "⏳", VerbosityLevel::Normal, "stderr")]
#[case("success", "✅", VerbosityLevel::Normal, "stderr")]
#[case("warning", "⚠️", VerbosityLevel::Normal, "stderr")]
#[case("error", "❌", VerbosityLevel::Quiet, "stderr")]
#[case("result", "", VerbosityLevel::Normal, "stdout")]
fn it_should_write_to_correct_channel(
    #[case] method: &str,
    #[case] symbol: &str,
    #[case] min_verbosity: VerbosityLevel,
    #[case] channel: &str,
) {
    // Using the existing TestUserOutput helper from test_support module
    let mut test_output = TestUserOutput::new(min_verbosity);
    let message = "Test message";

    // Call the appropriate method on the .output field
    match method {
        "progress" => test_output.output.progress(message),
        "success" => test_output.output.success(message),
        "warning" => test_output.output.warn(message),
        "error" => test_output.output.error(message),
        "result" => test_output.output.result(message),
        _ => panic!("Unknown method: {}", method),
    }

    // Check correct channel using helper methods
    let expected = if symbol.is_empty() {
        format!("{}\n", message)
    } else {
        format!("{} {}\n", symbol, message)
    };

    match channel {
        "stdout" => {
            assert_eq!(test_output.stdout(), expected);
            assert_eq!(test_output.stderr(), "");
        }
        "stderr" => {
            assert_eq!(test_output.stderr(), expected);
            assert_eq!(test_output.stdout(), "");
        }
        _ => panic!("Unknown channel: {}", channel),
    }
}
````

#### Parameterized Verbosity Level Tests

Test that messages respect verbosity levels:

```rust
#[rstest]
#[case(VerbosityLevel::Quiet, false)]
#[case(VerbosityLevel::Normal, true)]
#[case(VerbosityLevel::Verbose, true)]
fn it_should_respect_verbosity_for_progress(
    #[case] verbosity: VerbosityLevel,
    #[case] should_show: bool,
) {
    let mut test_output = TestUserOutput::new(verbosity);
    test_output.output.progress("Test");

    if should_show {
        assert!(!test_output.stderr().is_empty());
    } else {
        assert_eq!(test_output.stderr(), "");
    }
}

#[rstest]
#[case(VerbosityLevel::Quiet, false)]
#[case(VerbosityLevel::Normal, true)]
#[case(VerbosityLevel::Verbose, true)]
fn it_should_respect_verbosity_for_success(
    #[case] verbosity: VerbosityLevel,
    #[case] should_show: bool,
) {
    let mut test_output = TestUserOutput::new(verbosity);
    test_output.output.success("Test");

    if should_show {
        assert!(!test_output.stderr().is_empty());
    } else {
        assert_eq!(test_output.stderr(), "");
    }
}

#[rstest]
#[case(VerbosityLevel::Quiet, true)]
#[case(VerbosityLevel::Normal, true)]
#[case(VerbosityLevel::Verbose, true)]
fn it_should_always_show_errors_regardless_of_verbosity(
    #[case] verbosity: VerbosityLevel,
    #[case] should_show: bool,
) {
    let mut test_output = TestUserOutput::new(verbosity);
    test_output.output.error("Test");

    assert!(should_show);
    assert!(!test_output.stderr().is_empty());
}
```

#### Parameterized Formatting Tests

Test that messages are formatted correctly with their symbols:

```rust
#[rstest]
#[case("progress", "⏳ Test message\n")]
#[case("success", "✅ Test message\n")]
#[case("warning", "⚠️  Test message\n")]  // Note: warning has extra space in current code
#[case("error", "❌ Test message\n")]
fn it_should_format_messages_with_correct_symbol(
    #[case] method: &str,
    #[case] expected_output: &str,
) {
    let mut test_output = TestUserOutput::new(VerbosityLevel::Normal);

    match method {
        "progress" => test_output.output.progress("Test message"),
        "success" => test_output.output.success("Test message"),
        "warning" => test_output.output.warn("Test message"),
        "error" => test_output.output.error("Test message"),
        _ => panic!("Unknown method: {}", method),
    }

    assert_eq!(test_output.stderr(), expected_output);
}
```

#### Parameterized Multi-Method Tests

Test behavior across multiple methods at once:

```rust
#[rstest]
#[case(VerbosityLevel::Quiet)]
#[case(VerbosityLevel::Normal)]
#[case(VerbosityLevel::Verbose)]
fn it_should_handle_result_messages_at_all_verbosity_levels(
    #[case] verbosity: VerbosityLevel,
) {
    let mut test_output = TestUserOutput::new(verbosity);
    test_output.output.result("Result data");

    // Results always go to stdout, no symbol
    assert_eq!(test_output.stdout(), "Result data\n");
    assert_eq!(test_output.stderr(), "");
}
```

## Implementation Plan

### Phase 1: Setup and Dependencies (estimated: 30 minutes)

- [ ] Add `rstest = "0.18"` to `Cargo.toml` dev-dependencies
- [ ] Verify rstest works with a simple test case
- [ ] Document decision to use rstest in code comments

### Phase 2: Identify Test Patterns (estimated: 1 hour)

- [ ] Analyze existing tests to identify duplication patterns
- [ ] Group tests by what they're testing (channel routing, verbosity, formatting)
- [ ] Document the test matrix (what combinations need coverage)
- [ ] Create checklist of all existing test coverage to maintain

### Phase 3: Create Parameterized Channel Routing Tests (estimated: 2 hours)

- [ ] Write `it_should_write_to_correct_channel` parameterized test
- [ ] Add test cases for all message types (progress, success, warning, error, result)
- [ ] Verify all cases pass
- [ ] Document the test matrix in comments

### Phase 4: Create Parameterized Verbosity Tests (estimated: 2 hours)

- [ ] Write parameterized verbosity tests for normal messages (progress, success, warning)
- [ ] Write parameterized test for error messages (always shown)
- [ ] Write parameterized test for result messages
- [ ] Verify all verbosity combinations are covered
- [ ] Compare coverage with original tests

### Phase 5: Create Parameterized Formatting Tests (estimated: 1 hour)

- [ ] Write `it_should_format_messages_with_correct_symbol` parameterized test
- [ ] Add test cases for all symbols
- [ ] Verify formatting is correct for all message types
- [ ] Add edge cases if any exist

### Phase 6: Remove Duplicate Tests (estimated: 1 hour)

- [ ] Identify tests that are now covered by parameterized tests
- [ ] Remove duplicate test methods
- [ ] Keep any unique tests that aren't covered by parameterization
- [ ] Document why remaining non-parameterized tests exist

### Phase 7: Validation and Documentation (estimated: 1 hour)

- [ ] Run full test suite and verify 100% pass rate
- [ ] Check test coverage hasn't decreased (`cargo llvm-cov`)
- [ ] Document the test matrix in module-level comments
- [ ] Add examples of how to add new test cases
- [ ] Update any test documentation

### Phase 8: Integration Testing (estimated: 30 minutes)

- [ ] Run pre-commit checks
- [ ] Run all E2E tests
- [ ] Verify no regressions
- [ ] Check CI/CD pipeline if available

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
  - [ ] No unused dependencies (`cargo machete`)
  - [ ] All linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
  - [ ] All unit tests pass (`cargo test`)
  - [ ] Documentation builds successfully (`cargo doc`)
  - [ ] All E2E tests pass (config, provision, full suite)

**Task-Specific Criteria**:

- [ ] `rstest` dependency is added to dev-dependencies
- [ ] Parameterized tests exist for channel routing (stdout vs stderr)
- [ ] Parameterized tests exist for verbosity level behavior
- [ ] Parameterized tests exist for message formatting
- [ ] Duplicate test methods have been removed
- [ ] Test coverage is maintained or improved (verify with `cargo llvm-cov`)
- [ ] All tests pass without failures
- [ ] Test matrix is documented in code comments
- [ ] Code is more maintainable (less duplication, clearer intent)

**Coverage Requirements**:

- [ ] All message types are covered (progress, success, warning, error, result, steps)
- [ ] All verbosity levels are tested (Quiet, Normal, Verbose)
- [ ] Both output channels are tested (stdout, stderr)
- [ ] All message symbols are verified
- [ ] Edge cases are covered (empty messages, special characters if relevant)

**Documentation Requirements**:

- [ ] Parameterized test structure is documented
- [ ] Test matrix is explained (what combinations are tested)
- [ ] Instructions for adding new test cases are provided
- [ ] Rationale for using rstest is documented

## Related Documentation

- [User Output Refactoring Plan](../refactors/plans/user-output-architecture-improvements.md) - Complete refactoring context
- [Testing Conventions](../contributing/testing/) - Testing best practices
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [rstest Documentation](https://docs.rs/rstest/) - Official rstest documentation
- [Development Principles](../development-principles.md) - Testability and maintainability principles

## Notes

### Why rstest?

We chose `rstest` over other parameterized testing approaches because:

1. **Clear Syntax**: Test cases are defined with `#[case(...)]` attributes, making the test matrix immediately visible
2. **Type Safety**: Compile-time checking of parameter types
3. **Well Maintained**: Actively maintained crate with good community support
4. **Minimal Overhead**: Doesn't require significant boilerplate or complex setup
5. **Readable Output**: Test failures show which specific case failed with clear parameter values

### Dependencies

**✅ RESOLVED**: All dependencies are now complete:

- ✅ Proposal #1 (Simplified Test Infrastructure) - Issue #123 - `TestUserOutput` helper with `.stdout()` and `.stderr()` methods is implemented

This proposal can now proceed without waiting for other work. The existing `TestUserOutput` helper makes parameterized tests straightforward to implement.

### Benefits of Parameterized Tests

- ✅ **Reduced Duplication**: One test implementation covers multiple cases
- ✅ **Easier to Extend**: Adding new cases is as simple as adding a `#[case(...)]` line
- ✅ **Clear Specification**: Test cases act as behavior documentation
- ✅ **Maintainability**: Changes affect one test instead of many
- ✅ **Better Coverage**: Easier to ensure all combinations are tested
- ✅ **Consistent Testing**: All cases use the same testing logic

### Test Matrix Example

After this refactoring, the test matrix will be clearly documented:

```text
Message Type | Symbol | Verbosity | Channel | Always Shown
-------------|--------|-----------|---------|-------------
progress     | ⏳     | Normal    | stderr  | No
success      | ✅     | Normal    | stderr  | No
warning      | ⚠️     | Normal    | stderr  | No
error        | ❌     | Quiet     | stderr  | Yes
result       | (none) | Quiet     | stdout  | Yes
steps        | (none) | Normal    | stderr  | No
```

This matrix is encoded in the parameterized tests, making it easy to verify coverage.

### Maintenance Impact

**Before**: Adding a new message type requires writing 4-5 separate test methods (channel routing, verbosity, formatting, etc.)

**After**: Adding a new message type requires adding 1-2 `#[case(...)]` lines to existing parameterized tests

This dramatically reduces the maintenance burden when extending the system.

### Alternative Approaches Considered

1. **Macros**: Custom macros to generate tests - Rejected because less readable and harder to debug
2. **Test Generators**: Runtime test generation - Rejected because rstest provides better compile-time safety
3. **Manual Loops**: Loop over test cases in a single test - Rejected because failures don't show which case failed clearly

### Future Enhancements

After this implementation:

- New message types can be added to test matrix with minimal effort
- Test coverage reports will be more meaningful (less duplication)
- Test maintenance becomes significantly easier
- Pattern can be applied to other test suites in the project
