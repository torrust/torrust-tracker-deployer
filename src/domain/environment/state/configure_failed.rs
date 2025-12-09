//! `ConfigureFailed` State
//!
//! Error state - Application configuration failed
//!
//! The configuration command failed during execution. The `context` field
//! contains structured error information including the failed step, error kind,
//! timing information, and a reference to the detailed trace file.
//!
//! **Recovery Options:**
//! - Destroy and recreate the environment
//! - Manual configuration correction (advanced users)
//! - Review trace file for detailed error information

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, BaseFailureContext, StateTypeError};
use crate::domain::environment::Environment;
use crate::shared::ErrorKind;

// ============================================================================
// Configure Command Error Context
// ============================================================================

/// Error context for configure command failures
///
/// Captures comprehensive information about configuration failures including
/// the specific step that failed, error classification, timing, and trace details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFailureContext {
    /// Which step failed during configuration
    pub failed_step: ConfigureStep,

    /// Error category for type-safe handling
    pub error_kind: ErrorKind,

    /// Base failure context with common fields
    #[serde(flatten)]
    pub base: BaseFailureContext,
}

/// Steps in the configure workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigureStep {
    /// Installing Docker
    InstallDocker,
    /// Installing Docker Compose
    InstallDockerCompose,
    /// Configuring automatic security updates
    ConfigureSecurityUpdates,
    /// Configuring UFW firewall
    ConfigureFirewall,
    /// Configuring Tracker firewall rules
    ConfigureTrackerFirewall,
}

/// Error state - Application configuration failed
///
/// The configuration command failed during execution. The `context` field
/// contains structured error information including the failed step, error kind,
/// timing information, and a reference to the detailed trace file.
///
/// **Recovery Options:**
/// - Destroy and recreate the environment
/// - Manual configuration correction (advanced users)
/// - Review trace file for detailed error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureFailed {
    /// Structured error context with detailed failure information
    pub context: ConfigureFailureContext,
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<ConfigureFailed> {
    /// Converts typed `Environment<ConfigureFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ConfigureFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_configure_failed)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<ConfigureFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `ConfigureFailed` state.
    pub fn try_into_configure_failed(self) -> Result<Environment<ConfigureFailed>, StateTypeError> {
        match self {
            Self::ConfigureFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "configure_failed",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::environment::TraceId;
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_context() -> ConfigureFailureContext {
        ConfigureFailureContext {
            failed_step: ConfigureStep::InstallDocker,
            error_kind: ErrorKind::CommandExecution,
            base: BaseFailureContext {
                error_summary: "Docker installation failed".to_string(),
                failed_at: Utc::now(),
                execution_started_at: Utc::now(),
                execution_duration: Duration::from_secs(15),
                trace_id: TraceId::new(),
                trace_file_path: None,
            },
        }
    }

    #[test]
    fn it_should_create_configure_failed_state_with_context() {
        let context = create_test_context();
        let state = ConfigureFailed {
            context: context.clone(),
        };
        assert_eq!(state.context.failed_step, ConfigureStep::InstallDocker);
        assert_eq!(state.context.error_kind, ErrorKind::CommandExecution);
    }

    #[test]
    fn it_should_serialize_configure_failed_state_to_json() {
        let state = ConfigureFailed {
            context: create_test_context(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("InstallDocker"));
        assert!(json.contains("CommandExecution"));
    }

    #[test]
    fn it_should_deserialize_configure_failed_state_from_json() {
        let state = ConfigureFailed {
            context: create_test_context(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ConfigureFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.context.failed_step,
            ConfigureStep::InstallDocker
        );
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

        fn create_test_environment_configure_failed() -> Environment<ConfigureFailed> {
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
            .configure_failed(super::create_test_context())
        }

        #[test]
        fn it_should_convert_configure_failed_environment_into_any() {
            let env = create_test_environment_configure_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ConfigureFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_configure_failed_successfully() {
            let env = create_test_environment_configure_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_configure_failed();
            assert!(result.is_ok());
        }
    }

    mod context_tests {
        use super::*;

        #[test]
        fn it_should_serialize_configure_failure_context() {
            let context = ConfigureFailureContext {
                failed_step: ConfigureStep::InstallDocker,
                error_kind: ErrorKind::CommandExecution,
                base: BaseFailureContext {
                    error_summary: "Docker installation failed".to_string(),
                    failed_at: Utc::now(),
                    execution_started_at: Utc::now(),
                    execution_duration: Duration::from_secs(15),
                    trace_id: TraceId::new(),
                    trace_file_path: None,
                },
            };

            let json = serde_json::to_string(&context).unwrap();
            assert!(json.contains("InstallDocker"));
            assert!(json.contains("CommandExecution"));
        }

        #[test]
        fn it_should_deserialize_configure_failure_context() {
            let trace_id = TraceId::new();
            let json = format!(
                r#"{{
                    "failed_step": "InstallDockerCompose",
                    "error_kind": "CommandExecution",
                    "error_summary": "Command execution failed",
                    "failed_at": "2025-10-06T10:00:00Z",
                    "execution_started_at": "2025-10-06T09:59:30Z",
                    "execution_duration": {{"secs": 30, "nanos": 0}},
                    "trace_id": "{trace_id}",
                    "trace_file_path": null
                }}"#
            );

            let context: ConfigureFailureContext = serde_json::from_str(&json).unwrap();
            assert_eq!(context.failed_step, ConfigureStep::InstallDockerCompose);
            assert_eq!(context.error_kind, ErrorKind::CommandExecution);
        }
    }
}
