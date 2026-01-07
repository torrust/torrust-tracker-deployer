//! `RunFailed` State
//!
//! Error state - Application runtime failed
//!
//! The run command failed during execution. The `context` field
//! contains detailed information about the failure, including which step
//! failed, error classification, and trace file location.
//!
//! **Recovery Options:**
//! - Retry the run command
//! - Destroy and recreate the environment

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, BaseFailureContext, StateTypeError};
use crate::domain::environment::Environment;
use crate::shared::error::ErrorKind;

/// Steps in the run workflow
///
/// Each variant represents a distinct phase in the run process.
/// This allows precise tracking of which step failed during run.
///
/// The run workflow follows the three-level architecture:
/// - **Command** (Level 1): `RunCommandHandler` orchestrates the workflow
/// - **Step** (Level 2): Individual steps like `StartServicesStep`
/// - **Remote Action** (Level 3): Ansible playbooks execute on remote hosts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStep {
    /// Starting Docker Compose services on the remote host
    StartServices,
}

impl fmt::Display for RunStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::StartServices => "Start Services",
        };
        write!(f, "{name}")
    }
}

/// Structured failure context for run command errors
///
/// Contains comprehensive information about a run failure:
/// - Which step failed
/// - Error classification for recovery guidance
/// - Base failure metadata (timing, trace ID, error summary)
///
/// This enables:
/// - Accurate error reporting
/// - Recovery suggestions based on the specific failure
/// - Post-mortem analysis via trace files
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunFailureContext {
    /// The step that was executing when the failure occurred
    pub failed_step: RunStep,

    /// Classification of the error for recovery guidance
    pub error_kind: ErrorKind,

    /// Common failure metadata (timing, trace, error summary)
    pub base: BaseFailureContext,
}

/// Error state - Application runtime failed
///
/// The run command failed during execution. The `context` field
/// contains detailed information about the failure.
///
/// **Recovery Options:**
/// - Retry the run command
/// - Destroy and recreate the environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunFailed {
    /// Structured failure context with step info, error classification, and trace
    pub context: RunFailureContext,
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<RunFailed> {
    /// Converts typed `Environment<RunFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::RunFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_run_failed)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<RunFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `RunFailed` state.
    pub fn try_into_run_failed(self) -> Result<Environment<RunFailed>, StateTypeError> {
        match self {
            Self::RunFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "run_failed",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;

    use super::*;
    use crate::domain::environment::TraceId;

    fn create_test_failure_context() -> RunFailureContext {
        let now = Utc::now();
        RunFailureContext {
            failed_step: RunStep::StartServices,
            error_kind: ErrorKind::InfrastructureOperation,
            base: BaseFailureContext {
                error_summary: "Test error".to_string(),
                failed_at: now,
                execution_started_at: now,
                execution_duration: Duration::from_secs(10),
                trace_id: TraceId::new(),
                trace_file_path: None,
            },
        }
    }

    #[test]
    fn it_should_create_run_failed_state_with_context() {
        let context = create_test_failure_context();
        let state = RunFailed {
            context: context.clone(),
        };
        assert_eq!(state.context.failed_step, RunStep::StartServices);
        assert_eq!(state.context.error_kind, ErrorKind::InfrastructureOperation);
    }

    #[test]
    fn it_should_display_run_step() {
        assert_eq!(format!("{}", RunStep::StartServices), "Start Services");
    }

    #[test]
    fn it_should_serialize_run_step_to_snake_case() {
        let step = RunStep::StartServices;
        let json = serde_json::to_string(&step).unwrap();
        assert_eq!(json, r#""start_services""#);
    }

    #[test]
    fn it_should_deserialize_run_step_from_snake_case() {
        let step: RunStep = serde_json::from_str(r#""start_services""#).unwrap();
        assert_eq!(step, RunStep::StartServices);
    }

    mod conversion_tests {
        use super::*;
        use crate::adapters::ssh::SshCredentials;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::runtime_outputs::ProvisionMethod;
        use crate::domain::provider::{LxdConfig, ProviderConfig};
        use crate::domain::ProfileName;
        use crate::shared::Username;
        use std::net::{IpAddr, Ipv4Addr};
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

        fn create_test_environment_run_failed() -> Environment<RunFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let now = Utc::now();
            let context = RunFailureContext {
                failed_step: RunStep::StartServices,
                error_kind: ErrorKind::InfrastructureOperation,
                base: BaseFailureContext {
                    error_summary: "Docker compose failed".to_string(),
                    failed_at: now,
                    execution_started_at: now,
                    execution_duration: Duration::from_secs(5),
                    trace_id: TraceId::new(),
                    trace_file_path: None,
                },
            };
            Environment::new(
                name.clone(),
                default_lxd_provider_config(&name),
                ssh_creds,
                22,
                chrono::Utc::now(),
            )
            .start_provisioning()
            .provisioned(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
                ProvisionMethod::Provisioned,
            )
            .start_configuring()
            .configured()
            .start_releasing()
            .released()
            .start_running()
            .run_failed(context)
        }

        #[test]
        fn it_should_convert_run_failed_environment_into_any() {
            let env = create_test_environment_run_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::RunFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_run_failed_successfully() {
            let env = create_test_environment_run_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_run_failed();
            assert!(result.is_ok());
        }
    }
}
