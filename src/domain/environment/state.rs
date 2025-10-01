//! Environment State Marker Types
//!
//! This module defines the state marker types used in the type-state pattern
//! for the Environment entity. Each state represents a distinct phase in the
//! deployment lifecycle and is enforced at compile-time.
//!
//! ## Type-State Pattern
//!
//! The type-state pattern uses Rust's type system to enforce state machine
//! transitions at compile-time. Each state is a distinct type, and the
//! `Environment<S>` struct is generic over the state type. This ensures that
//! invalid state transitions are caught during compilation rather than at runtime.
//!
//! ## State Lifecycle
//!
//! ### Happy Path
//!
//! ```text
//! Created → Provisioning → Provisioned → Configuring → Configured
//!   → Releasing → Released → Running → Destroyed
//! ```
//!
//! ### Error States
//!
//! At each operational phase, the system can transition to a corresponding
//! failed state if an error occurs:
//!
//! - `Provisioning` → `ProvisionFailed`
//! - `Configuring` → `ConfigureFailed`
//! - `Releasing` → `ReleaseFailed`
//! - `Running` → `RunFailed`
//!
//! ## Usage Example
//!
//! ```rust
//! use torrust_tracker_deploy::domain::environment::state::{Created, Provisioning};
//!
//! // State types are used as type parameters for Environment<S>
//! // let env: Environment<Created> = Environment::new(name, credentials);
//! // let env: Environment<Provisioning> = env.start_provisioning();
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Initial state - Environment has been created but no operations performed
///
/// This is the entry state for all new environments. From this state, the
/// environment can transition to `Provisioning` when infrastructure setup begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Created;

/// Intermediate state - Infrastructure provisioning in progress
///
/// The environment is actively being provisioned (VM creation, network setup, etc.).
/// This state indicates that the provision command has started but not yet completed.
///
/// **Valid Transitions:**
/// - Success: `Provisioned`
/// - Failure: `ProvisionFailed`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provisioning;

/// Final state - Infrastructure provisioning completed successfully
///
/// The VM instance is running and accessible. The environment is ready for
/// application configuration.
///
/// **Valid Transitions:**
/// - `Configuring` (start application configuration)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provisioned;

/// Intermediate state - Application configuration in progress
///
/// The environment is being configured with application-specific settings
/// (Ansible playbooks, configuration files, etc.).
///
/// **Valid Transitions:**
/// - Success: `Configured`
/// - Failure: `ConfigureFailed`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuring;

/// Final state - Application configuration completed successfully
///
/// All application configuration has been applied. The environment is ready
/// for release preparation.
///
/// **Valid Transitions:**
/// - `Releasing` (start release process)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configured;

/// Intermediate state - Release preparation in progress
///
/// The environment is being prepared for production release (building artifacts,
/// final checks, etc.).
///
/// **Valid Transitions:**
/// - Success: `Released`
/// - Failure: `ReleaseFailed`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releasing;

/// Final state - Release preparation completed successfully
///
/// The environment is fully prepared and ready to run the application.
///
/// **Valid Transitions:**
/// - `Running` (start application)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Released;

/// Final state - Application is running
///
/// The application is actively running in the environment. This is the target
/// operational state.
///
/// **Valid Transitions:**
/// - `RunFailed` (if runtime error occurs)
/// - `Destroyed` (when shutting down)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Running;

/// Error state - Infrastructure provisioning failed
///
/// The provision command failed during execution. The `failed_step` field
/// contains the name of the step that caused the failure, providing context
/// for debugging and recovery.
///
/// **Recovery Options:**
/// - Destroy and recreate the environment
/// - Manual inspection and repair (advanced users)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvisionFailed {
    /// The name of the step that failed during provisioning
    pub failed_step: String,
}

/// Error state - Application configuration failed
///
/// The configuration command failed during execution. The `failed_step` field
/// contains the name of the step that caused the failure.
///
/// **Recovery Options:**
/// - Destroy and recreate the environment
/// - Manual configuration correction (advanced users)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigureFailed {
    /// The name of the step that failed during configuration
    pub failed_step: String,
}

/// Error state - Release preparation failed
///
/// The release command failed during execution. The `failed_step` field
/// contains the name of the step that caused the failure.
///
/// **Recovery Options:**
/// - Destroy and recreate the environment
/// - Manual release correction (advanced users)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseFailed {
    /// The name of the step that failed during release
    pub failed_step: String,
}

/// Error state - Application runtime failed
///
/// The application encountered a runtime error. The `failed_step` field
/// contains the name of the operation that caused the failure.
///
/// **Recovery Options:**
/// - Restart the application
/// - Destroy and recreate the environment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunFailed {
    /// The name of the operation that failed during runtime
    pub failed_step: String,
}

/// Terminal state - Environment has been destroyed
///
/// All infrastructure resources have been released and the environment no longer
/// exists. This is the final state in the lifecycle.
///
/// **No Valid Transitions:** This is a terminal state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destroyed;

/// Error type for invalid type conversions when working with type-erased environments
///
/// This error occurs when attempting to convert an `AnyEnvironmentState` to a specific
/// typed `Environment<S>` state, but the runtime state doesn't match the expected type.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deploy::domain::environment::state::AnyEnvironmentState;
///
/// // let any_env = AnyEnvironmentState::Provisioned(...);
/// // // This will fail because any_env is Provisioned, not Created
/// // let result = any_env.try_into_created();
/// // assert!(result.is_err());
/// ```
#[derive(Debug, Error)]
pub enum StateTypeError {
    /// The environment is in a different state than expected
    #[error("Expected state '{expected}', but found '{actual}'")]
    UnexpectedState {
        /// The state type that was expected
        expected: &'static str,
        /// The actual state type that was found
        actual: String,
    },
}

// Import Environment for type erasure enum
use crate::domain::environment::Environment;

/// Type-erased environment that can hold any typed `Environment<S>` at runtime
///
/// This enum enables runtime handling of `Environment<S>` instances without
/// knowing their specific state type at compile time. This is essential for:
///
/// - **Serialization**: Saving environments to disk (JSON files)
/// - **Deserialization**: Loading environments from disk
/// - **Collections**: Storing environments with different states together
/// - **Runtime Inspection**: Checking state without compile-time type knowledge
/// - **Generic Interfaces**: Passing through non-generic function parameters
///
/// ## Type Erasure Pattern
///
/// Each variant wraps a typed `Environment<S>` where `S` is one of the state
/// marker types defined in this module. The enum variant name acts as a
/// discriminator (similar to a `type` column in database Single Table Inheritance).
///
/// ## Usage Example
///
/// ```rust
/// use torrust_tracker_deploy::domain::environment::state::AnyEnvironmentState;
///
/// // Type erasure: typed -> runtime
/// // let env: Environment<Provisioned> = ...;
/// // let any_env: AnyEnvironmentState = env.into_any();
///
/// // Serialization
/// // let json = serde_json::to_string(&any_env)?;
///
/// // Deserialization
/// // let any_env: AnyEnvironmentState = serde_json::from_str(&json)?;
///
/// // Type restoration: runtime -> typed
/// // let env: Environment<Provisioned> = any_env.try_into_provisioned()?;
/// ```
///
/// ## Design Decision
///
/// See [ADR: Type Erasure for Environment States](../../docs/decisions/type-erasure-for-environment-states.md)
/// for detailed rationale behind this design choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyEnvironmentState {
    /// Environment in `Created` state
    Created(Environment<Created>),

    /// Environment in `Provisioning` state
    Provisioning(Environment<Provisioning>),

    /// Environment in `Provisioned` state
    Provisioned(Environment<Provisioned>),

    /// Environment in `Configuring` state
    Configuring(Environment<Configuring>),

    /// Environment in `Configured` state
    Configured(Environment<Configured>),

    /// Environment in `Releasing` state
    Releasing(Environment<Releasing>),

    /// Environment in `Released` state
    Released(Environment<Released>),

    /// Environment in `Running` state
    Running(Environment<Running>),

    /// Environment in `ProvisionFailed` error state
    ProvisionFailed(Environment<ProvisionFailed>),

    /// Environment in `ConfigureFailed` error state
    ConfigureFailed(Environment<ConfigureFailed>),

    /// Environment in `ReleaseFailed` error state
    ReleaseFailed(Environment<ReleaseFailed>),

    /// Environment in `RunFailed` error state
    RunFailed(Environment<RunFailed>),

    /// Environment in `Destroyed` terminal state
    Destroyed(Environment<Destroyed>),
}

// Type Restoration: Runtime → Typed conversions (try_into_<state>)
// These methods convert AnyEnvironmentState back to typed Environment<S>
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<Created>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Created` state.
    pub fn try_into_created(self) -> Result<Environment<Created>, StateTypeError> {
        match self {
            Self::Created(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "created",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Provisioning>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Provisioning` state.
    pub fn try_into_provisioning(self) -> Result<Environment<Provisioning>, StateTypeError> {
        match self {
            Self::Provisioning(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "provisioning",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Provisioned>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Provisioned` state.
    pub fn try_into_provisioned(self) -> Result<Environment<Provisioned>, StateTypeError> {
        match self {
            Self::Provisioned(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "provisioned",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Configuring>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Configuring` state.
    pub fn try_into_configuring(self) -> Result<Environment<Configuring>, StateTypeError> {
        match self {
            Self::Configuring(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "configuring",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Configured>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Configured` state.
    pub fn try_into_configured(self) -> Result<Environment<Configured>, StateTypeError> {
        match self {
            Self::Configured(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "configured",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Releasing>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Releasing` state.
    pub fn try_into_releasing(self) -> Result<Environment<Releasing>, StateTypeError> {
        match self {
            Self::Releasing(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "releasing",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Released>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Released` state.
    pub fn try_into_released(self) -> Result<Environment<Released>, StateTypeError> {
        match self {
            Self::Released(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "released",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Running>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Running` state.
    pub fn try_into_running(self) -> Result<Environment<Running>, StateTypeError> {
        match self {
            Self::Running(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "running",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<ProvisionFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `ProvisionFailed` state.
    pub fn try_into_provision_failed(self) -> Result<Environment<ProvisionFailed>, StateTypeError> {
        match self {
            Self::ProvisionFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "provision_failed",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<ConfigureFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `ConfigureFailed` state.
    pub fn try_into_configure_failed(self) -> Result<Environment<ConfigureFailed>, StateTypeError> {
        match self {
            Self::ConfigureFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "configure_failed",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<ReleaseFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `ReleaseFailed` state.
    pub fn try_into_release_failed(self) -> Result<Environment<ReleaseFailed>, StateTypeError> {
        match self {
            Self::ReleaseFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "release_failed",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<RunFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `RunFailed` state.
    pub fn try_into_run_failed(self) -> Result<Environment<RunFailed>, StateTypeError> {
        match self {
            Self::RunFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "run_failed",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Attempts to convert `AnyEnvironmentState` to `Environment<Destroyed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Destroyed` state.
    pub fn try_into_destroyed(self) -> Result<Environment<Destroyed>, StateTypeError> {
        match self {
            Self::Destroyed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "destroyed",
                actual: other.state_name().to_string(),
            }),
        }
    }

    /// Helper method to get the state name as a string
    ///
    /// This is used internally for error messages when type conversion fails.
    fn state_name(&self) -> &'static str {
        match self {
            Self::Created(_) => "created",
            Self::Provisioning(_) => "provisioning",
            Self::Provisioned(_) => "provisioned",
            Self::Configuring(_) => "configuring",
            Self::Configured(_) => "configured",
            Self::Releasing(_) => "releasing",
            Self::Released(_) => "released",
            Self::Running(_) => "running",
            Self::ProvisionFailed(_) => "provision_failed",
            Self::ConfigureFailed(_) => "configure_failed",
            Self::ReleaseFailed(_) => "release_failed",
            Self::RunFailed(_) => "run_failed",
            Self::Destroyed(_) => "destroyed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test module for state marker types
    ///
    /// These tests verify that state types can be created, cloned, and serialized
    /// correctly. They ensure basic functionality of the state marker types.

    #[test]
    fn it_should_create_created_state() {
        let state = Created;
        assert_eq!(state, Created);
    }

    #[test]
    fn it_should_clone_created_state() {
        let state = Created;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn it_should_create_provisioning_state() {
        let state = Provisioning;
        assert_eq!(state, Provisioning);
    }

    #[test]
    fn it_should_create_provisioned_state() {
        let state = Provisioned;
        assert_eq!(state, Provisioned);
    }

    #[test]
    fn it_should_create_configuring_state() {
        let state = Configuring;
        assert_eq!(state, Configuring);
    }

    #[test]
    fn it_should_create_configured_state() {
        let state = Configured;
        assert_eq!(state, Configured);
    }

    #[test]
    fn it_should_create_releasing_state() {
        let state = Releasing;
        assert_eq!(state, Releasing);
    }

    #[test]
    fn it_should_create_released_state() {
        let state = Released;
        assert_eq!(state, Released);
    }

    #[test]
    fn it_should_create_running_state() {
        let state = Running;
        assert_eq!(state, Running);
    }

    #[test]
    fn it_should_create_provision_failed_state_with_context() {
        let state = ProvisionFailed {
            failed_step: "cloud_init_execution".to_string(),
        };
        assert_eq!(state.failed_step, "cloud_init_execution");
    }

    #[test]
    fn it_should_clone_provision_failed_state() {
        let state = ProvisionFailed {
            failed_step: "cloud_init_execution".to_string(),
        };
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn it_should_create_configure_failed_state_with_context() {
        let state = ConfigureFailed {
            failed_step: "ansible_playbook_execution".to_string(),
        };
        assert_eq!(state.failed_step, "ansible_playbook_execution");
    }

    #[test]
    fn it_should_create_release_failed_state_with_context() {
        let state = ReleaseFailed {
            failed_step: "build_artifacts".to_string(),
        };
        assert_eq!(state.failed_step, "build_artifacts");
    }

    #[test]
    fn it_should_create_run_failed_state_with_context() {
        let state = RunFailed {
            failed_step: "application_startup".to_string(),
        };
        assert_eq!(state.failed_step, "application_startup");
    }

    #[test]
    fn it_should_create_destroyed_state() {
        let state = Destroyed;
        assert_eq!(state, Destroyed);
    }

    #[test]
    fn it_should_serialize_created_state_to_json() {
        let state = Created;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn it_should_deserialize_created_state_from_json() {
        let json = "null";
        let state: Created = serde_json::from_str(json).unwrap();
        assert_eq!(state, Created);
    }

    #[test]
    fn it_should_serialize_provision_failed_state_to_json() {
        let state = ProvisionFailed {
            failed_step: "cloud_init".to_string(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("cloud_init"));
    }

    #[test]
    fn it_should_deserialize_provision_failed_state_from_json() {
        let json = r#"{"failed_step":"cloud_init"}"#;
        let state: ProvisionFailed = serde_json::from_str(json).unwrap();
        assert_eq!(state.failed_step, "cloud_init");
    }

    #[test]
    fn it_should_serialize_configure_failed_state_to_json() {
        let state = ConfigureFailed {
            failed_step: "ansible_playbook".to_string(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("ansible_playbook"));
    }

    #[test]
    fn it_should_deserialize_configure_failed_state_from_json() {
        let json = r#"{"failed_step":"ansible_playbook"}"#;
        let state: ConfigureFailed = serde_json::from_str(json).unwrap();
        assert_eq!(state.failed_step, "ansible_playbook");
    }

    #[test]
    fn it_should_debug_format_states() {
        let created = format!("{Created:?}");
        assert_eq!(created, "Created");

        let provisioning = format!("{Provisioning:?}");
        assert_eq!(provisioning, "Provisioning");

        let failed = format!(
            "{:?}",
            ProvisionFailed {
                failed_step: "test".to_string()
            }
        );
        assert!(failed.contains("ProvisionFailed"));
        assert!(failed.contains("test"));
    }

    // Tests for AnyEnvironmentState enum (Type Erasure)
    mod any_environment_state_tests {
        use super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_ssh_credentials() -> SshCredentials {
            let username = Username::new("test-user".to_string()).unwrap();
            SshCredentials::new(
                PathBuf::from("/tmp/test_key"),
                PathBuf::from("/tmp/test_key.pub"),
                username,
            )
        }

        fn create_test_environment_created() -> Environment<Created> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
        }

        // Note: For testing other states, we'll use the state transition methods
        // once they're implemented in Subtask 2. For now, we test with Created state
        // which demonstrates that the enum can hold any Environment<S> type.

        #[test]
        fn it_should_create_any_environment_state_with_created_variant() {
            let env = create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            assert!(matches!(any_env, AnyEnvironmentState::Created(_)));
        }

        // Note: Tests for other state variants will be added in Subtask 2
        // once we have the conversion methods (into_any()) that properly
        // create environments in different states through state transitions.

        #[test]
        fn it_should_clone_any_environment_state() {
            let env = create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            let cloned = any_env.clone();
            assert!(matches!(cloned, AnyEnvironmentState::Created(_)));
        }

        #[test]
        fn it_should_debug_format_any_environment_state() {
            let env = create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            let debug_str = format!("{any_env:?}");
            assert!(debug_str.contains("Created"));
        }

        #[test]
        fn it_should_serialize_any_environment_state_to_json() {
            let env = create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            let json = serde_json::to_string(&any_env).unwrap();
            assert!(json.contains("Created"));
        }

        // Tests for Type Conversion Methods (Subtask 2)

        // Tests for into_any() - Typed to Runtime conversions

        #[test]
        fn it_should_convert_created_environment_into_any() {
            let env = create_test_environment_created();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Created(_)));
        }

        #[test]
        fn it_should_convert_provisioning_environment_into_any() {
            let env = create_test_environment_created().start_provisioning();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Provisioning(_)));
        }

        #[test]
        fn it_should_convert_provisioned_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Provisioned(_)));
        }

        #[test]
        fn it_should_convert_configuring_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Configuring(_)));
        }

        #[test]
        fn it_should_convert_configured_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Configured(_)));
        }

        #[test]
        fn it_should_convert_releasing_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Releasing(_)));
        }

        #[test]
        fn it_should_convert_released_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Released(_)));
        }

        #[test]
        fn it_should_convert_running_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Running(_)));
        }

        #[test]
        fn it_should_convert_provision_failed_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provision_failed("infrastructure error".to_string());
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ProvisionFailed(_)));
        }

        #[test]
        fn it_should_convert_configure_failed_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configure_failed("ansible error".to_string());
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ConfigureFailed(_)));
        }

        #[test]
        fn it_should_convert_release_failed_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .release_failed("release error".to_string());
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ReleaseFailed(_)));
        }

        #[test]
        fn it_should_convert_run_failed_environment_into_any() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running()
                .run_failed("runtime error".to_string());
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::RunFailed(_)));
        }

        #[test]
        fn it_should_convert_destroyed_environment_into_any() {
            let env = create_test_environment_created().destroy();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Destroyed(_)));
        }

        // Tests for try_into_<state>() - Runtime to Typed conversions (successful cases)

        #[test]
        fn it_should_convert_any_to_created_successfully() {
            let env = create_test_environment_created();
            let any_env = env.into_any();
            let result = any_env.try_into_created();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_provisioning_successfully() {
            let env = create_test_environment_created().start_provisioning();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioning();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_provisioned_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioned();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_configuring_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring();
            let any_env = env.into_any();
            let result = any_env.try_into_configuring();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_configured_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured();
            let any_env = env.into_any();
            let result = any_env.try_into_configured();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_releasing_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing();
            let any_env = env.into_any();
            let result = any_env.try_into_releasing();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_released_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released();
            let any_env = env.into_any();
            let result = any_env.try_into_released();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_running_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running();
            let any_env = env.into_any();
            let result = any_env.try_into_running();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_provision_failed_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provision_failed("test error".to_string());
            let any_env = env.into_any();
            let result = any_env.try_into_provision_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_configure_failed_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configure_failed("test error".to_string());
            let any_env = env.into_any();
            let result = any_env.try_into_configure_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_release_failed_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .release_failed("test error".to_string());
            let any_env = env.into_any();
            let result = any_env.try_into_release_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_run_failed_successfully() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running()
                .run_failed("test error".to_string());
            let any_env = env.into_any();
            let result = any_env.try_into_run_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_destroyed_successfully() {
            let env = create_test_environment_created().destroy();
            let any_env = env.into_any();
            let result = any_env.try_into_destroyed();
            assert!(result.is_ok());
        }

        // Tests for try_into_<state>() - Runtime to Typed conversions (failure cases)

        #[test]
        fn it_should_fail_converting_provisioning_to_created() {
            let env = create_test_environment_created().start_provisioning();
            let any_env = env.into_any();
            let result = any_env.try_into_created();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, StateTypeError::UnexpectedState { .. }));
            assert!(err.to_string().contains("created"));
            assert!(err.to_string().contains("provisioning"));
        }

        #[test]
        fn it_should_fail_converting_created_to_provisioning() {
            let env = create_test_environment_created();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioning();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("provisioning"));
            assert!(err.to_string().contains("created"));
        }

        #[test]
        fn it_should_fail_converting_provision_failed_to_provisioned() {
            let env = create_test_environment_created()
                .start_provisioning()
                .provision_failed("error".to_string());
            let any_env = env.into_any();
            let result = any_env.try_into_provisioned();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("provisioned"));
            assert!(err.to_string().contains("provision_failed"));
        }

        #[test]
        fn it_should_fail_converting_destroyed_to_running() {
            let env = create_test_environment_created().destroy();
            let any_env = env.into_any();
            let result = any_env.try_into_running();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("running"));
            assert!(err.to_string().contains("destroyed"));
        }

        // Tests for round-trip conversions (preserving data integrity)

        #[test]
        fn it_should_preserve_data_in_round_trip_conversion() {
            let original_name = "test-env";
            let env = create_test_environment_created();

            // Round-trip: typed -> erased -> typed
            let any_env = env.into_any();
            let env_restored = any_env.try_into_created().unwrap();
            let name_after = env_restored.name().as_str();

            assert_eq!(name_after, original_name);
        }

        #[test]
        fn it_should_preserve_error_details_in_failed_states() {
            let error_message = "infrastructure deployment failed";
            let env = create_test_environment_created()
                .start_provisioning()
                .provision_failed(error_message.to_string());

            // Round-trip conversion
            let any_env = env.into_any();
            let env_restored = any_env.try_into_provision_failed().unwrap();

            assert_eq!(env_restored.state().failed_step, error_message);
        }
    }
}
