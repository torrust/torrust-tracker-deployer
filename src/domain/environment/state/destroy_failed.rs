//! `DestroyFailed` State
//!
//! Error state - Infrastructure destruction failed
//!
//! The destroy command failed during execution. The `context` field
//! contains structured error information including the failed step, error kind,
//! timing information, and a reference to the detailed trace file.
//!
//! **Recovery Options:**
//! - Retry the destroy operation
//! - Manual cleanup of remaining resources
//! - Review trace file for detailed error information

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, BaseFailureContext, StateTypeError};
use crate::domain::environment::Environment;
use crate::shared::ErrorKind;

// ============================================================================
// Destroy Command Error Context
// ============================================================================

/// Error context for destroy command failures
///
/// Captures comprehensive information about destroy failures including
/// the specific step that failed, error classification, timing, and trace details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DestroyFailureContext {
    /// Which step failed during destruction
    pub failed_step: DestroyStep,

    /// Error category for type-safe handling
    pub error_kind: ErrorKind,

    /// Base failure context with common fields
    #[serde(flatten)]
    pub base: BaseFailureContext,
}

/// Steps in the destroy workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DestroyStep {
    /// Loading environment state
    LoadEnvironment,
    /// Destroying infrastructure via `OpenTofu`
    DestroyInfrastructure,
    /// Cleaning up state files
    CleanupStateFiles,
}

/// Error state - Infrastructure destruction failed
///
/// The destroy command failed during execution. The `context` field
/// contains structured error information including the failed step, error kind,
/// timing information, and a reference to the detailed trace file.
///
/// **Recovery Options:**
/// - Retry the destroy operation
/// - Manual cleanup of remaining resources
/// - Review trace file for detailed error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DestroyFailed {
    /// Structured error context with detailed failure information
    pub context: DestroyFailureContext,
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<DestroyFailed> {
    /// Converts typed `Environment<DestroyFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::DestroyFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_destroy_failed)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<DestroyFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `DestroyFailed` state.
    pub fn try_into_destroy_failed(self) -> Result<Environment<DestroyFailed>, StateTypeError> {
        match self {
            Self::DestroyFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "destroy_failed",
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
    use std::path::PathBuf;
    use std::time::Duration;

    fn create_test_context() -> DestroyFailureContext {
        DestroyFailureContext {
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
        }
    }

    #[test]
    fn it_should_create_destroy_failed_state_with_context() {
        let context = create_test_context();
        let state = DestroyFailed {
            context: context.clone(),
        };
        assert_eq!(
            state.context.failed_step,
            DestroyStep::DestroyInfrastructure
        );
        assert_eq!(state.context.error_kind, ErrorKind::InfrastructureOperation);
    }

    #[test]
    fn it_should_clone_destroy_failed_state() {
        let state = DestroyFailed {
            context: create_test_context(),
        };
        let cloned = state.clone();
        assert_eq!(state.context.failed_step, cloned.context.failed_step);
    }

    #[test]
    fn it_should_serialize_destroy_failed_state_to_json() {
        let state = DestroyFailed {
            context: create_test_context(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("DestroyInfrastructure"));
        assert!(json.contains("InfrastructureOperation"));
    }

    #[test]
    fn it_should_deserialize_destroy_failed_state_from_json() {
        let state = DestroyFailed {
            context: create_test_context(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: DestroyFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.context.failed_step,
            DestroyStep::DestroyInfrastructure
        );
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

        fn create_test_environment_destroy_failed() -> Environment<DestroyFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(
                name.clone(),
                default_lxd_provider_config(&name),
                ssh_creds,
                22,
            )
            .start_provisioning()
            .provisioned(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
                ProvisionMethod::Provisioned,
            )
            .start_destroying()
            .destroy_failed(super::create_test_context())
        }

        #[test]
        fn it_should_convert_destroy_failed_environment_into_any() {
            let env = create_test_environment_destroy_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::DestroyFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_destroy_failed_successfully() {
            let env = create_test_environment_destroy_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_destroy_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_preserve_error_details_in_failed_states() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let context = super::create_test_context();
            let env = Environment::new(
                name.clone(),
                default_lxd_provider_config(&name),
                ssh_creds,
                22,
            )
            .start_provisioning()
            .provisioned(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
                ProvisionMethod::Provisioned,
            )
            .start_destroying()
            .destroy_failed(context.clone());

            // Round-trip conversion
            let any_env = env.into_any();
            let env_restored = any_env.try_into_destroy_failed().unwrap();

            assert_eq!(
                env_restored.state().context.failed_step,
                context.failed_step
            );
            assert_eq!(
                env_restored.state().context.base.error_summary,
                context.base.error_summary
            );
        }
    }

    mod context_tests {
        use super::*;

        #[test]
        fn it_should_serialize_destroy_failure_context() {
            let context = DestroyFailureContext {
                failed_step: DestroyStep::CleanupStateFiles,
                error_kind: ErrorKind::StatePersistence,
                base: BaseFailureContext {
                    error_summary: "Failed to clean up state files".to_string(),
                    failed_at: Utc::now(),
                    execution_started_at: Utc::now(),
                    execution_duration: Duration::from_secs(5),
                    trace_id: TraceId::new(),
                    trace_file_path: Some(PathBuf::from("/data/env/traces/trace.log")),
                },
            };

            let json = serde_json::to_string(&context).unwrap();
            assert!(json.contains("CleanupStateFiles"));
            assert!(json.contains("StatePersistence"));
        }

        #[test]
        fn it_should_deserialize_destroy_failure_context() {
            let trace_id = TraceId::new();
            let json = format!(
                r#"{{
                    "failed_step": "LoadEnvironment",
                    "error_kind": "StatePersistence",
                    "error_summary": "Failed to load environment",
                    "failed_at": "2025-10-24T10:00:00Z",
                    "execution_started_at": "2025-10-24T09:59:00Z",
                    "execution_duration": {{"secs": 60, "nanos": 0}},
                    "trace_id": "{trace_id}",
                    "trace_file_path": null
                }}"#
            );

            let context: DestroyFailureContext = serde_json::from_str(&json).unwrap();
            assert_eq!(context.failed_step, DestroyStep::LoadEnvironment);
            assert_eq!(context.error_kind, ErrorKind::StatePersistence);
        }
    }
}
