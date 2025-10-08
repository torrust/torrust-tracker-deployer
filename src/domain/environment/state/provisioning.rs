//! Provisioning State
//!
//! Intermediate state - Infrastructure provisioning in progress
//!
//! The environment is actively being provisioned (VM creation, network setup, etc.).
//! This state indicates that the provision command has started but not yet completed.
//!
//! **Valid Transitions:**
//! - Success: `Provisioned`
//! - Failure: `ProvisionFailed`

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{
    AnyEnvironmentState, ProvisionFailed, Provisioned, StateTypeError,
};
use crate::domain::environment::Environment;

/// Intermediate state - Infrastructure provisioning in progress
///
/// The environment is actively being provisioned (VM creation, network setup, etc.).
/// This state indicates that the provision command has started but not yet completed.
///
/// **Valid Transitions:**
/// - Success: `Provisioned`
/// - Failure: `ProvisionFailed`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provisioning;

// State transition implementations
impl Environment<Provisioning> {
    /// Transitions from Provisioning to Provisioned state
    ///
    /// This method indicates that infrastructure provisioning completed successfully.
    #[must_use]
    pub fn provisioned(self) -> Environment<Provisioned> {
        self.with_state(Provisioned)
    }

    /// Transitions from Provisioning to `ProvisionFailed` state
    ///
    /// This method indicates that infrastructure provisioning failed.
    /// The context parameter provides structured error information including
    /// the failed step, error classification, and trace reference.
    #[must_use]
    pub fn provision_failed(
        self,
        context: crate::domain::environment::state::ProvisionFailureContext,
    ) -> Environment<ProvisionFailed> {
        self.with_state(ProvisionFailed { context })
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Provisioning> {
    /// Converts typed `Environment<Provisioning>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Provisioning(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_provisioning)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<Provisioning>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Provisioning` state.
    pub fn try_into_provisioning(self) -> Result<Environment<Provisioning>, StateTypeError> {
        match self {
            Self::Provisioning(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "provisioning",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_provisioning_state() {
        let state = Provisioning;
        assert_eq!(state, Provisioning);
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

        fn create_test_environment_provisioning() -> Environment<Provisioning> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds, 22).start_provisioning()
        }

        #[test]
        fn it_should_convert_provisioning_environment_into_any() {
            let env = create_test_environment_provisioning();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Provisioning(_)));
        }

        #[test]
        fn it_should_convert_any_to_provisioning_successfully() {
            let env = create_test_environment_provisioning();
            let any_env = env.into_any();
            let result = any_env.try_into_provisioning();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_fail_converting_created_to_provisioning() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let env = Environment::new(name, ssh_creds, 22);
            let any_env = env.into_any();
            let result = any_env.try_into_provisioning();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("provisioning"));
            assert!(err.to_string().contains("created"));
        }
    }

    mod state_transitions {
        use super::super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Provisioned;
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

        fn create_test_environment_provisioning() -> Environment<Provisioning> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds, 22).start_provisioning()
        }

        #[test]
        fn it_should_transition_from_provisioning_to_provisioned() {
            let env = create_test_environment_provisioning();
            let env = env.provisioned();

            assert_eq!(*env.state(), Provisioned);
            assert_eq!(env.name().as_str(), "test-env");
        }

        #[test]
        fn it_should_transition_from_provisioning_to_provision_failed() {
            use crate::domain::environment::state::{
                BaseFailureContext, ProvisionFailureContext, ProvisionStep,
            };
            use crate::domain::environment::TraceId;
            use crate::shared::ErrorKind;
            use chrono::Utc;
            use std::time::Duration;

            let env = create_test_environment_provisioning();
            let context = ProvisionFailureContext {
                failed_step: ProvisionStep::CloudInitWait,
                error_kind: ErrorKind::Timeout,
                base: BaseFailureContext {
                    error_summary: "cloud_init_timeout".to_string(),
                    failed_at: Utc::now(),
                    execution_started_at: Utc::now(),
                    execution_duration: Duration::from_secs(30),
                    trace_id: TraceId::new(),
                    trace_file_path: None,
                },
            };
            let env = env.provision_failed(context.clone());

            assert_eq!(
                env.state().context.failed_step,
                ProvisionStep::CloudInitWait
            );
            assert_eq!(env.name().as_str(), "test-env");
        }
    }
}
