//! `ProvisionFailed` State
//!
//! Error state - Infrastructure provisioning failed
//!
//! The provision command failed during execution. The `context` field
//! contains structured error information including the failed step, error kind,
//! timing information, and a reference to the detailed trace file.
//!
//! **Recovery Options:**
//! - Destroy and recreate the environment
//! - Manual inspection and repair (advanced users)
//! - Review trace file for detailed error information

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, BaseFailureContext, StateTypeError};
use crate::domain::environment::Environment;
use crate::shared::ErrorKind;

// ============================================================================
// Provision Command Error Context
// ============================================================================

/// Error context for provision command failures
///
/// Captures comprehensive information about provision failures including
/// the specific step that failed, error classification, timing, and trace details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvisionFailureContext {
    /// Which step failed during provisioning
    pub failed_step: ProvisionStep,

    /// Error category for type-safe handling
    pub error_kind: ErrorKind,

    /// Base failure context with common fields
    #[serde(flatten)]
    pub base: BaseFailureContext,
}

/// Steps in the provision workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvisionStep {
    /// Rendering `OpenTofu` templates
    RenderOpenTofuTemplates,
    /// Initializing `OpenTofu`
    OpenTofuInit,
    /// Validating infrastructure configuration
    OpenTofuValidate,
    /// Planning infrastructure changes
    OpenTofuPlan,
    /// Applying infrastructure changes
    OpenTofuApply,
    /// Retrieving instance information
    GetInstanceInfo,
    /// Rendering Ansible templates with runtime data
    RenderAnsibleTemplates,
    /// Waiting for SSH connectivity
    WaitSshConnectivity,
    /// Waiting for cloud-init completion
    CloudInitWait,
}

/// Error state - Infrastructure provisioning failed
///
/// The provision command failed during execution. The `context` field
/// contains structured error information including the failed step, error kind,
/// timing information, and a reference to the detailed trace file.
///
/// **Recovery Options:**
/// - Destroy and recreate the environment
/// - Manual inspection and repair (advanced users)
/// - Review trace file for detailed error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvisionFailed {
    /// Structured error context with detailed failure information
    pub context: ProvisionFailureContext,
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<ProvisionFailed> {
    /// Converts typed `Environment<ProvisionFailed>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::ProvisionFailed(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_provision_failed)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<ProvisionFailed>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `ProvisionFailed` state.
    pub fn try_into_provision_failed(self) -> Result<Environment<ProvisionFailed>, StateTypeError> {
        match self {
            Self::ProvisionFailed(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "provision_failed",
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

    fn create_test_context() -> ProvisionFailureContext {
        ProvisionFailureContext {
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
        }
    }

    #[test]
    fn it_should_create_provision_failed_state_with_context() {
        let context = create_test_context();
        let state = ProvisionFailed {
            context: context.clone(),
        };
        assert_eq!(state.context.failed_step, ProvisionStep::CloudInitWait);
        assert_eq!(state.context.error_kind, ErrorKind::Timeout);
    }

    #[test]
    fn it_should_clone_provision_failed_state() {
        let state = ProvisionFailed {
            context: create_test_context(),
        };
        let cloned = state.clone();
        assert_eq!(state.context.failed_step, cloned.context.failed_step);
    }

    #[test]
    fn it_should_serialize_provision_failed_state_to_json() {
        let state = ProvisionFailed {
            context: create_test_context(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("CloudInitWait"));
        assert!(json.contains("Timeout"));
    }

    #[test]
    fn it_should_deserialize_provision_failed_state_from_json() {
        let state = ProvisionFailed {
            context: create_test_context(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ProvisionFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.context.failed_step,
            ProvisionStep::CloudInitWait
        );
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

        fn create_test_environment_provision_failed() -> Environment<ProvisionFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
                .start_provisioning()
                .provision_failed(super::create_test_context())
        }

        #[test]
        fn it_should_convert_provision_failed_environment_into_any() {
            let env = create_test_environment_provision_failed();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::ProvisionFailed(_)));
        }

        #[test]
        fn it_should_convert_any_to_provision_failed_successfully() {
            let env = create_test_environment_provision_failed();
            let any_env = env.into_any();
            let result = any_env.try_into_provision_failed();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_preserve_error_details_in_failed_states() {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let context = super::create_test_context();
            let env = Environment::new(name, ssh_creds)
                .start_provisioning()
                .provision_failed(context.clone());

            // Round-trip conversion
            let any_env = env.into_any();
            let env_restored = any_env.try_into_provision_failed().unwrap();

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
        fn it_should_serialize_provision_failure_context() {
            let context = ProvisionFailureContext {
                failed_step: ProvisionStep::OpenTofuApply,
                error_kind: ErrorKind::InfrastructureOperation,
                base: BaseFailureContext {
                    error_summary: "Infrastructure provisioning failed".to_string(),
                    failed_at: Utc::now(),
                    execution_started_at: Utc::now(),
                    execution_duration: Duration::from_secs(30),
                    trace_id: TraceId::new(),
                    trace_file_path: Some(PathBuf::from("/data/env/traces/trace.log")),
                },
            };

            let json = serde_json::to_string(&context).unwrap();
            assert!(json.contains("OpenTofuApply"));
            assert!(json.contains("InfrastructureOperation"));
        }

        #[test]
        fn it_should_deserialize_provision_failure_context() {
            let trace_id = TraceId::new();
            let json = format!(
                r#"{{
                    "failed_step": "RenderOpenTofuTemplates",
                    "error_kind": "TemplateRendering",
                    "error_summary": "Template rendering failed",
                    "failed_at": "2025-10-06T10:00:00Z",
                    "execution_started_at": "2025-10-06T09:59:00Z",
                    "execution_duration": {{"secs": 60, "nanos": 0}},
                    "trace_id": "{trace_id}",
                    "trace_file_path": null
                }}"#
            );

            let context: ProvisionFailureContext = serde_json::from_str(&json).unwrap();
            assert_eq!(context.failed_step, ProvisionStep::RenderOpenTofuTemplates);
            assert_eq!(context.error_kind, ErrorKind::TemplateRendering);
        }
    }
}
