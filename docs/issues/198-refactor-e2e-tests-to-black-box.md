# Refactor E2E tests to use black-box CLI execution

**Epic Issue**: #198

**Related Issues**:

- Issue #199 (Task 1: Refactor e2e_provision_and_destroy_tests)
- Issue #200 (Task 2: Refactor e2e_tests_full)

## 📋 Overview

Currently, `src/bin/e2e_provision_and_destroy_tests.rs` and `src/bin/e2e_tests_full.rs` are implemented as integration tests that import and call internal library functions directly. This couples the tests to the internal implementation and doesn't test the actual CLI interface used by end-users.

This epic aims to refactor these binaries to be "real" E2E tests that execute the CLI as a black box, similar to `tests/e2e_create_command.rs` and `tests/e2e_destroy_command.rs`.

## 🎯 Goals

- [ ] Decouple E2E tests from internal implementation details (Application/Domain layers).
- [ ] Test the public CLI interface exactly as an end-user would use it.
- [ ] Simplify test maintenance (tests only change if CLI API changes).
- [ ] Ensure consistent testing patterns across the codebase.

## ⚙️ Specifications

The refactored tests should follow the pattern established in `tests/e2e_create_command.rs`:

1. **Isolation**: Create a temporary directory for the test execution.
2. **Black Box Execution**: Execute the deployer using the CLI command (e.g., via `cargo run`).
3. **Verification**: Check the process exit code and validate the side effects (e.g., infrastructure created, files generated).

### Example Pattern

```rust
// Arrange
let temp_workspace = TempWorkspace::new()?;
let config_file = temp_workspace.write_config_file(...)?;

// Act
let result = ProcessRunner::new()
    .working_dir(temp_workspace.path())
    .run_command(&["provision", "environment", ...])?;

// Assert
assert!(result.success());
// Verify infrastructure state...
```

### Architectural Requirements

- **DDD Layer**: `tests/` (or `src/bin/` for these specific binaries if they remain as binaries).
- **Module Path**: `src/bin/`
- **Architectural Constraints**:
  - Tests must NOT import `torrust_tracker_deployer_lib::application` or `torrust_tracker_deployer_lib::domain` directly for executing logic.
  - Tests should only use the library for shared types or test support utilities if necessary.
  - All operations must be performed via the CLI arguments.

## 📅 Implementation Plan

### Phase 1: Refactor `e2e_provision_and_destroy_tests`

1. **Generate New Version**: Create `src/bin/e2e_provision_and_destroy_tests_new.rs` implementing the black-box pattern.
2. **Verify**: Ensure it passes pre-commit checks and manual execution.
3. **CI Integration**: Add the new binary to `.github/workflows/test-e2e-provision.yml`.
4. **Replace**: Delete the old binary and rename the new one to `src/bin/e2e_provision_and_destroy_tests.rs`.
5. **Cleanup**: Update workflow to remove the temporary binary reference.

### Phase 2: Refactor `e2e_tests_full`

1. **Generate New Version**: Create `src/bin/e2e_tests_full_new.rs` implementing the black-box pattern.
2. **Verify**: Ensure it passes pre-commit checks and manual execution.
3. **Replace**: Delete the old binary and rename the new one to `src/bin/e2e_tests_full.rs`.

## ✅ Acceptance Criteria

- [ ] `src/bin/e2e_provision_and_destroy_tests.rs` executes `provision` and `destroy` commands via CLI.
- [ ] `src/bin/e2e_tests_full.rs` executes the full lifecycle via CLI commands.
- [ ] No direct calls to `command_handlers` from these test binaries.
- [ ] CI workflows pass with the refactored tests.
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
