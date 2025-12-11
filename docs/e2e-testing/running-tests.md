# Running E2E Tests

This guide explains how to run the E2E test suites and configure your environment.

## 🚀 Running Test Suites

### Infrastructure Lifecycle Tests

Test infrastructure provisioning and destruction lifecycle (VM creation, cloud-init, and destruction):

```bash
cargo run --bin e2e-infrastructure-lifecycle-tests
```

### Deployment Workflow Tests

Test software installation, configuration, release, and run workflows (Ansible playbooks):

```bash
cargo run --bin e2e-deployment-workflow-tests
```

### Complete Workflow Tests

For local development, you can run the complete end-to-end test:

```bash
cargo run --bin e2e-complete-workflow-tests
```

⚠️ **Note**: The `e2e-complete-workflow-tests` binary cannot run on GitHub Actions due to network connectivity issues, but is useful for local validation.

## ⚙️ Command Line Options

All test binaries support these options:

- `--keep` - Keep the test environment after completion (useful for debugging)
- `--templates-dir` - Specify custom templates directory path
- `--help` - Show help information

## 💡 Examples

```bash
# Run infrastructure lifecycle tests
cargo run --bin e2e-infrastructure-lifecycle-tests

# Run infrastructure lifecycle tests with debugging (keep environment)
cargo run --bin e2e-infrastructure-lifecycle-tests -- --keep

# Run deployment workflow tests with debugging
cargo run --bin e2e-deployment-workflow-tests -- --keep

# Run complete tests with custom templates
cargo run --bin e2e-complete-workflow-tests -- --templates-dir ./custom/templates
```

## 🛠️ Prerequisites

### Automated Setup (Recommended)

The project provides a dependency installer tool that automatically detects and installs required dependencies:

```bash
# Install all required dependencies
cargo run --bin dependency-installer install

# Check which dependencies are installed
cargo run --bin dependency-installer check

# List all dependencies with status
cargo run --bin dependency-installer list
```

The installer supports:

- **cargo-machete** - Detects unused Rust dependencies
- **OpenTofu** - Infrastructure provisioning tool
- **Ansible** - Configuration management tool
- **LXD** - VM-based testing infrastructure

For detailed information, see [`packages/dependency-installer/README.md`](../../packages/dependency-installer/README.md).

### Manual Setup

If you prefer manual installation or need to troubleshoot:

#### For Infrastructure Lifecycle Tests

1. **LXD installed and configured**

   ```bash
   sudo snap install lxd
   sudo lxd init  # Follow the setup prompts
   ```

2. **OpenTofu installed**

   ```bash
   # Installation instructions in docs/tech-stack/opentofu.md
   ```

#### For Deployment Workflow Tests

1. **Docker installed**

   ```bash
   # Docker is available on most systems or in CI environments
   docker --version
   ```

2. **Ansible installed**

   ```bash
   # Installation instructions in docs/tech-stack/ansible.md
   ```

#### For Complete Workflow Tests

Requires **all** of the above: LXD, OpenTofu, Docker, and Ansible.

### Verification

After setup (automated or manual), verify all dependencies are available:

```bash
# Quick check (exit code indicates success/failure)
cargo run --bin dependency-installer check

# Detailed check with logging
cargo run --bin dependency-installer check --verbose
```

## 🎯 Test Suite Selection Guide

**Use Infrastructure Lifecycle Tests (`e2e-infrastructure-lifecycle-tests`) when**:

- Testing infrastructure changes (OpenTofu, LXD configuration)
- Validating VM creation and cloud-init setup
- Working on provisioning-related features

**Use Deployment Workflow Tests (`e2e-deployment-workflow-tests`) when**:

- Testing Ansible playbooks and software installation
- Validating configuration management changes
- Working on application deployment features

**Use Complete Workflow Tests (`e2e-complete-workflow-tests`) when**:

- Comprehensive local validation before CI
- Integration testing of provision + configuration
- Debugging end-to-end deployment issues
