//! Provisioned State
//!
//! Final state - Infrastructure provisioning completed successfully
//!
//! The VM instance is running and accessible. The environment is ready for
//! application configuration.
//!
//! **Valid Transitions:**
//! - `Configuring` (start application configuration)

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, Configuring, StateTypeError};
use crate::domain::environment::Environment;

/// Final state - Infrastructure provisioning completed successfully
///
/// The VM instance is running and accessible. The environment is ready for
/// application configuration.
///
/// **Valid Transitions:**
/// - `Configuring` (start application configuration)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provisioned;

// State transition implementations
impl Environment<Provisioned> {
    /// Transitions from Provisioned to Configuring state
    ///
    /// This method indicates that application configuration has begun.
    #[must_use]
    pub fn start_configuring(self) -> Environment<Configuring> {
        self.with_state(Configuring)
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Provisioned> {
    /// Converts typed `Environment<Provisioned>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Provisioned(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_provisioned)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<Provisioned>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Provisioned` state.
    pub fn try_into_provisioned(self) -> Result<Environment<Provisioned>, StateTypeError> {
        match self {
            Self::Provisioned(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "provisioned",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_provisioned_state() {
        let state = Provisioned;
        assert_eq!(state, Provisioned);
    }

    mod conversion_tests {
        use super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::ProvisionFailureContext;
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

        fn create_test_environment_provisioned() -> Environment<Provisioned> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provisioned()
        }

        fn create_test_provision_context() -> ProvisionFailureContext {
            use crate::domain::environment::state::{ProvisionErrorKind, ProvisionStep};
            use crate::domain::environment::TraceId;
            use chrono::Utc;
            use std::time::Duration;

            ProvisionFailureContext {
                failed_step: ProvisionStep::CloudInitWait,
                error_kind: ProvisionErrorKind::ConfigurationTimeout,
                error_summary: "error".to_string(),
                failed_at: Utc::now(),
                execution_started_at: Utc::now(),
                execution_duration: Duration::from_secs(0),
                trace_id: TraceId::default(),
                trace_file_path: None,
            }
        }

        #[test]
        fn it_should_convert_provisioned_environment_into_any() {
            let env = create_test_environment_provisioned();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Provisioned(_)));
        }

        #[test]
        fn it_should_convert_any_to_provisioned_successfully() {
            let env = create_test_environment_provisioned();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioned();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_fail_converting_provision_failed_to_provisioned() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let env = Environment::new(name, ssh_creds)
                .start_provisioning()
                .provision_failed(create_test_provision_context());
            let any_env = env.into_any();
            let result = any_env.try_into_provisioned();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("provisioned"));
            assert!(err.to_string().contains("provision_failed"));
        }
    }

    mod transition_tests {
        use super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::{Configuring, Destroyed};
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_environment() -> Environment<Provisioned> {
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
        }

        #[test]
        fn it_should_transition_from_provisioned_to_configuring() {
            let env = create_test_environment();
            let env = env.start_configuring();

            assert_eq!(*env.state(), Configuring);
            assert_eq!(env.name().as_str(), "test-state");
        }

        #[test]
        fn it_should_transition_to_destroyed_from_provisioned() {
            let env = create_test_environment();
            let env = env.destroy();

            assert_eq!(*env.state(), Destroyed);
            assert_eq!(env.name().as_str(), "test-state");
        }
    }
}
