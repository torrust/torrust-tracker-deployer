//! Running State
//!
//! Final state - Application is running
//!
//! The application is actively running in the environment. This is the target
//! operational state.
//!
//! **Valid Transitions:**
//! - `RunFailed` (if runtime error occurs)
//! - `Destroyed` (when shutting down)

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, RunFailed, StateTypeError};
use crate::domain::environment::Environment;

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

// State transition implementations
impl Environment<Running> {
    /// Transitions from Running to `RunFailed` state
    ///
    /// This method indicates that the application encountered a runtime failure.
    #[must_use]
    pub fn run_failed(self, failed_step: String) -> Environment<RunFailed> {
        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: RunFailed { failed_step },
        }
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Running> {
    /// Converts typed `Environment<Running>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Running(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_running)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_running_state() {
        let state = Running;
        assert_eq!(state, Running);
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

        fn create_test_environment_running() -> Environment<Running> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running()
        }

        #[test]
        fn it_should_convert_running_environment_into_any() {
            let env = create_test_environment_running();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Running(_)));
        }

        #[test]
        fn it_should_convert_any_to_running_successfully() {
            let env = create_test_environment_running();
            let any_env = env.into_any();
            let result = any_env.try_into_running();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_fail_converting_destroyed_to_running() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let env = Environment::new(name, ssh_creds).destroy();
            let any_env = env.into_any();
            let result = any_env.try_into_running();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("running"));
            assert!(err.to_string().contains("destroyed"));
        }
    }

    mod transition_tests {
        use super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Destroyed;
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_environment() -> Environment<Running> {
            let env_name = EnvironmentName::new("test-state".to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let ssh_credentials = SshCredentials::new(
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
                ssh_username,
            );
            Environment::new(env_name, ssh_credentials)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running()
        }

        #[test]
        fn it_should_transition_from_running_to_run_failed() {
            let env = create_test_environment();
            let env = env.run_failed("application_crash".to_string());

            assert_eq!(env.state().failed_step, "application_crash");
            assert_eq!(env.name().as_str(), "test-state");
        }

        #[test]
        fn it_should_transition_to_destroyed_from_running() {
            let env = create_test_environment();
            let env = env.destroy();

            assert_eq!(*env.state(), Destroyed);
            assert_eq!(env.name().as_str(), "test-state");
        }
    }
}
