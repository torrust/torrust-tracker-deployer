# Refactor e2e_provision_and_destroy_tests to use black-box CLI execution

**Issue**: #199
**Parent Epic**: #198 - Refactor E2E tests to use black-box CLI execution

## 📋 Overview

Refactor `src/bin/e2e_provision_and_destroy_tests.rs` to be a "real" E2E test that executes the CLI as a black box, similar to `tests/e2e_create_command.rs`.

## 🎯 Goals

- Decouple the test from internal implementation details.
- Test the public CLI interface for provisioning and destruction.
- Ensure consistent testing patterns.

## ⚙️ Specifications

The refactored test should follow the pattern:

1. **Isolation**: Create a temporary directory.
2. **Black Box Execution**: Execute `cargo run -- provision ...` and `cargo run -- destroy ...`.
3. **Verification**: Check process exit codes and validate infrastructure state (e.g., using `lxc list` or checking for state files).

### Key Changes & Requirements

1. **Move Test Support Code**: Move `tests/support/process_runner.rs`, `tests/support/assertions.rs`, and `tests/support/temp_workspace.rs` to `src/testing/black_box/` so they can be used by the binaries in `src/bin/`.
2. **Dependencies**: Keep `verify_required_dependencies` to ensure the test fails for the right reasons if tools are missing.
3. **Fixtures**: Use SSH key fixtures from `fixtures/` and pass their paths to the CLI arguments.
4. **Cleanup**: Maintain "Preflight Cleanup" to handle interrupted previous runs.
5. **Context**: Remove `TestContext` as it simulates internal bootstrapping which is no longer needed for black-box testing.
6. **Tasks**: Replace internal tasks like `run_provision_command` and `run_destroy_command` with direct `ProcessRunner` calls to the CLI.

### Architectural Requirements

- **DDD Layer**: `src/bin/`
- **Module Path**: `src/bin/e2e_provision_and_destroy_tests.rs`
- **Architectural Constraints**:
  - No direct imports of `application` or `domain` layers for logic.
  - Use `ProcessRunner` (moved to `src/testing/black_box`) to execute the CLI.

## 📅 Implementation Plan

1. **Move Test Support Code**:

   - Move `tests/support/process_runner.rs` -> `src/testing/black_box/process_runner.rs`
   - Move `tests/support/assertions.rs` -> `src/testing/black_box/assertions.rs`
   - Move `tests/support/temp_workspace.rs` -> `src/testing/black_box/temp_workspace.rs`
   - Update `src/testing/mod.rs` to expose `black_box` module.
   - Update existing tests in `tests/` to import from `src/testing/black_box` (or re-export them in `tests/support/mod.rs` to minimize churn).

2. **Generate New Version**: Create `src/bin/e2e_provision_and_destroy_tests_new.rs` by copying the existing file.

   - **Refactor Strategy**:
     - Replace `TestContext` initialization with `TempWorkspace` creation.
     - Replace `run_provision_command` with `ProcessRunner::run_provision_command` (implement this method in `ProcessRunner`).
     - Replace `run_destroy_command` with `ProcessRunner::run_destroy_command`.
     - Ensure `verify_required_dependencies` is called.
     - Ensure `preflight_cleanup` is called (might need adaptation to work without `TestContext` or just use `lxc` commands directly via `std::process::Command` or a helper).

3. **Verify**: Ensure it passes pre-commit checks and manual execution.
4. **CI Integration**: Add the new binary to `.github/workflows/test-e2e-provision.yml`.
5. **Replace**: Delete the old binary and rename the new one to `src/bin/e2e_provision_and_destroy_tests.rs`.
6. **Cleanup**: Update workflow to remove the temporary binary reference.

## ✅ Acceptance Criteria

- [ ] `src/bin/e2e_provision_and_destroy_tests.rs` executes `provision` and `destroy` commands via CLI.
- [ ] No direct calls to `command_handlers`.
- [ ] CI workflow `test-e2e-provision.yml` passes.
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
