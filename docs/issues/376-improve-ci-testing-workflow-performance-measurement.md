# Improve CI Testing Workflow Performance Measurement

**Issue**: #376
**Parent Epic**: N/A
**Related**:

## Overview

The GitHub Actions testing workflow (`.github/workflows/testing.yml`) is slow on GitHub runners. This task improves performance measurement and visibility by:

1. Adding explicit `cargo fetch` step to measure dependency download time
2. Using `cargo nextest` for test execution with per-test timing information
3. Building test binaries before running tests to separate build time from test execution time

This enables identification of performance bottlenecks in the CI pipeline and provides data for future optimizations.

## Goals

- [ ] Add `cargo fetch` step to measure dependency download time separately
- [ ] Separate test binary build step from test execution step
- [ ] Replace `cargo test` with `cargo nextest` for better test timing visibility
- [ ] Document tests that take more than 1 second

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (GitHub Actions workflow configuration)
**Module Path**: `.github/workflows/testing.yml`
**Pattern**: CI/CD configuration

### Module Structure Requirements

- [ ] YAML workflow syntax compliance
- [ ] Maintain existing job dependencies and matrix strategies
- [ ] Preserve test isolation verification step

### Architectural Constraints

- [ ] No breaking changes to existing CI behavior
- [ ] All tests must continue to pass
- [ ] Test isolation verification must remain functional

### Anti-Patterns to Avoid

- ❌ Removing existing test coverage
- ❌ Breaking job dependencies between format → check → unit/build
- ❌ Changing behavior of other jobs (format, check, build)

## Specifications

### Changes to `unit` Job in `.github/workflows/testing.yml`

The current `unit` job structure in testing.yml line 114-147 needs three additions:

**Add 1: Install cargo-nextest** (after `cache` step):

```yaml
- id: tools
  name: Install Tools
  uses: taiki-e/install-action@v2
  with:
    tool: cargo-nextest
```

**Add 2: Explicit dependency fetch** (after `tools` step):

```yaml
- id: fetch
  name: Download Dependencies
  run: cargo fetch --verbose
```

**Add 3: Separate test binary build** (after `fetch`, before `test-docs`):

```yaml
- id: build
  name: Build Test Binaries
  run: cargo build --tests --benches --examples --workspace --all-targets --all-features --verbose
```

**Change 4: Replace test command**:

Replace:

```yaml
- id: test
  name: Run Unit Tests
  run: cargo test --tests --benches --examples --workspace --all-targets --all-features
```

With:

```yaml
- id: test
  name: Run Unit Tests
  run: cargo nextest run --all-targets --workspace --all-features
```

**Why these changes**:

- **cargo fetch**: Shows how long dependency download takes (important for cold cache scenarios)
- **cargo build**: Separates compilation time from test execution time
- **cargo nextest**: Provides per-test timing information to identify slow tests
- **--all-targets**: Ensures benches and examples are tested, maintaining parity with `cargo test`

### Tests Taking More Than 1 Second (cargo nextest run)

Based on local execution with `cargo nextest run --all-targets --workspace --all-features`, the following tests exceed 1 second:

**File lock multiprocess tests (1-6 seconds)**:

- `torrust-tracker-deployer::file_lock_multiprocess::it_should_acquire_lock_after_child_releases` - 1.010s
- `torrust-tracker-deployer::file_lock_multiprocess::it_should_prevent_lock_acquisition_across_processes` - 2.004s
- `torrust-tracker-deployer::file_lock_multiprocess::it_should_allow_sequential_acquisition_by_different_processes` - 3.004s
- `torrust-tracker-deployer::file_lock_multiprocess::it_should_handle_parent_acquiring_while_child_holds_lock` - 3.005s
- `torrust-tracker-deployer::file_lock_multiprocess::it_should_handle_rapid_lock_handoff_between_processes` - 5.034s
- `torrust-tracker-deployer::file_lock_multiprocess::it_should_handle_multiple_processes_competing_for_lock` - 6.054s

**Docker-based integration tests (3-5 seconds)**:

- `torrust-dependency-installer::check_command_docker_integration::it_should_list_all_dependencies_with_their_installation_status` - 3.295s
- `torrust-dependency-installer::check_command_docker_integration::it_should_display_debug_logs_when_verbose_flag_is_enabled` - 3.423s
- `torrust-dependency-installer::install_command_docker_integration::it_should_return_error_exit_code_when_installation_fails` - 3.480s
- `torrust-dependency-installer::check_command_docker_integration::it_should_report_missing_dependencies_when_checking_all_in_fresh_ubuntu_container` - 3.535s
- `torrust-dependency-installer::check_command_docker_integration::it_should_exit_with_error_code_when_checking_missing_specific_dependency` - 3.568s

**AI training example validation tests (4 seconds)**:

- `torrust-tracker-deployer::validate_ai_training_examples::it_should_validate_all_ai_training_example_configurations` - 4.138s
- `torrust-tracker-deployer::validate_ai_training_examples::it_should_render_all_ai_training_example_configurations` - 4.310s

**SSH connectivity timeout tests (5 seconds)**:

- `torrust-tracker-deployer::ssh_client_integration::ssh_client::connectivity_tests::it_should_handle_connectivity_timeouts` - 5.010s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::connectivity_tests::it_should_timeout_when_connecting_to_unreachable_host_with_real_ssh_infrastructure` - 5.013s

**Package installation tests (18-19 seconds)**:

- `torrust-dependency-installer::install_command_docker_integration::it_should_install_opentofu_successfully` - 18.319s
- `torrust-dependency-installer::install_command_docker_integration::it_should_handle_idempotent_installation_of_cargo_machete` - 18.664s
- `torrust-dependency-installer::install_command_docker_integration::it_should_install_cargo_machete_successfully` - 19.264s

**SSH server integration tests (10-11 seconds)**:

- `torrust-tracker-deployer::testing::integration::ssh_server::tests::it_should_start_real_ssh_server_container` - 10.914s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::connectivity_tests::it_should_connect_to_real_ssh_server_and_test_connectivity` - 10.870s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::command_execution_tests::it_should_execute_remote_command_on_real_ssh_server` - 10.968s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::command_execution_tests::it_should_allow_users_to_override_default_options` - 11.007s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::command_execution_tests::it_should_allow_users_to_override_strict_host_key_checking` - 11.038s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::command_execution_tests::it_should_allow_users_to_add_non_conflicting_ssh_options` - 10.863s

**E2E workflow tests (3-22 seconds)**:

- `torrust-tracker-deployer::e2e_integration::e2e::validate_command::it_should_validate_configuration_without_creating_deployment` - 3.443s
- `torrust-tracker-deployer::e2e_integration::e2e::validate_command::it_should_succeed_when_configuration_file_is_valid` - 3.831s
- `torrust-tracker-deployer::e2e_integration::e2e::validate_command::it_should_report_invalid_json_when_configuration_file_has_malformed_json` - 9.889s
- `torrust-tracker-deployer::e2e_integration::e2e::show_command::it_should_report_environment_not_found_when_environment_does_not_exist` - 17.346s
- `torrust-tracker-deployer::e2e_integration::e2e::validate_command::it_should_report_file_not_found_when_configuration_file_does_not_exist` - 17.309s
- `torrust-tracker-deployer::e2e_integration::e2e::show_command::it_should_show_created_environment_details` - 18.278s
- `torrust-tracker-deployer::e2e_integration::e2e::show_command::it_should_show_environment_state` - 18.307s
- `torrust-tracker-deployer::e2e_integration::e2e::show_command::it_should_show_provider_information` - 18.312s
- `torrust-tracker-deployer::e2e_integration::e2e::create_command::it_should_fail_when_config_file_not_found` - 19.067s
- `torrust-tracker-deployer::e2e_integration::e2e::list_command::it_should_report_no_data_directory_when_workspace_is_empty` - 19.096s
- `torrust-tracker-deployer::e2e_integration::e2e::create_command::it_should_fail_gracefully_with_invalid_config` - 19.128s
- `torrust-tracker-deployer::e2e_integration::e2e::destroy_command::it_should_fail_when_environment_not_found_in_working_directory` - 19.160s
- `torrust-tracker-deployer::e2e_integration::e2e::create_command::it_should_create_environment_from_config_file_black_box` - 19.191s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::it_should_fail_when_environment_not_found` - 19.443s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::it_should_fail_when_config_file_not_found` - 19.507s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::it_should_render_artifacts_using_config_file_successfully` - 19.604s
- `torrust-tracker-deployer::e2e_integration::e2e::purge_command::it_should_fail_when_purging_nonexistent_environment` - 19.613s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::it_should_render_artifacts_using_env_name_successfully` - 20.099s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::it_should_complete_full_lifecycle_from_create_to_render` - 20.158s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::it_should_work_with_custom_working_directory` - 20.176s
- `torrust-tracker-deployer::e2e_integration::e2e::destroy_command::it_should_destroy_environment_with_custom_working_directory` - 20.248s
- `torrust-tracker-deployer::e2e_integration::e2e::destroy_command::it_should_destroy_environment_with_default_working_directory` - 20.292s
- `torrust-tracker-deployer::e2e_integration::e2e::destroy_command::it_should_complete_full_lifecycle_with_custom_working_directory` - 20.328s
- `torrust-tracker-deployer::e2e_integration::e2e::list_command::it_should_list_created_environment` - 20.357s
- `torrust-tracker-deployer::e2e_integration::e2e::create_command::it_should_fail_when_environment_already_exists` - 20.527s
- `torrust-tracker-deployer::e2e_integration::e2e::list_command::it_should_list_multiple_environments` - 20.994s
- `torrust-tracker-deployer::e2e_integration::e2e::purge_command::it_should_purge_with_custom_working_directory` - 21.059s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::it_should_fail_when_output_directory_already_exists` - 21.088s
- `torrust-tracker-deployer::e2e_integration::e2e::purge_command::it_should_complete_full_lifecycle_from_create_to_purge` - 21.119s
- `torrust-tracker-deployer::e2e_integration::e2e::purge_command::it_should_purge_destroyed_environment_successfully` - 21.152s
- `torrust-tracker-deployer::e2e_integration::e2e::purge_command::it_should_remove_only_specified_environment_data` - 21.553s

**Root cause of slow E2E tests**: Every E2E test calls `ProcessRunner`, which spawns `cargo run -- <command>
...` for each sub-command. The `cargo run` overhead alone is ~13-14s even with a warm build cache
(Cargo linker/check phase). The actual binary (`./target/debug/torrust-tracker-deployer`) takes ~13ms to
execute. Each E2E test that calls `ProcessRunner` once pays ~19-21s total; tests calling it twice
(e.g. `it_should_fail_when_environment_already_exists`) pay ~21s due to parallel execution.

**Notes**:

- File lock tests use real multiprocess coordination with `sleep` delays — inherently slow
- Docker-based tests have overhead of container startup/teardown
- Package installation tests are slow due to actual package compilation inside containers
- SSH tests need real container initialization (~10s Docker overhead)
- E2E tests dominated by `cargo run` startup overhead (~13-14s per invocation), not application logic
- AI training validation tests render all environment configurations with template engine

## Implementation Plan

### Phase 1: Workflow Modifications (15 minutes)

- [ ] Modify `.github/workflows/testing.yml` `unit` job:
  - [ ] Add `tools` step to install cargo-nextest
  - [ ] Add `fetch` step before build
  - [ ] Add explicit `build` step for test binaries
  - [ ] Replace test command with `cargo nextest run`

### Phase 2: Documentation (10 minutes)

- [ ] Create this issue specification with test timing analysis
- [ ] Document slow tests (>1s) from local cargo nextest run
- [ ] Get user approval on specification

### Phase 3: Issue Creation (5 minutes)

- [ ] Create GitHub issue with specification
- [ ] Update specification document with issue number
- [ ] Rename file to include issue number

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Workflow Changes**:

- [ ] `cargo fetch` step added to `unit` job
- [ ] `cargo nextest` installed via `taiki-e/install-action@v2`
- [ ] Explicit test binary build step added before test execution
- [ ] `cargo nextest run --all-targets` replaces `cargo test` (maintains test coverage parity)

**Functional Requirements**:

- [ ] All existing tests continue to pass with cargo nextest
- [ ] Test isolation verification step still functions correctly
- [ ] Workflow completes successfully on GitHub Actions
- [ ] Job timing is now separated: fetch → build → test-docs → test

**Documentation**:

- [ ] Tests taking >1s documented in issue specification
- [ ] Issue specification reviewed and approved by user

## Related Documentation

- [GitHub Actions: Testing Workflow](../../.github/workflows/testing.yml)
- [cargo-nextest Documentation](https://nexte.st/)
- [Contributing: Pre-commit Process](../contributing/commit-process.md)

## Notes

**Why cargo nextest?**

cargo-nextest provides:

- Per-test timing information (helps identify slow tests)
- Better test failure output
- Parallel test execution with better scheduling
- Industry-standard tool used by many Rust projects

**Timing Breakdown Goals**:

After these changes, the GitHub Actions job logs will clearly show:

1. How long `cargo fetch` takes (dependency download)
2. How long `cargo build` takes (compilation)
3. How long `cargo test --doc` takes (doc tests)
4. How long `cargo nextest run` takes (unit tests)
5. Individual test timings from nextest output

**Future Optimization Opportunities**:

Based on the timing data collected:

- Package installation tests (18-21s) could be moved to separate job or marked as integration tests
- SSH/Docker tests (10-11s) might benefit from container image caching
- E2E tests (18-22s) could potentially use test fixtures instead of full provisioning
