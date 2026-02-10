# Add E2E Test Workflow for Dependency Installer

**Issue**: #330
**Parent Epic**: N/A (Standalone Feature)
**Related**:

- `packages/dependency-installer/` - The package being tested
- `.github/workflows/copilot-setup-steps.yml` - Current workflow that uses the installer
- `.github/workflows/test-e2e-deployment.yml` - Uses installer indirectly
- `.github/workflows/test-e2e-infrastructure.yml` - Uses installer indirectly
- `.github/workflows/test-lxd-provision.yml` - Uses installer indirectly

## Overview

Create a dedicated GitHub Actions workflow to test the `dependency-installer` package in isolation. Currently, the installer is only tested indirectly through other workflows (E2E deployment, E2E infrastructure, LXD provision), which means installer failures are discovered late and are harder to diagnose. This new workflow will provide fast, clear feedback when the installer itself has issues.

## Goals

- [ ] Create dedicated workflow that tests dependency-installer in isolation
- [ ] Test all three installer commands: `check`, `install`, and `list`
- [ ] Verify installer works correctly before other workflows depend on it
- [ ] Provide clear, actionable feedback when installer failures occur

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (CI/CD configuration)
**File Location**: `.github/workflows/test-dependency-installer.yml`
**Pattern**: GitHub Actions Workflow

### Module Structure Requirements

- [ ] Follow existing workflow conventions from the project
- [ ] Reuse steps from `copilot-setup-steps.yml` where appropriate
- [ ] Maintain consistency with existing E2E test workflows

### Architectural Constraints

- [ ] Must run on `ubuntu-latest` runner (same as other workflows)
- [ ] Must be triggered on push, pull request, and manual dispatch
- [ ] Should have an appropriate timeout (10-15 minutes recommended)
- [ ] Must follow the same Rust toolchain setup as other workflows

### Anti-Patterns to Avoid

- ❌ Don't duplicate complex logic from other workflows without extraction
- ❌ Don't make the workflow dependent on other workflows
- ❌ Don't add unnecessary steps that aren't installer-specific

## Specifications

### Workflow Structure

The workflow should:

1. **Build the installer binary** (similar to other workflows):

   ```bash
   cargo build --release -p torrust-dependency-installer --bin dependency-installer
   ```

2. **Run the installer** with non-interactive mode:

   ```bash
   target/release/dependency-installer install
   ```

3. **Verify installation** using the check command:

   ```bash
   target/release/dependency-installer check
   ```

4. **List all dependencies** to verify the list command works:

   ```bash
   target/release/dependency-installer list
   ```

### Workflow Triggers

```yaml
on:
  push:
    paths:
      - .github/workflows/test-dependency-installer.yml
      - packages/dependency-installer/**
      - Cargo.toml
      - Cargo.lock
  pull_request:
    paths:
      - .github/workflows/test-dependency-installer.yml
      - packages/dependency-installer/**
      - Cargo.toml
      - Cargo.lock
  workflow_dispatch: # Allow manual triggering
```

### Success Criteria Per Step

Each step should verify its specific aspect:

- **Build step**: Binary compiles successfully
- **Install step**: All dependencies install successfully (exit code 0)
- **Check step**: All dependencies are detected as installed (exit code 0)
- **List step**: All dependencies listed with correct status (exit code 0)

### Output Validation

The workflow should verify:

- Exit codes are 0 for success
- No error-level log messages appear in structured logging output
- All expected dependencies are reported as installed

### Error Handling

The workflow should:

- Fail fast on any step failure
- Provide clear error messages identifying which command failed
- Include relevant log output in failure artifacts

## Implementation Plan

### Phase 1: Create Basic Workflow (30-45 minutes)

- [ ] Create `.github/workflows/test-dependency-installer.yml`
- [ ] Add workflow metadata (name, description comments)
- [ ] Configure triggers (push, PR, workflow_dispatch) with path filters
- [ ] Set up job with timeout (10-15 minutes)
- [ ] Add required permissions (contents: read)

### Phase 2: Implement Core Steps (30 minutes)

- [ ] Add checkout step (`actions/checkout@v4`)
- [ ] Add Rust toolchain setup (`dtolnay/rust-toolchain@stable`)
- [ ] Add Rust cache setup (`Swatinem/rust-cache@v2`)
- [ ] Add build step for dependency-installer binary
- [ ] Add install step with `DEBIAN_FRONTEND=noninteractive`

### Phase 3: Add Verification Steps (20 minutes)

- [ ] Add check step to verify all dependencies installed correctly
- [ ] Add list step to verify all dependencies are reported
- [ ] Add specific tool version checks (opentofu, ansible, etc.)
- [ ] Add step to display structured output for debugging

### Phase 4: Testing and Documentation (20 minutes)

- [ ] Test workflow locally (if possible) or via PR
- [ ] Verify workflow runs successfully on GitHub Actions
- [ ] Add inline comments explaining critical steps
- [ ] Update this specification with any lessons learned
- [ ] Verify workflow badge appears correctly in repo

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Workflow file passes YAML linting (yamllint)

**Functional Requirements**:

- [ ] Workflow file created at `.github/workflows/test-dependency-installer.yml`
- [ ] Workflow builds the dependency-installer binary successfully
- [ ] Workflow runs `install` command and exits with code 0
- [ ] Workflow runs `check` command and exits with code 0
- [ ] Workflow runs `list` command and exits with code 0
- [ ] Workflow is triggered on:
  - Push events (when relevant paths change)
  - Pull request events (when relevant paths change)
  - Manual workflow dispatch
- [ ] Workflow has appropriate timeout (10-15 minutes)
- [ ] Workflow uses same Rust setup as other workflows

**Testing**:

- [ ] Workflow runs successfully on GitHub Actions
- [ ] Workflow correctly detects installer failures (manual negative testing)
- [ ] Workflow provides clear error messages when steps fail

**Documentation**:

- [ ] Workflow includes descriptive comments explaining each major step
- [ ] Workflow includes reference to this specification in comments
- [ ] Workflow appears in GitHub Actions tab with correct name

## Related Documentation

- [Dependency Installer README](../../packages/dependency-installer/README.md) - Package documentation
- [Copilot Setup Steps Workflow](../../.github/workflows/copilot-setup-steps.yml) - Similar workflow using installer
- [E2E Deployment Workflow](../../.github/workflows/test-e2e-deployment.yml) - Workflow using installer
- [Development Principles](../../docs/development-principles.md) - Testability principle
- [GitHub Actions Best Practices](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/workflow-commands-for-github-actions)

## Notes

### Why This Matters

Currently, when the dependency-installer fails, we only discover it when:

1. E2E deployment tests fail
2. E2E infrastructure tests fail
3. LXD provision tests fail
4. Copilot agent setup fails

This leads to:

- **Slower feedback**: Failures discovered 20-30 minutes into complex workflows
- **Ambiguous failures**: Is it the installer or the actual test logic?
- **Wasted CI resources**: Running unnecessary setup before installer failure

With a dedicated workflow:

- **Fast feedback**: Installer issues caught in ~5 minutes
- **Clear failures**: Immediately know if installer is the problem
- **Efficient CI**: Other workflows can assume installer works

### Estimated Total Time

Approximately 1.5-2 hours including testing and documentation.

### Future Enhancements

Potential future improvements (not in scope for this issue):

- Test installer on different Ubuntu versions
- Test individual dependency installers in isolation
- Add performance benchmarks for install time
- Test installer behavior with partial installations
- Add matrix testing for different dependency combinations
