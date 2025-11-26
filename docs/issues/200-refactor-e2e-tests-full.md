# Refactor e2e_tests_full to use black-box CLI execution

**Issue**: #200
**Parent Epic**: #198 - Refactor E2E tests to use black-box CLI execution

## 📋 Overview

Refactor `src/bin/e2e_tests_full.rs` to be a "real" E2E test that executes the CLI as a black box. This test covers the full lifecycle including configuration and validation.

## 🎯 Goals

- Decouple the test from internal implementation details.
- Test the public CLI interface for the full lifecycle.
- Ensure consistent testing patterns with `e2e_provision_and_destroy_tests.rs`.

## ⚙️ Specifications

The refactored test should follow the pattern established in Issue #199:

1. **Project-Local Directories**: Use project directories (`./data`, `./build`) with a fixed environment name (`e2e-full`) for predictable cleanup.
2. **Black Box Execution**: Execute CLI commands via `ProcessRunner`:
   - `cargo run -- create environment --env-file <config-file>`
   - `cargo run -- provision <environment-name>`
   - `cargo run -- configure <environment-name>`
   - `cargo run -- test <environment-name>`
   - `cargo run -- destroy <environment-name>`
3. **Verification**: Check process exit codes only (success = exit code 0).

### Key Changes & Requirements

1. **Use Existing Test Support Code**: Use `ProcessRunner` from `src/testing/black_box/` (already moved in Issue #199).
2. **Add New ProcessRunner Methods**: Add `run_configure_command` and `run_test_command` methods.
3. **Dependencies**: Keep `verify_required_dependencies` to ensure the test fails for the right reasons if tools are missing.
4. **Dynamic Config Generation**: Generate config file at runtime with absolute SSH key paths (learned from Issue #199: static fixtures don't work because Ansible runs from build directory).
5. **Cleanup**: Maintain "Preflight Cleanup" using utilities from `src/testing/` module.
6. **Context**: Remove `TestContext` as it simulates internal bootstrapping which is no longer needed for black-box testing.
7. **Tasks**: Replace internal tasks with direct `ProcessRunner` calls to the CLI.

### Architectural Requirements

- **DDD Layer**: `src/bin/`
- **Module Path**: `src/bin/e2e_tests_full.rs`
- **Architectural Constraints**:
  - No direct imports of `application` or `domain` layers for logic execution.
  - Imports from `src/testing/` module are allowed (cross-cutting concern, not a DDD layer).
  - Use `ProcessRunner` (in `src/testing/black_box`) to execute the CLI.

### Design Decisions (Inherited from Issue #199)

1. **No TempWorkspace**: Use project-local directories (`./data`, `./build`) instead of temporary directories for:

   - Predictable LXD resource cleanup (fixed instance name `torrust-tracker-vm-e2e-full`)
   - Easier debugging (artifacts persist after test run)
   - Simpler preflight cleanup (known paths)

2. **Dynamic Config Generation**: Generate `envs/e2e-full.json` at runtime with absolute paths (NOT static fixture):

   - Static fixtures don't work because relative paths fail when Ansible runs from build directory
   - Generate config with absolute SSH key paths at runtime

3. **Exit Code Verification Only**: Trust CLI exit codes for now. Future enhancement: use `list` command to verify environment state.

4. **Positive Boolean Semantics**: Use `destroy: bool` parameter (true = destroy) instead of `keep: bool` (true = skip destroy) for clearer intent.

## 📅 Implementation Plan

1. **Add ProcessRunner Methods**:

   - Add `run_configure_command` method to `ProcessRunner`
   - Add `run_test_command` method to `ProcessRunner`

2. **Refactor Binary**: Update `src/bin/e2e_tests_full.rs`:

   - Remove `TestContext` usage
   - Remove direct `command_handlers` imports
   - Remove `tokio` async runtime (CLI execution is synchronous)
   - Use `ProcessRunner` to execute CLI commands:
     - `create environment --env-file <generated-config>`
     - `provision e2e-full`
     - `configure e2e-full`
     - `test e2e-full`
     - `destroy e2e-full`
   - Keep `verify_required_dependencies`
   - Keep preflight cleanup (from `testing` module)
   - Verify success via exit codes only
   - Generate config file at runtime with absolute paths

3. **Verify**: Ensure it passes pre-commit checks and manual execution.

4. **CI Integration**: This test is LOCAL ONLY (cannot run on GitHub Actions due to network issues in LXD VMs on GitHub runners).

## ✅ Acceptance Criteria

- [ ] `src/bin/e2e_tests_full.rs` executes `create environment`, `provision`, `configure`, `test`, and `destroy` commands via CLI.
- [ ] No direct calls to `command_handlers` for logic execution.
- [ ] Uses dynamically generated config file with absolute SSH key paths.
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Binary can be run locally: `cargo run --bin e2e-tests-full`
