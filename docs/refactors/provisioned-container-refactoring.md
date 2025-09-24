# Provisioned Container Module Refactoring Plan

## 📋 Overview

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
    ├── docker_builder.rs      # Docker image building (Phase 1)
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

### 1. Extract Docker Image Builder

**Current Issue**: Docker image building logic is embedded in the container state machine.

**Proposed Solution**:

```rust
pub struct DockerImageBuilder {
    image_name: String,
    tag: String,
    dockerfile_path: PathBuf,
    context_path: PathBuf,
    build_timeout: Duration,
}

impl DockerImageBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn with_name(mut self, name: impl Into<String>) -> Self { /* ... */ }
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self { /* ... */ }
    pub fn with_dockerfile(mut self, path: PathBuf) -> Self { /* ... */ }
    pub fn build(&self) -> Result<()> { /* ... */ }
}
```

**Benefits**:

- Single Responsibility Principle
- Easier testing of build logic
- Configurable image parameters
- Reusable across different container types

### 2. Container Configuration Builder

**Current Issue**: Container configuration is hardcoded and not easily customizable.

**Proposed Solution**:

```rust
pub struct ContainerConfigBuilder {
    image: String,
    exposed_ports: Vec<ContainerPort>,
    environment: HashMap<String, String>,
    wait_conditions: Vec<WaitFor>,
    volumes: Vec<String>,
}

impl ContainerConfigBuilder {
    pub fn new(image: impl Into<String>) -> Self { /* ... */ }
    pub fn with_exposed_port(mut self, port: impl IntoContainerPort) -> Self { /* ... */ }
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self { /* ... */ }
    pub fn with_wait_condition(mut self, condition: WaitFor) -> Self { /* ... */ }
    pub fn build(self) -> GenericImage { /* ... */ }
}
```

**Benefits**:

- Explicit configuration
- Easy to test different configurations
- Flexibility for different use cases

### 3. Separate SSH Operations

**Current Issue**: SSH operations are tightly coupled with container lifecycle.

**Proposed Solution**:

```rust
pub trait ContainerSshManager {
    fn wait_for_ssh_ready(&self, timeout: Duration) -> Result<()>;
    fn setup_ssh_keys(&self, credentials: &SshCredentials) -> Result<()>;
    fn test_ssh_connection(&self) -> Result<()>;
}

pub struct DockerContainerSshManager {
    container: Arc<Container<GenericImage>>,
    ssh_port: u16,
    ssh_timeout: Duration,
}

impl ContainerSshManager for DockerContainerSshManager {
    // Implementation details...
}
```

**Benefits**:

- Decoupled SSH management
- Easier to test SSH operations
- Reusable for different container types
- Clear separation of concerns

## 🔧 Error Handling & Robustness

### 4. Improve Error Context

**Current Issue**: Errors lack sufficient context for debugging.

**Proposed Solution**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProvisionedContainerError {
    #[error("Docker build failed for image '{image_name}:{tag}' with stderr: {stderr}")]
    DockerBuildFailed {
        image_name: String,
        tag: String,
        stderr: String,
    },

    #[error("Container '{container_id}' failed to start: {source}")]
    ContainerStartFailed {
        container_id: Option<String>,
        #[source]
        source: testcontainers::TestcontainersError,
    },

    #[error("SSH setup timeout after {timeout_secs}s for container '{container_id}'")]
    SshSetupTimeout {
        container_id: String,
        timeout_secs: u64,
    },
}
```

**Benefits**:

- Better error messages with context
- Easier debugging and troubleshooting
- More actionable error information

### 5. Robust SSH Connectivity Testing

**Current Issue**: SSH readiness check is a simple sleep without actual connectivity testing.

**Proposed Solution**:

```rust
impl ContainerSshManager for DockerContainerSshManager {
    async fn wait_for_ssh_ready(&self, timeout: Duration) -> Result<()> {
        let start_time = Instant::now();
        let mut backoff = Duration::from_millis(100);

        while start_time.elapsed() < timeout {
            match self.test_ssh_connection().await {
                Ok(_) => {
                    info!("SSH connection successful after {:?}", start_time.elapsed());
                    return Ok(());
                }
                Err(_) => {
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(5));
                }
            }
        }

        Err(ProvisionedContainerError::SshSetupTimeout {
            container_id: self.container_id().to_string(),
            timeout_secs: timeout.as_secs(),
        })
    }
}
```

**Benefits**:

- Actual SSH connectivity verification
- Exponential backoff for efficiency
- Configurable timeouts
- Better failure detection

### 6. Add Timeout Configurations

**Current Issue**: Hardcoded timeouts make the system inflexible.

**Proposed Solution**:

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

## 📦 Configuration & Constants

### 7. Extract Magic Numbers and Strings

**Current Issue**: Hardcoded values scattered throughout the code.

**Proposed Solution**:

```rust
pub mod constants {
    use std::time::Duration;

    pub const DEFAULT_IMAGE_NAME: &str = "torrust-provisioned-instance";
    pub const DEFAULT_IMAGE_TAG: &str = "latest";
    pub const DEFAULT_SSH_PORT: u16 = 22;
    pub const DEFAULT_DOCKERFILE_PATH: &str = "docker/provisioned-instance/Dockerfile";
    pub const DEFAULT_SSH_READY_MESSAGE: &str = "sshd entered RUNNING state";
    pub const DEFAULT_SSH_WAIT_DURATION: Duration = Duration::from_secs(5);
    pub const MAX_SSH_SETUP_RETRIES: usize = 3;
}
```

**Benefits**:

- Centralized configuration
- Easy to modify behavior
- Self-documenting code
- Consistent values across the module

### 8. Container Configuration Options

**Current Issue**: Limited flexibility in container setup.

**Proposed Solution**:

```rust
#[derive(Debug, Clone)]
pub struct ContainerOptions {
    pub image_name: String,
    pub image_tag: String,
    pub dockerfile_path: PathBuf,
    pub ssh_port: u16,
    pub environment_vars: HashMap<String, String>,
    pub volumes: Vec<String>,
    pub timeouts: ContainerTimeouts,
}

impl Default for ContainerOptions {
    fn default() -> Self {
        Self {
            image_name: constants::DEFAULT_IMAGE_NAME.to_string(),
            image_tag: constants::DEFAULT_IMAGE_TAG.to_string(),
            dockerfile_path: PathBuf::from(constants::DEFAULT_DOCKERFILE_PATH),
            ssh_port: constants::DEFAULT_SSH_PORT,
            environment_vars: HashMap::new(),
            volumes: Vec::new(),
            timeouts: ContainerTimeouts::default(),
        }
    }
}
```

## 🧪 Testing & Observability

### 9. Improve Test Coverage

**Current Issue**: Limited test coverage, especially for integration scenarios.

**Proposed Solution**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        ContainerSshManager {}
        impl ContainerSshManager for ContainerSshManager {
            fn wait_for_ssh_ready(&self, timeout: Duration) -> Result<()>;
            fn setup_ssh_keys(&self, credentials: &SshCredentials) -> Result<()>;
            fn test_ssh_connection(&self) -> Result<()>;
        }
    }

    #[test]
    fn it_should_build_docker_image_with_custom_config() { /* ... */ }

    #[test]
    fn it_should_handle_ssh_setup_timeout() { /* ... */ }

    #[test]
    fn it_should_retry_ssh_connection_with_backoff() { /* ... */ }

    #[tokio::test]
    async fn it_should_wait_for_ssh_with_exponential_backoff() { /* ... */ }
}
```

**Benefits**:

- Better test coverage
- Mock-based testing for isolation
- Testing of error scenarios
- Async test support

### 10. Enhanced Logging

**Current Issue**: Limited logging context and structure.

**Proposed Solution**:

```rust
use tracing::{info, warn, error, debug, instrument, Span};

impl RunningProvisionedContainer {
    #[instrument(skip(self, ssh_credentials), fields(container_id = %self.container.id()))]
    pub async fn setup_ssh_keys(&self, ssh_credentials: &SshCredentials) -> Result<()> {
        let span = Span::current();
        span.record("ssh_user", &ssh_credentials.ssh_username);
        span.record("ssh_port", &self.ssh_port);

        info!("Starting SSH key authentication setup");

        // Implementation with structured logging...

        info!(
            setup_duration_ms = start_time.elapsed().as_millis(),
            "SSH key authentication configured successfully"
        );

        Ok(())
    }
}
```

**Benefits**:

- Structured logging with context
- Performance metrics
- Better debugging information
- Distributed tracing support

### 11. Add Health Checks

**Current Issue**: Limited container health verification.

**Proposed Solution**:

```rust
pub trait ContainerHealthChecker {
    async fn check_system_resources(&self) -> Result<SystemResources>;
    async fn check_required_services(&self) -> Result<Vec<ServiceStatus>>;
    async fn check_network_connectivity(&self) -> Result<NetworkStatus>;
}

#[derive(Debug)]
pub struct SystemResources {
    pub memory_usage_mb: u64,
    pub disk_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub is_running: bool,
}
```

**Benefits**:

- Comprehensive health monitoring
- Early detection of issues
- Better debugging information

## 🔄 API & Usability

### 12. Builder Pattern for Container Creation

**Current Issue**: Limited configurability during container creation.

**Proposed Solution**:

```rust
impl StoppedProvisionedContainer {
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }
}

pub struct ContainerBuilder {
    options: ContainerOptions,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self {
            options: ContainerOptions::default(),
        }
    }

    pub fn with_image(mut self, name: impl Into<String>) -> Self {
        self.options.image_name = name.into();
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.options.image_tag = tag.into();
        self
    }

    pub fn with_timeout(mut self, timeout_type: TimeoutType, duration: Duration) -> Self {
        match timeout_type {
            TimeoutType::DockerBuild => self.options.timeouts.docker_build = duration,
            TimeoutType::ContainerStart => self.options.timeouts.container_start = duration,
            TimeoutType::SshReady => self.options.timeouts.ssh_ready = duration,
            TimeoutType::SshSetup => self.options.timeouts.ssh_setup = duration,
        }
        self
    }

    pub fn build(self) -> StoppedProvisionedContainer {
        StoppedProvisionedContainer::with_options(self.options)
    }
}
```

**Usage Example**:

```rust
let container = StoppedProvisionedContainer::builder()
    .with_image("custom-provisioned-instance")
    .with_tag("v2.0")
    .with_timeout(TimeoutType::SshReady, Duration::from_secs(60))
    .build()
    .start()
    .await?;
```

## 📋 Implementation Priority

### ✅ Phase 0: Module Restructuring (Completed)

1. ✅ **Module Organization** - Moved `src/e2e/provisioned_container.rs` to `src/e2e/containers/` structure
2. ✅ **Backward Compatibility** - Added re-exports to maintain existing import paths
3. ✅ **Documentation Updates** - Updated all references to new module structure
4. ✅ **Test Validation** - Ensured all tests pass with new structure

### Phase 1: Foundation (High Priority)

1. Extract Docker Image Builder
2. Improve Error Context
3. Extract Magic Numbers and Strings
4. Split Large Functions

### Phase 2: Robustness (High Priority)

1. Robust SSH Connectivity Testing
2. Add Timeout Configurations
3. Add Retry Logic
4. Resource Management

### Phase 3: API Improvements (Medium Priority)

1. Builder Pattern for Container Creation
2. Container Configuration Builder
3. Separate SSH Operations
4. Improve Type Safety

### Phase 4: Observability (Medium Priority)

1. Enhanced Logging
2. Add Health Checks
3. Add Container Inspection Methods
4. Improve Test Coverage

### Phase 5: Advanced Features (Low Priority)

1. Async Support
2. Secure SSH Key Handling
3. Documentation Improvements
4. Advanced Monitoring

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
