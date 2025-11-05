# Update Pre-Commit Script for GitHub Runner-Compatible E2E Tests

**Issue**: [#146](https://github.com/torrust/torrust-tracker-deployer/issues/146)
**Parent Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution
**Related**:

- [#121](https://github.com/torrust/torrust-tracker-deployer/issues/121) - Install Git Pre-Commit Hooks for Copilot Agent
- [#120](https://github.com/torrust/torrust-tracker-deployer/issues/120) - Configure GitHub Copilot Agent Environment
- [docs/e2e-testing.md](../e2e-testing.md) - E2E Testing Documentation
- [GitHub Docs: Preinstalling tools in Copilot's environment](https://docs.github.com/en/enterprise-cloud@latest/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment#preinstalling-tools-or-dependencies-in-copilots-environment)

## Overview

Update the pre-commit verification script (`scripts/pre-commit.sh`) to run GitHub runner-compatible E2E tests instead of the full E2E test suite. This enables GitHub Copilot agents to successfully execute pre-commit checks in GitHub Actions environments where LXD VM network connectivity is limited.

## Problem Statement

### Current Situation

The pre-commit script currently runs the E2E full test suite:

```bash
"Running comprehensive E2E tests|All E2E tests passed|(Filtering logs to WARNING level and above - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-tests-full"
```

**Issues**:

- `e2e-tests-full` requires LXD VMs with full network connectivity
- GitHub Actions runners experience network connectivity issues inside LXD VMs
- Cannot download Docker GPG keys, package repositories timeout
- GitHub Copilot agents run in GitHub Actions infrastructure and **cannot execute** this test

### Why E2E Full Tests Can't Run on GitHub Runners

**Root Cause**: Known GitHub Actions networking limitations with nested virtualization:

- [GitHub Issue 13003](https://github.com/actions/runner-images/issues/13003) - Network connectivity issues with LXD VMs
- [GitHub Issue 1187](https://github.com/actions/runner-images/issues/1187) - Original networking issue
- [GitHub Issue 2890](https://github.com/actions/runner-images/issues/2890) - Specific apt repository timeout issues

### Why We Keep E2E Full Tests

The E2E full test suite (`src/bin/e2e_tests_full.rs`) is valuable for **human developers** with local environments where:

- All dependencies are installed (LXD, OpenTofu, Ansible)
- LXD VM networking works correctly
- Can validate the complete deployment pipeline in a single run

## Goals

- [ ] Replace `e2e-tests-full` with GitHub runner-compatible E2E tests in pre-commit script
- [ ] Maintain comprehensive E2E coverage through split test execution
- [ ] Enable GitHub Copilot agents to run pre-commit checks successfully
- [ ] Keep clear documentation about test suite purposes and limitations

## 🏗️ Architecture Requirements

**Script Location**: `scripts/pre-commit.sh`
**Pattern**: Shell script configuration with step definitions
**Integration**: Used by Git pre-commit hooks (see [#121](https://github.com/torrust/torrust-tracker-deployer/issues/121))

### Testing Strategy

**Current Approach**: Single E2E full test (local-only)
**New Approach**: Split E2E test execution (GitHub runner-compatible)

**Test Suite Compatibility**:

- ✅ `e2e-provision-and-destroy-tests` - Uses LXD VMs but no nested networking
- ✅ `e2e-config-tests` - Uses Docker containers with reliable networking
- ❌ `e2e-tests-full` - Requires LXD VM networking (local development only)

## Specifications

### Test Execution Split

The pre-commit script should run **two separate E2E test commands** instead of one:

**Before** (current):

```bash
"Running comprehensive E2E tests|All E2E tests passed|(Filtering logs to WARNING level and above - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-tests-full"
```

**After** (proposed):

```bash
"Running E2E provision and destroy tests|Provision and destroy tests passed|(Testing infrastructure lifecycle - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-provision-and-destroy-tests"
"Running E2E configuration tests|Configuration tests passed|(Testing software installation and configuration)|RUST_LOG=warn|cargo run --bin e2e-config-tests"
```

### Step Configuration Format

Each step follows the format:

```text
"description|success_message|special_note|env_vars|command"
```

**Key Points**:

- Two separate steps for better granularity
- Clear descriptions matching test purposes
- Appropriate timing notes for each test type
- Same log level filtering (`RUST_LOG=warn`)

### Coverage Comparison

**E2E Full Tests Coverage**:

- Infrastructure provisioning (LXD VMs)
- Cloud-init completion
- Docker installation
- Docker Compose installation
- Infrastructure destruction

**Split Tests Coverage** (combined):

- ✅ Infrastructure provisioning (LXD VMs) - `e2e-provision-and-destroy-tests`
- ✅ Cloud-init completion - `e2e-provision-and-destroy-tests`
- ✅ Docker installation - `e2e-config-tests`
- ✅ Docker Compose installation - `e2e-config-tests`
- ✅ Infrastructure destruction - `e2e-provision-and-destroy-tests`

**Result**: Same coverage, GitHub runner-compatible execution

## Implementation Plan

### Phase 1: Update Pre-Commit Script (30 minutes)

- [ ] Task 1.1: Replace the E2E full test step with provision and destroy test step
- [ ] Task 1.2: Add the configuration test step after provision and destroy test step
- [ ] Task 1.3: Verify step numbering and total step count are correct
- [ ] Task 1.4: Test the script locally to ensure both tests run successfully

### Phase 2: Documentation Updates (15 minutes)

- [ ] Task 2.1: Update `docs/contributing/commit-process.md` to document the split E2E test execution
- [ ] Task 2.2: Update `.github/copilot-instructions.md` if needed
- [ ] Task 2.3: Ensure documentation clarifies when to use E2E full tests vs split tests

### Phase 3: Validation (15 minutes)

- [ ] Task 3.1: Run `./scripts/pre-commit.sh` locally to verify both E2E tests execute
- [ ] Task 3.2: Verify pre-commit script passes when both E2E tests succeed
- [ ] Task 3.3: Verify pre-commit script fails when either E2E test fails
- [ ] Task 3.4: Run shellcheck on the modified script

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Shellcheck passes for `scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] Pre-commit script runs provision and destroy E2E tests
- [ ] Pre-commit script runs configuration E2E tests
- [ ] Both E2E test steps execute in sequence
- [ ] Script succeeds when both tests pass
- [ ] Script fails appropriately when either test fails
- [ ] Step numbering and count are correct
- [ ] Timing notes accurately describe each test's duration expectations
- [ ] Documentation reflects the split E2E test approach

**Testing Criteria**:

- [ ] Verify locally: `./scripts/pre-commit.sh` completes successfully
- [ ] Verify provision test step runs: Check for "Running E2E provision and destroy tests" message
- [ ] Verify config test step runs: Check for "Running E2E configuration tests" message
- [ ] Both test steps show success messages with timing

## Related Documentation

- [docs/e2e-testing.md](../e2e-testing.md) - E2E Testing Guide (documents test suite split strategy)
- [docs/contributing/commit-process.md](../contributing/commit-process.md) - Commit Process (documents pre-commit requirements)
- [.github/workflows/test-e2e-provision.yml](../../.github/workflows/test-e2e-provision.yml) - CI workflow for provision tests
- [.github/workflows/test-e2e-config.yml](../../.github/workflows/test-e2e-config.yml) - CI workflow for config tests
- [#120](https://github.com/torrust/torrust-tracker-deployer/issues/120) - Configure GitHub Copilot Agent Environment
- [#121](https://github.com/torrust/torrust-tracker-deployer/issues/121) - Git pre-commit hook installation (depends on this issue)
- [GitHub Actions Runner Images: Ubuntu 24.04](https://github.com/actions/runner-images/blob/ubuntu24/20251030.96/images/ubuntu/Ubuntu2404-Readme.md) - Pre-installed software on runners
- [GitHub Docs: Preinstalling tools in Copilot's environment](https://docs.github.com/en/enterprise-cloud@latest/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment#preinstalling-tools-or-dependencies-in-copilots-environment) - Official documentation on customizing Copilot agent environment

## Implementation Details

### Script Modification

The `STEPS` array in `scripts/pre-commit.sh` currently contains 6 steps. After the modification, it will still contain 6 steps but with step 5 split into two separate steps:

**Current Steps**:

1. Checking for unused dependencies (cargo machete)
2. Running linters
3. Running tests
4. Testing cargo documentation
5. Running comprehensive E2E tests ← **This will be replaced**
6. Running code coverage check

**New Steps**:

1. Checking for unused dependencies (cargo machete)
2. Running linters
3. Running tests
4. Testing cargo documentation
5. Running E2E provision and destroy tests ← **New**
6. Running E2E configuration tests ← **New**
7. Running code coverage check

**Total Steps**: 7 (was 6)

### Code Changes

Replace this line in the `STEPS` array:

```bash
"Running comprehensive E2E tests|All E2E tests passed|(Filtering logs to WARNING level and above - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-tests-full"
```

With these two lines:

```bash
"Running E2E provision and destroy tests|Provision and destroy tests passed|(Testing infrastructure lifecycle - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-provision-and-destroy-tests"
"Running E2E configuration tests|Configuration tests passed|(Testing software installation and configuration)|RUST_LOG=warn|cargo run --bin e2e-config-tests"
```

### Human Developer Workflow

**For Local Development** (human developers):

- Can still run `cargo run --bin e2e-tests-full` manually for comprehensive single-run testing
- Pre-commit hook will run split tests (same coverage, GitHub compatible)
- Choice between convenience (manual full test) vs compatibility (automated split tests)

**For GitHub Copilot Agents**:

- Pre-commit hook always runs split tests
- Both tests execute successfully in GitHub Actions environment
- Comprehensive coverage without networking limitations

## Notes

### GitHub Actions Runner Pre-installed Dependencies

The GitHub Actions Ubuntu 24.04 runners come with many dependencies **already installed**:

- ✅ **Ansible 2.19.3** - Pre-installed ([runner image docs](https://github.com/actions/runner-images/blob/ubuntu24/20251030.96/images/ubuntu/Ubuntu2404-Readme.md))
- ✅ **Rust 1.90.0 + Cargo 1.90.0** - Pre-installed
- ✅ **LXD** - Pre-installed (verified working in CI)
- ✅ **Docker** - Pre-installed (Docker 28.0.4, Docker Compose v2 2.38.2)
- ✅ **cargo-machete** - Installed via dependency-installer package (issue #113-#117)
- ✅ **OpenTofu** - Installed via dependency-installer package (issue #113-#117)

This means the split E2E tests (`e2e-provision-and-destroy-tests` and `e2e-config-tests`) have **all required dependencies available** in the GitHub Actions environment.

### Rationale for This Change

**Problem**: GitHub Copilot agents need to run pre-commit checks but cannot execute `e2e-tests-full` due to GitHub Actions networking limitations.

**Solution**: Use split E2E tests that provide the same coverage but are compatible with GitHub Actions infrastructure.

**Benefits**:

- ✅ Enables Copilot agents to run pre-commit checks
- ✅ Maintains comprehensive E2E coverage
- ✅ Aligns pre-commit script with CI workflows
- ✅ Human developers can still use `e2e-tests-full` manually for convenience

### Why Not Remove E2E Full Tests?

The E2E full test binary (`src/bin/e2e_tests_full.rs`) provides value for human developers:

- **Convenience**: Single command to validate complete deployment pipeline
- **Speed**: Faster than running two separate commands manually
- **Local Development**: Works perfectly in local environments with proper networking
- **Debugging**: Easier to debug full flow in one execution

It's kept as a **local development convenience tool**, not removed from the codebase.

### Alternative Considered: Conditional Execution

**Alternative**: Detect environment (local vs GitHub) and conditionally run different tests.

**Rejected Because**:

- ❌ Adds complexity to the script
- ❌ Different behavior in different environments (confusing)
- ❌ Split tests provide same coverage regardless of environment
- ❌ Harder to maintain and reason about

**Better Approach**: Always run split tests in pre-commit hook, let developers manually run full tests when desired.

## Time Estimate

**Total Estimated Time**: 1-1.5 hours

- Script modification: 30 minutes
- Documentation updates: 15 minutes
- Testing and validation: 15 minutes
- Buffer for edge cases: 15 minutes
