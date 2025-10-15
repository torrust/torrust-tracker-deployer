//! `RunFailed` State
//!
//! Error state - Application runtime failed
//!
//! The application encountered a runtime error. The `failed_step` field
//! contains the name of the operation that caused the failure.
//!
//! **Recovery Options:**
//! - Restart the application
//! - Destroy and recreate the environment

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, StateTypeError};
use crate::domain::environment::Environment;

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

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<RunFailed> {
    /// Converts typed `Environment<RunFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::RunFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_run_failed)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_run_failed_state_with_context() {
        let state = RunFailed {
            failed_step: "application_startup".to_string(),
        };
        assert_eq!(state.failed_step, "application_startup");
    }

    mod conversion_tests {
        use super::*;
        use crate::adapters::ssh::SshCredentials;
        use crate::domain::environment::name::EnvironmentName;
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

        fn create_test_environment_run_failed() -> Environment<RunFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds, 22)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running()
                .run_failed("test error".to_string())
        }

        #[test]
        fn it_should_convert_run_failed_environment_into_any() {
            let env = create_test_environment_run_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::RunFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_run_failed_successfully() {
            let env = create_test_environment_run_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_run_failed();
            assert!(result.is_ok());
        }
    }
}
