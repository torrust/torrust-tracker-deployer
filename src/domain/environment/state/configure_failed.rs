//! `ConfigureFailed` State
//!
//! Error state - Application configuration failed
//!
//! The configuration command failed during execution. The `failed_step` field
//! contains the name of the step that caused the failure.
//!
//! **Recovery Options:**
//! - Destroy and recreate the environment
//! - Manual configuration correction (advanced users)

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, StateTypeError};
use crate::domain::environment::Environment;

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

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<ConfigureFailed> {
    /// Converts typed `Environment<ConfigureFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ConfigureFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_configure_failed)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_configure_failed_state_with_context() {
        let state = ConfigureFailed {
            failed_step: "ansible_playbook_execution".to_string(),
        };
        assert_eq!(state.failed_step, "ansible_playbook_execution");
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

        fn create_test_environment_configure_failed() -> Environment<ConfigureFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configure_failed("test error".to_string())
        }

        #[test]
        fn it_should_convert_configure_failed_environment_into_any() {
            let env = create_test_environment_configure_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ConfigureFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_configure_failed_successfully() {
            let env = create_test_environment_configure_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_configure_failed();
            assert!(result.is_ok());
        }
    }
}
