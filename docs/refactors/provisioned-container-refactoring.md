# Provisioned Container Module Refactoring Plan

## � MAJOR UPDATE (September 2025)

**Status**: This document has been updated to reflect the current implementation state. Many features initially marked as "not implemented" have actually been completed.

### Key Findings from Code Review

**✅ COMPLETED FEATURES** (that were previously listed as not-implemented):

- **Add Timeout Configurations** - `ContainerTimeouts` struct fully implemented with configurable timeouts
- **Extract Magic Numbers and Strings** - Constants extracted and configurable parameters implemented
- **Enhanced Logging** - Structured logging with `tracing` implemented throughout all modules
- **Robust SSH Connectivity Testing** - `SshWaitAction` with exponential backoff and actual connectivity testing

**🎯 REMAINING HIGH-PRIORITY ITEMS**:

- **Improve Test Coverage** - More comprehensive unit and integration tests needed
- **Add Health Checks** - Container health monitoring beyond just SSH connectivity

**⚠️ REASSESSED ITEMS**:

- **Container Configuration Options** - Partially implemented; current modular approach may be preferable
- **Builder Pattern for Container Creation** - Questionable relevance; current modular builders are cleaner

---

## �📋 Overview

This document outlines a comprehensive refactoring plan for the provisioned container module (now located at `src/e2e/containers/provisioned.rs`) to improve maintainability, readability, testability, and reliability. The refactoring follows Rust best practices and the project's established patterns.

## ✅ Completed Changes

### Module Restructuring (Phase 0 - Completed)

**What was done**: Reorganized the module structure to better accommodate future container types:

- **Before**: `src/e2e/provisioned_container.rs` (single file)
- **After**: `src/e2e/containers/` (dedicated module directory)
  - `src/e2e/containers/mod.rs` - Module root with re-exports
  - `src/e2e/containers/provisioned.rs` - Provisioned container implementation

**Benefits achieved**:

- Better organization for future container types
- Cleaner separation of concerns
- Maintained backward compatibility through re-exports
- Prepared foundation for extracting collaborators

**Import paths updated**:

```rust
// Old import path (still works via re-export)
use torrust_tracker_deploy::e2e::provisioned_container::StoppedProvisionedContainer;

// New preferred import path
use torrust_tracker_deploy::e2e::containers::StoppedProvisionedContainer;
```

## 🎯 Goals

- Improve code maintainability and readability
- Enhance error handling and robustness
- Increase testability and test coverage
- Better separation of concerns
- Add configurability and flexibility
- Improve observability and debugging capabilities

## 📁 New Module Structure

The refactoring begins with a restructured module organization that will accommodate future container types and collaborators:

```text
src/e2e/containers/
├── mod.rs              # Module root with re-exports
├── provisioned.rs      # Current provisioned container implementation
└── [future modules]    # Space for additional container types and collaborators
    ├── image_builder.rs      # Container image building (Phase 1)
    ├── ssh_manager.rs          # SSH operations (Phase 1)
    ├── health_checker.rs       # Health checking (Phase 4)
    └── config_builder.rs       # Configuration management (Phase 3)
```

This structure enables:

- **Separation of concerns** - Each collaborator in its own module
- **Testability** - Individual components can be tested in isolation
- **Reusability** - Components can be shared across different container types
- **Backward compatibility** - Existing imports continue to work via re-exports

## 🏗️ Architecture & Design Patterns

### ✅ 1. Extract Docker Image Builder (Completed)

**Issue Resolved**: Docker image building logic was embedded in the container state machine.

**Implementation Completed**:

```rust
pub struct ContainerImageBuilder {
    image_name: Option<String>,        // Required field validation
    tag: String,                       // Default: "latest"
    dockerfile_path: Option<PathBuf>,  // Required field validation
    context_path: PathBuf,             // Default: "."
    build_timeout: Duration,           // Default: 300 seconds
}

impl ContainerImageBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn with_name(mut self, name: impl Into<String>) -> Self { /* ... */ }
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self { /* ... */ }
    pub fn with_dockerfile(mut self, path: PathBuf) -> Self { /* ... */ }
    pub fn with_context(mut self, path: PathBuf) -> Self { /* ... */ }
    pub fn with_build_timeout(mut self, timeout: Duration) -> Self { /* ... */ }
    pub fn build(&self) -> Result<()> { /* ... */ }
    pub fn image_tag(&self) -> String { /* ... */ }
}

// Comprehensive error handling
#[derive(Debug, thiserror::Error)]
pub enum DockerBuildError {
    #[error("Failed to execute docker build command for image '{image_name}:{tag}': {source}")]
    DockerBuildExecution { /* ... */ },
    #[error("Docker build failed for image '{image_name}:{tag}' with stderr: {stderr}")]
    DockerBuildFailed { /* ... */ },
    #[error("Image name is required but was not provided")]
    ImageNameRequired,
    #[error("Dockerfile path is required but was not provided")]
    DockerfilePathRequired,
}
```

**Benefits Achieved**:

- ✅ Single Responsibility Principle
- ✅ Explicit configuration with required field validation
- ✅ Comprehensive error handling with specific error types
- ✅ Builder pattern with method chaining
- ✅ Full test coverage (13 unit tests)
- ✅ Integration with existing provisioned container error chain
- ✅ Reusable across different container types
- ✅ Configurable image parameters with sensible defaults

**Module Location**: `src/e2e/containers/image_builder.rs`

### ✅ 2. Container Configuration Builder (Completed)

**Issue Resolved**: Container configuration was hardcoded and not easily customizable.

**Implementation Completed**:

```rust
pub struct ContainerConfigBuilder {
    image: String,                     // Docker image with tag
    exposed_ports: Vec<u16>,          // Ports to expose (simplified to u16)
    wait_conditions: Vec<WaitFor>,    // Wait conditions for readiness
}

impl ContainerConfigBuilder {
    pub fn new(image: impl Into<String>) -> Self { /* ... */ }
    pub fn with_exposed_port(mut self, port: u16) -> Self { /* ... */ }
    pub fn with_wait_condition(mut self, condition: WaitFor) -> Self { /* ... */ }
    pub fn build(self) -> GenericImage { /* ... */ }
}

// Usage in provisioned container
let image = ContainerConfigBuilder::new(format!("{}:{}", DEFAULT_IMAGE_NAME, DEFAULT_IMAGE_TAG))
    .with_exposed_port(22)
    .with_wait_condition(WaitFor::message_on_stdout("sshd entered RUNNING state"))
    .build();
```

**Benefits Achieved**:

- ✅ Explicit configuration with builder pattern
- ✅ Removed hardcoded container configuration
- ✅ Easy to test different configurations through builder
- ✅ Flexibility for provisioned container use case
- ✅ Full test coverage (9 unit tests)
- ✅ Integration with existing provisioned container module
- ✅ Backwards compatibility maintained
- ✅ Focused only on features actually needed by provisioned container

**Module Location**: `src/e2e/containers/config_builder.rs`

**Key Design Decision**: Simplified to only include features actually used by the provisioned container (image, ports, wait conditions) rather than implementing unused features (environment variables, volumes).

### ✅ 3. Separate SSH Operations - Container Actions Architecture (Completed)

**Issue Resolved**: SSH operations were tightly coupled with container lifecycle.

**Implementation Completed**: Implemented a trait-based container actions architecture that decouples container operations from the container state machine.

**Implementation Completed**:

```rust
// Trait for containers that can execute commands
pub trait ContainerExecutor {
    fn exec(&self, command: testcontainers::core::ExecCommand) -> Result<testcontainers::core::ExecOutput, testcontainers::TestcontainersError>;
}

// Container actions module structure - IMPLEMENTED
src/e2e/containers/actions/
├── mod.rs                    # Module root with re-exports ✅
├── ssh_key_setup.rs         # SSH key setup action ✅
└── ssh_wait.rs              # Wait for SSH connectivity action ✅
```

**Container Actions Design**:

```rust
// SSH Key Setup Action
pub struct SshKeySetupAction;

impl SshKeySetupAction {
    pub fn execute<T: ContainerExecutor>(
        &self,
        container: &T,
        ssh_credentials: &SshCredentials,
    ) -> Result<()> {
        // Implementation using container.exec()
    }
}

// SSH Wait Action (doesn't need container exec - uses external SSH connection)
pub struct SshWaitAction {
    pub timeout: Duration,
    pub max_attempts: usize,
}

impl SshWaitAction {
    pub fn execute(&self, host: &str, port: u16) -> Result<()> {
        // Implementation using actual SSH connection attempts in a loop
    }
}
```

**Benefits Achieved**:

- ✅ **Decoupled container actions**: Operations are separate from container lifecycle
- ✅ **Trait-based architecture**: Easy to test and mock for different container types
- ✅ **Reusable across container types**: Any container implementing `ContainerExecutor` can use these actions
- ✅ **Clear separation of concerns**: Container manages lifecycle, actions manage operations
- ✅ **Extensible**: Easy to add new container actions in the future
- ✅ **Better testability**: Actions can be tested independently of container state

## 🔧 Error Handling & Robustness

### ✅ 4. Improve Error Context (Completed)

**Issue Resolved**: Errors lacked sufficient context for debugging.

**Implementation Completed**: Enhanced error handling across all container modules with detailed context for better debugging experience:

```rust
// Enhanced ProvisionedContainerError with comprehensive context
#[derive(Debug, thiserror::Error)]
pub enum ProvisionedContainerError {
    #[error("Container start failed - Container ID: {container_id}, Image: {image_name}:{image_tag}, Start time: {start_time_ms}ms: {source}")]
    ContainerStartFailed {
        container_id: String,
        image_name: String,
        image_tag: String,
        start_time_ms: u64,
        #[source]
        source: Box<ContainerBuildError>,
    },

    #[error("SSH setup timeout after {timeout_ms}ms for container {container_id} (user: {ssh_user})")]
    SshSetupTimeout {
        container_id: String,
        timeout_ms: u64,
        ssh_user: String,
    },

    // Additional context-rich error variants...
}

// ContainerBuildError with build context
#[derive(Debug, thiserror::Error)]
pub enum ContainerBuildError {
    #[error("Container build failed - Dockerfile: {dockerfile_path}, Context: {context_path}, Build time: {build_duration_ms}ms: {source}")]
    ContainerBuildFailed {
        dockerfile_path: PathBuf,
        context_path: PathBuf,
        build_duration_ms: u64,
        #[source]
        source: testcontainers::TestcontainersError,
    },
}

// ContainerConfigError with validation context
#[derive(Debug, thiserror::Error)]
pub enum ContainerConfigError {
    #[error("Invalid port {port}: {reason}")]
    InvalidPort { port: u16, reason: String },

    #[error("Invalid image name '{image_name}': {reason}")]
    InvalidImageName { image_name: String, reason: String },
}

// SshKeySetupError with SSH user context
#[derive(Debug, thiserror::Error)]
pub enum SshKeySetupError {
    #[error("SSH key setup failed for user '{ssh_user}': Failed to create SSH directory: {source}")]
    SshDirectoryCreationFailed {
        ssh_user: String,
        #[source]
        source: testcontainers::TestcontainersError,
    },
}

// SshWaitError with connection details
#[derive(Debug, thiserror::Error)]
pub enum SshWaitError {
    #[error("SSH connection timeout to {host}:{port} after {timeout_ms}ms - Last error: {last_error_context}")]
    SshConnectionTimeout {
        host: String,
        port: u16,
        timeout_ms: u64,
        last_error_context: String,
    },
}
```

**Modules Enhanced**:

- ✅ `provisioned.rs`: Container ID, image details, timing information
- ✅ `image_builder.rs`: Dockerfile path, context path, build timing
- ✅ `config_builder.rs`: Comprehensive validation with specific error types
- ✅ `ssh_key_setup.rs`: SSH user context for key operations
- ✅ `ssh_wait.rs`: Host, port, and connection attempt details

**Benefits Achieved**:

- ✅ Better error messages with context
- ✅ Easier debugging and troubleshooting
- ✅ More actionable error information
- ✅ All error types now include specific debugging context
- ✅ Proper error chain preservation using `#[source]` attributes
- ✅ Comprehensive test coverage for all enhanced error types

### ✅ 5. Robust SSH Connectivity Testing (Completed)

**Status**: ✅ **COMPLETED**

**Issue Resolved**: SSH readiness check now implemented with actual connectivity testing instead of simple sleep.

**Implementation Completed**:

```rust
pub struct SshWaitAction {
    pub timeout: Duration,
    pub max_attempts: usize,
}

impl SshWaitAction {
    pub fn new(timeout: Duration, max_attempts: usize) -> Self {
        Self { timeout, max_attempts }
    }

    pub fn execute(&self, host: &str, port: u16) -> Result<()> {
        let start_time = Instant::now();
        let mut attempt = 0;
        let mut backoff = Duration::from_millis(100);
        let mut last_error_context = "No connection attempts made".to_string();

        while start_time.elapsed() < self.timeout && attempt < self.max_attempts {
            attempt += 1;

            match test_ssh_connection(host, port) {
                Ok(_) => {
                    info!(
                        host = host,
                        port = port,
                        attempts = attempt,
                        duration_ms = start_time.elapsed().as_millis(),
                        "SSH connectivity established"
                    );
                    return Ok(());
                }
                Err(e) => {
                    last_error_context = e.to_string();
                    std::thread::sleep(backoff);
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(5));
                }
            }
        }

        Err(SshWaitError::SshConnectionTimeout {
            host: host.to_string(),
            port,
            timeout_secs: self.timeout.as_secs(),
            max_attempts: self.max_attempts,
            last_error_context,
        })
    }
}
```

**Benefits Achieved**:

- ✅ **Actual SSH connectivity verification** - No more sleep-based waiting
- ✅ **Exponential backoff for efficiency** - Prevents aggressive polling
- ✅ **Configurable timeouts and retry attempts** - Flexible for different environments
- ✅ **Better failure detection** - Detailed error context and timing info
- ✅ **Proper error handling** - Comprehensive error information for debugging

### ✅ 6. Add Timeout Configurations (Completed)

**Status**: ✅ **COMPLETED**

**Issue Resolved**: Hardcoded timeouts made the system inflexible.

**Implementation Completed**:

```rust
#[derive(Debug, Clone)]
pub struct ContainerTimeouts {
    pub docker_build: Duration,
    pub container_start: Duration,
    pub ssh_ready: Duration,
    pub ssh_setup: Duration,
}

impl Default for ContainerTimeouts {
    fn default() -> Self {
        Self {
            docker_build: Duration::from_secs(300),    // 5 minutes
            container_start: Duration::from_secs(60),   // 1 minute
            ssh_ready: Duration::from_secs(30),         // 30 seconds
            ssh_setup: Duration::from_secs(15),         // 15 seconds
        }
    }
}
```

**Benefits Achieved**:

- ✅ Configurable timeouts for all container operations
- ✅ Sensible defaults with ability to customize
- ✅ Used throughout provisioned container implementation
- ✅ Constructor methods like `with_timeouts()` available

## 📦 Configuration & Constants

### ✅ 7. Extract Magic Numbers and Strings (Completed)

**Status**: ✅ **COMPLETED**

**Issue Resolved**: Hardcoded values scattered throughout the code.

**Implementation Completed**:

```rust
// In src/e2e/containers/provisioned.rs
const DEFAULT_IMAGE_NAME: &str = "torrust-provisioned-instance";
const DEFAULT_IMAGE_TAG: &str = "latest";

// Timeout configurations in ContainerTimeouts struct
pub struct ContainerTimeouts {
    pub docker_build: Duration,     // Default: 300 seconds
    pub container_start: Duration,  // Default: 60 seconds
    pub ssh_ready: Duration,        // Default: 30 seconds
    pub ssh_setup: Duration,        // Default: 15 seconds
}

// In various modules - configurable parameters with defaults
impl ContainerImageBuilder {
    fn default() -> Self {
        Self {
            build_timeout: Duration::from_secs(300), // Default timeout
            // Other configurable parameters...
        }
    }
}
```

**Benefits Achieved**:

- ✅ Centralized configuration constants
- ✅ Configurable timeouts replace hardcoded values
- ✅ Easy to modify behavior across modules
- ✅ Self-documenting code with meaningful constant names
- ✅ Consistent values across the entire container system

### 8. Container Configuration Options - ⚠️ PARTIALLY IMPLEMENTED

**Status**: ⚠️ **PARTIALLY IMPLEMENTED** - Timeouts are configurable, but other container options could benefit from consolidation

**Current State**: The system already has configurable timeouts via `ContainerTimeouts` and modular builders for different aspects:

- `ContainerImageBuilder` - Handles image name, tag, dockerfile path, context
- `ContainerConfigBuilder` - Handles container configuration (ports, wait conditions)
- `ContainerTimeouts` - Handles all timeout configurations

**Assessment**: The current modular approach provides good separation of concerns. A unified `ContainerOptions` struct may introduce unnecessary coupling.

**Recommendation**: **CONSIDER SKIPPING** - The current modular approach with separate builders for different concerns is cleaner than a monolithic configuration struct. Only implement if there's a clear need to pass around a unified configuration object.

## 🧪 Testing & Observability

### 9. Improve Test Coverage - 🎯 STILL RELEVANT

**Status**: 🎯 **STILL RELEVANT** - Test coverage improvements are always beneficial

**Current State**: The container modules have basic unit tests, but could benefit from:

- More comprehensive integration test scenarios
- Mock-based testing for better isolation
- Error scenario testing with proper mocking
- Async test patterns for SSH operations

**Updated Proposed Solution**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Integration tests with actual containers (existing approach)
    #[tokio::test]
    async fn it_should_start_and_configure_provisioned_container() {
        let container = StoppedProvisionedContainer::new()
            .start()
            .await
            .expect("Container should start");

        // Test actual SSH connectivity and operations
        let ssh_credentials = SshCredentials::new("root", &test_ssh_key());
        container.setup_ssh_keys(&ssh_credentials).await.expect("SSH setup should work");
    }

    // Unit tests for individual components (using current modular architecture)
    #[test]
    fn it_should_validate_container_configuration() {
        let config = ContainerConfigBuilder::new("test-image:latest")
            .with_exposed_port(22)
            .with_exposed_port(8080);

        assert!(config.build().exposed_ports().contains(&22));
    }

    // Error scenario tests
    #[test]
    fn it_should_handle_ssh_timeout_gracefully() {
        let ssh_wait = SshWaitAction::new(Duration::from_millis(1), 1);
        let result = ssh_wait.execute("unreachable-host", 22);

        assert!(matches!(result, Err(SshWaitError::SshConnectionTimeout { .. })));
    }
}
```

**Benefits**:

- ✅ **Builds on existing architecture** - Works with current modular design
- ✅ **Realistic integration tests** - Uses actual containers for end-to-end validation
- ✅ **Focused unit tests** - Tests individual components in isolation
- ✅ **Error scenario coverage** - Ensures proper error handling

### ✅ 10. Enhanced Logging (Completed)

**Status**: ✅ **COMPLETED**

**Issue Resolved**: Limited logging context and structure.

**Implementation Completed**:

```rust
use tracing::{info, warn, error, debug};

impl RunningProvisionedContainer {
    pub async fn start() -> Result<Self> {
        info!("Starting provisioned instance container");

        // Implementation with structured logging...

        info!(
            container_id = %container.id(),
            ssh_port = ssh_port,
            startup_duration_ms = start_time.elapsed().as_millis(),
            "Provisioned container started successfully"
        );

        Ok(container)
    }

    pub fn stop(self) -> Result<()> {
        info!(container_id = %self.container.id(), "Stopping container");
        // Implementation...
    }
}

impl SshWaitAction {
    pub fn execute(&self, host: &str, port: u16) -> Result<()> {
        info!(
            host = host,
            port = port,
            timeout_secs = self.timeout.as_secs(),
            max_attempts = self.max_attempts,
            "Starting SSH connectivity check"
        );
        // Implementation...
    }
}
```

**Benefits Achieved**:

- ✅ Structured logging with `tracing` throughout all modules
- ✅ Contextual information (container IDs, ports, timing)
- ✅ Performance metrics and timing information
- ✅ Better debugging information for troubleshooting
- ✅ Consistent logging patterns across all container modules

### 11. Add Health Checks - 🎯 STILL RELEVANT

**Status**: 🎯 **STILL RELEVANT** - Would provide valuable container monitoring capabilities

**Current State**: SSH connectivity is verified through `SshWaitAction`, but there's no comprehensive health checking beyond SSH availability.

**Updated Proposed Solution**:

```rust
pub trait ContainerHealthChecker {
    async fn check_ssh_connectivity(&self) -> Result<SshHealthStatus>;
    async fn check_system_resources(&self) -> Result<SystemResources>;
    async fn check_required_services(&self) -> Result<Vec<ServiceStatus>>;
    async fn comprehensive_health_check(&self) -> Result<ContainerHealthReport>;
}

#[derive(Debug)]
pub struct ContainerHealthReport {
    pub ssh_status: SshHealthStatus,
    pub system_resources: SystemResources,
    pub services: Vec<ServiceStatus>,
    pub overall_status: HealthStatus,
}

#[derive(Debug)]
pub enum HealthStatus {
    Healthy,
    Warning { issues: Vec<String> },
    Unhealthy { critical_issues: Vec<String> },
}

impl ContainerHealthChecker for RunningProvisionedContainer {
    async fn comprehensive_health_check(&self) -> Result<ContainerHealthReport> {
        let ssh_status = self.check_ssh_connectivity().await?;
        let system_resources = self.check_system_resources().await?;
        let services = self.check_required_services().await?;

        let overall_status = evaluate_overall_health(&ssh_status, &system_resources, &services);

        Ok(ContainerHealthReport {
            ssh_status,
            system_resources,
            services,
            overall_status,
        })
    }
}
```

**Benefits**:

- ✅ **Comprehensive monitoring** - Beyond just SSH connectivity
- ✅ **Early issue detection** - Identify problems before they cause failures
- ✅ **Debugging support** - Detailed health information for troubleshooting
- ✅ **Extensible design** - Easy to add new health checks as needed

## 🔄 API & Usability

### 12. Builder Pattern for Container Creation - ❓ QUESTIONABLE RELEVANCE

**Status**: ❓ **QUESTIONABLE RELEVANCE** - Current modular approach may be preferable

**Current State**: The provisioned container supports some configuration through methods like `with_timeouts()`, and uses separate specialized builders:

- `ContainerImageBuilder` - Focused on image building concerns
- `ContainerConfigBuilder` - Focused on container configuration
- `ContainerTimeouts` - Focused on timeout configuration

**Assessment**: The current modular approach provides better separation of concerns than a monolithic builder pattern.

**Alternative Approach** (if builder pattern is still desired):

```rust
// Instead of a monolithic builder, enhance the existing configuration methods
impl StoppedProvisionedContainer {
    pub fn with_custom_image(image_name: &str, tag: &str) -> Self {
        // Configure with custom image details
    }

    pub fn with_extended_timeouts() -> Self {
        // Pre-configured for longer operations
    }

    pub fn with_debug_config() -> Self {
        // Pre-configured for debugging scenarios
    }
}

// Or use a lightweight configuration struct for common scenarios
#[derive(Debug)]
pub struct ContainerPreset {
    pub timeouts: ContainerTimeouts,
    pub image_config: (String, String), // name, tag
}

impl ContainerPreset {
    pub fn development() -> Self { /* fast timeouts, latest images */ }
    pub fn ci_testing() -> Self { /* moderate timeouts, stable images */ }
    pub fn debugging() -> Self { /* long timeouts, debug images */ }
}

impl StoppedProvisionedContainer {
    pub fn with_preset(preset: ContainerPreset) -> Self { /* ... */ }
}
```

**Recommendation**: **CONSIDER ALTERNATIVE** - Focus on presets and configuration methods rather than a complex builder pattern, which fits better with the current architecture.

## 📋 Implementation Priority

### ✅ Phase 0: Module Restructuring (Completed)

1. ✅ **Module Organization** - Moved `src/e2e/provisioned_container.rs` to `src/e2e/containers/` structure
2. ✅ **Backward Compatibility** - Added re-exports to maintain existing import paths
3. ✅ **Documentation Updates** - Updated all references to new module structure
4. ✅ **Test Validation** - Ensured all tests pass with new structure

## 📋 Updated Implementation Priority

### ✅ Phase 0: Module Restructuring (COMPLETED)

1. ✅ **Module Organization** - Moved `src/e2e/provisioned_container.rs` to `src/e2e/containers/` structure
2. ✅ **Backward Compatibility** - Added re-exports to maintain existing import paths
3. ✅ **Documentation Updates** - Updated all references to new module structure
4. ✅ **Test Validation** - Ensured all tests pass with new structure

### ✅ Phase 1: Foundation (COMPLETED)

1. ✅ **Extract Container Image Builder** - Independent `ContainerImageBuilder` with comprehensive features
2. ✅ **Extract Container Configuration Builder** - Flexible `ContainerConfigBuilder` for container setup
3. ✅ **Separate SSH Operations** - Container actions architecture with `SshKeySetupAction` and `SshWaitAction`
4. ✅ **Improve Error Context** - Enhanced error handling with detailed context across all modules
5. ✅ **Add Timeout Configurations** - Configurable `ContainerTimeouts` with sensible defaults
6. ✅ **Extract Magic Numbers and Strings** - Constants and configurable parameters
7. ✅ **Enhanced Logging** - Structured logging with `tracing` throughout all modules
8. ✅ **Robust SSH Connectivity Testing** - `SshWaitAction` with exponential backoff

### 🎯 Phase 2: Remaining Improvements (HIGH PRIORITY)

1. **Improve Test Coverage** - Comprehensive unit and integration tests
2. **Add Health Checks** - Container health monitoring beyond SSH connectivity

### ⚠️ Phase 3: Optional Enhancements (LOW PRIORITY)

1. **Container Configuration Consolidation** - Unified configuration approach (if needed)
2. **Alternative Builder Pattern** - Configuration presets instead of monolithic builder

## 🧪 Testing Strategy

Each refactoring phase should include:

- Unit tests for new components
- Integration tests for end-to-end workflows
- Regression tests to ensure existing functionality
- Performance tests for async operations
- Error scenario testing

## 📊 Success Metrics

- **Maintainability**: Reduced cyclomatic complexity, improved code organization
- **Testability**: Increased test coverage from ~30% to >90%
- **Reliability**: Reduced failure rate through retry logic and health checks
- **Performance**: Improved startup times through async operations
- **Observability**: Better logging and monitoring capabilities
- **Security**: Proper SSH key validation and secure cleanup

This refactoring plan provides a structured approach to improving the provisioned container module while maintaining backward compatibility and following the project's established patterns.
