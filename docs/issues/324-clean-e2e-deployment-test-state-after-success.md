# Clean up E2E deployment test state after successful test run

**Issue**: #324
**Parent Epic**: None
**Related**: None

## Overview

The E2E deployment workflow tests (`src/bin/e2e_deployment_workflow_tests.rs`) leave stale environment state after a successful test run. The environment `e2e-deployment` remains listed as "Running" when querying `cargo run -- list`, even though the Docker container was already removed by testcontainers.

This is misleading and pollutes the environment list with phantom environments.

## Goals

- [ ] Automatically clean up internal state (`data/`, `build/`, `templates/`) after a successful E2E deployment test run
- [ ] Preserve state on failure for debugging purposes
- [ ] Ensure `cargo run -- list` does not show `e2e-deployment` after a successful test

## 🏗️ Architecture Requirements

**DDD Layer**: Testing (test infrastructure code, not production DDD layers)
**Module Path**: `src/bin/e2e_deployment_workflow_tests.rs`
**Pattern**: Test binary post-test cleanup

### Module Structure Requirements

- [ ] Changes are confined to test infrastructure code (`src/bin/` and `src/testing/`)
- [ ] No changes to production DDD layers needed

### Architectural Constraints

- [ ] Post-test cleanup uses direct function calls (not CLI commands) for consistency with the container-based test pattern (see implementation approach rationale)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Mixing CLI purge command with direct container management in the same test (increases cognitive load)
- ❌ Cleaning up on failure (state must be preserved for debugging)

## Specifications

### Current Behavior

After running `cargo run --bin e2e-deployment-workflow-tests`, the test:

1. Runs preflight cleanup (`run_container_preflight_cleanup`) — removes `build/`, `templates/`, `data/` from previous runs
2. Creates environment, registers instance, configures, releases, runs services
3. Stops the Docker container (via `stop_test_infrastructure`)
4. **Does NOT clean up** internal state (`data/e2e-deployment/`, `build/e2e-deployment/`, `templates/e2e-deployment/`)

Result: `cargo run -- list` shows `e2e-deployment` in "Running" state.

### Expected Behavior

On **success**: the test should clean up the environment state by removing `data/`, `build/`, and `templates/` directories. After a successful run, `cargo run -- list` should not show `e2e-deployment`.

On **failure**: state is preserved as-is for debugging. No post-test cleanup runs.

### Existing Infrastructure

The following building blocks already exist:

- **`run_container_preflight_cleanup()`** in `src/testing/e2e/tasks/black_box/preflight_cleanup.rs` — cleans `build/`, `templates/`, `data/` directories + hanging Docker containers before a test run
- **`ProcessRunner::run_purge_command()`** in `src/testing/e2e/process_runner.rs` — executes `cargo run -- purge <env> --force` as a CLI command
- **`E2eTestRunner`** in `src/testing/e2e/tasks/black_box/test_runner.rs` — orchestrates CLI commands; has `destroy_infrastructure()` but no `purge_environment()` method

### Implementation Approach: Reuse Cleanup Functions (Option A)

Reuse the existing `run_container_preflight_cleanup()` function (or its underlying directory cleanup logic) in the success path of the test binary.

**Why not use the CLI `purge` command (Option B)?**

The deployment E2E test uses Docker containers — provision and destroy of the instance are managed manually outside the deployer workflow. The `purge` command only affects the local machine (not the instance), so it _could_ technically work, but using it would mix the black-box CLI approach with the direct container management approach. This creates unnecessary cognitive load: the same cleanup operation would be done in two different ways within the same test. Reusing the cleanup functions is more consistent with the test's existing pattern.

Additionally, coupling the test to the internal state (knowing which directories to clean) is acceptable — even beneficial — for a test. It explicitly documents what artifacts the test produces and what must be cleaned up.

## Implementation Plan

### Phase 1: Add post-success cleanup to test binary

- [ ] Task 1.1: In `src/bin/e2e_deployment_workflow_tests.rs`, in the `main()` function's success branch (after `match test_result { Ok(()) => { ... } }`), call `run_container_preflight_cleanup(ENVIRONMENT_NAME)` to remove `data/`, `build/`, and `templates/` directories for the environment
- [ ] Task 1.2: Wrap the cleanup call so that a cleanup failure does not mask the test success — log a warning and continue

### Phase 2: Verify and test

- [ ] Task 2.1: Run `cargo run --bin e2e-deployment-workflow-tests` and verify `cargo run -- list` does not show `e2e-deployment` afterward
- [ ] Task 2.2: Verify that on test failure, the state is preserved (check `data/e2e-deployment/environment.json` still exists)
- [ ] Task 2.3: Check if `e2e_complete_workflow_tests.rs` has the same issue (it runs `destroy` so state shows "Destroyed" — less misleading but still leaves artifacts; decide if purge is needed there too as a follow-up)

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] After a successful E2E deployment test run, `cargo run -- list` does not show `e2e-deployment`
- [ ] After a failed E2E deployment test run, `e2e-deployment` state is preserved for debugging
- [ ] Cleanup is logged appropriately (info level on success)
- [ ] Cleanup failure does not mask the original test success (warn and continue, don't return error)

## Related Documentation

- Preflight cleanup: `src/testing/e2e/tasks/black_box/preflight_cleanup.rs`
- Test runner: `src/testing/e2e/tasks/black_box/test_runner.rs`
- Process runner (purge command): `src/testing/e2e/process_runner.rs`
- Purge command handler: `src/application/command_handlers/purge/mod.rs`

## Notes

- The `e2e_infrastructure_lifecycle_tests.rs` does not have this problem because it runs `destroy` as part of its test workflow, which transitions the state to "Destroyed"
- The `e2e_complete_workflow_tests.rs` also runs `destroy`, so the state shows "Destroyed" rather than "Running" — less misleading but still leaves local artifacts. Consider purging there too as a follow-up
- **Decision**: Use Option A (reuse `run_container_preflight_cleanup()`) instead of the CLI `purge` command. Rationale: the deployment test manages containers manually outside the deployer workflow. Using the CLI purge would mix two different approaches (black-box CLI + direct container management) in the same test, increasing cognitive load. Reusing cleanup functions is consistent with the test's existing pattern and explicitly documents what artifacts must be cleaned
