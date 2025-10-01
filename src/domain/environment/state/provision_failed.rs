//! `ProvisionFailed` State
//!
//! Error state - Infrastructure provisioning failed
//!
//! The provision command failed during execution. The `failed_step` field
//! contains the name of the step that caused the failure, providing context
//! for debugging and recovery.
//!
//! **Recovery Options:**
//! - Destroy and recreate the environment
//! - Manual inspection and repair (advanced users)

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, StateTypeError};
use crate::domain::environment::Environment;

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

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<ProvisionFailed> {
    /// Converts typed `Environment<ProvisionFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ProvisionFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_provision_failed)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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

    mod conversion_tests {
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

        fn create_test_environment_provision_failed() -> Environment<ProvisionFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provision_failed("test error".to_string())
        }

        #[test]
        fn it_should_convert_provision_failed_environment_into_any() {
            let env = create_test_environment_provision_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ProvisionFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_provision_failed_successfully() {
            let env = create_test_environment_provision_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_provision_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_preserve_error_details_in_failed_states() {
            let error_message = "infrastructure deployment failed";
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let env = Environment::new(name, ssh_creds)
                .start_provisioning()
                .provision_failed(error_message.to_string());

            // Round-trip conversion
            let any_env = env.into_any();
            let env_restored = any_env.try_into_provision_failed().unwrap();

            assert_eq!(env_restored.state().failed_step, error_message);
        }
    }
}
