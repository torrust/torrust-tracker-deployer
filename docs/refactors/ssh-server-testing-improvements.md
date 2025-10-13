# SSH Server Testing Module Improvements

## 📋 Overview

This refactoring addresses code quality, maintainability, and testability issues in the SSH server testing module (`src/testing/integration/ssh_server.rs`). The module provides Docker-based SSH server containers for integration testing but suffers from code duplication, poor abstraction, inadequate error handling, and hardcoded values.

**Target Files:**

- `src/testing/integration/ssh_server.rs`
- New file: `src/testing/integration/ssh_server/docker_ops.rs` (to be created)
- New file: `src/testing/integration/ssh_server/config.rs` (to be created)
- New file: `src/testing/integration/ssh_server/errors.rs` (to be created)

**Scope:**

- Eliminate code duplication between Mock and Real implementations
- Introduce trait-based abstraction for polymorphism
- Replace generic error types with explicit, actionable error types
- Extract hardcoded values into configurable constants
- Abstract Docker operations for better testability
- Improve error messages with actionable guidance
- Add comprehensive test coverage for error scenarios
- Refactor debug function into testable, structured components

## 📊 Progress Tracking

**Total Active Proposals**: 10
**Total Postponed**: 3
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 10

### Phase Summary

- **Phase 0 - Quick Wins (High Impact, Low Effort)**: ⏳ 0/5 completed (0%)
  - #0: Convert File to Module Structure
  - #1: Extract Common SSH Server Trait
  - #2: Extract Hardcoded Constants
  - #3: Add Explicit Error Types with Thiserror
  - #4: Replace Unwrap with Proper Error Handling
- **Phase 1 - Core Improvements (High Impact, Medium Effort)**: ⏳ 0/4 completed (0%)
  - #5: Create General Docker Command Adapter
  - #6: Add Configuration Struct
  - #7: Refactor Debug Function into Testable Components
  - #8: Improve Error Messages with Actionable Guidance
- **Phase 2 - Enhanced Testing (Medium Impact, Medium Effort)**: ⏳ 0/2 completed (0%)
  - #9: Add Tests for Error Scenarios
  - #10: Implement Cleanup Methods

### Postponed Proposals

- **Docker Image Caching**: Deferred until performance becomes a measurable issue in CI
- **Advanced Test Coverage**: Deferred until core refactoring is complete
- **SSH Connectivity Health Check** (original #10): Removed - not needed for test-only code, the server is tested through usage in tests
- **Module Reorganization** (original #11): Merged into Proposal #0

## 🎯 Key Problems Identified

### 1. Code Duplication (DRY Violation)

`MockSshServerContainer` and `RealSshServerContainer` have identical public API methods with identical implementations. This violates the DRY principle and makes maintenance harder.

```rust
// Duplicated across both structs
pub fn ssh_port(&self) -> u16 { self.ssh_port }
pub fn host_ip(&self) -> IpAddr { self.host_ip }
pub fn test_username(&self) -> &str { &self.test_username }
pub fn test_password(&self) -> &str { &self.test_password }
```

### 2. Poor Error Handling

Uses generic `Box<dyn std::error::Error + Send + Sync>` instead of explicit error types, making it impossible to pattern match on specific error cases and providing poor user experience.

```rust
// Current: Generic error
pub async fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    return Err(format!("Docker build failed...").into());
}
```

### 3. Missing Abstraction

No trait defines the SSH server container interface, forcing callers to know whether they're using Mock or Real implementations. This breaks polymorphism and makes testing harder.

### 4. Hardcoded Values

Multiple hardcoded values scattered throughout the code make configuration inflexible and violate the separation of concerns principle.

```rust
"torrust-ssh-server:latest"  // Image name
22_u16  // SSH port
"testuser" / "testpass"  // Credentials
WaitFor::seconds(10)  // Timeout
"docker/ssh-server"  // Dockerfile path
```

### 5. Untestable Docker Operations

Docker build and container operations are directly embedded in production code with no abstraction, making them impossible to unit test.

### 6. Problematic Debug Function

The `print_docker_debug_info` function is 100+ lines, does too many things, is untested, and just prints to stdout without structured error handling.

### 7. Unwrap Usage

Code uses `.unwrap()` in production paths, violating the project's error handling guidelines:

```rust
dockerfile_dir.to_str().unwrap()  // Can panic
```

## 🚀 Refactoring Phases

---

## Phase 0: Quick Wins (Highest Priority)

High-impact improvements that require minimal effort and provide immediate benefits.

### Proposal #0: Convert File to Module Structure

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

The entire SSH server testing functionality is in a single file (`ssh_server.rs`), which will become harder to maintain as we add more types and functionality. Following the project's module organization conventions from the start will make future refactoring easier.

#### Proposed Solution

Convert the single file into a module structure with separate files for each type/concern:

```text
src/testing/integration/ssh_server/
    mod.rs              - Main module with traits, re-exports, and module documentation
    mock_container.rs   - MockSshServerContainer implementation
    real_container.rs   - RealSshServerContainer implementation
    debug.rs            - Debug information utilities (print_docker_debug_info)
```

Initial `mod.rs` structure:

```rust
//! SSH Server Container for Integration Testing
//!
//! This module provides SSH server containers for testing SSH client functionality.
//! Two implementations are available:
//!
//! - `MockSshServerContainer`: Fast mock for tests that don't need real SSH connectivity
//! - `RealSshServerContainer`: Actual Docker SSH server for full integration tests

// ============================================================================
// IMPORTS - Standard Library
// ============================================================================
use std::net::IpAddr;

// ============================================================================
// SUBMODULES
// ============================================================================
mod mock_container;
mod real_container;
mod debug;

// ============================================================================
// PUBLIC RE-EXPORTS
// ============================================================================
pub use mock_container::MockSshServerContainer;
pub use real_container::RealSshServerContainer;
pub use debug::print_docker_debug_info;

// ============================================================================
// TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    // Integration tests that test the module as a whole
}
```

Each implementation file will contain:

- The struct definition
- All impl blocks for that struct
- Unit tests specific to that implementation

#### Rationale

- Follows project module organization guidelines
- Makes the codebase easier to navigate from the start
- Prepares for future additions (config, errors, docker ops modules)
- Each file has a clear, single responsibility
- Easier to review changes (smaller diffs per file)
- Sets up proper structure before adding more complexity

#### Benefits

- ✅ Better organization from the start
- ✅ Easier to locate specific implementations
- ✅ Prepares structure for future modules (config, errors, docker adapter)
- ✅ Follows project conventions consistently
- ✅ Makes code reviews easier with smaller files
- ✅ Reduces merge conflicts in multi-contributor scenarios

#### Implementation Checklist

- [x] Create `src/testing/integration/ssh_server/` directory
- [x] Create `mod.rs` with module documentation and re-exports
- [x] Move `MockSshServerContainer` to `mock_container.rs`
- [x] Move `RealSshServerContainer` to `real_container.rs`
- [x] Move `print_docker_debug_info` to `debug.rs`
- [x] Move tests to appropriate files (impl-specific tests stay with impl)
- [x] Verify all imports work correctly
- [x] Ensure all tests still pass
- [x] Run linter and fix any issues
- [x] Update any external imports if needed

#### Testing Strategy

- Verify all existing tests still pass after reorganization
- Check that module can be imported from external code
- Confirm documentation builds correctly

---

### Proposal #1: Extract Common SSH Server Trait

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: Proposal #0 (module structure)

#### Problem

Mock and Real implementations share identical public APIs but have no common interface. This prevents polymorphism and forces callers to know the concrete type.

```rust
// Current: Callers must know the concrete type
let container = RealSshServerContainer::start().await?;
let port = container.ssh_port();

// Cannot write generic code that works with both
```

#### Proposed Solution

Create a trait defining the SSH server container interface:

```rust
/// Common interface for SSH server containers (mock and real)
pub trait SshServerContainer {
    /// Get the SSH port mapped by the container
    fn ssh_port(&self) -> u16;

    /// Get the container's host IP address
    fn host_ip(&self) -> IpAddr;

    /// Get the test username configured in the container
    fn test_username(&self) -> &str;

    /// Get the test password configured in the container
    fn test_password(&self) -> &str;
}

impl SshServerContainer for MockSshServerContainer {
    fn ssh_port(&self) -> u16 { self.ssh_port }
    fn host_ip(&self) -> IpAddr { self.host_ip }
    fn test_username(&self) -> &str { &self.test_username }
    fn test_password(&self) -> &str { &self.test_password }
}

impl SshServerContainer for RealSshServerContainer {
    fn ssh_port(&self) -> u16 { self.ssh_port }
    fn host_ip(&self) -> IpAddr { self.host_ip }
    fn test_username(&self) -> &str { &self.test_username }
    fn test_password(&self) -> &str { &self.test_password }
}
```

#### Rationale

- Enables polymorphic code that works with both Mock and Real containers
- Follows Interface Segregation Principle
- Makes testing easier (can use Mock in unit tests, Real in integration tests)
- Reduces coupling between test code and concrete implementations
- Standard Rust pattern for providing common behavior

#### Benefits

- ✅ Enables writing generic test code that works with both implementations
- ✅ Improves testability of code using SSH containers
- ✅ Makes the module's public API clearer and more maintainable
- ✅ Follows standard Rust trait-based design patterns
- ✅ Zero runtime cost (trait methods can be inlined)

#### Implementation Checklist

- [x] Define `SshServerContainer` trait in `mod.rs` with four methods
- [x] Implement trait for `MockSshServerContainer` in `mock_container.rs`
- [x] Implement trait for `RealSshServerContainer` in `real_container.rs`
- [x] Update documentation to explain trait usage
- [x] Add example showing polymorphic usage in module docs
- [x] Verify all tests pass
- [x] Run linter and fix any issues

#### Testing Strategy

- Verify existing tests still pass
- Add test demonstrating polymorphic usage with `Box<dyn SshServerContainer>`
- Confirm trait methods can be called through trait objects

---

### Proposal #2: Extract Hardcoded Constants

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: Proposal #0 (module structure)

#### Problem

Multiple hardcoded values are scattered throughout the code, making configuration inflexible and violating separation of concerns.

```rust
// Hardcoded throughout the file
"torrust-ssh-server:latest"
22_u16
"testuser"
"testpass"
2222  // Mock port
10  // Wait seconds
"docker/ssh-server"
```

#### Proposed Solution

Extract all constants to a dedicated constants section at the top of the module:

```rust
// ============================================================================
// CONSTANTS
// ============================================================================

/// Docker image name for the SSH server container
pub const SSH_SERVER_IMAGE_NAME: &str = "torrust-ssh-server";

/// Docker image tag for the SSH server container
pub const SSH_SERVER_IMAGE_TAG: &str = "latest";

/// SSH port inside the container
pub const SSH_CONTAINER_PORT: u16 = 22;

/// Mock SSH server port (for testing without Docker)
pub const MOCK_SSH_PORT: u16 = 2222;

/// Default test username configured in the SSH server
pub const DEFAULT_TEST_USERNAME: &str = "testuser";

/// Default test password configured in the SSH server
pub const DEFAULT_TEST_PASSWORD: &str = "testpass";

/// Container startup wait time in seconds
pub const CONTAINER_STARTUP_WAIT_SECS: u64 = 10;

/// Relative path to SSH server Dockerfile directory
pub const DOCKERFILE_DIR: &str = "docker/ssh-server";
```

#### Rationale

- Makes configuration values explicit and discoverable
- Enables easy adjustment without hunting through code
- Follows standard Rust practice of constants at module top
- Improves maintainability and readability
- Makes values self-documenting through naming and comments

#### Benefits

- ✅ Configuration values are explicit and easy to find
- ✅ Changes to values require updating only one location
- ✅ Values are self-documenting through names and doc comments
- ✅ Enables future configuration injection if needed
- ✅ Follows project module organization guidelines

#### Implementation Checklist

- [x] Add constants section at top of module after imports
- [x] Extract all hardcoded values to named constants
- [x] Add documentation for each constant
- [x] Replace all hardcoded values with constant references
- [x] Verify all tests pass
- [x] Run linter and fix any issues

#### Testing Strategy

- Verify existing tests pass after constant extraction
- Confirm constants can be accessed from test code
- Check that changing a constant affects behavior as expected

---

### Proposal #3: Add Explicit Error Types with Thiserror

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: Proposal #0 (module structure), Proposal #2 (constants)

#### Problem

Current error handling uses generic `Box<dyn std::error::Error + Send + Sync>`, preventing pattern matching and providing poor error messages without context or actionability.

```rust
// Current: Cannot pattern match
pub async fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    return Err(format!("Docker build failed:\n{}", stderr).into());
}

// Cannot do this:
match container.start().await {
    Err(SshServerError::DockerBuildFailed { .. }) => { /* specific handling */ }
    // ...
}
```

#### Proposed Solution

Create explicit error types using `thiserror` following the project's error handling guidelines:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshServerError {
    #[error("SSH server Dockerfile not found at '{expected_path}'
Tip: Ensure 'docker/ssh-server/Dockerfile' exists in the project root")]
    DockerfileNotFound {
        expected_path: String,
    },

    #[error("Docker build command failed for image '{image_name}'
Tip: Run 'docker build -t {image_name} {dockerfile_dir}' manually to see detailed errors")]
    DockerBuildFailed {
        image_name: String,
        dockerfile_dir: String,
        stdout: String,
        stderr: String,
    },

    #[error("Failed to start SSH server container
Tip: Check if Docker daemon is running with 'docker ps'")]
    ContainerStartFailed {
        #[source]
        source: testcontainers::core::error::Error,
    },

    #[error("Failed to get mapped port for SSH container
Tip: Verify container is running with 'docker ps'")]
    PortMappingFailed {
        #[source]
        source: testcontainers::core::error::Error,
    },

    #[error("Docker command execution failed: {command}
Tip: Verify Docker is installed and accessible: 'docker --version'")]
    DockerCommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid UTF-8 in Dockerfile path: {path}")]
    InvalidUtf8InPath {
        path: String,
    },
}

impl SshServerError {
    pub fn help(&self) -> &'static str {
        match self {
            Self::DockerfileNotFound { .. } => {
                "Dockerfile Not Found - Detailed Troubleshooting:

1. Verify the Dockerfile exists:
   ls -la docker/ssh-server/Dockerfile

2. Check you're running from project root:
   pwd  # Should show the torrust-tracker-deployer directory

3. If using a custom Dockerfile location:
   - Update the DOCKERFILE_DIR constant
   - Or pass the path as a parameter (if configuration is implemented)

For more information, see the SSH server documentation."
            }

            Self::DockerBuildFailed { .. } => {
                "Docker Build Failed - Detailed Troubleshooting:

1. Run the build command manually to see full output:
   docker build -t torrust-ssh-server:latest docker/ssh-server

2. Common issues:
   - Check Dockerfile syntax
   - Verify base image is accessible: docker pull ubuntu:22.04
   - Check network connectivity for package downloads
   - Review build logs for specific error messages

3. Check Docker daemon status:
   systemctl status docker  # Linux
   docker info  # General information

4. Try cleaning Docker build cache:
   docker builder prune

For more information, see Docker documentation."
            }

            Self::ContainerStartFailed { .. } => {
                "Container Start Failed - Detailed Troubleshooting:

1. Check if Docker daemon is running:
   docker ps

2. Verify sufficient resources:
   docker system df  # Check disk space
   docker info  # Check memory/CPU limits

3. Check for port conflicts:
   netstat -tlnp | grep :22
   ss -tlnp | grep :22

4. Review Docker logs:
   docker logs <container_id>

For more information, see testcontainers documentation."
            }

            Self::PortMappingFailed { .. } => {
                "Port Mapping Failed - Detailed Troubleshooting:

1. Verify container is running:
   docker ps

2. Check container port configuration:
   docker port <container_id>

3. Check if the required port is already in use:
   netstat -tlnp  # Linux
   ss -tlnp  # Alternative

For more information, see Docker networking documentation."
            }

            Self::DockerCommandFailed { .. } => {
                "Docker Command Execution Failed - Detailed Troubleshooting:

1. Verify Docker is installed:
   docker --version

2. Check Docker daemon is running:
   systemctl status docker  # Linux systemd
   docker ps  # Quick check

3. Verify user permissions:
   groups  # Check if user is in 'docker' group
   sudo usermod -aG docker $USER  # Add user to docker group
   # Log out and log back in for group changes to take effect

4. Try running Docker with sudo (temporary workaround):
   sudo docker ps

For more information, see Docker installation documentation."
            }

            Self::InvalidUtf8InPath { .. } => {
                "Invalid UTF-8 in Path - Detailed Troubleshooting:

1. Check the Dockerfile path contains only valid UTF-8 characters
2. Avoid special characters, emoji, or non-ASCII characters in paths
3. Use ASCII characters for file and directory names

This is typically a configuration or system encoding issue."
            }
        }
    }
}
```

#### Rationale

- Follows project error handling guidelines (explicit errors over anyhow)
- Enables pattern matching for specific error handling
- Provides clear, actionable error messages with context
- Implements tiered help system (brief tip + .help() method)
- Preserves source errors for complete traceability
- Each error variant includes sufficient context for diagnosis

#### Benefits

- ✅ Enables pattern matching and specific error handling
- ✅ Provides clear, actionable error messages
- ✅ Maintains error chain for complete traceability
- ✅ Follows project error handling guidelines
- ✅ Improves debugging and user experience
- ✅ Makes errors testable and verifiable

#### Implementation Checklist

- [x] Create error types with `thiserror` derive macros
- [x] Add `#[error]` attributes with brief, clear messages
- [x] Include tips in error messages
- [x] Implement `.help()` method with detailed troubleshooting
- [x] Add `#[source]` attributes for error chaining
- [x] Update all `Result` return types to use `SshServerError`
- [x] Replace all `.into()` error conversions with explicit error constructors
- [x] Add error-specific context (paths, commands, etc.)
- [x] Verify all tests pass
- [x] Run linter and fix any issues
- [x] Update documentation with error handling examples

#### Testing Strategy

- Add tests verifying each error variant can be constructed
- Test error messages contain expected information
- Verify source error chaining works correctly
- Add integration tests for common error scenarios

---

### Proposal #4: Replace Unwrap with Proper Error Handling

**Status**: ✅ Completed (done together with Proposal #3)  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: Proposal #3 (error types)

#### Problem

Code uses `.unwrap()` in production paths, which can panic and violates the project's error handling guidelines.

```rust
// Current: Can panic
.args([
    "build",
    "-t",
    "torrust-ssh-server:latest",
    dockerfile_dir.to_str().unwrap(),  // PANIC if path is not valid UTF-8
])
```

#### Proposed Solution

Replace all `unwrap()` calls with proper error handling:

```rust
// Use the error type from Proposal #2
let dockerfile_dir_str = dockerfile_dir
    .to_str()
    .ok_or_else(|| SshServerError::InvalidUtf8InPath {
        path: dockerfile_dir.to_string_lossy().to_string(),
    })?;

let build_output = Command::new("docker")
    .args([
        "build",
        "-t",
        "torrust-ssh-server:latest",
        dockerfile_dir_str,
    ])
    .output()
    .map_err(|source| SshServerError::DockerCommandFailed {
        command: "docker build".to_string(),
        source,
    })?;
```

#### Rationale

- Follows project guidelines: prefer `expect()` over `unwrap()`, but prefer proper error handling over both
- Prevents panics in production code
- Provides better error messages with context
- Makes error cases explicit and handleable
- Aligns with Rust best practices and project standards

#### Benefits

- ✅ Eliminates panic risk in production code
- ✅ Provides clear error messages for failure cases
- ✅ Follows project error handling guidelines
- ✅ Makes error cases explicit and testable
- ✅ Improves code reliability and maintainability

#### Implementation Checklist

- [x] Identify all `.unwrap()` calls in the module
- [x] Replace each with appropriate error handling
- [x] Use error types from Proposal #3
- [x] Add proper error context to each conversion
- [x] Verify all tests pass
- [x] Run linter and fix any issues

**Note**: This was completed together with Proposal #3. The `.unwrap()` call on `dockerfile_dir.to_str()` was replaced with `ok_or_else()` and the `InvalidUtf8InPath` error variant.

#### Testing Strategy

- Add tests that trigger the error conditions (invalid UTF-8 paths, etc.)
- Verify error messages are clear and actionable
- Confirm no panics occur in error scenarios

---

## Phase 1: Core Improvements

High-impact improvements that require moderate effort and build on Phase 0 foundations.

### Proposal #5: Create General Docker Command Adapter

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #3 (error types)

#### Problem

Docker operations are directly embedded in `RealSshServerContainer` using raw `Command` execution, making them impossible to unit test and not reusable across the project. We make heavy use of Docker throughout the project and would benefit from a general-purpose Docker adapter following the same pattern as our Ansible, OpenTofu, and LXD adapters.

```rust
// Current: Raw Docker commands in production code
impl RealSshServerContainer {
    pub async fn start() -> Result<Self, SshServerError> {
        // Direct Command execution - not testable, not reusable
        let build_output = Command::new("docker")
            .args(["build", "-t", "image:tag", "/path"])
            .output()?;
        // ...
    }
}
```

#### Proposed Solution

Create a general Docker adapter in `src/shared/docker` following the same pattern as our other external tool adapters (Ansible, OpenTofu, LXD). The adapter will use our `CommandExecutor` wrapper and provide a clean, testable interface for Docker operations.

**Module Structure:**

```text
src/shared/docker/
    mod.rs          - Module exports and documentation
    adapter/
        mod.rs      - Adapter re-exports
        client.rs   - DockerAdapter implementation
        errors.rs   - Docker-specific error types
    commands/
        mod.rs      - Command builders re-exports
        build.rs    - docker build command
        images.rs   - docker images command
        ps.rs       - docker ps command
        logs.rs     - docker logs command
```

**Docker Adapter (`src/shared/docker/adapter/client.rs`):**

```rust
use crate::shared::command::CommandExecutor;
use std::path::Path;
use std::sync::Arc;

/// Adapter for Docker CLI operations
///
/// This adapter wraps Docker CLI commands using our CommandExecutor,
/// following the same pattern as Ansible, OpenTofu, and LXD adapters.
pub struct DockerAdapter {
    executor: Arc<dyn CommandExecutor>,
}

impl DockerAdapter {
    pub fn new(executor: Arc<dyn CommandExecutor>) -> Self {
        Self { executor }
    }

    /// Build a Docker image from a Dockerfile
    pub fn build_image(
        &self,
        dockerfile_dir: &Path,
        image_name: &str,
        tag: &str,
    ) -> Result<BuildOutput, DockerError> {
        let build_cmd = BuildCommand::new(dockerfile_dir, image_name, tag);
        let output = self.executor.execute(build_cmd.to_command())?;

        if !output.status.success() {
            return Err(DockerError::BuildFailed {
                image: format!("{}:{}", image_name, tag),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(BuildOutput::from(output))
    }

    /// List Docker images
    pub fn list_images(&self, filter: Option<&str>) -> Result<Vec<ImageInfo>, DockerError> {
        let images_cmd = ImagesCommand::new().with_filter(filter);
        let output = self.executor.execute(images_cmd.to_command())?;

        ImageInfo::parse_from_output(&output.stdout)
    }

    /// List Docker containers
    pub fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>, DockerError> {
        let ps_cmd = PsCommand::new().all(all);
        let output = self.executor.execute(ps_cmd.to_command())?;

        ContainerInfo::parse_from_output(&output.stdout)
    }

    /// Get logs from a container
    pub fn get_logs(
        &self,
        container_id: &str,
        tail: Option<usize>,
    ) -> Result<String, DockerError> {
        let logs_cmd = LogsCommand::new(container_id).tail(tail);
        let output = self.executor.execute(logs_cmd.to_command())?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if a Docker image exists locally
    pub fn image_exists(&self, image_name: &str, tag: &str) -> Result<bool, DockerError> {
        let filter = format!("{}:{}", image_name, tag);
        let images = self.list_images(Some(&filter))?;
        Ok(!images.is_empty())
    }
}
```

**Command Builders (`src/shared/docker/commands/`):**

```rust
// build.rs
pub struct BuildCommand {
    dockerfile_dir: PathBuf,
    image_name: String,
    tag: String,
    no_cache: bool,
}

impl BuildCommand {
    pub fn new(dockerfile_dir: &Path, image_name: &str, tag: &str) -> Self {
        Self {
            dockerfile_dir: dockerfile_dir.to_path_buf(),
            image_name: image_name.to_string(),
            tag: tag.to_string(),
            no_cache: false,
        }
    }

    pub fn no_cache(mut self, enabled: bool) -> Self {
        self.no_cache = enabled;
        self
    }

    pub fn to_command(&self) -> Command {
        let mut cmd = Command::new("docker");
        cmd.args(["build", "-t", &format!("{}:{}", self.image_name, self.tag)]);

        if self.no_cache {
            cmd.arg("--no-cache");
        }

        cmd.arg(&self.dockerfile_dir);
        cmd
    }
}
```

**Error Types (`src/shared/docker/adapter/errors.rs`):**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DockerError {
    #[error("Docker build failed for image '{image}'
Tip: Run the build command manually to see detailed output")]
    BuildFailed {
        image: String,
        stdout: String,
        stderr: String,
    },

    #[error("Docker command execution failed: {command}
Tip: Verify Docker is installed and running: 'docker ps'")]
    CommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse Docker command output
Tip: Docker CLI output format may have changed")]
    ParseError {
        output: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl DockerError {
    pub fn help(&self) -> &'static str {
        match self {
            Self::BuildFailed { .. } => {
                "Docker Build Failed - Detailed Troubleshooting:

1. Run build manually: docker build -t <image>:<tag> <path>
2. Check Dockerfile syntax and base image availability
3. Verify network connectivity for package downloads
4. Check Docker daemon logs: journalctl -u docker (Linux)
5. Try with --no-cache flag to rebuild from scratch

For more information, see Docker documentation."
            }

            Self::CommandFailed { .. } => {
                "Docker Command Failed - Detailed Troubleshooting:

1. Verify Docker is installed: docker --version
2. Check Docker daemon is running: docker ps
3. Verify permissions: add user to docker group
   sudo usermod -aG docker $USER
4. Try with sudo as temporary workaround

For more information, see Docker installation guide."
            }

            Self::ParseError { .. } => {
                "Docker Output Parse Error - Detailed Troubleshooting:

1. Check Docker CLI version compatibility
2. Verify output format hasn't changed: docker version
3. Report issue with Docker and application versions

This may indicate a Docker CLI version incompatibility."
            }
        }
    }
}
```

**Usage in `RealSshServerContainer`:**

```rust
use crate::shared::docker::DockerAdapter;
use crate::shared::command::DefaultCommandExecutor;

impl RealSshServerContainer {
    pub async fn start_with_adapter(
        docker: Arc<DockerAdapter>,
    ) -> Result<Self, SshServerError> {
        // Use the Docker adapter instead of raw commands
        docker.build_image(
            Path::new(DOCKERFILE_DIR),
            SSH_SERVER_IMAGE_NAME,
            SSH_SERVER_IMAGE_TAG,
        )?;

        // Start container with built image
        // ...
    }

    pub async fn start() -> Result<Self, SshServerError> {
        let executor = Arc::new(DefaultCommandExecutor::new());
        let docker = Arc::new(DockerAdapter::new(executor));
        Self::start_with_adapter(docker).await
    }
}
```

**Benefits of `DockerDebugInfo` Using Docker Adapter:**

The `DockerDebugInfo` type from Proposal #6 can then use the Docker adapter as a collaborator:

```rust
impl DockerDebugInfo {
    pub fn collect_with_adapter(
        docker: &DockerAdapter,
        container_port: u16,
    ) -> Self {
        Self {
            all_containers: docker.list_containers(true)
                .map(|containers| format!("{:?}", containers))
                .map_err(|e| e.to_string()),
            // ... use adapter for all Docker operations
        }
    }
}
```

#### Rationale

- **Follows Project Patterns**: Uses the same adapter pattern as Ansible, OpenTofu, LXD
- **Uses CommandExecutor**: Leverages our existing command execution wrapper
- **Reusable**: Can be used throughout the project wherever Docker is needed
- **Testable**: Easy to mock with a test implementation of CommandExecutor
- **Maintainable**: Centralized Docker logic in one place
- **Consistent**: All external tools follow the same pattern
- **Future-Proof**: Easy to add more Docker operations as needed

#### Benefits

- ✅ Follows established project patterns for external tools
- ✅ Reusable across the entire project (not just SSH tests)
- ✅ Testable through CommandExecutor mocking
- ✅ Consistent with Ansible, OpenTofu, and LXD adapters
- ✅ Clean separation of concerns
- ✅ Easy to extend with new Docker operations
- ✅ Better error handling with specific error types
- ✅ Enables `DockerDebugInfo` to use a proper collaborator

#### Implementation Checklist

- [ ] Create `src/shared/docker/` module structure
- [ ] Implement `DockerAdapter` in `src/shared/docker/adapter/client.rs`
- [ ] Create `DockerError` types in `src/shared/docker/adapter/errors.rs`
- [ ] Implement command builders in `src/shared/docker/commands/`
  - [ ] `BuildCommand` for docker build
  - [ ] `ImagesCommand` for docker images
  - [ ] `PsCommand` for docker ps
  - [ ] `LogsCommand` for docker logs
- [ ] Add output parsing types (`ImageInfo`, `ContainerInfo`, etc.)
- [ ] Update `RealSshServerContainer` to use `DockerAdapter`
- [ ] Add unit tests for Docker adapter using mock `CommandExecutor`
- [ ] Add integration tests with real Docker
- [ ] Update `DockerDebugInfo` to use `DockerAdapter` as collaborator
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues
- [ ] Update documentation

#### Testing Strategy

- Unit test `DockerAdapter` methods with mock `CommandExecutor`
- Unit test command builders produce correct arguments
- Unit test output parsing logic
- Integration test with real Docker daemon
- Test error scenarios and error messages
- Verify `RealSshServerContainer` works with adapter

#### Testing Strategy

- Unit test `DockerCliBuilder` methods with mock filesystem/commands
- Unit test `RealSshServerContainer` with `MockDockerBuilder`
- Add tests for Docker build failure scenarios
- Verify integration tests still pass with real Docker

---

### Proposal #6: Add Configuration Struct

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #0 (module structure), Proposal #2 (constants)

#### Problem

All configuration is hardcoded or embedded in constants, making it impossible to customize container behavior without modifying code.

```rust
// Current: No way to customize
let container = RealSshServerContainer::start().await?;
// Always uses default credentials, port, image name, etc.
```

#### Proposed Solution

Create a configuration struct with builder pattern:

```rust
// src/testing/integration/ssh_server/config.rs

/// Configuration for SSH server containers
#[derive(Debug, Clone)]
pub struct SshServerConfig {
    /// Docker image name
    pub image_name: String,

    /// Docker image tag
    pub image_tag: String,

    /// SSH port inside container
    pub container_port: u16,

    /// Test username
    pub username: String,

    /// Test password
    pub password: String,

    /// Container startup wait time
    pub startup_wait_secs: u64,

    /// Dockerfile directory path
    pub dockerfile_dir: PathBuf,
}

impl SshServerConfig {
    /// Create configuration with default values from constants
    pub fn default() -> Self {
        Self {
            image_name: SSH_SERVER_IMAGE_NAME.to_string(),
            image_tag: SSH_SERVER_IMAGE_TAG.to_string(),
            container_port: SSH_CONTAINER_PORT,
            username: DEFAULT_TEST_USERNAME.to_string(),
            password: DEFAULT_TEST_PASSWORD.to_string(),
            startup_wait_secs: CONTAINER_STARTUP_WAIT_SECS,
            dockerfile_dir: PathBuf::from(DOCKERFILE_DIR),
        }
    }

    /// Create a builder for custom configuration
    pub fn builder() -> SshServerConfigBuilder {
        SshServerConfigBuilder::default()
    }
}

/// Builder for SSH server configuration
#[derive(Debug, Default)]
pub struct SshServerConfigBuilder {
    image_name: Option<String>,
    image_tag: Option<String>,
    container_port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    startup_wait_secs: Option<u64>,
    dockerfile_dir: Option<PathBuf>,
}

impl SshServerConfigBuilder {
    pub fn image_name(mut self, name: impl Into<String>) -> Self {
        self.image_name = Some(name.into());
        self
    }

    pub fn image_tag(mut self, tag: impl Into<String>) -> Self {
        self.image_tag = Some(tag.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn startup_wait_secs(mut self, secs: u64) -> Self {
        self.startup_wait_secs = Some(secs);
        self
    }

    pub fn dockerfile_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dockerfile_dir = Some(dir.into());
        self
    }

    pub fn build(self) -> SshServerConfig {
        let defaults = SshServerConfig::default();
        SshServerConfig {
            image_name: self.image_name.unwrap_or(defaults.image_name),
            image_tag: self.image_tag.unwrap_or(defaults.image_tag),
            container_port: self.container_port.unwrap_or(defaults.container_port),
            username: self.username.unwrap_or(defaults.username),
            password: self.password.unwrap_or(defaults.password),
            startup_wait_secs: self.startup_wait_secs.unwrap_or(defaults.startup_wait_secs),
            dockerfile_dir: self.dockerfile_dir.unwrap_or(defaults.dockerfile_dir),
        }
    }
}
```

Update containers to accept configuration:

```rust
impl RealSshServerContainer {
    /// Start container with custom configuration
    pub async fn start_with_config(config: SshServerConfig) -> Result<Self, SshServerError> {
        // Use config values instead of constants
    }

    /// Start container with default configuration
    pub async fn start() -> Result<Self, SshServerError> {
        Self::start_with_config(SshServerConfig::default()).await
    }
}

impl MockSshServerContainer {
    /// Create mock with custom configuration
    pub fn new_with_config(config: SshServerConfig) -> Self {
        // Use config values
    }

    /// Create mock with default configuration
    pub fn start() -> Result<Self, SshServerError> {
        Ok(Self::new_with_config(SshServerConfig::default()))
    }
}
```

#### Rationale

- Enables customization without modifying code
- Follows Builder pattern for ergonomic API
- Maintains backward compatibility (default configuration)
- Separates configuration from behavior
- Makes testing with different configurations easier
- Standard Rust pattern for flexible configuration

#### Benefits

- ✅ Enables customizing container behavior without code changes
- ✅ Makes testing with different configurations easier
- ✅ Maintains backward compatibility with existing code
- ✅ Provides clear, type-safe configuration API
- ✅ Follows standard Rust configuration patterns

#### Implementation Checklist

- [ ] Create `src/testing/integration/ssh_server/config.rs`
- [ ] Define `SshServerConfig` struct
- [ ] Implement `SshServerConfigBuilder` with builder pattern
- [ ] Update `RealSshServerContainer` to accept configuration
- [ ] Update `MockSshServerContainer` to accept configuration
- [ ] Add `start()` methods with default configuration for backward compatibility
- [ ] Add documentation with usage examples
- [ ] Add tests verifying configuration behavior
- [ ] Verify all existing tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- Test default configuration matches current hardcoded values
- Test builder pattern allows customizing each field
- Test containers use provided configuration
- Verify backward compatibility (existing tests still pass)

---

### Proposal #7: Refactor Debug Function into Testable Components

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #5 (Docker adapter)

#### Problem

The `print_docker_debug_info` function is 100+ lines, does too many things, is untested, and just prints to stdout without structured error handling.

```rust
// Current: 100+ line function that does everything
pub fn print_docker_debug_info(container_port: u16) {
    println!("\n=== Docker Debug Information ===");
    // Run docker ps
    // Run docker images
    // Run docker logs
    // Check port usage
    // ... etc
}
```

#### Proposed Solution

Split into smaller, testable functions with structured data:

```rust
/// Debug information collected about Docker containers
#[derive(Debug)]
pub struct DockerDebugInfo {
    pub all_containers: Result<String, String>,
    pub ssh_images: Result<String, String>,
    pub running_ssh_containers: Result<Vec<ContainerInfo>, String>,
    pub port_usage: Result<Vec<PortInfo>, String>,
}

/// Information about a Docker container
#[derive(Debug)]
pub struct ContainerInfo {
    pub id: String,
    pub status: String,
    pub logs: Result<String, String>,
}

/// Information about port usage
#[derive(Debug)]
pub struct PortInfo {
    pub port: u16,
    pub process: String,
}

impl DockerDebugInfo {
    /// Collect all Docker debug information
    pub fn collect(container_port: u16) -> Self {
        Self {
            all_containers: Self::list_all_containers(),
            ssh_images: Self::list_ssh_images(),
            running_ssh_containers: Self::find_ssh_containers(),
            port_usage: Self::check_port_usage(container_port),
        }
    }

    /// List all Docker containers (docker ps -a)
    fn list_all_containers() -> Result<String, String> {
        Command::new("docker")
            .args(["ps", "-a"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
            .map_err(|e| format!("Failed to list containers: {e}"))
    }

    /// List SSH server Docker images
    fn list_ssh_images() -> Result<String, String> {
        Command::new("docker")
            .args(["images", "torrust-ssh-server"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
            .map_err(|e| format!("Failed to list images: {e}"))
    }

    /// Find containers using SSH server image
    fn find_ssh_containers() -> Result<Vec<ContainerInfo>, String> {
        let output = Command::new("docker")
            .args(["ps", "-a", "--filter", "ancestor=torrust-ssh-server:latest"])
            .output()
            .map_err(|e| format!("Failed to filter containers: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = Vec::new();

        for line in stdout.lines().skip(1) {  // Skip header
            if let Some(id) = line.split_whitespace().next() {
                let logs = Self::get_container_logs(id);
                containers.push(ContainerInfo {
                    id: id.to_string(),
                    status: line.to_string(),
                    logs,
                });
            }
        }

        Ok(containers)
    }

    /// Get logs for a specific container
    fn get_container_logs(container_id: &str) -> Result<String, String> {
        Command::new("docker")
            .args(["logs", "--tail", "20", container_id])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
            .map_err(|e| format!("Failed to get logs: {e}"))
    }

    /// Check port usage
    fn check_port_usage(port: u16) -> Result<Vec<PortInfo>, String> {
        // Try netstat first, fallback to ss
        Self::check_port_with_netstat(port)
            .or_else(|_| Self::check_port_with_ss(port))
    }

    fn check_port_with_netstat(port: u16) -> Result<Vec<PortInfo>, String> {
        // Implementation
    }

    fn check_port_with_ss(port: u16) -> Result<Vec<PortInfo>, String> {
        // Implementation
    }

    /// Print the debug information in a formatted way
    pub fn print(&self) {
        println!("\n=== Docker Debug Information ===");

        if let Ok(containers) = &self.all_containers {
            println!("Docker containers (docker ps -a):");
            println!("{containers}");
        } else if let Err(e) = &self.all_containers {
            println!("Failed to list containers: {e}");
        }

        // ... rest of printing logic

        println!("=== End Docker Debug Information ===\n");
    }
}

/// Collect and print Docker debug information
///
/// This is a convenience function that collects and prints debug info.
/// For structured access to the data, use `DockerDebugInfo::collect()`.
pub fn print_docker_debug_info(container_port: u16) {
    let debug_info = DockerDebugInfo::collect(container_port);
    debug_info.print();
}
```

#### Rationale

- Separates data collection from presentation
- Makes each function testable in isolation
- Returns structured data instead of just printing
- Enables programmatic access to debug information
- Follows Single Responsibility Principle
- Each function is small, focused, and easy to understand

#### Benefits

- ✅ Debug functions are now testable
- ✅ Structured data can be used programmatically
- ✅ Each function has a single, clear responsibility
- ✅ Easier to maintain and extend
- ✅ Better error handling with Result types
- ✅ Can be used in automated diagnostics

#### Implementation Checklist

- [ ] Define `DockerDebugInfo` struct with all debug data
- [ ] Define helper structs (`ContainerInfo`, `PortInfo`)
- [ ] Implement `DockerDebugInfo::collect()` method
- [ ] Extract each operation into a separate method
- [ ] Implement `print()` method for formatted output
- [ ] Keep original `print_docker_debug_info()` as convenience wrapper
- [ ] Add unit tests for each data collection method
- [ ] Add integration test verifying full debug info collection
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues
- [ ] Update documentation

#### Testing Strategy

- Unit test each data collection method with mock commands
- Test error handling for each failure scenario
- Test structured data construction
- Test printing functionality
- Add integration test collecting real debug info

---

### Proposal #8: Improve Error Messages with Actionable Guidance

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #3 (error types)

#### Problem

Current error messages lack actionable guidance and specific context to help users resolve issues.

#### Proposed Solution

This was largely addressed in Proposal #3 with the error types including tips and `.help()` methods. This proposal focuses on enhancing those messages further based on real-world usage.

#### Implementation Checklist

- [ ] Review all error messages in `SshServerError`
- [ ] Ensure each error has a clear tip in the message
- [ ] Verify `.help()` methods provide comprehensive troubleshooting
- [ ] Add platform-specific guidance (Linux vs macOS vs Windows)
- [ ] Include links to relevant documentation
- [ ] Test error messages with real failure scenarios
- [ ] Get feedback from users on error clarity

#### Testing Strategy

- Manually trigger each error scenario
- Verify error messages are clear and actionable
- Test that following the guidance resolves the issue
- Collect feedback on error message quality

---

## Phase 2: Enhanced Testing

Medium-impact improvements focused on test coverage and reliability.

### Proposal #9: Add Tests for Error Scenarios

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposal #3 (error types), Proposal #5 (Docker adapter)

#### Problem

Current tests only verify happy path scenarios. Error conditions are not tested, leaving gaps in test coverage.

```rust
// Current: Only tests successful container start
#[tokio::test]
async fn it_should_start_real_ssh_server_container() {
    let container = RealSshServerContainer::start().await;
    // Only checks success case
}
```

#### Proposed Solution

Add comprehensive error scenario tests using the mock Docker builder:

```rust
#[cfg(test)]
mod error_tests {
    use super::*;

    #[tokio::test]
    async fn it_should_return_error_when_dockerfile_not_found() {
        // Create config with non-existent Dockerfile
        let config = SshServerConfig::builder()
            .dockerfile_dir("/nonexistent/path")
            .build();

        let result = RealSshServerContainer::start_with_config(config).await;

        assert!(matches!(
            result,
            Err(SshServerError::DockerfileNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn it_should_return_error_when_docker_build_fails() {
        // Use mock builder that fails
        let mock_builder = Arc::new(MockDockerBuilder::new_failing());

        let result = RealSshServerContainer::start_with_builder(mock_builder).await;

        assert!(matches!(
            result,
            Err(SshServerError::DockerBuildFailed { .. })
        ));
    }

    #[tokio::test]
    async fn it_should_return_error_when_docker_command_fails() {
        // Test with Docker not available
        // (requires mocking Command execution)
    }

    #[test]
    fn it_should_return_error_for_invalid_utf8_path() {
        // Test path validation
    }

    #[tokio::test]
    async fn it_should_provide_actionable_error_message() {
        let config = SshServerConfig::builder()
            .dockerfile_dir("/nonexistent/path")
            .build();

        let result = RealSshServerContainer::start_with_config(config).await;

        let err = result.unwrap_err();
        let message = err.to_string();

        // Verify error message is clear
        assert!(message.contains("Dockerfile not found"));
        assert!(message.contains("Tip:"));

        // Verify help is available
        let help = err.help();
        assert!(!help.is_empty());
        assert!(help.contains("Troubleshooting"));
    }
}
```

#### Rationale

- Ensures error handling code is actually tested
- Verifies error messages are correct and actionable
- Catches regressions in error handling
- Improves overall code reliability
- Follows project testing conventions

#### Benefits

- ✅ Error handling code is now tested
- ✅ Verifies error messages are helpful
- ✅ Catches regressions in error scenarios
- ✅ Improves confidence in error handling
- ✅ Documents expected error behaviors

#### Implementation Checklist

- [ ] Add test for `DockerfileNotFound` error
- [ ] Add test for `DockerBuildFailed` error
- [ ] Add test for `ContainerStartFailed` error
- [ ] Add test for `PortMappingFailed` error
- [ ] Add test for `DockerCommandFailed` error
- [ ] Add test for `InvalidUtf8InPath` error
- [ ] Verify error messages contain expected information
- [ ] Test that `.help()` methods return useful guidance
- [ ] Add tests for error source chaining
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- Use mock Docker builder to simulate failures
- Test each error variant is constructible
- Verify error messages match expectations
- Test error source preservation
- Ensure pattern matching works correctly

---

### Proposal #10: Implement Cleanup Methods

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposal #1 (trait)

#### Problem

Containers have no explicit cleanup or stop methods. Resource cleanup relies entirely on Drop implementation, which may not be sufficient in all scenarios.

#### Proposed Solution

Add explicit cleanup methods to the trait and implementations:

```rust
pub trait SshServerContainer {
    // ... existing methods

    /// Stop and remove the container
    fn stop(&mut self) -> Result<(), SshServerError>;

    /// Check if container is still running
    fn is_running(&self) -> bool;
}

impl SshServerContainer for RealSshServerContainer {
    fn stop(&mut self) -> Result<(), SshServerError> {
        // Stop and remove the container
        // testcontainers should handle this, but we can be explicit
        Ok(())
    }

    fn is_running(&self) -> bool {
        // Check container status
        true  // testcontainers manages lifecycle
    }
}

impl SshServerContainer for MockSshServerContainer {
    fn stop(&mut self) -> Result<(), SshServerError> {
        // No-op for mock
        Ok(())
    }

    fn is_running(&self) -> bool {
        true  // Mock always "running"
    }
}
```

#### Rationale

- Makes cleanup explicit and controllable
- Enables cleanup verification in tests
- Provides better control over resource lifecycle
- Documents cleanup behavior clearly

#### Benefits

- ✅ Explicit control over container lifecycle
- ✅ Better resource management in tests
- ✅ Clearer contract for cleanup behavior
- ✅ Enables cleanup verification

#### Implementation Checklist

- [ ] Add `stop()` method to `SshServerContainer` trait
- [ ] Add `is_running()` method to trait
- [ ] Implement methods for `RealSshServerContainer`
- [ ] Implement methods for `MockSshServerContainer`
- [ ] Add tests verifying cleanup behavior
- [ ] Update documentation
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- Test `stop()` can be called successfully
- Verify `is_running()` reflects container state
- Test cleanup in Drop implementation still works
- Verify no resource leaks in tests

---

## Phase 2: Enhanced Testing

Medium-impact improvements focused on test coverage and reliability.

**Note**: The original Proposal #10 (SSH Connectivity Health Check) has been removed as it's not needed for test-only code. The SSH server is adequately tested through its usage in integration tests.

---

This proposal was actually implemented in Proposal #0. The module structure reorganization happens there. This section can be considered complete once Proposal #0 is implemented.

---

## 📈 Timeline

- **Start Date**: To be determined
- **Estimated Duration**:
  - Phase 0: 1-2 days
  - Phase 1: 3-5 days
  - Phase 2: 2-3 days
  - **Total**: ~2 weeks for complete implementation

## 🔍 Review Process

### Approval Criteria

- [ ] All proposals reviewed by project maintainers
- [ ] Technical feasibility validated
- [ ] Aligns with [Development Principles](../development-principles.md)
- [ ] Aligns with [Error Handling Guide](../contributing/error-handling.md)
- [ ] Aligns with [Testing Conventions](../contributing/testing.md)
- [ ] Aligns with [Module Organization](../contributing/module-organization.md)
- [ ] Implementation plan is clear and actionable
- [ ] Impact/effort prioritization is reasonable

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing (unit, integration, E2E)
- [ ] All linters passing (clippy, rustfmt, etc.)
- [ ] Documentation updated with new APIs
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch
- [ ] Integration tests verify no regressions

## 📚 Related Documentation

- [Development Principles](../development-principles.md) - Core quality principles
- [Error Handling Guide](../contributing/error-handling.md) - Error handling best practices
- [Testing Conventions](../contributing/testing.md) - Testing standards and patterns
- [Module Organization](../contributing/module-organization.md) - Code organization guidelines

## 💡 Notes

### Implementation Strategy

1. **Start with Phase 0**: High-impact, low-effort changes provide immediate value
2. **Build incrementally**: Each proposal builds on previous ones
3. **Test continuously**: Run tests after each proposal implementation
4. **Review frequently**: Get feedback early and often

### Potential Challenges

- **Docker environment differences**: Tests may behave differently in CI vs local
- **Backward compatibility**: Ensure existing code using this module still works
- **Test reliability**: Docker-based tests can be flaky; health checks will help

### Future Considerations

- Consider adding support for other virtualization providers (Podman, Finch)
- Consider caching Docker image builds to speed up tests
- Consider parallel container management for faster test execution

---

**Created**: 2025-10-13  
**Last Updated**: 2025-10-13  
**Status**: 📋 Planning
