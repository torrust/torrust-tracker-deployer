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
//! use torrust_tracker_deploy::domain::environment_state::{Created, Provisioning};
//!
//! // State types are used as type parameters for Environment<S>
//! // let env: Environment<Created> = Environment::new(name, credentials);
//! // let env: Environment<Provisioning> = env.start_provisioning();
//! ```

use serde::{Deserialize, Serialize};

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
}
