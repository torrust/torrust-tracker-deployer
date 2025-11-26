# Refactor e2e_tests_full to use black-box CLI execution

**Issue**: #200
**Parent Epic**: #198 - Refactor E2E tests to use black-box CLI execution

## 📋 Overview

Refactor `src/bin/e2e_tests_full.rs` to be a "real" E2E test that executes the CLI as a black box. This test covers the full lifecycle including configuration.

## 🎯 Goals

- Decouple the test from internal implementation details.
- Test the public CLI interface for the full lifecycle.
- Ensure consistent testing patterns.

## ⚙️ Specifications

The refactored test should follow the pattern:

1. **Isolation**: Create a temporary directory.
2. **Black Box Execution**: Execute `cargo run -- ...` for all steps (provision, configure, test, destroy).
3. **Verification**: Check process exit codes and validate system state.

### Key Changes & Requirements

1. **Move Test Support Code**: (Same as Issue #199) Ensure `ProcessRunner` and friends are available in `src/testing/black_box/`.
2. **Dependencies**: Keep `verify_required_dependencies`.
3. **Fixtures**: Use SSH key fixtures from `fixtures/` and pass their paths to the CLI arguments.
4. **Cleanup**: Maintain "Preflight Cleanup".
5. **Context**: Remove `TestContext`.
6. **Tasks**: Replace internal tasks with direct `ProcessRunner` calls to the CLI for:
   - `provision`
   - `configure`
   - `test` (verify)
   - `destroy`

### Architectural Requirements

- **DDD Layer**: `src/bin/`
- **Module Path**: `src/bin/e2e_tests_full.rs`
- **Architectural Constraints**:
  - No direct imports of `application` or `domain` layers for logic.
  - Use `ProcessRunner` (moved to `src/testing/black_box`) to execute the CLI.

## 📅 Implementation Plan

1. **Generate New Version**: Create `src/bin/e2e_tests_full_new.rs` by copying the existing file.

   - **Refactor Strategy**:
     - Replace `TestContext` initialization with `TempWorkspace` creation.
     - Replace `run_provision_command` with `ProcessRunner::run_provision_command`.
     - Replace `run_configure_command` with `ProcessRunner::run_configure_command` (implement this method).
     - Replace `run_test_command` with `ProcessRunner::run_test_command` (implement this method).
     - Replace `run_destroy_command` with `ProcessRunner::run_destroy_command`.
     - Ensure `verify_required_dependencies` and `preflight_cleanup` are called.

2. **Verify**: Ensure it passes pre-commit checks and manual execution.
3. **Replace**: Delete the old binary and rename the new one to `src/bin/e2e_tests_full.rs`.

## ✅ Acceptance Criteria

- [ ] `src/bin/e2e_tests_full.rs` executes the full lifecycle via CLI commands.
- [ ] No direct calls to `command_handlers`.
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
