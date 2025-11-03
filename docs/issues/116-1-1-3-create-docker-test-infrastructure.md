# Create Docker Test Infrastructure

**Issue**: [#116](https://github.com/torrust/torrust-tracker-deployer/issues/116)
**Parent Issue**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests  
**Depends On**: [#115](https://github.com/torrust/torrust-tracker-deployer/issues/115) - Create CLI Binary with Check Command (Issue 1-1-2)  
**Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution  
**Related**: [docs/e2e-testing.md](../e2e-testing.md)

## Overview

Create Docker-based testing infrastructure to verify the CLI binary works correctly in a clean Ubuntu 24.04 environment. This enables testing the `check` command in isolated containers before implementing installation logic.

## Objectives

- [ ] Create Dockerfile based on Ubuntu 24.04
- [ ] Set up testcontainers integration tests
- [ ] Copy existing Docker helper utilities
- [ ] Test the CLI binary in containers
- [ ] Verify check command works correctly
- [ ] Document testing approach for future phases

## Context

This is **Phase 3** of creating the dependency installation package. It validates that the CLI binary from Issue 1-1-2 works correctly in a clean environment, preparing for installation testing in Phase 4 (Issue 1-1-4).

### Why Docker Testing Matters

Docker testing ensures:

1. **Clean environment** - No accidentally relying on tools in development machine
2. **Reproducibility** - Tests work the same way on all machines
3. **CI readiness** - Same tests can run in GitHub Actions
4. **Installation verification** - Phase 4 will use this infrastructure to test actual installation

### Dependencies

- **Requires**: Issue 1-1-2 (CLI binary) must be completed first - we need a binary to test
- **Enables**: Issue 1-1-4 (installation logic) will extend this infrastructure

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (testing utilities)  
**Module Path**: `packages/dependency-installer/docker/` and `packages/dependency-installer/tests/`  
**Pattern**: Testcontainers + Docker helpers from existing codebase

### Directory Structure

```text
packages/dependency-installer/
├── docker/
│   ├── ubuntu-24.04.Dockerfile    # Base image for testing
│   └── README.md                   # Docker setup documentation
├── tests/
│   ├── docker_check_command.rs     # Integration tests
│   └── containers/
│       ├── mod.rs
│       ├── ubuntu.rs               # Ubuntu container helper
│       └── helpers.rs              # Copied from existing codebase
└── src/
    ├── bin/
    │   └── dependency-installer.rs # Binary to test
    └── lib.rs
```

## Specifications

### Dockerfile for Testing

Create `docker/ubuntu-24.04.Dockerfile`:

```dockerfile
# Base image: Ubuntu 24.04
FROM ubuntu:24.04

# Install minimal dependencies for running the binary
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Working directory
WORKDIR /app

# The binary will be copied by testcontainers at runtime
# No need to copy it here - testcontainers handles that

# Default command (can be overridden by tests)
CMD ["/bin/bash"]
```

### Docker README Documentation

Create `docker/README.md`:

````markdown
# Docker Testing Infrastructure

This directory contains Docker configurations for testing the dependency-installer CLI.

## Images

### ubuntu-24.04.Dockerfile

Base Ubuntu 24.04 image for testing the CLI binary in a clean environment.

**Purpose**: Verify that the `dependency-installer check` command correctly detects missing tools.

**Usage in tests**:

\```rust
let image = GenericImage::new("dependency-installer-test", "ubuntu-24.04")
.with_wait_for(WaitFor::message_on_stdout("Ready"));
\```

## Testing Strategy

1. Build the binary: `cargo build --bin dependency-installer`
2. Copy binary into container using testcontainers
3. Run `check` command in container
4. Verify it correctly reports missing tools
5. (Phase 4) Install tools and verify installation

## Building Images

\```bash
docker build -f docker/ubuntu-24.04.Dockerfile -t dependency-installer-test:ubuntu-24.04 .
\```

## Related

- `tests/docker_check_command.rs` - Integration tests using this infrastructure
- `tests/containers/` - Container helper utilities
````

### Testcontainers Integration Test

Create `tests/docker_check_command.rs`:

```rust
//! Integration tests for the dependency-installer CLI using Docker containers.
//!
//! These tests verify that the CLI binary works correctly in a clean Ubuntu 24.04
//! environment. They use testcontainers to spin up isolated Docker containers.

use std::path::PathBuf;
use testcontainers::clients::Cli as DockerCli;

mod containers;
use containers::ubuntu::UbuntuContainer;

/// Test that the check command correctly identifies missing dependencies
/// in a fresh Ubuntu 24.04 container
#[test]
fn test_check_all_reports_missing_dependencies() {
    // Set up Docker client
    let docker = DockerCli::default();

    // Get the binary path (built by cargo before running tests)
    let binary_path = get_binary_path();

    // Start Ubuntu container with the binary
    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .start();

    // Run the check command
    let output = container.exec(&["dependency-installer", "check"]);

    // Verify it reports missing dependencies
    assert!(output.contains("cargo-machete: not installed"));
    assert!(output.contains("OpenTofu: not installed"));
    assert!(output.contains("Ansible: not installed"));
    assert!(output.contains("LXD: not installed"));

    // Verify exit code is non-zero (failure)
    let exit_code = container.exec_with_exit_code(&["dependency-installer", "check"]);
    assert_eq!(exit_code, 1, "check command should exit with 1 when dependencies missing");
}

/// Test that the check command works for specific tools
#[test]
fn test_check_specific_tool() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .start();

    // Check a specific tool (OpenTofu)
    let output = container.exec(&["dependency-installer", "check", "--tool", "opentofu"]);

    assert!(output.contains("OpenTofu: not installed"));

    let exit_code = container.exec_with_exit_code(&["dependency-installer", "check", "--tool", "opentofu"]);
    assert_eq!(exit_code, 1);
}

/// Test that the list command works correctly
#[test]
fn test_list_command() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .start();

    let output = container.exec(&["dependency-installer", "list"]);

    // Verify all tools are listed
    assert!(output.contains("cargo-machete"));
    assert!(output.contains("OpenTofu"));
    assert!(output.contains("Ansible"));
    assert!(output.contains("LXD"));

    // Verify status is shown
    assert!(output.contains("not installed"));
}

/// Test verbose output flag
#[test]
fn test_verbose_output() {
    let docker = DockerCli::default();
    let binary_path = get_binary_path();

    let container = UbuntuContainer::new(&docker)
        .with_binary(&binary_path)
        .start();

    let output = container.exec(&["dependency-installer", "check", "--verbose"]);

    // Verify debug logs are present
    assert!(output.contains("DEBUG") || output.contains("Checking if"));
}

/// Get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    // Assumes the binary was built before running tests
    // E.g., with: cargo build --bin dependency-installer
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("dependency-installer");

    assert!(
        path.exists(),
        "Binary not found at {:?}. Run 'cargo build --bin dependency-installer' first",
        path
    );

    path
}
```

### Container Helper Module

Create `tests/containers/ubuntu.rs`:

```rust
//! Ubuntu container helper for testing the dependency-installer CLI.

use std::path::Path;
use testcontainers::{clients::Cli, Container, Image, RunnableImage};
use testcontainers::core::{WaitFor, ExecCommand};

use super::helpers::copy_file_to_container;

/// Helper for managing Ubuntu test containers
pub struct UbuntuContainer<'d> {
    container: Container<'d, UbuntuImage>,
}

impl<'d> UbuntuContainer<'d> {
    /// Create a new Ubuntu container builder
    pub fn new(docker: &'d Cli) -> UbuntuContainerBuilder<'d> {
        UbuntuContainerBuilder { docker }
    }

    /// Execute a command in the container and return stdout
    pub fn exec(&self, command: &[&str]) -> String {
        let output = self.container.exec(ExecCommand::new(command));
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    /// Execute a command and return the exit code
    pub fn exec_with_exit_code(&self, command: &[&str]) -> i32 {
        let output = self.container.exec(ExecCommand::new(command));
        output.exit_code.unwrap_or(0)
    }
}

/// Builder for Ubuntu containers
pub struct UbuntuContainerBuilder<'d> {
    docker: &'d Cli,
}

impl<'d> UbuntuContainerBuilder<'d> {
    /// Add the binary to the container
    pub fn with_binary(self, binary_path: &Path) -> UbuntuContainerWithBinary<'d> {
        UbuntuContainerWithBinary {
            docker: self.docker,
            binary_path: binary_path.to_path_buf(),
        }
    }
}

pub struct UbuntuContainerWithBinary<'d> {
    docker: &'d Cli,
    binary_path: PathBuf,
}

impl<'d> UbuntuContainerWithBinary<'d> {
    /// Start the container
    pub fn start(self) -> UbuntuContainer<'d> {
        let image = UbuntuImage::default();
        let container = self.docker.run(image);

        // Copy the binary into the container
        copy_file_to_container(&container, &self.binary_path, "/usr/local/bin/dependency-installer");

        // Make the binary executable
        container.exec(ExecCommand::new(vec!["chmod", "+x", "/usr/local/bin/dependency-installer"]));

        UbuntuContainer { container }
    }
}

/// Ubuntu 24.04 image for testing
#[derive(Debug, Default, Clone)]
struct UbuntuImage;

impl Image for UbuntuImage {
    fn name(&self) -> &str {
        "ubuntu"
    }

    fn tag(&self) -> &str {
        "24.04"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Ready")]
    }
}
```

### Copy Existing Docker Helpers

Copy helper utilities from `src/testing/e2e/containers/helpers.rs` to `tests/containers/helpers.rs`:

```rust
//! Helper utilities for working with testcontainers.
//!
//! Copied and adapted from src/testing/e2e/containers/helpers.rs

use std::path::Path;
use testcontainers::Container;

/// Copy a file from the host into a running container
pub fn copy_file_to_container<I: Image>(
    container: &Container<'_, I>,
    source_path: &Path,
    dest_path: &str,
) {
    // Implementation copied from existing codebase
    // This is a placeholder - actual implementation will use Docker API
    // or testcontainers utilities to copy files
    todo!("Copy implementation from src/testing/e2e/containers/helpers.rs")
}
```

## Implementation Tasks

### Docker Infrastructure Setup

- [ ] Create `docker/` directory in the package
- [ ] Create `docker/ubuntu-24.04.Dockerfile` with Ubuntu 24.04 base
- [ ] Create `docker/README.md` documenting the testing strategy
- [ ] Test building the Docker image manually:

  ```bash
  cd packages/dependency-installer
  docker build -f docker/ubuntu-24.04.Dockerfile -t dependency-installer-test:ubuntu-24.04 .
  ```

### Container Helper Implementation

**Source Location**: Copy existing Docker helpers from `src/testing/e2e/containers/` (main project repository)

- [ ] Create `tests/containers/` directory in the package
- [ ] Create `tests/containers/mod.rs` with module exports
- [ ] Copy `helpers.rs` from `src/testing/e2e/containers/helpers.rs`:
  - [ ] Copy `copy_file_to_container()` function
  - [ ] Adapt imports for new package location
  - [ ] Update any path references if needed
- [ ] Create `tests/containers/ubuntu.rs` with:
  - [ ] `UbuntuContainer` struct
  - [ ] `UbuntuContainerBuilder` for fluent API
  - [ ] `with_binary()` method to copy binary
  - [ ] `exec()` method to run commands
  - [ ] `exec_with_exit_code()` method for exit codes
- [ ] Test that containers can be started and stopped

### Integration Tests

- [ ] Create `tests/docker_check_command.rs`
- [ ] Add `get_binary_path()` helper function
- [ ] Implement test: `test_check_all_reports_missing_dependencies`
  - [ ] Start container
  - [ ] Run `dependency-installer check`
  - [ ] Verify output lists all missing tools
  - [ ] Verify exit code is 1
- [ ] Implement test: `test_check_specific_tool`
  - [ ] Test `--tool opentofu` flag
  - [ ] Verify correct output
  - [ ] Verify exit code
- [ ] Implement test: `test_list_command`
  - [ ] Verify all tools are listed
  - [ ] Verify statuses are shown
- [ ] Implement test: `test_verbose_output`
  - [ ] Test `--verbose` flag
  - [ ] Verify debug logs appear

### Cargo Configuration

- [ ] Add testcontainers dependencies to `Cargo.toml`:

  ```toml
  [dev-dependencies]
  testcontainers = "0.20"
  ```

- [ ] Configure integration tests in `Cargo.toml`:

  ```toml
  [[test]]
  name = "docker_check_command"
  path = "tests/docker_check_command.rs"
  ```

### Testing

- [ ] Build the binary: `cargo build --bin dependency-installer`
- [ ] Run integration tests: `cargo test --test docker_check_command`
- [ ] Verify all tests pass
- [ ] Check test output shows correct detection
- [ ] Verify containers are cleaned up after tests

### Documentation

- [ ] Document testing approach in `docker/README.md`
- [ ] Add usage examples to README
- [ ] Document how to run tests locally
- [ ] Explain the testing strategy

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All integration tests pass
- [ ] Docker containers are properly cleaned up after tests

**Docker Infrastructure**:

- [ ] Dockerfile builds successfully
- [ ] Dockerfile is based on Ubuntu 24.04
- [ ] Dockerfile is minimal and focused on testing

**Container Helpers**:

- [ ] `UbuntuContainer` helper works correctly
- [ ] Binary can be copied into containers
- [ ] Commands can be executed in containers
- [ ] Exit codes are captured correctly
- [ ] Helpers are reusable for Phase 4 (installation testing)

**Integration Tests**:

- [ ] Test `check` command in clean environment
- [ ] Test `check --tool <name>` for specific tools
- [ ] Test `list` command output
- [ ] Test `--verbose` flag
- [ ] All tests run in isolated Docker containers
- [ ] Tests verify exit codes correctly
- [ ] Tests verify output format

**Documentation**:

- [ ] `docker/README.md` explains the testing approach
- [ ] Examples show how to run tests
- [ ] Build instructions are clear
- [ ] Testing strategy is documented

## Example Test Output

```bash
$ cargo test --test docker_check_command

running 4 tests
test test_check_all_reports_missing_dependencies ... ok
test test_check_specific_tool ... ok
test test_list_command ... ok
test test_verbose_output ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Related Documentation

- [Issue 1-1-2](./115-1-1-2-create-cli-binary-with-check-command.md) - CLI binary being tested
- [Parent Issue 1-1](./create-dependency-installation-package-for-e2e-tests.md) - Overall package specification
- [docs/e2e-testing.md](../e2e-testing.md) - E2E testing documentation
- [src/testing/e2e/containers/](../../src/testing/e2e/containers/) - Existing container helpers to copy

## Notes

### Estimated Time

**2-3 hours** total for this phase.

### Next Steps

After completing this phase:

1. **Issue 1-1-4**: Implement installation logic and extend these tests to verify actual installation

### Design Decisions

**Ubuntu 24.04**: Chosen as the base image to match the target environment for E2E tests.

**Testcontainers**: Using testcontainers library provides automatic cleanup and works well in CI environments.

**Copy Docker helpers**: Reusing existing container utilities from `src/testing/e2e/containers/` ensures consistency and saves implementation time.

**Binary testing approach**: Tests copy the pre-built binary into containers rather than building inside containers, which is faster and simpler.

### Integration with Phase 4

This Docker infrastructure will be extended in Issue 1-1-4 to:

1. Test the `install` command
2. Verify tools are correctly installed
3. Re-run `check` command to verify successful installation
4. Test installation failures and error handling

The container helpers and test patterns established here will be reused and extended in Phase 4.
