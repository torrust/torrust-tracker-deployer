# Configure GitHub Copilot Agent Environment Setup

**Issue**: [#120](https://github.com/torrust/torrust-tracker-deployer/issues/120)
**Parent Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution  
**Depends On**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests (Issue 1-1)  
**Related**: [GitHub Docs - Customizing Copilot Agent Environment](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment)

## Overview

Create a GitHub Actions workflow file (`.github/workflows/copilot-setup-steps.yml`) that preinstalls all required development dependencies before the Copilot coding agent starts working. This ensures the agent has the same environment as human contributors, improving reliability and speed when working on assigned issues.

## Objectives

- [ ] Create `.github/workflows/copilot-setup-steps.yml` workflow file
- [ ] Use the dependency-installer binary from Issue 1-1 to install all tools
- [ ] Ensure workflow runs on the default branch for Copilot to use
- [ ] Test workflow manually through GitHub Actions tab
- [ ] Verify workflow runs successfully before agent starts
- [ ] Document the setup in project documentation

## Context

### Why This Matters

When you assign an issue to GitHub Copilot coding agent, it starts in a fresh ephemeral environment powered by GitHub Actions. While the agent **can** discover and install dependencies through trial and error, this approach is:

- **Slow**: LLM-based discovery is non-deterministic
- **Unreliable**: May fail to install complex dependencies
- **Inconsistent**: Environment may differ from local development

### Current Environment Requirements

Human contributors use these tools locally:

- **cargo-machete** - Detects unused dependencies (pre-commit checks)
- **OpenTofu** - Infrastructure provisioning (E2E tests)
- **Ansible** - Configuration management (E2E tests)
- **LXD** - Virtualization (E2E tests)

The agent needs the **exact same environment** to work effectively.

### Solution

GitHub provides a mechanism to preinstall tools via a special workflow file at `.github/workflows/copilot-setup-steps.yml`. This workflow:

1. Runs **before** the agent starts working
2. Sets up the environment with required dependencies
3. Uses the dependency-installer binary we created in Issue 1-1
4. Ensures consistency with human development environments

## 🏗️ Architecture Requirements

**File Location**: `.github/workflows/copilot-setup-steps.yml`  
**Runner**: `ubuntu-latest` (Copilot only supports Ubuntu x64 Linux)  
**Integration**: Uses dependency-installer binary from packages/dependency-installer/

## Specifications

### Copilot Setup Steps Workflow

Create `.github/workflows/copilot-setup-steps.yml`:

```yaml
name: "Copilot Setup Steps"

# Automatically run the setup steps when they are changed to allow for easy validation,
# and allow manual testing through the repository's "Actions" tab
on:
  workflow_dispatch:
  push:
    paths:
      - .github/workflows/copilot-setup-steps.yml
  pull_request:
    paths:
      - .github/workflows/copilot-setup-steps.yml

jobs:
  # The job MUST be called `copilot-setup-steps` or it will not be picked up by Copilot.
  copilot-setup-steps:
    runs-on: ubuntu-latest

    # Set the permissions to the lowest permissions possible needed for your steps.
    # Copilot will be given its own token for its operations.
    permissions:
      # We need to clone the repository to build and run the dependency-installer binary
      contents: read

    steps:
      - name: Checkout code
        uses: actions/checkout@v5

      - name: Set up Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable

      - name: Build dependency-installer binary
        run: |
          cd packages/dependency-installer
          cargo build --release --bin dependency-installer

      - name: Install all development dependencies
        run: |
          # Use the binary we just built to install all dependencies
          sudo packages/dependency-installer/target/release/dependency-installer install --yes
        env:
          # Ensure non-interactive installation
          DEBIAN_FRONTEND: noninteractive

      - name: Verify installations
        run: |
          # Verify all tools are installed correctly
          packages/dependency-installer/target/release/dependency-installer check
```

### Key Workflow Features

**Job Name**: Must be exactly `copilot-setup-steps` for GitHub to recognize it

**Triggers**:

- `workflow_dispatch` - Manual testing from Actions tab
- `push` on workflow changes - Auto-validation when workflow is modified
- `pull_request` on workflow changes - Validation before merging

**Permissions**: Minimal `contents: read` for checkout (Copilot gets its own token)

**Steps**:

1. **Checkout**: Clone the repository
2. **Rust Setup**: Install Rust toolchain (needed to build our binary)
3. **Build**: Compile dependency-installer in release mode
4. **Install**: Run binary with `--yes` flag (non-interactive)
5. **Verify**: Check all installations succeeded

### Why Use Our Binary Instead of Bash Scripts

**Advantages**:

- ✅ **Consistent with Issue 1-1**: Uses the same tool we created
- ✅ **Tested**: Binary has comprehensive tests (Issues 1-1-1 through 1-1-4)
- ✅ **Maintainable**: Single source of truth for dependency installation
- ✅ **Idempotent**: Safe to run multiple times
- ✅ **Error Handling**: Better error reporting than bash scripts
- ✅ **Future-proof**: When we update installation logic, workflow automatically uses new version

## Implementation Tasks

### Prerequisites

- [ ] **Verify directory structure exists**

  ```bash
  # Check if .github/workflows/ directory exists
  if [ ! -d ".github/workflows" ]; then
    echo "Creating .github/workflows directory..."
    mkdir -p .github/workflows
  fi
  ```

- [ ] **Verify .github directory is properly configured**

  - Ensure `.github/` is NOT in `.gitignore`
  - Workflows should be version-controlled
  - GitHub Actions should have permissions to read workflows

### Create Workflow File

- [ ] Create `.github/workflows/` directory if it doesn't exist
- [ ] Create `copilot-setup-steps.yml` file
- [ ] Copy the workflow specification above
- [ ] Ensure job name is exactly `copilot-setup-steps`
- [ ] Verify workflow syntax is correct

### Configure Workflow Settings

- [ ] Set `runs-on: ubuntu-latest` (Copilot only supports Ubuntu x64)
- [ ] Set minimal `permissions: contents: read`
- [ ] Configure triggers:
  - [ ] `workflow_dispatch` for manual testing
  - [ ] `push` on workflow file changes
  - [ ] `pull_request` on workflow file changes

### Implement Setup Steps

- [ ] Add checkout step using `actions/checkout@v5`
- [ ] Add Rust toolchain setup using `actions-rust-lang/setup-rust-toolchain@v1`
- [ ] Add build step for dependency-installer binary:
  - [ ] Use `cargo build --release` for optimized binary
  - [ ] Build in packages/dependency-installer directory
- [ ] Add installation step:
  - [ ] Run with `sudo` (some tools require it)
  - [ ] Use `--yes` flag for non-interactive installation
  - [ ] Set `DEBIAN_FRONTEND=noninteractive` environment variable
- [ ] Add verification step:
  - [ ] Run `dependency-installer check` to verify all tools installed
  - [ ] Exit with error if any tools missing

### Testing and Validation

- [ ] Commit workflow file to a feature branch
- [ ] Create pull request to test workflow runs on PR
- [ ] Verify workflow appears in GitHub Actions tab
- [ ] Test manual workflow run via `workflow_dispatch`
- [ ] Check workflow logs show all steps completing successfully
- [ ] Verify all dependencies are installed correctly
- [ ] Merge to default branch for Copilot to use

### Documentation Updates

- [ ] Add section to README about Copilot agent environment
- [ ] Document workflow purpose and how it works
- [ ] Explain why dependencies are preinstalled
- [ ] Add troubleshooting section if workflow fails

## Acceptance Criteria

**Workflow File**:

- [ ] File created at `.github/workflows/copilot-setup-steps.yml`
- [ ] Job name is exactly `copilot-setup-steps`
- [ ] Runs on `ubuntu-latest`
- [ ] Has minimal permissions (`contents: read`)

**Workflow Functionality**:

- [ ] Workflow builds dependency-installer binary successfully
- [ ] Workflow installs all 4 dependencies (cargo-machete, OpenTofu, Ansible, LXD)
- [ ] Verification step passes (all tools detected)
- [ ] Workflow completes without errors
- [ ] Workflow can be manually triggered from Actions tab

**Testing**:

- [ ] Workflow runs successfully on pull requests
- [ ] Workflow runs successfully on push to default branch
- [ ] Manual workflow run completes successfully
- [ ] All steps show green checkmarks in GitHub Actions UI
- [ ] Logs show successful installation of all tools

**Integration**:

- [ ] Workflow is merged to default branch (required for Copilot)
- [ ] Copilot agent uses preinstalled environment when assigned issues
- [ ] Agent doesn't waste time discovering/installing dependencies

**Documentation**:

- [ ] README documents Copilot environment setup
- [ ] Instructions explain how to test workflow manually
- [ ] Troubleshooting guide for common failures
- [ ] Links to GitHub documentation on Copilot environment customization

## Workflow Behavior

### When Workflow Runs

**Automatically**:

- When you create/update the workflow file in a PR
- When workflow file is pushed to any branch

**Manually**:

- From GitHub Actions tab (workflow_dispatch trigger)
- Useful for testing before assigning issues to Copilot

**For Copilot Agent**:

- Automatically before agent starts working on assigned issue
- Only if workflow file exists on default branch

### If Setup Fails

From GitHub documentation:

> If any setup step fails by returning a non-zero exit code, Copilot will skip the remaining setup steps and begin working with the current state of its development environment.

This means:

- Copilot will still start, but may lack dependencies
- May fall back to trial-and-error installation
- Better to fix workflow immediately if it fails

## Example Workflow Run Output

```text
Run actions/checkout@v5
  Checking out repository...
  ✓ Checkout complete

Run actions-rust-lang/setup-rust-toolchain@v1
  Installing Rust stable toolchain...
  ✓ Rust toolchain installed

Run cargo build --release --bin dependency-installer
  Compiling dependency-installer...
  ✓ Binary built: packages/dependency-installer/target/release/dependency-installer

Run dependency-installer install --yes
  Installing dependencies...
  ✓ cargo-machete: installed successfully
  ✓ OpenTofu: installed successfully
  ✓ Ansible: installed successfully
  ✓ LXD: installed successfully

  All dependencies installed successfully

Run dependency-installer check
  Checking dependencies...
  ✓ cargo-machete: installed
  ✓ OpenTofu: installed
  ✓ Ansible: installed
  ✓ LXD: installed

  All dependencies are installed
```

## Related Documentation

- [Issue 1-1](./1-1-create-dependency-installation-package-for-e2e-tests.md) - Dependency installer package this workflow uses
- [GitHub Docs - Customize Copilot Agent Environment](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment)
- [GitHub Actions - Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [GitHub Actions - Manual Workflow Run](https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-workflow-runs/manually-running-a-workflow)

## Notes

### Estimated Time

**2-3 hours** total for this issue.

### Dependencies

**Requires**:

- Issue 1-1 (or at least Issue 1-1-4) must be completed first
- Dependency-installer binary must exist and work correctly

**Blocks**:

- None directly, but improves Copilot agent effectiveness

### Design Decisions

**Why build binary in workflow**: We build from source rather than using pre-built releases because:

- Ensures we always use latest version from the branch
- No need to manage binary releases/artifacts
- Build is fast (~30 seconds with caching)

**Why use sudo**: Some tools (OpenTofu, Ansible, LXD) require system-level installation, so we need sudo privileges.

**Why --yes flag**: Copilot environment runs non-interactively, so we skip confirmation prompts.

**Why verify after install**: Ensures environment is correctly set up before Copilot starts working. If verification fails, we know immediately.

### Future Enhancements

- **Caching**: Add Rust build caching to speed up workflow
- **Conditional installation**: Only install tools if not already present (though binary handles this)
- **Larger runners**: If workflow is slow, consider using GitHub larger runners (see [GitHub Docs](https://docs.github.com/en/actions/using-github-hosted-runners/using-larger-runners/about-larger-runners))
- **Custom environment variables**: Add any project-specific environment variables if needed

### Copilot-Specific Notes

**Ubuntu x64 only**: Copilot coding agent only supports Ubuntu x64 Linux runners. Don't use Windows, macOS, or other operating systems.

**Self-hosted runners not supported**: Standard Copilot uses GitHub-hosted runners only. Self-hosted runners require ARC (Actions Runner Controller) setup.

**Workflow must be on default branch**: The workflow won't be used by Copilot unless it's present on your repository's default branch (usually `main`).

**Security**: The workflow runs before Copilot starts, but Copilot gets its own token with appropriate permissions. Your workflow only needs minimal permissions.
