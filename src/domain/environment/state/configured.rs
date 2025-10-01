//! Configured State
//!
//! Final state - Application configuration completed successfully
//!
//! All application configuration has been applied. The environment is ready
//! for release preparation.
//!
//! **Valid Transitions:**
//! - `Releasing` (start release process)

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, Releasing, StateTypeError};
use crate::domain::environment::Environment;

/// Final state - Application configuration completed successfully
///
/// All application configuration has been applied. The environment is ready
/// for release preparation.
///
/// **Valid Transitions:**
/// - `Releasing` (start release process)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configured;

// State transition implementations
impl Environment<Configured> {
    /// Transitions from Configured to Releasing state
    ///
    /// This method indicates that release preparation has begun.
    #[must_use]
    pub fn start_releasing(self) -> Environment<Releasing> {
        self.with_state(Releasing)
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Configured> {
    /// Converts typed `Environment<Configured>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Configured(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_configured)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_configured_state() {
        let state = Configured;
        assert_eq!(state, Configured);
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

        fn create_test_environment_configured() -> Environment<Configured> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
        }

        #[test]
        fn it_should_convert_configured_environment_into_any() {
            let env = create_test_environment_configured();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Configured(_)));
        }

        #[test]
        fn it_should_convert_any_to_configured_successfully() {
            let env = create_test_environment_configured();
            let any_env = env.into_any();
            let result = any_env.try_into_configured();
            assert!(result.is_ok());
        }
    }

    mod transition_tests {
        use super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Releasing;
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_environment() -> Environment<Configured> {
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
        }

        #[test]
        fn it_should_transition_from_configured_to_releasing() {
            let env = create_test_environment();
            let env = env.start_releasing();

            assert_eq!(*env.state(), Releasing);
            assert_eq!(env.name().as_str(), "test-state");
        }
    }
}
