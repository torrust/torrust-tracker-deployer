# Refactor and Improve E2E Test Execution

**Epic Issue**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112)
**Parent**: #1 (Project Roadmap)

## Sub-Issues

This epic is broken down into sub-issues:

```text
1-1. #113 - Create Dependency Installation Package for E2E Tests (10-14 hours)
     ├── 1-1-1. #114 - Create Detection Logic Package (2-3 hours)
     ├── 1-1-2. #115 - Create CLI Binary with Check Command (2-3 hours)
     ├── 1-1-3. #116 - Create Docker Test Infrastructure (2-3 hours)
     └── 1-1-4. #117 - Implement Installation Logic (4-5 hours)
1-2. #118 - Integrate Dependency Installer with E2E Tests (3-5 hours)
1-3. #119 - Update CI Workflows and Remove Bash Scripts (2-4 hours)
1-4. #120 - Configure GitHub Copilot Agent Environment (2-3 hours)
1-5. #121 - Install Git Pre-Commit Hooks for Copilot Agent (2-3 hours)
```

**Total Estimated Time**: 21-32 hours split across multiple focused sub-issues

## Overview

This epic focuses on standardizing, documenting, and improving the End-to-End (E2E) test execution infrastructure to make it more consistent, maintainable, and easier for AI agents to execute. The work involves refactoring the current mixed approach to E2E testing into a unified, black-box testing strategy that uses the application's public CLI interface.

## Goals

- [ ] Standardize all E2E tests to use the same approach (black-box testing via CLI)
- [ ] Document different types of E2E tests and their purposes
- [ ] Make E2E tests easier for AI agents to understand and execute
- [ ] Improve test maintainability by reducing direct coupling to internal layers
- [ ] Provide clear guidelines for future E2E test development

## Current Situation

### Terminology Clarification: Rust Tests vs E2E Tests

**Rust Test Types** (by implementation):

Rust has two native test types distinguished by how they're executed:

- **Unit tests**: Live alongside code, can test private items, compiled into the main crate
- **Integration tests**: Separate binaries in `tests/`, can only exercise the public API of the crate

**Our E2E Test Classification** (by testing level):

We use "E2E" (End-to-End) as our terminology because the **main goal is to test the deployer at the end-user level**, regardless of Rust's implementation mechanism. However, not all current "E2E tests" achieve true end-user testing yet:

- **True E2E tests**: Use the public CLI interface as an end-user would (black-box testing)
- **Temporary integration tests**: Called "E2E" but currently test at the application layer level because presentation layer commands don't exist yet for all features

These "temporary integration tests" will become true E2E tests once the presentation layer commands are implemented.

### Existing E2E Test Files

We currently have **6 files** for E2E testing with different Rust implementations:

**Rust Integration Tests** (newer, true E2E approach):

- `tests/e2e_create_command.rs` - Tests via public CLI
- `tests/e2e_destroy_command.rs` - Tests via public CLI

**Explicit Binaries** (older, temporary integration test approach):

- `src/bin/e2e_config_tests.rs` - Calls application layer directly
- `src/bin/e2e_provision_and_destroy_tests.rs` - Calls application layer directly
- `src/bin/e2e_tests_full.rs` - Calls application layer directly

**Why explicit binaries?** We use explicit binaries (not Rust integration tests) because we need full control over the binary, its arguments, and execution flow. This is similar to integration tests in that they test the crate from outside, but with custom entry points.

### Why Two Different Approaches Exist

#### Rust Integration Tests (True E2E Tests)

The two Rust integration test files (`e2e_create_command.rs`, `e2e_destroy_command.rs`) represent the **newer, preferred approach** - true E2E testing:

- **True black-box testing**: Create a temporary directory and run the tracker deployer as an end-user would
- **Use public CLI interface**: Tests interact with the application through its console commands (presentation layer)
- **Implementation context**: These tests were implemented later after the DDD presentation layer commands were available
- **Development strategy alignment**: Follow the inside-outside strategy where user-facing presentation layer commands came after core functionality
- **Infrastructure independence**: These specific commands don't handle infrastructure (VMs or Docker), making true E2E testing feasible with Rust integration tests

#### Explicit Binaries (Temporary Integration Tests)

The three explicit binary tests represent the **older approach** and are temporarily integration tests rather than true E2E tests:

- **Not black-box**: They don't use the console app as a user would
- **Direct layer access**: They simulate presentation logic and call the DDD application layer command handlers directly
- **Historical context**: These were the first E2E tests, created before presentation logic for commands existed
- **Explicit binary control**: Use custom binaries (not Rust integration tests) to control arguments, execution flow, and binary behavior
- **Goal**: These are **intended to be E2E tests** (testing at end-user level) but are **temporarily implemented as integration tests** because presentation layer commands don't exist yet for all features
- **Future migration needed**: These tests should eventually use the public CLI at a higher level once presentation commands are available

#### Infrastructure Provider Split

Within the explicit binary tests, there are **two subcategories** based on infrastructure:

1. **Virtual Machine-based tests**: Simulate 100% production environment
2. **Docker container-based tests**: Used for provisioning tests due to networking problems with VMs in GitHub shared runners

### Challenges Preventing Full Standardization

We cannot immediately standardize all E2E tests to the black-box approach because:

1. **Missing presentation layer commands**:

   - No public CLI commands for provisioning
   - No public CLI commands for configuration

2. **Infrastructure setup limitations**:
   - Some tests require Docker containers for GitHub Actions compatibility
   - No provider implementation for "externally-provisioned instances"
   - Current tests manually set up infrastructure state

### Future Vision: Externally-Provisioned Instance Provider

A **nice-to-have feature** (not yet in roadmap) would be adding a provider for externally-provisioned instances. This would:

- Allow skipping the "provisioning" step in E2E tests
- Enable users to deploy the tracker to instances they provisioned previously
- Support scenarios where:
  - No provider exists for their infrastructure
  - Users prefer not sharing tokens with the deployer
  - Reusing existing machines for the tracker
  - Custom security/compliance requirements

This feature would greatly simplify E2E testing and expand deployment flexibility.

## Target State

After completing this epic:

- **Unified approach**: All E2E tests use black-box testing via public CLI
- **Clear documentation**: Different test types are well-documented with their purposes
- **AI-friendly**: Tests are structured to be easily understood and executed by AI agents
- **Maintainable**: Tests are decoupled from internal implementation details
- **Comprehensive coverage**: Tests cover all major workflows using the public interface

## Tasks

### Phase 1: Dependency Management Infrastructure

- [ ] #TBD - Create dependency installation package for E2E tests (Rust package to detect/install tools like OpenTofu, Ansible, LXD, cargo-machete)
- [ ] #TBD - Integrate dependency installer with E2E tests (Add dependency verification to E2E test binaries)
- [ ] #TBD - Update CI workflows and remove bash scripts (Migrate GitHub Actions to use new dependency installer)

### Phase 2: Test Standardization (Future)

(Additional tasks will be created as Phase 1 completes and we have better visibility into standardization needs)

## Related

- **Parent**: #1 (Project Roadmap)
- **Documentation**:
  - [docs/e2e-testing.md](../e2e-testing.md) - Current E2E testing documentation
  - [docs/codebase-architecture.md](../codebase-architecture.md) - DDD architecture guide
  - [docs/contributing/testing/](../contributing/testing/) - Testing conventions
- **Test Files**:
  - Integration tests: `tests/e2e_create_command.rs`, `tests/e2e_destroy_command.rs`
  - Binary tests: `src/bin/e2e_config_tests.rs`, `src/bin/e2e_provision_and_destroy_tests.rs`, `src/bin/e2e_tests_full.rs`

## Notes

- This epic addresses technical debt accumulated during the inside-outside development strategy
- Standardization will improve test reliability and reduce maintenance burden
- Clear documentation will help onboard contributors and AI agents
- Some tasks may depend on implementing missing presentation layer commands
- Consider the externally-provisioned instance provider as a separate future epic
