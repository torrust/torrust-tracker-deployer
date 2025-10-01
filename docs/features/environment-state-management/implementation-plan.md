# Implementation Plan

> **📋 Roadmap**  
> Detailed implementation plan for Environment State Management feature.

## 🏗️ Implementation Phases

### Phase 1: Foundation

#### 1. Domain Model Enhancement

- [ ] Create state marker types (`Created`, `Provisioning`, `Provisioned`, etc.)
- [ ] Implement `Environment<S>` with type-state pattern
- [ ] Add state-specific transition methods with compile-time validation
- [ ] Create basic type-safe state transitions

### Phase 2: Serialization & Type Erasure

#### 2. Serialization Implementation

- [ ] Create `AnyEnvironmentState` enum for type erasure
- [ ] Implement conversion methods between typed and erased states
- [ ] Add serialization/deserialization for all state types
- [ ] Implement helper methods for state introspection and display

### Phase 3: Persistence

#### 3. Repository Implementation

- [ ] Create `StateRepository` trait working with `AnyEnvironmentState`
- [ ] Implement JSON file-based repository with type erasure support
- [ ] Add atomic write operations (temp file + rename)
- [ ] Add error handling for storage operations

### Phase 4: Command Integration

#### 4. Command Enhancement

- [ ] Update command handlers to accept and return specific state types
- [ ] Implement type-safe state transitions in command execution
- [ ] Add orchestration layer for chaining commands with type safety
- [ ] Add error state handling with compile-time guarantees
- [ ] Ensure commands can only be called on valid state types

### Phase 4: Testing & Documentation

#### 4. Unit Tests

- [ ] Test state machine transitions with compile-time validation
- [ ] Test repository operations with type erasure
- [ ] Test command integration with type-safe state transitions
- [ ] Test error scenarios and invalid state access attempts
- [ ] Test serialization/deserialization of all state types

#### 5. E2E Tests

- [ ] Test state persistence across command executions
- [ ] Test error state handling
- [ ] Test state recovery after interruptions

#### 6. Documentation

- [ ] Update deployment overview with state management
- [ ] Add state management to architecture documentation
- [ ] Create troubleshooting guide for state issues

- [ ] Update deployment overview with state management
- [ ] Add state management to architecture documentation
- [ ] Create troubleshooting guide for state issues

## 🔧 Technical Implementation Details

### State Machine Implementation (Type-State Pattern)

```rust
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// State marker types - each state is a distinct type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Created;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provisioning;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provisioned;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuring;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configured;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Releasing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Released;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Running;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionFailed {
    pub failed_step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureFailed {
    pub failed_step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseFailed {
    pub failed_step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunFailed {
    pub failed_step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destroyed;

// Environment with type-state pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment<S> {
    name: EnvironmentName,
    ssh_credentials: SshCredentials,
    data_dir: PathBuf,
    build_dir: PathBuf,
    state: S,
}

// Implementation for all states
impl<S> Environment<S> {
    pub fn name(&self) -> &EnvironmentName {
        &self.name
    }

    pub fn ssh_credentials(&self) -> &SshCredentials {
        &self.ssh_credentials
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn build_dir(&self) -> &Path {
        &self.build_dir
    }

    pub fn state(&self) -> &S {
        &self.state
    }
}

// State-specific implementations with compile-time enforced transitions
impl Environment<Created> {
    pub fn new(name: EnvironmentName, ssh_credentials: SshCredentials) -> Self {
        let data_dir = PathBuf::from("./data").join(name.as_str());
        let build_dir = PathBuf::from("./build").join(name.as_str());

        Self {
            name,
            ssh_credentials,
            data_dir,
            build_dir,
            state: Created,
        }
    }

    pub fn start_provisioning(self) -> Environment<Provisioning> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Provisioning,
        }
    }
}

impl Environment<Provisioning> {
    pub fn provisioned(self) -> Environment<Provisioned> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Provisioned,
        }
    }

    pub fn provision_failed(self, failed_step: String) -> Environment<ProvisionFailed> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: ProvisionFailed { failed_step },
        }
    }
}

impl Environment<Provisioned> {
    pub fn start_configuring(self) -> Environment<Configuring> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Configuring,
        }
    }
}

impl Environment<Configuring> {
    pub fn configured(self) -> Environment<Configured> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Configured,
        }
    }

    pub fn configure_failed(self, failed_step: String) -> Environment<ConfigureFailed> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: ConfigureFailed { failed_step },
        }
    }
}

impl Environment<Configured> {
    pub fn start_releasing(self) -> Environment<Releasing> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Releasing,
        }
    }
}

impl Environment<Releasing> {
    pub fn released(self) -> Environment<Released> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Released,
        }
    }

    pub fn release_failed(self, failed_step: String) -> Environment<ReleaseFailed> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: ReleaseFailed { failed_step },
        }
    }
}

impl Environment<Released> {
    pub fn start_running(self) -> Environment<Running> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Running,
        }
    }
}

impl Environment<Running> {
    pub fn run_failed(self, failed_step: String) -> Environment<RunFailed> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: RunFailed { failed_step },
        }
    }
}

// Common transitions for any state (like destroy)
impl<S> Environment<S> {
    pub fn destroy(self) -> Environment<Destroyed> {
        Environment {
            name: self.name,
            ssh_credentials: self.ssh_credentials,
            data_dir: self.data_dir,
            build_dir: self.build_dir,
            state: Destroyed,
        }
    }
}

// Type erasure for storage - we need this for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyEnvironmentState {
    Created(Environment<Created>),
    Provisioning(Environment<Provisioning>),
    Provisioned(Environment<Provisioned>),
    Configuring(Environment<Configuring>),
    Configured(Environment<Configured>),
    Releasing(Environment<Releasing>),
    Released(Environment<Released>),
    Running(Environment<Running>),
    ProvisionFailed(Environment<ProvisionFailed>),
    ConfigureFailed(Environment<ConfigureFailed>),
    ReleaseFailed(Environment<ReleaseFailed>),
    RunFailed(Environment<RunFailed>),
    Destroyed(Environment<Destroyed>),
}

// Type erasure implementation - one for each state type
impl Environment<Created> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Created(self)
    }
}

impl Environment<Provisioning> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Provisioning(self)
    }
}

impl Environment<Provisioned> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Provisioned(self)
    }
}

impl Environment<Configuring> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Configuring(self)
    }
}

impl Environment<Configured> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Configured(self)
    }
}

impl Environment<Releasing> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Releasing(self)
    }
}

impl Environment<Released> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Released(self)
    }
}

impl Environment<Running> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Running(self)
    }
}

impl Environment<ProvisionFailed> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ProvisionFailed(self)
    }
}

impl Environment<ConfigureFailed> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ConfigureFailed(self)
    }
}

impl Environment<ReleaseFailed> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ReleaseFailed(self)
    }
}

impl Environment<RunFailed> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::RunFailed(self)
    }
}

impl Environment<Destroyed> {
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Destroyed(self)
    }
}
```

### Repository Implementation (Updated for Type-State Pattern)

```rust
use std::path::{Path, PathBuf};
use std::fs;
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateRepositoryError {
    #[error("Failed to read state file: {path}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write state file: {path}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid JSON in state file: {path}")]
    SerializationError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

pub trait StateRepository {
    fn save(&self, env: &AnyEnvironmentState) -> Result<(), StateRepositoryError>;
    fn load(&self, env_name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>, StateRepositoryError>;
    fn exists(&self, env_name: &EnvironmentName) -> Result<bool, StateRepositoryError>;
}

pub struct JsonStateRepository {
    base_path: PathBuf,
}

impl JsonStateRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn state_file_path(&self, env_name: &EnvironmentName) -> PathBuf {
        self.base_path
            .join("data")
            .join(env_name.as_str())
            .join("state.json")
    }
}

impl StateRepository for JsonStateRepository {
    fn save(&self, env: &AnyEnvironmentState) -> Result<(), StateRepositoryError> {
        let state_path = self.state_file_path(env.name());

        // Ensure parent directory exists
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent).map_err(|e| StateRepositoryError::WriteError {
                path: state_path.clone(),
                source: e,
            })?;
        }

        // Use atomic write (temp file + rename)
        let temp_file = NamedTempFile::new_in(state_path.parent().unwrap())
            .map_err(|e| StateRepositoryError::WriteError {
                path: state_path.clone(),
                source: e,
            })?;

        serde_json::to_writer_pretty(&temp_file, env)
            .map_err(|e| StateRepositoryError::SerializationError {
                path: state_path.clone(),
                source: e,
            })?;

        temp_file.persist(&state_path)
            .map_err(|e| StateRepositoryError::WriteError {
                path: state_path,
                source: e.error,
            })?;

        Ok(())
    }

    fn load(&self, env_name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>, StateRepositoryError> {
        let state_path = self.state_file_path(env_name);

        if !state_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&state_path)
            .map_err(|e| StateRepositoryError::ReadError {
                path: state_path.clone(),
                source: e,
            })?;

        let environment = serde_json::from_str(&content)
            .map_err(|e| StateRepositoryError::SerializationError {
                path: state_path,
                source: e,
            })?;

        Ok(Some(environment))
    }

    fn exists(&self, env_name: &EnvironmentName) -> Result<bool, StateRepositoryError> {
        Ok(self.state_file_path(env_name).exists())
    }
}

// File locking mechanism to prevent concurrent access
use std::process;
use std::time::{Duration, SystemTime};

#[derive(Debug, Error)]
pub enum LockError {
    #[error("Failed to acquire lock for environment '{name}': lock held by process {pid}")]
    LockHeld {
        name: String,
        pid: u32,
    },

    #[error("Failed to create lock file: {path}")]
    LockCreationError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read lock file: {path}")]
    LockReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct LockFile {
    pid: u32,
    timestamp: SystemTime,
    environment_name: String,
}

pub struct StateLock {
    lock_path: PathBuf,
    _lock_file: Option<std::fs::File>,
}

impl StateLock {
    pub fn acquire(env_name: &EnvironmentName, base_path: &Path) -> Result<Self, LockError> {
        let lock_path = base_path
            .join("data")
            .join(env_name.as_str())
            .join("state.lock");

        // Check if lock file exists and is still valid
        if lock_path.exists() {
            let lock_content = fs::read_to_string(&lock_path)
                .map_err(|e| LockError::LockReadError {
                    path: lock_path.clone(),
                    source: e,
                })?;

            if let Ok(lock_info) = serde_json::from_str::<LockFile>(&lock_content) {
                // Check if the process still exists (platform-specific check would be better)
                // For now, just check if lock is recent (less than 5 minutes old)
                if let Ok(elapsed) = SystemTime::now().duration_since(lock_info.timestamp) {
                    if elapsed < Duration::from_secs(300) {
                        return Err(LockError::LockHeld {
                            name: env_name.as_str().to_string(),
                            pid: lock_info.pid,
                        });
                    }
                }
            }
        }

        // Create lock file
        let lock_info = LockFile {
            pid: process::id(),
            timestamp: SystemTime::now(),
            environment_name: env_name.as_str().to_string(),
        };

        // Ensure parent directory exists
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).map_err(|e| LockError::LockCreationError {
                path: lock_path.clone(),
                source: e,
            })?;
        }

        fs::write(&lock_path, serde_json::to_string_pretty(&lock_info).unwrap())
            .map_err(|e| LockError::LockCreationError {
                path: lock_path.clone(),
                source: e,
            })?;

        tracing::info!(
            "Acquired state lock for environment '{}' (PID: {})",
            env_name.as_str(),
            lock_info.pid
        );

        Ok(StateLock {
            lock_path: lock_path.clone(),
            _lock_file: None,
        })
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        // Clean up lock file
        if self.lock_path.exists() {
            let _ = fs::remove_file(&self.lock_path);
            tracing::info!("Released state lock: {}", self.lock_path.display());
        }
    }
}

// Updated repository with locking
pub struct LockedJsonStateRepository {
    base_path: PathBuf,
}

impl LockedJsonStateRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn state_file_path(&self, env_name: &EnvironmentName) -> PathBuf {
        self.base_path
            .join("data")
            .join(env_name.as_str())
            .join("state.json")
    }
}

impl StateRepository for LockedJsonStateRepository {
    fn save(&self, env: &AnyEnvironmentState) -> Result<(), StateRepositoryError> {
        let _lock = StateLock::acquire(env.name(), &self.base_path)
            .map_err(|e| StateRepositoryError::WriteError {
                path: self.state_file_path(env.name()),
                source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;

        let state_path = self.state_file_path(env.name());

        // Ensure parent directory exists
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent).map_err(|e| StateRepositoryError::WriteError {
                path: state_path.clone(),
                source: e,
            })?;
        }

        // Use atomic write (temp file + rename)
        let temp_file = NamedTempFile::new_in(state_path.parent().unwrap())
            .map_err(|e| StateRepositoryError::WriteError {
                path: state_path.clone(),
                source: e,
            })?;

        serde_json::to_writer_pretty(&temp_file, env)
            .map_err(|e| StateRepositoryError::SerializationError {
                path: state_path.clone(),
                source: e,
            })?;

        temp_file.persist(&state_path)
            .map_err(|e| StateRepositoryError::WriteError {
                path: state_path,
                source: e.error,
            })?;

        tracing::info!(
            "Saved state for environment '{}': {}",
            env.name().as_str(),
            env.state_name()
        );

        Ok(())
    }

    fn load(&self, env_name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>, StateRepositoryError> {
        let _lock = StateLock::acquire(env_name, &self.base_path)
            .map_err(|e| StateRepositoryError::ReadError {
                path: self.state_file_path(env_name),
                source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;

        let state_path = self.state_file_path(env_name);

        if !state_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&state_path)
            .map_err(|e| StateRepositoryError::ReadError {
                path: state_path.clone(),
                source: e,
            })?;

        let environment = serde_json::from_str(&content)
            .map_err(|e| StateRepositoryError::SerializationError {
                path: state_path,
                source: e,
            })?;

        tracing::info!(
            "Loaded state for environment '{}': {}",
            env_name.as_str(),
            environment.state_name()
        );

        Ok(Some(environment))
    }

    fn exists(&self, env_name: &EnvironmentName) -> Result<bool, StateRepositoryError> {
        Ok(self.state_file_path(env_name).exists())
    }
}

// Helper methods for AnyEnvironmentState
impl AnyEnvironmentState {
    pub fn name(&self) -> &EnvironmentName {
        match self {
            AnyEnvironmentState::Created(env) => env.name(),
            AnyEnvironmentState::Provisioning(env) => env.name(),
            AnyEnvironmentState::Provisioned(env) => env.name(),
            AnyEnvironmentState::Configuring(env) => env.name(),
            AnyEnvironmentState::Configured(env) => env.name(),
            AnyEnvironmentState::Releasing(env) => env.name(),
            AnyEnvironmentState::Released(env) => env.name(),
            AnyEnvironmentState::Running(env) => env.name(),
            AnyEnvironmentState::ProvisionFailed(env) => env.name(),
            AnyEnvironmentState::ConfigureFailed(env) => env.name(),
            AnyEnvironmentState::ReleaseFailed(env) => env.name(),
            AnyEnvironmentState::RunFailed(env) => env.name(),
            AnyEnvironmentState::Destroyed(env) => env.name(),
        }
    }

    pub fn state_name(&self) -> &'static str {
        match self {
            AnyEnvironmentState::Created(_) => "created",
            AnyEnvironmentState::Provisioning(_) => "provisioning",
            AnyEnvironmentState::Provisioned(_) => "provisioned",
            AnyEnvironmentState::Configuring(_) => "configuring",
            AnyEnvironmentState::Configured(_) => "configured",
            AnyEnvironmentState::Releasing(_) => "releasing",
            AnyEnvironmentState::Released(_) => "released",
            AnyEnvironmentState::Running(_) => "running",
            AnyEnvironmentState::ProvisionFailed(_) => "provision_failed",
            AnyEnvironmentState::ConfigureFailed(_) => "configure_failed",
            AnyEnvironmentState::ReleaseFailed(_) => "release_failed",
            AnyEnvironmentState::RunFailed(_) => "run_failed",
            AnyEnvironmentState::Destroyed(_) => "destroyed",
        }
    }
}
```

### Command Integration Pattern (Type-State Pattern)

```rust
// Example: Provision Command Integration with Type-State Pattern
impl ProvisionCommand {
    pub async fn execute(&self, environment: Environment<Created>, repository: &dyn StateRepository) -> Result<Environment<Provisioned>, ProvisionError> {
        // Transition to intermediate state - compile-time enforced!
        let provisioning_env = environment.start_provisioning();

        // Log state transition at info level with timestamp
        tracing::info!(
            "Environment '{}' transitioning: created -> provisioning",
            provisioning_env.name().as_str()
        );

        // Save intermediate state
        repository.save(&provisioning_env.clone().into_any())
            .map_err(|e| ProvisionError::StatePersistence { source: e })?;

        // Execute provision steps
        match self.execute_steps(&provisioning_env).await {
            Ok(_) => {
                // Success: transition to final state - compile-time enforced!
                let provisioned_env = provisioning_env.provisioned();

                tracing::info!(
                    "Environment '{}' transitioning: provisioning -> provisioned",
                    provisioned_env.name().as_str()
                );

                repository.save(&provisioned_env.clone().into_any())
                    .map_err(|e| ProvisionError::StatePersistence { source: e })?;
                Ok(provisioned_env)
            },
            Err(step_error) => {
                // Failure: transition to error state - compile-time enforced!
                let failed_step = step_error.step_name();
                let failed_env = provisioning_env.provision_failed(failed_step.clone());

                tracing::error!(
                    "Environment '{}' transitioning: provisioning -> provision_failed (step: {})",
                    failed_env.name().as_str(),
                    failed_step
                );

                repository.save(&failed_env.into_any())
                    .map_err(|e| ProvisionError::StatePersistence { source: e })?;
                Err(ProvisionError::StepExecution { source: step_error })
            }
        }
    }
}

// Configure Command - can only be called on Provisioned environments
impl ConfigureCommand {
    pub async fn execute(&self, environment: Environment<Provisioned>, repository: &dyn StateRepository) -> Result<Environment<Configured>, ConfigureError> {
        // Transition to intermediate state
        let configuring_env = environment.start_configuring();

        repository.save(&configuring_env.clone().into_any())
            .map_err(|e| ConfigureError::StatePersistence { source: e })?;

        match self.execute_steps(&configuring_env).await {
            Ok(_) => {
                let configured_env = configuring_env.configured();
                repository.save(&configured_env.clone().into_any())
                    .map_err(|e| ConfigureError::StatePersistence { source: e })?;
                Ok(configured_env)
            },
            Err(step_error) => {
                let failed_step = step_error.step_name();
                let failed_env = configuring_env.configure_failed(failed_step);
                repository.save(&failed_env.into_any())
                    .map_err(|e| ConfigureError::StatePersistence { source: e })?;
                Err(ConfigureError::StepExecution { source: step_error })
            }
        }
    }
}

// Command orchestration with type safety
pub struct DeploymentOrchestrator {
    repository: Box<dyn StateRepository>,
}

impl DeploymentOrchestrator {
    pub async fn full_deployment(&self, env_name: EnvironmentName, ssh_credentials: SshCredentials) -> Result<Environment<Running>, DeploymentError> {
        // Create new environment - starts in Created state
        let environment = Environment::new(env_name, ssh_credentials);

        // Each command transition is compile-time verified!
        let provision_cmd = ProvisionCommand::new();
        let provisioned_env = provision_cmd.execute(environment, self.repository.as_ref()).await?;

        let configure_cmd = ConfigureCommand::new();
        let configured_env = configure_cmd.execute(provisioned_env, self.repository.as_ref()).await?;

        let release_cmd = ReleaseCommand::new();
        let released_env = release_cmd.execute(configured_env, self.repository.as_ref()).await?;

        let run_cmd = RunCommand::new();
        let running_env = run_cmd.execute(released_env, self.repository.as_ref()).await?;

        Ok(running_env)
    }
}

// Error handling - you can only destroy from any state
impl<S> Environment<S> {
    pub async fn emergency_destroy(self, repository: &dyn StateRepository) -> Result<Environment<Destroyed>, DestroyError> {
        let destroyed_env = self.destroy();
        repository.save(&destroyed_env.clone().into_any())
            .map_err(|e| DestroyError::StatePersistence { source: e })?;

        // Perform actual cleanup operations
        // ...

        Ok(destroyed_env)
    }
}
```

### Status Query Implementation (Type-State Pattern)

```rust
// New module: src/application/queries/mod.rs
pub mod status_query;

// src/application/queries/status_query.rs
use crate::domain::{AnyEnvironmentState, EnvironmentName};
use crate::infrastructure::StateRepository;

#[derive(Debug)]
pub struct StatusQuery {
    repository: Box<dyn StateRepository>,
}

#[derive(Debug, Error)]
pub enum StatusQueryError {
    #[error("Environment '{name}' not found")]
    EnvironmentNotFound { name: String },
    #[error("Failed to load environment state")]
    StateLoadError {
        #[source]
        source: StateRepositoryError,
    },
}

#[derive(Debug)]
pub struct EnvironmentStatus {
    pub name: String,
    pub state_name: String,
    pub state_details: String,
    pub last_updated: Option<SystemTime>,
}

impl StatusQuery {
    pub fn new(repository: Box<dyn StateRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, env_name: &EnvironmentName) -> Result<EnvironmentStatus, StatusQueryError> {
        let environment = self.repository.load(env_name)
            .map_err(|e| StatusQueryError::StateLoadError { source: e })?
            .ok_or_else(|| StatusQueryError::EnvironmentNotFound {
                name: env_name.as_str().to_string()
            })?;

        let state_details = self.format_state_details(&environment);

        Ok(EnvironmentStatus {
            name: environment.name().as_str().to_string(),
            state_name: environment.state_name().to_string(),
            state_details,
            last_updated: None, // TODO: Add timestamp to state file
        })
    }

    fn format_state_details(&self, env: &AnyEnvironmentState) -> String {
        match env {
            AnyEnvironmentState::Created(_) => "Environment created, ready for provisioning".to_string(),
            AnyEnvironmentState::Provisioning(_) => "Infrastructure provisioning in progress".to_string(),
            AnyEnvironmentState::Provisioned(_) => "Infrastructure provisioned, ready for configuration".to_string(),
            AnyEnvironmentState::Configuring(_) => "System configuration in progress".to_string(),
            AnyEnvironmentState::Configured(_) => "System configured, ready for application release".to_string(),
            AnyEnvironmentState::Releasing(_) => "Application release in progress".to_string(),
            AnyEnvironmentState::Released(_) => "Application released, ready to run".to_string(),
            AnyEnvironmentState::Running(_) => "Application running and accessible".to_string(),
            AnyEnvironmentState::ProvisionFailed(env) => format!("Provisioning failed at step: {}", env.state().failed_step),
            AnyEnvironmentState::ConfigureFailed(env) => format!("Configuration failed at step: {}", env.state().failed_step),
            AnyEnvironmentState::ReleaseFailed(env) => format!("Release failed at step: {}", env.state().failed_step),
            AnyEnvironmentState::RunFailed(env) => format!("Runtime failed at step: {}", env.state().failed_step),
            AnyEnvironmentState::Destroyed(_) => "Environment destroyed and cleaned up".to_string(),
        }
    }
}

// Type-safe state recovery for development/debugging
impl StatusQuery {
    pub async fn load_environment_typed(&self, env_name: &EnvironmentName) -> Result<AnyEnvironmentState, StatusQueryError> {
        self.repository.load(env_name)
            .map_err(|e| StatusQueryError::StateLoadError { source: e })?
            .ok_or_else(|| StatusQueryError::EnvironmentNotFound {
                name: env_name.as_str().to_string()
            })
    }

    // Helper method to extract specific state types for command execution
    pub async fn require_provisioned(&self, env_name: &EnvironmentName) -> Result<Environment<Provisioned>, StatusQueryError> {
        match self.load_environment_typed(env_name).await? {
            AnyEnvironmentState::Provisioned(env) => Ok(env),
            other => Err(StatusQueryError::StateLoadError {
                source: StateRepositoryError::SerializationError {
                    path: PathBuf::from("memory"),
                    source: serde_json::Error::custom(format!("Expected provisioned state, found: {}", other.state_name()))
                }
            })
        }
    }

    pub async fn require_configured(&self, env_name: &EnvironmentName) -> Result<Environment<Configured>, StatusQueryError> {
        match self.load_environment_typed(env_name).await? {
            AnyEnvironmentState::Configured(env) => Ok(env),
            other => Err(StatusQueryError::StateLoadError {
                source: StateRepositoryError::SerializationError {
                    path: PathBuf::from("memory"),
                    source: serde_json::Error::custom(format!("Expected configured state, found: {}", other.state_name()))
                }
            })
        }
    }
}
```

## 🧪 Testing Strategy

### Unit Tests

1. **State Machine Tests**

   - Test all valid state transitions
   - Test invalid transition rejection
   - Test error state handling

2. **Repository Tests**

   - Test save/load operations
   - Test atomic write behavior
   - Test error handling for I/O failures

3. **Command Integration Tests**
   - Test state updates during command execution
   - Test error state transitions
   - Test state persistence

### E2E Tests

1. **State Persistence Tests**

   - Run command, verify state is saved
   - Restart application, verify state is loaded
   - Test interrupted command state handling

2. **Error Scenario Tests**
   - Trigger command failures
   - Verify error states are recorded
   - Test error state information

### Integration Tests

1. **Repository Integration**
   - Test with real file system
   - Test atomic write behavior
   - Test concurrent access scenarios

## 📅 Implementation Timeline

### Phase 1: Foundation

- [ ] Implement state marker types (`Created`, `Provisioning`, `Provisioned`, etc.)
- [ ] Implement `Environment<S>` with type-state pattern
- [ ] Add state-specific transition methods with compile-time validation
- [ ] Create basic type-safe state transitions

### Phase 2: Serialization & Type Erasure

- [ ] Create `AnyEnvironmentState` enum for type erasure
- [ ] Implement conversion methods between typed and erased states
- [ ] Add serialization/deserialization for all state types
- [ ] Implement helper methods for state introspection and display

### Phase 3: Persistence

- [ ] Implement `StateRepository` trait working with `AnyEnvironmentState`
- [ ] Add atomic write operations with type erasure support
- [ ] Implement state loading/saving with proper type reconstruction
- [ ] Add error handling for storage operations

### Phase 4: Command Integration

- [ ] Update commands to accept and return specific state types
- [ ] Implement type-safe command orchestration
- [ ] Add compile-time state validation for command execution
- [ ] Ensure commands can only be called on valid state types

### Phase 4: Testing & Documentation

- [ ] Add comprehensive unit tests for type-state pattern
- [ ] Update E2E tests with state persistence validation
- [ ] Test error scenarios and type safety guarantees
- [ ] Update documentation and create troubleshooting guides

## 🔮 Future Enhancements (Post-MVP)

### Status Query & CLI Integration

The status query functionality is planned for a future iteration after the production CLI scaffolding is implemented.

#### Prerequisites for Status Query Implementation

- [ ] Production CLI framework with subcommand structure
- [ ] Application main entry point and argument parsing
- [ ] Error handling and user output formatting for CLI
- [ ] Integration testing framework for CLI commands

#### Status Query Implementation (Future Phase)

- [ ] Create new `queries` module in `src/application/`
- [ ] Implement `StatusQuery` handler working with `AnyEnvironmentState`
- [ ] Add type-safe state extraction methods for command execution
- [ ] Add detailed state formatting for user-friendly display
- [ ] Add `status` command integration with CLI framework
- [ ] Add comprehensive tests for status query functionality

#### Status Query Benefits (When Implemented)

- **Environment State Visibility**: Users can check current state without reading logs
- **Type-Safe State Recovery**: Load specific state types for command continuation
- **User-Friendly Display**: Clear status messages with actionable information
- **Debug Support**: Help users understand where deployments failed

### Manual Recovery Documentation

Since there is no automated destroy command yet, users must manually clean up failed environments using OpenTofu commands. This section provides the manual recovery procedures.

#### Recovery Process for Failed Environments

When an environment fails during any phase (provisioning, configuring, releasing, or running), follow these steps to clean up and restart:

1. **Check the Error State**

   Review the logs to understand which step failed:

   ```bash
   # Look for error messages and state transitions in recent logs
   grep "ERROR" logs/*.log | tail -20
   grep "transitioning.*failed" logs/*.log
   ```

2. **Manual Cleanup Using OpenTofu**

   Navigate to the build directory and run OpenTofu destroy:

   ```bash
   cd build/{ENV_NAME}/tofu
   tofu destroy -auto-approve
   ```

   If OpenTofu reports errors, you may need to manually remove resources:

   ```bash
   # List current resources in state
   tofu state list

   # Remove specific resources if needed
   tofu state rm 'resource.name'
   ```

3. **Clean Up Local State Files**

   Remove the environment's data and build directories:

   ```bash
   rm -rf data/{ENV_NAME}
   rm -rf build/{ENV_NAME}
   ```

4. **Restart Deployment**

   After cleanup, restart the deployment from the beginning:

   ```bash
   # Re-run your deployment command (via E2E tests for now)
   cargo test --test your_deployment_test
   ```

#### Common Failure Scenarios

##### Scenario 1: Provisioning Failed During VM Creation

- **State**: `provision_failed`
- **Recovery**: Run OpenTofu destroy, clean up directories, restart

##### Scenario 2: Configuration Failed During Ansible Execution

- **State**: `configure_failed`
- **Recovery**: Destroy VM via OpenTofu, clean up directories, restart
- **Note**: The VM exists but configuration is incomplete

##### Scenario 3: Release Failed During Application Deployment

- **State**: `release_failed`
- **Recovery**: Destroy VM via OpenTofu, clean up directories, restart
- **Note**: VM is configured but application deployment failed

##### Scenario 4: Interrupted Command (Ctrl+C)

- **State**: Intermediate state (e.g., `provisioning`, `configuring`)
- **Recovery**: Follow same cleanup procedure as failed scenarios
- **Note**: Partial resources may exist and need cleanup

#### Future Enhancement: Automated Destroy Command

In a future iteration, an automated destroy command will be implemented to simplify recovery:

```bash
# Future command (not yet implemented)
torrust-deploy destroy {ENV_NAME}
```

This command will:

- Automatically run OpenTofu destroy
- Clean up local state files
- Remove build and data directories
- Provide clear feedback about cleanup progress

#### Implementation Approach (Future)

```rust
// Example future status command usage
// $ torrust-deploy status myenv
// Environment: myenv
// State: provision_failed
// Details: Provisioning failed at step: cloud_init_execution
//
// To recover:
// 1. Check logs for detailed error information
// 2. Destroy environment: cd build/myenv/tofu && tofu destroy -auto-approve
// 3. Clean up: rm -rf data/myenv build/myenv
// 4. Restart deployment from the beginning

pub struct StatusCommand {
    status_query: StatusQuery,
}

impl StatusCommand {
    pub async fn execute(&self, env_name: EnvironmentName) -> Result<(), StatusCommandError> {
        let status = self.status_query.execute(&env_name).await?;

        println!("Environment: {}", status.name);
        println!("State: {}", status.state_name);
        println!("Details: {}", status.state_details);

        // Add recovery suggestions based on state
        self.print_recovery_suggestions(&status);

        Ok(())
    }
}
```

## 🚨 Risk Mitigation

### Potential Risks

1. **State File Corruption**: Mitigated by atomic writes
2. **Concurrent Access**: Initial implementation assumes single-user usage
3. **State Inconsistency**: Validation can be added in future iterations
4. **Migration Complexity**: Start with simple JSON format, plan for migration strategy

### Contingency Plans

1. **Rollback Strategy**: Keep existing E2E tests working during implementation
2. **Incremental Deployment**: Implement feature behind feature flag initially
3. **State Recovery**: Document manual state file repair procedures
