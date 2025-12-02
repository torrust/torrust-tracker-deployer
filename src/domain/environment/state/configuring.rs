//! Configuring State
//!
//! Intermediate state - Application configuration in progress
//!
//! The environment is being configured with application-specific settings
//! (Ansible playbooks, configuration files, etc.).
//!
//! **Valid Transitions:**
//! - Success: `Configured`
//! - Failure: `ConfigureFailed`

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{
    AnyEnvironmentState, ConfigureFailed, Configured, StateTypeError,
};
use crate::domain::environment::Environment;

/// Intermediate state - Application configuration in progress
///
/// The environment is being configured with application-specific settings
/// (Ansible playbooks, configuration files, etc.).
///
/// **Valid Transitions:**
/// - Success: `Configured`
/// - Failure: `ConfigureFailed`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuring;

// State transition implementations
impl Environment<Configuring> {
    /// Transitions from Configuring to Configured state
    ///
    /// This method indicates that application configuration completed successfully.
    #[must_use]
    pub fn configured(self) -> Environment<Configured> {
        self.with_state(Configured)
    }

    /// Transitions from Configuring to `ConfigureFailed` state
    ///
    /// This method indicates that application configuration failed.
    /// The context parameter provides structured error information including
    /// the failed step, error classification, and trace reference.
    #[must_use]
    pub fn configure_failed(
        self,
        context: crate::domain::environment::state::ConfigureFailureContext,
    ) -> Environment<ConfigureFailed> {
        self.with_state(ConfigureFailed { context })
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Configuring> {
    /// Converts typed `Environment<Configuring>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Configuring(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_configuring)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<Configuring>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Configuring` state.
    pub fn try_into_configuring(self) -> Result<Environment<Configuring>, StateTypeError> {
        match self {
            Self::Configuring(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "configuring",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_configuring_state() {
        let state = Configuring;
        assert_eq!(state, Configuring);
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

        fn create_test_environment_configuring() -> Environment<Configuring> {
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
            .start_configuring()
        }

        #[test]
        fn it_should_convert_configuring_environment_into_any() {
            let env = create_test_environment_configuring();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Configuring(_)));
        }

        #[test]
        fn it_should_convert_any_to_configuring_successfully() {
            let env = create_test_environment_configuring();
            let any_env = env.into_any();
            let result = any_env.try_into_configuring();
            assert!(result.is_ok());
        }
    }

    mod transition_tests {
        use super::*;
        use crate::adapters::ssh::SshCredentials;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Configured;
        use crate::domain::provider::{LxdConfig, ProviderConfig};
        use crate::domain::ProfileName;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn default_lxd_provider_config(env_name: &EnvironmentName) -> ProviderConfig {
            ProviderConfig::Lxd(LxdConfig {
                profile_name: ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap(),
            })
        }

        fn create_test_environment() -> Environment<Configuring> {
            let env_name = EnvironmentName::new("test-state".to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let ssh_credentials = SshCredentials::new(
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
                ssh_username,
            );
            Environment::new(
                env_name.clone(),
                default_lxd_provider_config(&env_name),
                ssh_credentials,
                22,
            )
            .start_provisioning()
            .provisioned()
            .start_configuring()
        }

        #[test]
        fn it_should_transition_from_configuring_to_configured() {
            let env = create_test_environment();
            let env = env.configured();

            assert_eq!(*env.state(), Configured);
            assert_eq!(env.name().as_str(), "test-state");
        }

        #[test]
        fn it_should_transition_from_configuring_to_configure_failed() {
            use crate::domain::environment::state::{
                BaseFailureContext, ConfigureFailureContext, ConfigureStep,
            };
            use crate::domain::environment::TraceId;
            use crate::shared::ErrorKind;
            use chrono::Utc;
            use std::time::Duration;

            let env = create_test_environment();
            let context = ConfigureFailureContext {
                failed_step: ConfigureStep::InstallDocker,
                error_kind: ErrorKind::CommandExecution,
                base: BaseFailureContext {
                    error_summary: "ansible_playbook_error".to_string(),
                    failed_at: Utc::now(),
                    execution_started_at: Utc::now(),
                    execution_duration: Duration::from_secs(15),
                    trace_id: TraceId::new(),
                    trace_file_path: None,
                },
            };
            let env = env.configure_failed(context.clone());

            assert_eq!(
                env.state().context.failed_step,
                ConfigureStep::InstallDocker
            );
            assert_eq!(env.name().as_str(), "test-state");
        }
    }
}
