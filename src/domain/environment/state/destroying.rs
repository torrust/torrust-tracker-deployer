//! Destroying State
//!
//! Intermediate state - Infrastructure destruction in progress
//!
//! The environment is actively being destroyed (VM deletion, resource cleanup, etc.).
//! This state indicates that the destroy command has started but not yet completed.
//!
//! **Valid Transitions:**
//! - Success: `Destroyed`
//! - Failure: `DestroyFailed`

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{
    AnyEnvironmentState, DestroyFailed, Destroyed, StateTypeError,
};
use crate::domain::environment::Environment;

/// Intermediate state - Infrastructure destruction in progress
///
/// The environment is actively being destroyed (VM deletion, resource cleanup, etc.).
/// This state indicates that the destroy command has started but not yet completed.
///
/// **Valid Transitions:**
/// - Success: `Destroyed`
/// - Failure: `DestroyFailed`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destroying;

// State transition implementations
impl Environment<Destroying> {
    /// Transitions from Destroying to Destroyed state
    ///
    /// This method indicates that infrastructure destruction completed successfully.
    #[must_use]
    pub fn destroyed(self) -> Environment<Destroyed> {
        self.with_state(Destroyed)
    }

    /// Transitions from Destroying to `DestroyFailed` state
    ///
    /// This method indicates that infrastructure destruction failed.
    /// The context parameter provides structured error information including
    /// the failed step, error classification, and trace reference.
    #[must_use]
    pub fn destroy_failed(
        self,
        context: crate::domain::environment::state::DestroyFailureContext,
    ) -> Environment<DestroyFailed> {
        self.with_state(DestroyFailed { context })
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Destroying> {
    /// Converts typed `Environment<Destroying>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Destroying(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_destroying)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<Destroying>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Destroying` state.
    pub fn try_into_destroying(self) -> Result<Environment<Destroying>, StateTypeError> {
        match self {
            Self::Destroying(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "destroying",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_destroying_state() {
        let state = Destroying;
        assert_eq!(state, Destroying);
    }

    mod conversion_tests {
        use super::*;
        use crate::adapters::ssh::SshCredentials;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::provider::{LxdConfig, ProviderConfig};
        use crate::domain::ProfileName;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn default_lxd_provider_config(env_name: &EnvironmentName) -> ProviderConfig {
            ProviderConfig::Lxd(LxdConfig {
                profile_name: ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap(),
            })
        }

        fn create_test_ssh_credentials() -> SshCredentials {
            let username = Username::new("test-user".to_string()).unwrap();
            SshCredentials::new(
                PathBuf::from("/tmp/test_key"),
                PathBuf::from("/tmp/test_key.pub"),
                username,
            )
        }

        fn create_test_environment_destroying() -> Environment<Destroying> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(
                name.clone(),
                default_lxd_provider_config(&name),
                ssh_creds,
                22,
            )
            .start_provisioning()
            .provisioned()
            .start_destroying()
        }

        #[test]
        fn it_should_convert_destroying_environment_into_any() {
            let env = create_test_environment_destroying();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Destroying(_)));
        }

        #[test]
        fn it_should_convert_any_to_destroying_successfully() {
            let env = create_test_environment_destroying();
            let any_env = env.into_any();
            let result = any_env.try_into_destroying();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_fail_converting_created_to_destroying() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let env = Environment::new(
                name.clone(),
                default_lxd_provider_config(&name),
                ssh_creds,
                22,
            );
            let any_env = env.into_any();
            let result = any_env.try_into_destroying();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("destroying"));
            assert!(err.to_string().contains("created"));
        }
    }

    mod state_transitions {
        use super::super::*;
        use crate::adapters::ssh::SshCredentials;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Destroyed;
        use crate::domain::provider::{LxdConfig, ProviderConfig};
        use crate::domain::ProfileName;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn default_lxd_provider_config(env_name: &EnvironmentName) -> ProviderConfig {
            ProviderConfig::Lxd(LxdConfig {
                profile_name: ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap(),
            })
        }

        fn create_test_ssh_credentials() -> SshCredentials {
            let username = Username::new("test-user".to_string()).unwrap();
            SshCredentials::new(
                PathBuf::from("/tmp/test_key"),
                PathBuf::from("/tmp/test_key.pub"),
                username,
            )
        }

        fn create_test_environment_destroying() -> Environment<Destroying> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(
                name.clone(),
                default_lxd_provider_config(&name),
                ssh_creds,
                22,
            )
            .start_provisioning()
            .provisioned()
            .start_destroying()
        }

        #[test]
        fn it_should_transition_from_destroying_to_destroyed() {
            let env = create_test_environment_destroying();
            let env = env.destroyed();

            assert_eq!(*env.state(), Destroyed);
            assert_eq!(env.name().as_str(), "test-env");
        }

        #[test]
        fn it_should_transition_from_destroying_to_destroy_failed() {
            use crate::domain::environment::state::{
                BaseFailureContext, DestroyFailureContext, DestroyStep,
            };
            use crate::domain::environment::TraceId;
            use crate::shared::ErrorKind;
            use chrono::Utc;
            use std::time::Duration;

            let env = create_test_environment_destroying();
            let context = DestroyFailureContext {
                failed_step: DestroyStep::DestroyInfrastructure,
                error_kind: ErrorKind::InfrastructureOperation,
                base: BaseFailureContext {
                    error_summary: "infrastructure_destroy_failed".to_string(),
                    failed_at: Utc::now(),
                    execution_started_at: Utc::now(),
                    execution_duration: Duration::from_secs(30),
                    trace_id: TraceId::new(),
                    trace_file_path: None,
                },
            };
            let env = env.destroy_failed(context.clone());

            assert_eq!(
                env.state().context.failed_step,
                DestroyStep::DestroyInfrastructure
            );
            assert_eq!(env.name().as_str(), "test-env");
        }
    }
}
