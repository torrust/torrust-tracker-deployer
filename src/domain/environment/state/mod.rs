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
//! use torrust_tracker_deployer_lib::domain::environment::state::{Created, Provisioning};
//!
//! // State types are used as type parameters for Environment<S>
//! // let env: Environment<Created> = Environment::new(name, credentials);
//! // let env: Environment<Provisioning> = env.start_provisioning();
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

// State modules
mod common;
mod configure_failed;
mod configured;
mod configuring;
mod created;
mod destroy_failed;
mod destroyed;
mod destroying;
mod provision_failed;
mod provisioned;
mod provisioning;
mod release_failed;
mod released;
mod releasing;
mod run_failed;
mod running;

// Re-export state types
pub use common::BaseFailureContext;
pub use configure_failed::{ConfigureFailed, ConfigureFailureContext, ConfigureStep};
pub use configured::Configured;
pub use configuring::Configuring;
pub use created::Created;
pub use destroy_failed::{DestroyFailed, DestroyFailureContext, DestroyStep};
pub use destroyed::Destroyed;
pub use destroying::Destroying;
pub use provision_failed::{ProvisionFailed, ProvisionFailureContext, ProvisionStep};
pub use provisioned::Provisioned;
pub use provisioning::Provisioning;
pub use release_failed::ReleaseFailed;
pub use released::Released;
pub use releasing::Releasing;
pub use run_failed::RunFailed;
pub use running::Running;

/// Error type for invalid type conversions when working with type-erased environments
///
/// This error occurs when attempting to convert an `AnyEnvironmentState` to a specific
/// typed `Environment<S>` state, but the runtime state doesn't match the expected type.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::state::AnyEnvironmentState;
///
/// // let any_env = AnyEnvironmentState::Provisioned(...);
/// // // This will fail because any_env is Provisioned, not Created
/// // let result = any_env.try_into_created();
/// // assert!(result.is_err());
/// ```
#[derive(Debug, Clone, Error)]
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
use crate::domain::environment::{Environment, EnvironmentName};

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
/// use torrust_tracker_deployer_lib::domain::environment::state::AnyEnvironmentState;
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

    /// Environment in `Destroying` state
    Destroying(Environment<Destroying>),

    /// Environment in `ProvisionFailed` error state
    ProvisionFailed(Environment<ProvisionFailed>),

    /// Environment in `ConfigureFailed` error state
    ConfigureFailed(Environment<ConfigureFailed>),

    /// Environment in `ReleaseFailed` error state
    ReleaseFailed(Environment<ReleaseFailed>),

    /// Environment in `RunFailed` error state
    RunFailed(Environment<RunFailed>),

    /// Environment in `DestroyFailed` error state
    DestroyFailed(Environment<DestroyFailed>),

    /// Environment in `Destroyed` terminal state
    Destroyed(Environment<Destroyed>),
}

// Introspection methods for AnyEnvironmentState
impl AnyEnvironmentState {
    /// Get a reference to the environment context regardless of current state
    ///
    /// This helper method centralizes state matching for accessing
    /// state-independent data. Instead of pattern matching 6 times
    /// (once per field accessor), we match once and reuse the result.
    ///
    /// This is a private implementation detail that simplifies the
    /// public accessor methods.
    ///
    /// # Returns
    ///
    /// A reference to the `EnvironmentContext` contained within the environment.
    fn context(&self) -> &crate::domain::environment::EnvironmentContext {
        match self {
            Self::Created(env) => env.context(),
            Self::Provisioning(env) => env.context(),
            Self::Provisioned(env) => env.context(),
            Self::Configuring(env) => env.context(),
            Self::Configured(env) => env.context(),
            Self::Releasing(env) => env.context(),
            Self::Released(env) => env.context(),
            Self::Running(env) => env.context(),
            Self::Destroying(env) => env.context(),
            Self::ProvisionFailed(env) => env.context(),
            Self::ConfigureFailed(env) => env.context(),
            Self::ReleaseFailed(env) => env.context(),
            Self::RunFailed(env) => env.context(),
            Self::DestroyFailed(env) => env.context(),
            Self::Destroyed(env) => env.context(),
        }
    }

    /// Get the environment name regardless of current state
    ///
    /// This method provides access to the environment name without needing to
    /// pattern match on the specific state variant.
    ///
    /// # Returns
    ///
    /// A reference to the `EnvironmentName` contained within the environment.
    #[must_use]
    pub fn name(&self) -> &EnvironmentName {
        &self.context().user_inputs.name
    }

    /// Get the state name as a string
    ///
    /// Returns a static string identifier for the current state. This is useful
    /// for logging, error messages, and displaying state information to users.
    ///
    /// # Returns
    ///
    /// A static string representing the state name (e.g., "created", "provisioning").
    #[must_use]
    pub fn state_name(&self) -> &'static str {
        match self {
            Self::Created(_) => "created",
            Self::Provisioning(_) => "provisioning",
            Self::Provisioned(_) => "provisioned",
            Self::Configuring(_) => "configuring",
            Self::Configured(_) => "configured",
            Self::Releasing(_) => "releasing",
            Self::Released(_) => "released",
            Self::Running(_) => "running",
            Self::Destroying(_) => "destroying",
            Self::ProvisionFailed(_) => "provision_failed",
            Self::ConfigureFailed(_) => "configure_failed",
            Self::ReleaseFailed(_) => "release_failed",
            Self::RunFailed(_) => "run_failed",
            Self::DestroyFailed(_) => "destroy_failed",
            Self::Destroyed(_) => "destroyed",
        }
    }

    /// Check if the environment is in a success (non-error) state
    ///
    /// Success states are those representing normal operation flow, including
    /// transient states (like `Provisioning`) and terminal success states
    /// (like `Running`, `Destroyed`).
    ///
    /// # Returns
    ///
    /// `true` if the environment is in a success state, `false` for error states.
    #[must_use]
    pub fn is_success_state(&self) -> bool {
        matches!(
            self,
            Self::Created(_)
                | Self::Provisioning(_)
                | Self::Provisioned(_)
                | Self::Configuring(_)
                | Self::Configured(_)
                | Self::Releasing(_)
                | Self::Released(_)
                | Self::Running(_)
                | Self::Destroying(_)
                | Self::Destroyed(_)
        )
    }

    /// Check if the environment is in an error state
    ///
    /// Error states indicate that an operation failed during the environment's
    /// lifecycle (provisioning, configuration, release, or runtime).
    ///
    /// # Returns
    ///
    /// `true` if the environment is in an error state, `false` otherwise.
    #[must_use]
    pub fn is_error_state(&self) -> bool {
        matches!(
            self,
            Self::ProvisionFailed(_)
                | Self::ConfigureFailed(_)
                | Self::ReleaseFailed(_)
                | Self::RunFailed(_)
                | Self::DestroyFailed(_)
        )
    }

    /// Check if the environment is in a terminal state
    ///
    /// Terminal states are final states where no more transitions are expected.
    /// This includes both successful terminal states (`Running`, `Destroyed`)
    /// and error states (all `*Failed` variants).
    ///
    /// # Returns
    ///
    /// `true` if the environment is in a terminal state, `false` otherwise.
    #[must_use]
    pub fn is_terminal_state(&self) -> bool {
        matches!(
            self,
            Self::Running(_)
                | Self::Destroyed(_)
                | Self::ProvisionFailed(_)
                | Self::ConfigureFailed(_)
                | Self::ReleaseFailed(_)
                | Self::RunFailed(_)
                | Self::DestroyFailed(_)
        )
    }

    /// Get error details if the environment is in an error state
    ///
    /// For error states (`*Failed`), this returns the description of the
    /// operation that failed. For non-error states, returns `None`.
    ///
    /// # Returns
    ///
    /// - `Some(&str)` containing the failed operation description for error states
    /// - `None` for success states
    #[must_use]
    pub fn error_details(&self) -> Option<&str> {
        match self {
            Self::ProvisionFailed(env) => Some(&env.state().context.base.error_summary),
            Self::ConfigureFailed(env) => Some(&env.state().context.base.error_summary),
            Self::ReleaseFailed(env) => Some(&env.state().failed_step),
            Self::RunFailed(env) => Some(&env.state().failed_step),
            Self::DestroyFailed(env) => Some(&env.state().context.base.error_summary),
            _ => None,
        }
    }

    /// Get the instance name regardless of current state
    ///
    /// This method provides access to the instance name without needing to
    /// pattern match on the specific state variant.
    ///
    /// # Returns
    ///
    /// A reference to the `InstanceName` contained within the environment.
    #[must_use]
    pub fn instance_name(&self) -> &crate::domain::environment::InstanceName {
        &self.context().user_inputs.instance_name
    }

    /// Get the profile name regardless of current state
    ///
    /// This method provides access to the profile name without needing to
    /// pattern match on the specific state variant.
    ///
    /// # Returns
    ///
    /// A reference to the `ProfileName` contained within the environment.
    #[must_use]
    pub fn profile_name(&self) -> &crate::domain::environment::ProfileName {
        &self.context().user_inputs.profile_name
    }

    /// Get the SSH credentials regardless of current state
    ///
    /// This method provides access to the SSH credentials without needing to
    /// pattern match on the specific state variant.
    ///
    /// # Returns
    ///
    /// A reference to the `SshCredentials` contained within the environment.
    #[must_use]
    pub fn ssh_credentials(&self) -> &crate::adapters::ssh::SshCredentials {
        &self.context().user_inputs.ssh_credentials
    }

    /// Get the SSH port regardless of current state
    ///
    /// This method provides access to the SSH port without needing to
    /// pattern match on the specific state variant.
    ///
    /// # Returns
    ///
    /// The SSH port number.
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.context().user_inputs.ssh_port
    }

    /// Get the instance IP address if available, regardless of current state
    ///
    /// This method provides access to the instance IP without needing to
    /// pattern match on the specific state variant.
    ///
    /// # Returns
    ///
    /// - `Some(IpAddr)` if the environment has been provisioned
    /// - `None` if the environment hasn't been provisioned yet
    #[must_use]
    pub fn instance_ip(&self) -> Option<std::net::IpAddr> {
        self.context().runtime_outputs.instance_ip
    }

    /// Destroy the environment, transitioning it to the Destroyed state
    ///
    /// This method provides a unified interface to destroy an environment
    /// regardless of its current state. It encapsulates the repetitive match
    /// pattern that would otherwise be needed in calling code.
    ///
    /// # Returns
    ///
    /// - `Ok(Environment<Destroyed>)` if the environment was successfully destroyed
    /// - `Err(StateTypeError)` if the environment is already in the `Destroyed` state
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if called on an environment
    /// already in the `Destroyed` state.
    pub fn destroy(self) -> Result<Environment<Destroyed>, StateTypeError> {
        match self {
            Self::Created(env) => Ok(env.destroy()),
            Self::Provisioning(env) => Ok(env.destroy()),
            Self::Provisioned(env) => Ok(env.destroy()),
            Self::Configuring(env) => Ok(env.destroy()),
            Self::Configured(env) => Ok(env.destroy()),
            Self::Releasing(env) => Ok(env.destroy()),
            Self::Released(env) => Ok(env.destroy()),
            Self::Running(env) => Ok(env.destroy()),
            Self::Destroying(env) => Ok(env.destroy()),
            Self::ProvisionFailed(env) => Ok(env.destroy()),
            Self::ConfigureFailed(env) => Ok(env.destroy()),
            Self::ReleaseFailed(env) => Ok(env.destroy()),
            Self::RunFailed(env) => Ok(env.destroy()),
            Self::DestroyFailed(env) => Ok(env.destroy()),
            Self::Destroyed(_) => Err(StateTypeError::UnexpectedState {
                expected: "any state except destroyed",
                actual: "destroyed".to_string(),
            }),
        }
    }

    /// Get the `OpenTofu` build directory path regardless of current state
    ///
    /// This method provides a unified interface to access the build directory
    /// for `OpenTofu` operations without needing to pattern match on the
    /// specific state variant.
    ///
    /// The path is returned consistently regardless of the environment's state.
    /// The caller is responsible for determining how to use the path based on
    /// their specific needs and the environment's current state.
    ///
    /// # Returns
    ///
    /// The path to the `OpenTofu` build directory for the LXD provider.
    #[must_use]
    pub fn tofu_build_dir(&self) -> std::path::PathBuf {
        self.context().tofu_build_dir()
    }
}

/// Display implementation for user-friendly state representation
///
/// Formats the environment state in a human-readable way, including the
/// environment name, current state, and error details if applicable.
///
/// # Examples
///
/// ```text
/// Environment 'my-env' is in state: provisioning
/// Environment 'my-env' is in state: provision_failed (failed at: network timeout)
/// ```
impl std::fmt::Display for AnyEnvironmentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Environment '{}' is in state: {}",
            self.name().as_str(),
            self.state_name()
        )?;

        if let Some(error_details) = self.error_details() {
            write!(f, " (failed at: {error_details})")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ssh::SshCredentials;
    use crate::domain::environment::name::EnvironmentName;
    use crate::shared::Username;
    use std::path::PathBuf;

    /// Helper to create test SSH credentials
    fn create_test_ssh_credentials() -> SshCredentials {
        let username = Username::new("test-user".to_string()).unwrap();
        SshCredentials::new(
            PathBuf::from("/tmp/test_key"),
            PathBuf::from("/tmp/test_key.pub"),
            username,
        )
    }

    /// Helper to create a test environment in Created state
    fn create_test_environment_created() -> Environment<Created> {
        let name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_creds = create_test_ssh_credentials();
        Environment::new(name, ssh_creds, 22)
    }

    /// Helper to create a test `ProvisionFailureContext` with custom error message
    fn create_test_provision_context(error_message: &str) -> ProvisionFailureContext {
        use crate::domain::environment::TraceId;
        use crate::shared::ErrorKind;
        use chrono::Utc;
        use std::time::Duration;

        ProvisionFailureContext {
            failed_step: ProvisionStep::OpenTofuApply,
            error_kind: ErrorKind::InfrastructureOperation,
            base: BaseFailureContext {
                error_summary: error_message.to_string(),
                failed_at: Utc::now(),
                execution_started_at: Utc::now(),
                execution_duration: Duration::from_secs(0),
                trace_id: TraceId::default(),
                trace_file_path: None,
            },
        }
    }

    /// Helper to create a test `ConfigureFailureContext` with custom error message
    fn create_test_configure_context(error_message: &str) -> ConfigureFailureContext {
        use crate::domain::environment::TraceId;
        use crate::shared::ErrorKind;
        use chrono::Utc;
        use std::time::Duration;

        ConfigureFailureContext {
            failed_step: ConfigureStep::InstallDocker,
            error_kind: ErrorKind::CommandExecution,
            base: BaseFailureContext {
                error_summary: error_message.to_string(),
                failed_at: Utc::now(),
                execution_started_at: Utc::now(),
                execution_duration: Duration::from_secs(0),
                trace_id: TraceId::default(),
                trace_file_path: None,
            },
        }
    }

    /// Test module for state marker types
    ///
    /// These tests verify that state types can be created, cloned, and serialized
    /// correctly. They ensure basic functionality of the state marker types.

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
            context: create_test_provision_context("cloud_init_execution"),
        };
        assert_eq!(state.context.base.error_summary, "cloud_init_execution");
    }

    #[test]
    fn it_should_clone_provision_failed_state() {
        let state = ProvisionFailed {
            context: create_test_provision_context("cloud_init_execution"),
        };
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn it_should_create_configure_failed_state_with_context() {
        let state = ConfigureFailed {
            context: create_test_configure_context("ansible_playbook_execution"),
        };
        assert_eq!(
            state.context.base.error_summary,
            "ansible_playbook_execution"
        );
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
    fn it_should_serialize_provision_failed_state_to_json() {
        let state = ProvisionFailed {
            context: create_test_provision_context("cloud_init"),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("cloud_init"));
        assert!(json.contains("context"));
    }

    #[test]
    fn it_should_deserialize_provision_failed_state_from_json() {
        // Note: This test now uses the full context structure
        let context = create_test_provision_context("cloud_init");
        let state = ProvisionFailed {
            context: context.clone(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ProvisionFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.context.base.error_summary, "cloud_init");
    }

    #[test]
    fn it_should_serialize_configure_failed_state_to_json() {
        let state = ConfigureFailed {
            context: create_test_configure_context("ansible_playbook"),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("InstallDocker"));
        assert!(json.contains("CommandExecution"));
        let deserialized: ConfigureFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.context.base.error_summary, "ansible_playbook");
    }

    #[test]
    fn it_should_deserialize_configure_failed_state_from_json() {
        // Note: This test now uses the full context structure
        let context = create_test_configure_context("ansible_playbook");
        let state = ConfigureFailed {
            context: context.clone(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ConfigureFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.context.base.error_summary, "ansible_playbook");
    }

    // Tests for AnyEnvironmentState enum (Type Erasure)
    mod any_environment_state_tests {
        use super::*;

        // Note: Helper functions for creating test environments and contexts
        // are defined in the parent module and can be accessed via super::

        #[test]
        fn it_should_create_any_environment_state_with_created_variant() {
            let env = super::create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            assert!(matches!(any_env, AnyEnvironmentState::Created(_)));
        }

        // Note: Tests for other state variants will be added in Subtask 2
        // once we have the conversion methods (into_any()) that properly
        // create environments in different states through state transitions.

        #[test]
        fn it_should_clone_any_environment_state() {
            let env = super::create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            let cloned = any_env.clone();
            assert!(matches!(cloned, AnyEnvironmentState::Created(_)));
        }

        #[test]
        fn it_should_debug_format_any_environment_state() {
            let env = super::create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            let debug_str = format!("{any_env:?}");
            assert!(debug_str.contains("Created"));
        }

        #[test]
        fn it_should_serialize_any_environment_state_to_json() {
            let env = super::create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);
            let json = serde_json::to_string(&any_env).unwrap();
            assert!(json.contains("Created"));
        }

        // Tests for Type Conversion Methods (Subtask 2)

        // Tests for into_any() - Typed to Runtime conversions

        #[test]
        fn it_should_convert_provisioning_environment_into_any() {
            let env = super::create_test_environment_created().start_provisioning();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Provisioning(_)));
        }

        #[test]
        fn it_should_convert_provisioned_environment_into_any() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provisioned();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Provisioned(_)));
        }

        #[test]
        fn it_should_convert_configuring_environment_into_any() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Configuring(_)));
        }

        #[test]
        fn it_should_convert_configured_environment_into_any() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Configured(_)));
        }

        #[test]
        fn it_should_convert_releasing_environment_into_any() {
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provision_failed(super::create_test_provision_context("infrastructure error"));
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ProvisionFailed(_)));
        }

        #[test]
        fn it_should_convert_configure_failed_environment_into_any() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configure_failed(super::create_test_configure_context("ansible error"));
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ConfigureFailed(_)));
        }

        #[test]
        fn it_should_convert_release_failed_environment_into_any() {
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created().destroy();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Destroyed(_)));
        }

        // Tests for try_into_<state>() - Runtime to Typed conversions (successful cases)

        #[test]
        fn it_should_convert_any_to_provisioning_successfully() {
            let env = super::create_test_environment_created().start_provisioning();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioning();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_provisioned_successfully() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provisioned();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioned();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_configuring_successfully() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring();
            let any_env = env.into_any();
            let result = any_env.try_into_configuring();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_configured_successfully() {
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provision_failed(super::create_test_provision_context("test error"));
            let any_env = env.into_any();
            let result = any_env.try_into_provision_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_configure_failed_successfully() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configure_failed(super::create_test_configure_context("test error"));
            let any_env = env.into_any();
            let result = any_env.try_into_configure_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_convert_any_to_release_failed_successfully() {
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created()
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
            let env = super::create_test_environment_created().destroy();
            let any_env = env.into_any();
            let result = any_env.try_into_destroyed();
            assert!(result.is_ok());
        }

        // Tests for try_into_<state>() - Runtime to Typed conversions (failure cases)

        #[test]
        fn it_should_fail_converting_created_to_provisioning() {
            let env = super::create_test_environment_created();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioning();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("provisioning"));
            assert!(err.to_string().contains("created"));
        }

        #[test]
        fn it_should_fail_converting_provision_failed_to_provisioned() {
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provision_failed(super::create_test_provision_context("error"));
            let any_env = env.into_any();
            let result = any_env.try_into_provisioned();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("provisioned"));
            assert!(err.to_string().contains("provision_failed"));
        }

        #[test]
        fn it_should_fail_converting_destroyed_to_running() {
            let env = super::create_test_environment_created().destroy();
            let any_env = env.into_any();
            let result = any_env.try_into_running();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("running"));
            assert!(err.to_string().contains("destroyed"));
        }

        // Tests for round-trip conversions (preserving data integrity)

        #[test]
        fn it_should_preserve_error_details_in_failed_states() {
            let error_message = "infrastructure deployment failed";
            let context = super::create_test_provision_context(error_message);
            let env = super::create_test_environment_created()
                .start_provisioning()
                .provision_failed(context.clone());

            // Round-trip conversion
            let any_env = env.into_any();
            let env_restored = any_env.try_into_provision_failed().unwrap();

            assert_eq!(
                env_restored.state().context.base.error_summary,
                error_message
            );
        }

        // Tests for new utility methods

        #[test]
        fn it_should_destroy_environment_from_any_state() {
            let env = super::create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);

            let destroyed = any_env.destroy().unwrap();
            // Convert back to any state to check state name
            let destroyed_any = destroyed.into_any();
            assert_eq!(destroyed_any.state_name(), "destroyed");
        }

        #[test]
        fn it_should_get_tofu_build_dir_from_any_state() {
            let env = super::create_test_environment_created();
            let any_env = AnyEnvironmentState::Created(env);

            let build_dir = any_env.tofu_build_dir();
            assert!(build_dir.ends_with("lxd"));
        }

        #[test]
        fn it_should_error_when_destroying_already_destroyed_environment() {
            let env = super::create_test_environment_created().destroy();
            let any_env = AnyEnvironmentState::Destroyed(env);

            // This should return an error
            let result = any_env.destroy();
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(
                error.to_string(),
                "Expected state 'any state except destroyed', but found 'destroyed'"
            );
        }

        #[test]
        fn it_should_get_tofu_build_dir_from_destroyed_environment() {
            let env = super::create_test_environment_created().destroy();
            let any_env = AnyEnvironmentState::Destroyed(env);

            // This should now always return the path, even for destroyed environments
            let build_dir = any_env.tofu_build_dir();
            assert!(build_dir.ends_with("lxd"));
        }
    }

    mod introspection_tests {
        use super::{
            create_test_configure_context, create_test_environment_created,
            create_test_provision_context,
        };

        mod name {
            use super::super::EnvironmentName;

            #[test]
            fn it_should_return_environment_name_for_created_state() {
                let any_env = super::create_test_environment_created().into_any();
                let env_name = EnvironmentName::new("test-env".to_string()).unwrap();

                assert_eq!(any_env.name(), &env_name);
                assert_eq!(any_env.name().as_str(), "test-env");
            }

            #[test]
            fn it_should_return_same_name_for_provisioning_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .into_any();
                let env_name = EnvironmentName::new("test-env".to_string()).unwrap();

                assert_eq!(any_env.name(), &env_name);
            }

            #[test]
            fn it_should_return_same_name_for_error_states() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provision_failed(super::create_test_provision_context("error"))
                    .into_any();
                let env_name = EnvironmentName::new("test-env".to_string()).unwrap();

                assert_eq!(any_env.name(), &env_name);
            }
        }

        mod state_name {

            #[test]
            fn it_should_return_created_for_created_state() {
                let any_env = super::create_test_environment_created().into_any();
                assert_eq!(any_env.state_name(), "created");
            }

            #[test]
            fn it_should_return_provisioning_for_provisioning_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .into_any();
                assert_eq!(any_env.state_name(), "provisioning");
            }

            #[test]
            fn it_should_return_provisioned_for_provisioned_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .into_any();
                assert_eq!(any_env.state_name(), "provisioned");
            }

            #[test]
            fn it_should_return_configuring_for_configuring_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .into_any();
                assert_eq!(any_env.state_name(), "configuring");
            }

            #[test]
            fn it_should_return_configured_for_configured_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .into_any();
                assert_eq!(any_env.state_name(), "configured");
            }

            #[test]
            fn it_should_return_releasing_for_releasing_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .into_any();
                assert_eq!(any_env.state_name(), "releasing");
            }

            #[test]
            fn it_should_return_released_for_released_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .into_any();
                assert_eq!(any_env.state_name(), "released");
            }

            #[test]
            fn it_should_return_running_for_running_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .into_any();
                assert_eq!(any_env.state_name(), "running");
            }

            #[test]
            fn it_should_return_provision_failed_for_provision_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provision_failed(super::create_test_provision_context("error"))
                    .into_any();
                assert_eq!(any_env.state_name(), "provision_failed");
            }

            #[test]
            fn it_should_return_configure_failed_for_configure_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configure_failed(super::create_test_configure_context("error"))
                    .into_any();
                assert_eq!(any_env.state_name(), "configure_failed");
            }

            #[test]
            fn it_should_return_release_failed_for_release_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .release_failed("error".to_string())
                    .into_any();
                assert_eq!(any_env.state_name(), "release_failed");
            }

            #[test]
            fn it_should_return_run_failed_for_run_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .run_failed("error".to_string())
                    .into_any();
                assert_eq!(any_env.state_name(), "run_failed");
            }

            #[test]
            fn it_should_return_destroyed_for_destroyed_state() {
                let any_env = super::create_test_environment_created()
                    .destroy()
                    .into_any();
                assert_eq!(any_env.state_name(), "destroyed");
            }
        }

        mod is_success_state {

            #[test]
            fn it_should_return_true_for_created_state() {
                let any_env = super::create_test_environment_created().into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_provisioning_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_provisioned_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_configuring_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_configured_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_releasing_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_released_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_running_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_true_for_destroyed_state() {
                let any_env = super::create_test_environment_created()
                    .destroy()
                    .into_any();
                assert!(any_env.is_success_state());
            }

            #[test]
            fn it_should_return_false_for_provision_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provision_failed(super::create_test_provision_context("error"))
                    .into_any();
                assert!(!any_env.is_success_state());
            }

            #[test]
            fn it_should_return_false_for_configure_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configure_failed(super::create_test_configure_context("error"))
                    .into_any();
                assert!(!any_env.is_success_state());
            }

            #[test]
            fn it_should_return_false_for_release_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .release_failed("error".to_string())
                    .into_any();
                assert!(!any_env.is_success_state());
            }

            #[test]
            fn it_should_return_false_for_run_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .run_failed("error".to_string())
                    .into_any();
                assert!(!any_env.is_success_state());
            }
        }

        mod is_error_state {

            #[test]
            fn it_should_return_false_for_success_states() {
                let success_states = vec![
                    super::create_test_environment_created().into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .released()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .released()
                        .start_running()
                        .into_any(),
                    super::create_test_environment_created()
                        .destroy()
                        .into_any(),
                ];

                for state in success_states {
                    assert!(!state.is_error_state());
                }
            }

            #[test]
            fn it_should_return_true_for_provision_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provision_failed(super::create_test_provision_context("error"))
                    .into_any();
                assert!(any_env.is_error_state());
            }

            #[test]
            fn it_should_return_true_for_configure_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configure_failed(super::create_test_configure_context("error"))
                    .into_any();
                assert!(any_env.is_error_state());
            }

            #[test]
            fn it_should_return_true_for_release_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .release_failed("error".to_string())
                    .into_any();
                assert!(any_env.is_error_state());
            }

            #[test]
            fn it_should_return_true_for_run_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .run_failed("error".to_string())
                    .into_any();
                assert!(any_env.is_error_state());
            }
        }

        mod is_terminal_state {

            #[test]
            fn it_should_return_false_for_transient_states() {
                let transient_states = vec![
                    super::create_test_environment_created().into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .released()
                        .into_any(),
                ];

                for state in transient_states {
                    assert!(!state.is_terminal_state());
                }
            }

            #[test]
            fn it_should_return_true_for_running_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .into_any();
                assert!(any_env.is_terminal_state());
            }

            #[test]
            fn it_should_return_true_for_destroyed_state() {
                let any_env = super::create_test_environment_created()
                    .destroy()
                    .into_any();
                assert!(any_env.is_terminal_state());
            }

            #[test]
            fn it_should_return_true_for_provision_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provision_failed(super::create_test_provision_context("error"))
                    .into_any();
                assert!(any_env.is_terminal_state());
            }

            #[test]
            fn it_should_return_true_for_configure_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configure_failed(super::create_test_configure_context("error"))
                    .into_any();
                assert!(any_env.is_terminal_state());
            }

            #[test]
            fn it_should_return_true_for_release_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .release_failed("error".to_string())
                    .into_any();
                assert!(any_env.is_terminal_state());
            }

            #[test]
            fn it_should_return_true_for_run_failed_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .run_failed("error".to_string())
                    .into_any();
                assert!(any_env.is_terminal_state());
            }
        }

        mod error_details {

            #[test]
            fn it_should_return_none_for_success_states() {
                let success_states = vec![
                    super::create_test_environment_created().into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .released()
                        .into_any(),
                    super::create_test_environment_created()
                        .start_provisioning()
                        .provisioned()
                        .start_configuring()
                        .configured()
                        .start_releasing()
                        .released()
                        .start_running()
                        .into_any(),
                    super::create_test_environment_created()
                        .destroy()
                        .into_any(),
                ];

                for state in success_states {
                    assert!(state.error_details().is_none());
                }
            }

            #[test]
            fn it_should_return_error_message_for_provision_failed_state() {
                let error_message = "network timeout during provisioning";
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provision_failed(super::create_test_provision_context(error_message))
                    .into_any();

                assert_eq!(any_env.error_details(), Some(error_message));
            }

            #[test]
            fn it_should_return_error_message_for_configure_failed_state() {
                let error_message = "ansible playbook failed";
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configure_failed(super::create_test_configure_context(error_message))
                    .into_any();

                assert_eq!(any_env.error_details(), Some(error_message));
            }

            #[test]
            fn it_should_return_error_message_for_release_failed_state() {
                let error_message = "release process error";
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .release_failed(error_message.to_string())
                    .into_any();

                assert_eq!(any_env.error_details(), Some(error_message));
            }

            #[test]
            fn it_should_return_error_message_for_run_failed_state() {
                let error_message = "application crash";
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .run_failed(error_message.to_string())
                    .into_any();

                assert_eq!(any_env.error_details(), Some(error_message));
            }
        }

        mod display {

            #[test]
            fn it_should_format_success_state_without_error_details() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .into_any();

                let output = format!("{any_env}");
                assert_eq!(output, "Environment 'test-env' is in state: provisioning");
            }

            #[test]
            fn it_should_format_running_state() {
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured()
                    .start_releasing()
                    .released()
                    .start_running()
                    .into_any();

                let output = format!("{any_env}");
                assert_eq!(output, "Environment 'test-env' is in state: running");
            }

            #[test]
            fn it_should_format_error_state_with_error_details() {
                let error_message = "network timeout";
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provision_failed(super::create_test_provision_context(error_message))
                    .into_any();

                let output = format!("{any_env}");
                assert_eq!(
                    output,
                    format!("Environment 'test-env' is in state: provision_failed (failed at: {error_message})")
                );
            }

            #[test]
            fn it_should_format_configure_failed_with_error_details() {
                let error_message = "ansible error";
                let any_env = super::create_test_environment_created()
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configure_failed(super::create_test_configure_context(error_message))
                    .into_any();

                let output = format!("{any_env}");
                assert_eq!(
                    output,
                    format!("Environment 'test-env' is in state: configure_failed (failed at: {error_message})")
                );
            }

            #[test]
            fn it_should_format_destroyed_state() {
                let any_env = super::create_test_environment_created()
                    .destroy()
                    .into_any();

                let output = format!("{any_env}");
                assert_eq!(output, "Environment 'test-env' is in state: destroyed");
            }

            #[test]
            fn it_should_work_with_println_macro() {
                let any_env = super::create_test_environment_created().into_any();

                // This test verifies that Display can be used with println!
                // We can't capture println output easily, but we can verify it compiles
                let output = format!("{any_env}");
                assert!(output.contains("test-env"));
                assert!(output.contains("created"));
            }
        }
    }
}
