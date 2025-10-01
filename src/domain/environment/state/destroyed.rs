//! Destroyed State
//!
//! Terminal state - Environment has been destroyed
//!
//! All infrastructure resources have been released and the environment no longer
//! exists. This is the final state in the lifecycle.
//!
//! **No Valid Transitions:** This is a terminal state.

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, StateTypeError};
use crate::domain::environment::Environment;

/// Terminal state - Environment has been destroyed
///
/// All infrastructure resources have been released and the environment no longer
/// exists. This is the final state in the lifecycle.
///
/// **No Valid Transitions:** This is a terminal state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destroyed;

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Destroyed> {
    /// Converts typed `Environment<Destroyed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Destroyed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_destroyed)
impl AnyEnvironmentState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_destroyed_state() {
        let state = Destroyed;
        assert_eq!(state, Destroyed);
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

        fn create_test_environment_destroyed() -> Environment<Destroyed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds).destroy()
        }

        #[test]
        fn it_should_convert_destroyed_environment_into_any() {
            let env = create_test_environment_destroyed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Destroyed(_)));
        }

        #[test]
        fn it_should_convert_any_to_destroyed_successfully() {
            let env = create_test_environment_destroyed();
            let any_env = env.into_any();
            let result = any_env.try_into_destroyed();
            assert!(result.is_ok());
        }
    }
}
