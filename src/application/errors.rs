//! Application-layer wrapper types for domain error types
//!
//! These wrapper types shield the SDK's public API from internal domain error
//! types. They mirror the domain types structurally but live in the application
//! layer, so SDK consumers never need to import from `crate::domain`.
//!
//! # Design Rationale
//!
//! Domain errors (`RepositoryError`, `StateTypeError`, `ReleaseStep`) are
//! implementation details of the persistence and state-machine layers.
//! Exposing them directly in the SDK's public error surface would:
//! 1. Require SDK consumers to add `#[allow(unused_imports)]` for domain modules
//! 2. Make internal domain changes breaking changes for SDK consumers
//! 3. Violate the DDD Dependency Rule (presentation ← application, not ← domain)
//!
//! The `From` conversions on each `*CommandHandlerError` use these wrappers so
//! that application handler code using `?` on domain-returning calls continues
//! to work unchanged.

use std::fmt;

use thiserror::Error;

/// Application-layer wrapper for domain `RepositoryError`.
///
/// Mirrors the three variants of the domain type using plain types (no domain
/// imports required by SDK consumers).
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::errors::PersistenceError;
///
/// let err = PersistenceError::NotFound;
/// assert_eq!(err.to_string(), "Environment not found");
///
/// let internal = PersistenceError::Internal(
///     anyhow::anyhow!("disk full")
/// );
/// assert!(internal.to_string().contains("Internal error"));
/// ```
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// Environment not found in storage
    #[error("Environment not found")]
    NotFound,

    /// Conflict with concurrent operation
    #[error("Conflict: another process is accessing this environment")]
    Conflict,

    /// Internal implementation-specific error
    #[error("Internal error: {0}")]
    Internal(#[source] anyhow::Error),
}

impl From<crate::domain::environment::repository::RepositoryError> for PersistenceError {
    fn from(e: crate::domain::environment::repository::RepositoryError) -> Self {
        use crate::domain::environment::repository::RepositoryError;
        match e {
            RepositoryError::NotFound => Self::NotFound,
            RepositoryError::Conflict => Self::Conflict,
            RepositoryError::Internal(inner) => Self::Internal(inner),
        }
    }
}

/// Application-layer wrapper for domain `StateTypeError`.
///
/// Uses owned `String` fields instead of the domain's `&'static str` / `String`
/// mix, so SDK consumers only deal with plain strings.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::errors::InvalidStateError;
///
/// let err = InvalidStateError {
///     expected: "provisioned".to_string(),
///     actual: "created".to_string(),
/// };
/// assert!(err.to_string().contains("provisioned"));
/// ```
#[derive(Debug, Error)]
#[error("Expected state '{expected}', but found '{actual}'")]
pub struct InvalidStateError {
    /// The state that was expected
    pub expected: String,
    /// The actual state at the time of the error
    pub actual: String,
}

impl From<crate::domain::environment::state::StateTypeError> for InvalidStateError {
    fn from(e: crate::domain::environment::state::StateTypeError) -> Self {
        use crate::domain::environment::state::StateTypeError;
        match e {
            StateTypeError::UnexpectedState { expected, actual } => Self {
                expected: expected.to_string(),
                actual,
            },
        }
    }
}

/// Application-layer representation of a release workflow step.
///
/// Mirrors [`crate::domain::environment::state::ReleaseStep`] for error
/// reporting in `ReleaseCommandHandlerError` variants that need to surface
/// the step that failed. SDK consumers can display or match on these variants
/// without importing the domain module.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::errors::ReleaseWorkflowStep;
///
/// let step = ReleaseWorkflowStep::InitTrackerDatabase;
/// assert_eq!(step.to_string(), "Initialize Tracker Database");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseWorkflowStep {
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
    /// Creating Grafana storage directories on remote host
    CreateGrafanaStorage,
    /// Rendering Grafana provisioning templates to the build directory
    RenderGrafanaTemplates,
    /// Deploying Grafana provisioning configuration to the remote host via Ansible
    DeployGrafanaProvisioning,
    /// Creating `MySQL` storage directories on remote host
    CreateMysqlStorage,
    /// Rendering Backup configuration templates to the build directory
    RenderBackupTemplates,
    /// Creating Backup storage directories on remote host
    CreateBackupStorage,
    /// Deploying Backup configuration to the remote host via Ansible
    DeployBackupConfigToRemote,
    /// Installing backup crontab and maintenance script
    InstallBackupCrontab,
    /// Rendering Caddy configuration templates to the build directory
    RenderCaddyTemplates,
    /// Deploying Caddy configuration to the remote host via Ansible
    DeployCaddyConfigToRemote,
    /// Rendering Docker Compose templates to the build directory
    RenderDockerComposeTemplates,
    /// Deploying compose files to the remote host via Ansible
    DeployComposeFilesToRemote,
}

impl fmt::Display for ReleaseWorkflowStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::CreateTrackerStorage => "Create Tracker Storage",
            Self::InitTrackerDatabase => "Initialize Tracker Database",
            Self::RenderTrackerTemplates => "Render Tracker Templates",
            Self::DeployTrackerConfigToRemote => "Deploy Tracker Config to Remote",
            Self::CreatePrometheusStorage => "Create Prometheus Storage",
            Self::RenderPrometheusTemplates => "Render Prometheus Templates",
            Self::DeployPrometheusConfigToRemote => "Deploy Prometheus Config to Remote",
            Self::CreateGrafanaStorage => "Create Grafana Storage",
            Self::RenderGrafanaTemplates => "Render Grafana Templates",
            Self::DeployGrafanaProvisioning => "Deploy Grafana Provisioning",
            Self::CreateMysqlStorage => "Create MySQL Storage",
            Self::RenderBackupTemplates => "Render Backup Templates",
            Self::CreateBackupStorage => "Create Backup Storage",
            Self::DeployBackupConfigToRemote => "Deploy Backup Config to Remote",
            Self::InstallBackupCrontab => "Install Backup Crontab",
            Self::RenderCaddyTemplates => "Render Caddy Templates",
            Self::DeployCaddyConfigToRemote => "Deploy Caddy Config to Remote",
            Self::RenderDockerComposeTemplates => "Render Docker Compose Templates",
            Self::DeployComposeFilesToRemote => "Deploy Compose Files to Remote",
        };
        write!(f, "{name}")
    }
}

impl From<crate::domain::environment::state::ReleaseStep> for ReleaseWorkflowStep {
    fn from(s: crate::domain::environment::state::ReleaseStep) -> Self {
        use crate::domain::environment::state::ReleaseStep;
        match s {
            ReleaseStep::CreateTrackerStorage => Self::CreateTrackerStorage,
            ReleaseStep::InitTrackerDatabase => Self::InitTrackerDatabase,
            ReleaseStep::RenderTrackerTemplates => Self::RenderTrackerTemplates,
            ReleaseStep::DeployTrackerConfigToRemote => Self::DeployTrackerConfigToRemote,
            ReleaseStep::CreatePrometheusStorage => Self::CreatePrometheusStorage,
            ReleaseStep::RenderPrometheusTemplates => Self::RenderPrometheusTemplates,
            ReleaseStep::DeployPrometheusConfigToRemote => Self::DeployPrometheusConfigToRemote,
            ReleaseStep::CreateGrafanaStorage => Self::CreateGrafanaStorage,
            ReleaseStep::RenderGrafanaTemplates => Self::RenderGrafanaTemplates,
            ReleaseStep::DeployGrafanaProvisioning => Self::DeployGrafanaProvisioning,
            ReleaseStep::CreateMysqlStorage => Self::CreateMysqlStorage,
            ReleaseStep::RenderBackupTemplates => Self::RenderBackupTemplates,
            ReleaseStep::CreateBackupStorage => Self::CreateBackupStorage,
            ReleaseStep::DeployBackupConfigToRemote => Self::DeployBackupConfigToRemote,
            ReleaseStep::InstallBackupCrontab => Self::InstallBackupCrontab,
            ReleaseStep::RenderCaddyTemplates => Self::RenderCaddyTemplates,
            ReleaseStep::DeployCaddyConfigToRemote => Self::DeployCaddyConfigToRemote,
            ReleaseStep::RenderDockerComposeTemplates => Self::RenderDockerComposeTemplates,
            ReleaseStep::DeployComposeFilesToRemote => Self::DeployComposeFilesToRemote,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_display_not_found_persistence_error() {
        let err = PersistenceError::NotFound;
        assert_eq!(err.to_string(), "Environment not found");
    }

    #[test]
    fn it_should_display_conflict_persistence_error() {
        let err = PersistenceError::Conflict;
        assert_eq!(
            err.to_string(),
            "Conflict: another process is accessing this environment"
        );
    }

    #[test]
    fn it_should_display_invalid_state_error() {
        let err = InvalidStateError {
            expected: "provisioned".to_string(),
            actual: "created".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Expected state 'provisioned', but found 'created'"
        );
    }

    #[test]
    fn it_should_convert_from_repository_error_not_found() {
        use crate::domain::environment::repository::RepositoryError;
        let domain_err = RepositoryError::NotFound;
        let app_err = PersistenceError::from(domain_err);
        assert!(matches!(app_err, PersistenceError::NotFound));
    }

    #[test]
    fn it_should_convert_from_repository_error_conflict() {
        use crate::domain::environment::repository::RepositoryError;
        let domain_err = RepositoryError::Conflict;
        let app_err = PersistenceError::from(domain_err);
        assert!(matches!(app_err, PersistenceError::Conflict));
    }

    #[test]
    fn it_should_convert_from_state_type_error() {
        use crate::domain::environment::state::StateTypeError;
        let domain_err = StateTypeError::UnexpectedState {
            expected: "provisioned",
            actual: "created".to_string(),
        };
        let app_err = InvalidStateError::from(domain_err);
        assert_eq!(app_err.expected, "provisioned");
        assert_eq!(app_err.actual, "created");
    }

    #[test]
    fn it_should_convert_release_step_to_workflow_step() {
        use crate::domain::environment::state::ReleaseStep;
        let step = ReleaseStep::InitTrackerDatabase;
        let ws: ReleaseWorkflowStep = step.into();
        assert_eq!(ws, ReleaseWorkflowStep::InitTrackerDatabase);
        assert_eq!(ws.to_string(), "Initialize Tracker Database");
    }
}
