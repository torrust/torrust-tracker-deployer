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

1. **Project-Local Directories**: Use project directories (`./data`, `./build`) with a fixed environment name (`e2e-provision`) for predictable cleanup.
2. **Black Box Execution**: Execute CLI commands via `ProcessRunner`:
   - `cargo run -- create environment --env-file <config-file>`
   - `cargo run -- provision <environment-name>`
   - `cargo run -- destroy <environment-name>`
3. **Verification**: Check process exit codes only (success = exit code 0). Future enhancement: use `list` command to verify environment state.

### Key Changes & Requirements

1. **Move Shared Test Support Code**: Move only `tests/support/process_runner.rs` to `src/testing/black_box/process_runner.rs` (shared between `src/bin/` and `tests/`). Keep `TempWorkspace` and `assertions` in `tests/support/` (only used by tests).
2. **Dependencies**: Keep `verify_required_dependencies` to ensure the test fails for the right reasons if tools are missing.
3. **Fixtures**: Use a static fixture file `fixtures/e2e-provision/environment.json` with pre-filled configuration (no runtime JSON generation).
4. **Cleanup**: Maintain "Preflight Cleanup" using utilities from `src/testing/` module (allowed since `testing` is not a DDD layer).
5. **Context**: Remove `TestContext` as it simulates internal bootstrapping which is no longer needed for black-box testing.
6. **Tasks**: Replace internal tasks like `run_provision_command` and `run_destroy_command` with direct `ProcessRunner` calls to the CLI.

### Architectural Requirements

- **DDD Layer**: `src/bin/`
- **Module Path**: `src/bin/e2e_provision_and_destroy_tests.rs`
- **Architectural Constraints**:
  - No direct imports of `application` or `domain` layers for logic execution.
  - Imports from `src/testing/` module are allowed (cross-cutting concern, not a DDD layer).
  - Use `ProcessRunner` (in `src/testing/black_box`) to execute the CLI.

### Design Decisions

1. **No TempWorkspace**: Use project-local directories (`./data`, `./build`) instead of temporary directories for:

   - Predictable LXD resource cleanup (fixed instance name `torrust-tracker-vm-e2e-provision`)
   - Easier debugging (artifacts persist after test run)
   - Simpler preflight cleanup (known paths)

2. **Static Fixture Configuration**: Use `fixtures/e2e-provision/environment.json` instead of generating JSON programmatically:

   - Simpler implementation
   - If config schema changes, update fixture once (same effort as updating programmatic generation)
   - No runtime template processing

3. **Minimal Code Movement**: Only move `ProcessRunner` to `src/testing/black_box/` since it's shared. Keep `TempWorkspace` and `assertions` in `tests/support/` (local to tests).

4. **Exit Code Verification Only**: Trust CLI exit codes for now. Future enhancement: use `list` command to verify environment state after provisioning.

## 📅 Implementation Plan

1. **Create Fixture File**: Create `fixtures/e2e-provision/environment.json` with pre-filled configuration.

2. **Move ProcessRunner**:

   - Move `tests/support/process_runner.rs` -> `src/testing/black_box/process_runner.rs`
   - Add `run_provision_command` method to `ProcessRunner`
   - Update `src/testing/mod.rs` to expose `black_box` module
   - Update `tests/support/mod.rs` to re-export `ProcessRunner` from `src/testing/black_box` (minimize churn)

3. **Refactor Binary**: Update `src/bin/e2e_provision_and_destroy_tests.rs`:

   - Remove `TestContext` usage
   - Remove direct `command_handlers` imports
   - Use `ProcessRunner` to execute CLI commands:
     - `create environment --env-file fixtures/e2e-provision/environment.json`
     - `provision e2e-provision`
     - `destroy e2e-provision`
   - Keep `verify_required_dependencies`
   - Keep preflight cleanup (from `testing` module)
   - Verify success via exit codes only

4. **Verify**: Ensure it passes pre-commit checks and manual execution.

5. **CI Integration**: Verify `.github/workflows/test-e2e-provision.yml` still works (no changes expected).

## ✅ Acceptance Criteria

- [ ] `src/bin/e2e_provision_and_destroy_tests.rs` executes `create environment`, `provision`, and `destroy` commands via CLI.
- [ ] No direct calls to `command_handlers` for logic execution.
- [ ] Uses static fixture file `fixtures/e2e-provision/environment.json`.
- [ ] CI workflow `test-e2e-provision.yml` passes.
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
