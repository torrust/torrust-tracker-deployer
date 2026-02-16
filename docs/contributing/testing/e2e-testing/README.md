# E2E Testing

End-to-end (E2E) tests verify complete workflows using real infrastructure and external systems.

## Overview

E2E tests for the Torrust Tracker Deployer validate the entire deployment lifecycle from environment creation through infrastructure provisioning, software configuration, and cleanup.

For comprehensive E2E testing documentation, see the main E2E testing guide:

**[E2E Testing Documentation](../../../e2e-testing/README.md)**

## E2E Test Binaries

The project includes specialized E2E test binaries:

- **`e2e-infrastructure-lifecycle-tests`** - Infrastructure provisioning and destruction
- **`e2e-deployment-workflow-tests`** - Software installation, configuration, and release
- **`e2e-complete-workflow-tests`** - Complete end-to-end workflows (local only)

## Key Characteristics

- **Real Infrastructure**: Uses actual LXD VMs, OpenTofu, and Ansible
- **Complete Workflows**: Tests entire command pipelines
- **Cleanup Handling**: Ensures resources are cleaned up even on failure
- **CI/CD Compatible**: Infrastructure tests run on GitHub Actions

## When to Use E2E Tests

Use E2E tests for:

- ✅ Validating complete deployment workflows
- ✅ Testing infrastructure provisioning and cleanup
- ✅ Verifying integration with external tools
- ✅ End-to-end state transitions

Do NOT use E2E tests for:

- ❌ Testing individual functions or methods (use unit tests)
- ❌ Testing error handling logic (use unit/integration tests)
- ❌ Fast feedback during development (too slow)
