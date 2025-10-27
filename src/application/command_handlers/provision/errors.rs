//! Error types for the Provision command handler

use crate::adapters::ssh::SshError;
use crate::adapters::tofu::client::OpenTofuError;
use crate::application::steps::RenderAnsibleTemplatesError;
use crate::infrastructure::external_tools::tofu::ProvisionTemplateError;
use crate::shared::command::CommandError;

/// Comprehensive error type for the `ProvisionCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum ProvisionCommandHandlerError {
    #[error("OpenTofu template rendering failed: {0}")]
    OpenTofuTemplateRendering(#[from] ProvisionTemplateError),

    #[error("Ansible template rendering failed: {0}")]
    AnsibleTemplateRendering(#[from] RenderAnsibleTemplatesError),

    #[error("OpenTofu command failed: {0}")]
    OpenTofu(#[from] OpenTofuError),

    #[error("Command execution failed: {0}")]
    Command(#[from] CommandError),

    #[error("SSH connectivity failed: {0}")]
    SshConnectivity(#[from] SshError),

    #[error("Failed to persist environment state: {0}")]
    StatePersistence(#[from] crate::domain::environment::repository::RepositoryError),
}

impl crate::shared::Traceable for ProvisionCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::OpenTofuTemplateRendering(e) => {
                format!("ProvisionCommandHandlerError: OpenTofu template rendering failed - {e}")
            }
            Self::AnsibleTemplateRendering(e) => {
                format!("ProvisionCommandHandlerError: Ansible template rendering failed - {e}")
            }
            Self::OpenTofu(e) => {
                format!("ProvisionCommandHandlerError: OpenTofu command failed - {e}")
            }
            Self::Command(e) => {
                format!("ProvisionCommandHandlerError: Command execution failed - {e}")
            }
            Self::SshConnectivity(e) => {
                format!("ProvisionCommandHandlerError: SSH connectivity failed - {e}")
            }
            Self::StatePersistence(e) => {
                format!("ProvisionCommandHandlerError: Failed to persist environment state - {e}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::OpenTofuTemplateRendering(e) => Some(e),
            Self::AnsibleTemplateRendering(e) => Some(e),
            Self::OpenTofu(e) => Some(e),
            Self::Command(e) => Some(e),
            Self::SshConnectivity(e) => Some(e),
            Self::StatePersistence(_) => None,
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        match self {
            Self::OpenTofuTemplateRendering(_) | Self::AnsibleTemplateRendering(_) => {
                crate::shared::ErrorKind::TemplateRendering
            }
            Self::OpenTofu(_) => crate::shared::ErrorKind::InfrastructureOperation,
            Self::SshConnectivity(_) => crate::shared::ErrorKind::NetworkConnectivity,
            Self::Command(_) => crate::shared::ErrorKind::CommandExecution,
            Self::StatePersistence(_) => crate::shared::ErrorKind::StatePersistence,
        }
    }
}
