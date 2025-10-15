//! Released State
//!
//! Final state - Release preparation completed successfully
//!
//! The environment is fully prepared and ready to run the application.
//!
//! **Valid Transitions:**
//! - `Running` (start application)

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, Running, StateTypeError};
use crate::domain::environment::Environment;

/// Final state - Release preparation completed successfully
///
/// The environment is fully prepared and ready to run the application.
///
/// **Valid Transitions:**
/// - `Running` (start application)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Released;

// State transition implementations
impl Environment<Released> {
    /// Transitions from Released to Running state
    ///
    /// This method indicates that the application has started running.
    #[must_use]
    pub fn start_running(self) -> Environment<Running> {
        self.with_state(Running)
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Released> {
    /// Converts typed `Environment<Released>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Released(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_released)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_released_state() {
        let state = Released;
        assert_eq!(state, Released);
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

        fn create_test_environment_released() -> Environment<Released> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds, 22)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
        }

        #[test]
        fn it_should_convert_released_environment_into_any() {
            let env = create_test_environment_released();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Released(_)));
        }

        #[test]
        fn it_should_convert_any_to_released_successfully() {
            let env = create_test_environment_released();
            let any_env = env.into_any();
            let result = any_env.try_into_released();
            assert!(result.is_ok());
        }
    }

    mod transition_tests {
        use super::*;
        use crate::adapters::ssh::SshCredentials;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Running;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_environment() -> Environment<Released> {
            let env_name = EnvironmentName::new("test-state".to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let ssh_credentials = SshCredentials::new(
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
                ssh_username,
            );
            Environment::new(env_name, ssh_credentials, 22)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
        }

        #[test]
        fn it_should_transition_from_released_to_running() {
            let env = create_test_environment();
            let env = env.start_running();

            assert_eq!(*env.state(), Running);
            assert_eq!(env.name().as_str(), "test-state");
        }
    }
}
