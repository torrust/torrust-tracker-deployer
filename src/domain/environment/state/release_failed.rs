//! `ReleaseFailed` State
//!
//! Error state - Release preparation failed
//!
//! The release command failed during execution. The `failed_step` field
//! contains the name of the step that caused the failure.
//!
//! **Recovery Options:**
//! - Destroy and recreate the environment
//! - Manual release correction (advanced users)

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, StateTypeError};
use crate::domain::environment::Environment;

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

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<ReleaseFailed> {
    /// Converts typed `Environment<ReleaseFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ReleaseFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_release_failed)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<ReleaseFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `ReleaseFailed` state.
    pub fn try_into_release_failed(self) -> Result<Environment<ReleaseFailed>, StateTypeError> {
        match self {
            Self::ReleaseFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "release_failed",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_release_failed_state_with_context() {
        let state = ReleaseFailed {
            failed_step: "build_artifacts".to_string(),
        };
        assert_eq!(state.failed_step, "build_artifacts");
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

        fn create_test_environment_release_failed() -> Environment<ReleaseFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .release_failed("test error".to_string())
        }

        #[test]
        fn it_should_convert_release_failed_environment_into_any() {
            let env = create_test_environment_release_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ReleaseFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_release_failed_successfully() {
            let env = create_test_environment_release_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_release_failed();
            assert!(result.is_ok());
        }
    }
}
