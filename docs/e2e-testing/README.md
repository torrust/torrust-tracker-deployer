# E2E Testing Guide

This guide explains how to run and understand the End-to-End (E2E) tests for the Torrust Tracker Deployer project.

## 📖 Documentation Structure

- **[README.md](README.md)** - This overview and quick start guide
- **[architecture.md](architecture.md)** - E2E testing architecture, design decisions, and Docker strategy
- **[running-tests.md](running-tests.md)** - How to run automated tests, command-line options, and prerequisites
- **[manual/](manual/)** - Manual E2E testing guides:
  - **[README.md](manual/README.md)** - Complete manual test workflow (generic deployment guide)
  - **[mysql-verification.md](manual/mysql-verification.md)** - MySQL service verification and troubleshooting
  - **[prometheus-verification.md](manual/prometheus-verification.md)** - Prometheus metrics verification and troubleshooting
- **[test-suites.md](test-suites.md)** - Detailed description of each test suite and what they validate
- **[troubleshooting.md](troubleshooting.md)** - Common issues, debugging techniques, and cleanup procedures
- **[contributing.md](contributing.md)** - Guidelines for extending E2E tests
- **[advanced.md](advanced.md)** - Advanced techniques including cross-environment registration

## 🧪 What are E2E Tests?

The E2E tests validate the complete deployment process using two independent test suites:

1. **E2E Infrastructure Lifecycle Tests** - Test infrastructure provisioning and destruction lifecycle using LXD VMs
2. **E2E Deployment Workflow Tests** - Test software installation and configuration using Docker containers

This split approach ensures reliable testing in CI environments while maintaining comprehensive coverage.

## 🚀 Quick Start

### Run Infrastructure Lifecycle Tests

Test infrastructure provisioning and destruction lifecycle (VM creation, cloud-init, and destruction):

```bash
cargo run --bin e2e-infrastructure-lifecycle-tests
```

### Run Deployment Workflow Tests

Test software installation, configuration, release, and run workflows (Ansible playbooks):

```bash
cargo run --bin e2e-deployment-workflow-tests
```

### Run Full Local Testing

For local development, you can run the complete end-to-end test:

```bash
cargo run --bin e2e-complete-workflow-tests
```

⚠️ **Note**: The `e2e-complete-workflow-tests` binary cannot run on GitHub Actions due to network connectivity issues, but is useful for local validation.

## 🛠️ Quick Prerequisites Setup

The project provides a dependency installer tool that automatically detects and installs required dependencies:

```bash
# Install all required dependencies
cargo run --bin dependency-installer install

# Check which dependencies are installed
cargo run --bin dependency-installer check
```

For detailed prerequisites and manual setup, see [running-tests.md](running-tests.md).

## 📚 Learn More

- **New to E2E testing?** Start with [test-suites.md](test-suites.md) to understand what each test does
- **Want to run manual tests?** Follow [manual/README.md](manual/README.md) for step-by-step CLI workflow
- **Testing specific services?** See service-specific guides:
  - [manual/mysql-verification.md](manual/mysql-verification.md) - MySQL verification
  - [manual/prometheus-verification.md](manual/prometheus-verification.md) - Prometheus verification
- **Running into issues?** Check [troubleshooting.md](troubleshooting.md)
- **Want to understand the architecture?** Read [architecture.md](architecture.md)
- **Adding new tests?** See [contributing.md](contributing.md)
- **Advanced workflows?** Explore [advanced.md](advanced.md)

## 🔗 Related Documentation

For information about writing unit tests and testing conventions, see:

- **[docs/contributing/testing/](../contributing/testing/)** - Unit testing guidelines, conventions, and best practices
- **[docs/contributing/testing/unit-testing.md](../contributing/testing/unit-testing.md)** - Unit test organization and patterns
- **[docs/contributing/testing/coverage.md](../contributing/testing/coverage.md)** - Test coverage guidelines

E2E tests focus on system-level validation of the complete deployment workflow, while unit tests validate individual components in isolation.
