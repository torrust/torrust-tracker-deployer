# E2E Testing Guide

This guide explains how to run and understand the End-to-End (E2E) tests for the Torrust Tracker Deploy project.

## 🧪 What are E2E Tests?

The E2E tests validate the complete deployment process by:

1. **Provisioning infrastructure** using OpenTofu to create an LXD container
2. **Running Ansible playbooks** in the correct production order
3. **Validating each step** to ensure proper installation and configuration

## 🚀 Running E2E Tests

To run the full E2E test suite:

```bash
cargo run --bin e2e-tests
```

### Command Line Options

- `--keep` - Keep the test environment after completion (useful for debugging)
- `--verbose` - Enable verbose output to see detailed execution steps
- `--help` - Show help information

### Examples

```bash
# Run with verbose output
cargo run --bin e2e-tests -- --verbose

# Keep environment for debugging
cargo run --bin e2e-tests -- --keep

# Combine options
cargo run --bin e2e-tests -- --verbose --keep
```

## 📋 Test Sequence

The E2E tests execute the following steps in production order:

1. **Infrastructure Provisioning**

   - Uses OpenTofu configuration from `config/tofu/lxd/`
   - Creates LXD container with Ubuntu and cloud-init configuration

2. **Cloud-init Completion** (`wait-cloud-init.yml`)

   - Waits for cloud-init to finish system initialization
   - Validates user accounts and SSH key setup

3. **Docker Installation** (`install-docker.yml`)

   - Installs Docker Community Edition
   - Configures Docker service
   - Validates Docker daemon is running

4. **Docker Compose Installation** (`install-docker-compose.yml`)
   - Installs Docker Compose binary
   - Validates installation with test configuration

## 🔍 What Gets Validated

Each step includes validation to ensure proper setup:

### Cloud-init Validation

- ✅ Cloud-init status is "done"
- ✅ Boot completion marker file exists (`/var/lib/cloud/instance/boot-finished`)

### Docker Validation

- ✅ Docker version command works
- ✅ Docker daemon service is active

### Docker Compose Validation

- ✅ Docker Compose version command works
- ✅ Can parse and validate a test docker-compose.yml file

## 🛠️ Prerequisites

Before running E2E tests, ensure you have:

1. **LXD installed and configured**

   ```bash
   sudo snap install lxd
   sudo lxd init  # Follow the setup prompts
   ```

2. **OpenTofu installed**

   ```bash
   # Installation instructions in docs/tech-stack/opentofu.md
   ```

3. **Ansible installed**

   ```bash
   # Installation instructions in docs/tech-stack/ansible.md
   ```

## 🐛 Troubleshooting

### Test Environment Cleanup

If tests fail and leave resources behind, you can manually clean up:

```bash
# Check running containers
lxc list

# Stop and delete the test container
lxc stop torrust-vm
lxc delete torrust-vm

# Or use OpenTofu to clean up
cd config/tofu/lxd
tofu destroy -auto-approve
```

### Common Issues

- **SSH connectivity failures**: Usually means cloud-init is still running or SSH configuration failed
- **Ansible connection errors**: Check if the container IP is accessible and SSH key permissions are correct
- **OpenTofu errors**: Ensure LXD is properly configured and you have sufficient privileges

### Debug Mode

Use the `--keep` flag to inspect the environment after test completion:

```bash
cargo run --bin e2e-tests -- --keep --verbose

# After test completion, connect to the container:
lxc exec torrust-vm -- /bin/bash
```

## 🏗️ Architecture

The E2E tests use a layered approach:

```text
┌─────────────────────────────────────┐
│             E2E Tests               │
│    (Rust binary: e2e-tests)         │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│           OpenTofu/LXD              │
│       (Infrastructure Layer)        │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│        Ansible Playbooks            │
│      (Configuration Layer)          │
└─────────────────────────────────────┘
```

This mirrors the production deployment process where:

1. Infrastructure is provisioned first
2. Configuration management handles software installation
3. Validation ensures each step completed successfully

## 📝 Contributing to E2E Tests

When adding new playbooks or infrastructure changes:

1. **Update the test sequence** in `run_full_deployment_test()`
2. **Add validation methods** for new components
3. **Update this documentation** to reflect changes
4. **Test locally** before submitting PR

The E2E tests are designed to catch integration issues that unit tests cannot, ensuring the entire deployment pipeline works correctly.
