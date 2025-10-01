//! Releasing State
//!
//! Intermediate state - Release preparation in progress
//!
//! The environment is being prepared for production release (building artifacts,
//! final checks, etc.).
//!
//! **Valid Transitions:**
//! - Success: `Released`
//! - Failure: `ReleaseFailed`

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{
    AnyEnvironmentState, ReleaseFailed, Released, StateTypeError,
};
use crate::domain::environment::Environment;

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

// State transition implementations
impl Environment<Releasing> {
    /// Transitions from Releasing to Released state
    ///
    /// This method indicates that release preparation completed successfully.
    #[must_use]
    pub fn released(self) -> Environment<Released> {
        self.with_state(Released)
    }

    /// Transitions from Releasing to `ReleaseFailed` state
    ///
    /// This method indicates that release preparation failed at a specific step.
    #[must_use]
    pub fn release_failed(self, failed_step: String) -> Environment<ReleaseFailed> {
        self.with_state(ReleaseFailed { failed_step })
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Releasing> {
    /// Converts typed `Environment<Releasing>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Releasing(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_releasing)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_releasing_state() {
        let state = Releasing;
        assert_eq!(state, Releasing);
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

        fn create_test_environment_releasing() -> Environment<Releasing> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
        }

        #[test]
        fn it_should_convert_releasing_environment_into_any() {
            let env = create_test_environment_releasing();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Releasing(_)));
        }

        #[test]
        fn it_should_convert_any_to_releasing_successfully() {
            let env = create_test_environment_releasing();
            let any_env = env.into_any();
            let result = any_env.try_into_releasing();
            assert!(result.is_ok());
        }
    }

    mod transition_tests {
        use super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Released;
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_environment() -> Environment<Releasing> {
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
        }

        #[test]
        fn it_should_transition_from_releasing_to_released() {
            let env = create_test_environment();
            let env = env.released();

            assert_eq!(*env.state(), Released);
            assert_eq!(env.name().as_str(), "test-state");
        }

        #[test]
        fn it_should_transition_from_releasing_to_release_failed() {
            let env = create_test_environment();
            let env = env.release_failed("build_artifacts_missing".to_string());

            assert_eq!(env.state().failed_step, "build_artifacts_missing");
            assert_eq!(env.name().as_str(), "test-state");
        }
    }
}
