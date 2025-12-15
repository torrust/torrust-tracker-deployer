//! `ReleaseFailed` State
//!
//! Error state - Release preparation failed
//!
//! The release command failed during execution. The `context` field
//! contains detailed information about the failure, including which step
//! failed, error classification, and trace file location.
//!
//! **Recovery Options:**
//! - Destroy and recreate the environment
//! - Manual release correction (advanced users)

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, BaseFailureContext, StateTypeError};
use crate::domain::environment::Environment;
use crate::shared::error::ErrorKind;

/// Steps in the release workflow
///
/// Each variant represents a distinct phase in the release process.
/// This allows precise tracking of which step failed during release.
///
/// The release workflow follows the three-level architecture:
/// - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the workflow
/// - **Step** (Level 2): Individual steps like `RenderDockerComposeTemplatesStep` and `DeployComposeFilesStep`
/// - **Remote Action** (Level 3): Ansible playbooks execute on remote hosts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStep {
    /// Creating tracker storage directories on remote host
    CreateTrackerStorage,
    /// Initializing tracker `SQLite` database file
    InitTrackerDatabase,
    /// Rendering Tracker configuration templates to the build directory
    RenderTrackerTemplates,
    /// Deploying tracker configuration to the remote host via Ansible
    DeployTrackerConfigToRemote,
    /// Creating Prometheus storage directories on remote host
    CreatePrometheusStorage,
    /// Rendering Prometheus configuration templates to the build directory
    RenderPrometheusTemplates,
    /// Deploying Prometheus configuration to the remote host via Ansible
    DeployPrometheusConfigToRemote,
    /// Rendering Docker Compose templates to the build directory
    RenderDockerComposeTemplates,
    /// Deploying compose files to the remote host via Ansible
    DeployComposeFilesToRemote,
}

impl fmt::Display for ReleaseStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::CreateTrackerStorage => "Create Tracker Storage",
            Self::InitTrackerDatabase => "Initialize Tracker Database",
            Self::RenderTrackerTemplates => "Render Tracker Templates",
            Self::DeployTrackerConfigToRemote => "Deploy Tracker Config to Remote",
            Self::CreatePrometheusStorage => "Create Prometheus Storage",
            Self::RenderPrometheusTemplates => "Render Prometheus Templates",
            Self::DeployPrometheusConfigToRemote => "Deploy Prometheus Config to Remote",
            Self::RenderDockerComposeTemplates => "Render Docker Compose Templates",
            Self::DeployComposeFilesToRemote => "Deploy Compose Files to Remote",
        };
        write!(f, "{name}")
    }
}

/// Structured failure context for release command errors
///
/// Contains comprehensive information about a release failure:
/// - Which step failed
/// - Error classification for recovery guidance
/// - Base failure metadata (timing, trace ID, error summary)
///
/// This enables:
/// - Accurate error reporting
/// - Recovery suggestions based on the specific failure
/// - Post-mortem analysis via trace files
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseFailureContext {
    /// The step that was executing when the failure occurred
    pub failed_step: ReleaseStep,

    /// Classification of the error for recovery guidance
    pub error_kind: ErrorKind,

    /// Common failure metadata (timing, trace, error summary)
    pub base: BaseFailureContext,
}

/// Error state - Release preparation failed
///
/// The release command failed during execution. The `context` field
/// contains detailed information about the failure.
///
/// **Recovery Options:**
/// - Destroy and recreate the environment
/// - Manual release correction (advanced users)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseFailed {
    /// Structured failure context with step info, error classification, and trace
    pub context: ReleaseFailureContext,
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
    use std::time::Duration;

    use chrono::Utc;

    use super::*;
    use crate::domain::environment::TraceId;

    fn create_test_failure_context() -> ReleaseFailureContext {
        let now = Utc::now();
        ReleaseFailureContext {
            failed_step: ReleaseStep::RenderDockerComposeTemplates,
            error_kind: ErrorKind::Configuration,
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
    fn it_should_create_release_failed_state_with_context() {
        let context = create_test_failure_context();
        let state = ReleaseFailed {
            context: context.clone(),
        };
        assert_eq!(
            state.context.failed_step,
            ReleaseStep::RenderDockerComposeTemplates
        );
        assert_eq!(state.context.error_kind, ErrorKind::Configuration);
    }

    #[test]
    fn it_should_display_release_step() {
        assert_eq!(
            format!("{}", ReleaseStep::RenderDockerComposeTemplates),
            "Render Docker Compose Templates"
        );
        assert_eq!(
            format!("{}", ReleaseStep::DeployComposeFilesToRemote),
            "Deploy Compose Files to Remote"
        );
    }

    #[test]
    fn it_should_serialize_release_step_to_snake_case() {
        let step = ReleaseStep::RenderDockerComposeTemplates;
        let json = serde_json::to_string(&step).unwrap();
        assert_eq!(json, r#""render_docker_compose_templates""#);

        let step = ReleaseStep::DeployComposeFilesToRemote;
        let json = serde_json::to_string(&step).unwrap();
        assert_eq!(json, r#""deploy_compose_files_to_remote""#);
    }

    #[test]
    fn it_should_deserialize_release_step_from_snake_case() {
        let step: ReleaseStep =
            serde_json::from_str(r#""render_docker_compose_templates""#).unwrap();
        assert_eq!(step, ReleaseStep::RenderDockerComposeTemplates);

        let step: ReleaseStep =
            serde_json::from_str(r#""deploy_compose_files_to_remote""#).unwrap();
        assert_eq!(step, ReleaseStep::DeployComposeFilesToRemote);
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

        fn create_test_environment_release_failed() -> Environment<ReleaseFailed> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            let now = Utc::now();
            let context = ReleaseFailureContext {
                failed_step: ReleaseStep::DeployComposeFilesToRemote,
                error_kind: ErrorKind::InfrastructureOperation,
                base: BaseFailureContext {
                    error_summary: "SSH connection failed".to_string(),
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
            )
            .start_provisioning()
            .provisioned(
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
                ProvisionMethod::Provisioned,
            )
            .start_configuring()
            .configured()
            .start_releasing()
            .release_failed(context)
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
