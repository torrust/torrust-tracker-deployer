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

Based on local execution with `cargo nextest run --workspace --all-features`, the following tests exceed 1 second:

**Docker-based integration tests (3-5 seconds)**:

- `torrust-dependency-installer::check_command_docker_integration::it_should_exit_with_error_code_when_checking_missing_specific_dependency` - 3.382s
- `torrust-dependency-installer::install_command_docker_integration::it_should_return_error_exit_code_when_installation_fails` - 3.379s
- `torrust-dependency-installer::check_command_docker_integration::it_should_report_missing_dependencies_when_checking_all_in_fresh_ubuntu_container` - 3.483s
- `torrust-dependency-installer::check_command_docker_integration::it_should_list_all_dependencies_with_their_installation_status` - 3.520s
- `torrust-dependency-installer::check_command_docker_integration::it_should_display_debug_logs_when_verbose_flag_is_enabled` - 3.555s

**Package installation tests (18-21 seconds)**:

- `torrust-dependency-installer::install_command_docker_integration::it_should_install_opentofu_successfully` - 18.204s
- `torrust-dependency-installer::install_command_docker_integration::it_should_install_cargo_machete_successfully` - 20.005s
- `torrust-dependency-installer::install_command_docker_integration::it_should_handle_idempotent_installation_of_cargo_machete` - 20.845s

**SSH server integration tests (10-11 seconds)**:

- `torrust-tracker-deployer::testing::integration::ssh_server::tests::it_should_start_real_ssh_server_container` - 11.138s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::connectivity_tests::it_should_connect_to_real_ssh_server_and_test_connectivity` - 10.723s
- `torrust-tracker-deployer::ssh_client_integration::ssh_client::command_execution_tests::*` - 10.616s to 10.911s

**E2E workflow tests (18-22 seconds)**:

- `torrust-tracker-deployer::e2e_integration::e2e::create_command::*` - 19.623s to 20.076s
- `torrust-tracker-deployer::e2e_integration::e2e::destroy_command::*` - 20.369s to 20.933s
- `torrust-tracker-deployer::e2e_integration::e2e::purge_command::*` - 19.770s to 22.278s
- `torrust-tracker-deployer::e2e_integration::e2e::render_command::*` - 20.076s to 21.746s
- `torrust-tracker-deployer::e2e_integration::e2e::show_command::*` - 18.013s to 18.911s
- `torrust-tracker-deployer::e2e_integration::e2e::validate_command::*` - 3.444s to 9.544s

**Notes**:

- Docker-based tests have overhead of container startup/teardown
- Package installation tests are slow due to actual package compilation
- SSH tests need real container initialization
- E2E tests create temporary workspaces and execute full command workflows

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
