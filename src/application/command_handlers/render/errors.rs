//! Error types for the Render command handler

use std::path::PathBuf;

use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::RepositoryError;
use crate::shared::error::{ErrorKind, Traceable};

/// Comprehensive error type for the `RenderCommandHandler`
#[derive(Debug, thiserror::Error)]
pub enum RenderCommandHandlerError {
    /// Environment was not found in repository (env-name mode)
    #[error("Environment '{name}' not found")]
    EnvironmentNotFound {
        /// The name of the environment that was not found
        name: EnvironmentName,
    },

    /// Environment is not in Created state (already provisioned)
    ///
    /// This is not a hard error - artifacts already exist in build directory
    #[error("Environment '{name}' is already in '{current_state}' state")]
    EnvironmentAlreadyProvisioned {
        /// The name of the environment
        name: EnvironmentName,
        /// The actual state of the environment
        current_state: String,
        /// Path to existing artifacts
        artifacts_path: PathBuf,
    },

    /// Configuration file not found (env-file mode)
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound {
        /// Path to the configuration file that was not found
        path: PathBuf,
    },

    /// Configuration file parsing failed (env-file mode)
    #[error("Failed to parse configuration file: {path}")]
    ConfigParsingFailed {
        /// Path to the configuration file
        path: PathBuf,
        /// JSON parsing error
        #[source]
        source: serde_json::Error,
    },

    /// Domain validation failed during config-to-params conversion
    #[error("Configuration validation failed: {reason}")]
    DomainValidationFailed {
        /// Description of the validation failure
        reason: String,
    },

    /// Invalid IP address format provided
    #[error("Invalid IP address format: {value}")]
    InvalidIpAddress {
        /// The invalid IP address string
        value: String,
    },

    /// Failed to render templates
    #[error("Template rendering failed: {reason}")]
    TemplateRenderingFailed {
        /// Description of why rendering failed
        reason: String,
    },

    /// Repository read error (env-name mode)
    #[error("Failed to load environment: {0}")]
    RepositoryLoad(#[from] RepositoryError),

    /// No input mode specified (neither env-name nor env-file)
    #[error("No input mode specified")]
    NoInputMode,
}

impl Traceable for RenderCommandHandlerError {
    fn trace_format(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!("RenderCommandHandlerError: Environment '{name}' not found")
            }
            Self::EnvironmentAlreadyProvisioned {
                name,
                current_state,
                artifacts_path,
            } => {
                format!(
                    "RenderCommandHandlerError: Environment '{name}' is in '{current_state}' state - artifacts at {}",
                    artifacts_path.display()
                )
            }
            Self::ConfigFileNotFound { path } => {
                format!(
                    "RenderCommandHandlerError: Configuration file not found: {}",
                    path.display()
                )
            }
            Self::ConfigParsingFailed { path, source } => {
                format!(
                    "RenderCommandHandlerError: Failed to parse {}: {source}",
                    path.display()
                )
            }
            Self::DomainValidationFailed { reason } => {
                format!("RenderCommandHandlerError: Configuration validation failed - {reason}")
            }
            Self::InvalidIpAddress { value } => {
                format!("RenderCommandHandlerError: Invalid IP address: {value}")
            }
            Self::TemplateRenderingFailed { reason } => {
                format!("RenderCommandHandlerError: Template rendering failed - {reason}")
            }
            Self::RepositoryLoad(e) => {
                format!("RenderCommandHandlerError: Failed to load environment - {e}")
            }
            Self::NoInputMode => "RenderCommandHandlerError: No input mode specified".to_string(),
        }
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        // RepositoryError doesn't implement Traceable, so we don't return sources
        None
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::EnvironmentNotFound { .. } | Self::ConfigFileNotFound { .. } => {
                ErrorKind::FileSystem
            }
            Self::EnvironmentAlreadyProvisioned { .. }
            | Self::ConfigParsingFailed { .. }
            | Self::DomainValidationFailed { .. }
            | Self::InvalidIpAddress { .. }
            | Self::NoInputMode => ErrorKind::Configuration,
            Self::TemplateRenderingFailed { .. } => ErrorKind::TemplateRendering,
            Self::RepositoryLoad(_) => ErrorKind::StatePersistence,
        }
    }
}

impl RenderCommandHandlerError {
    /// Provide user-friendly help text for errors
    ///
    /// Each error variant includes actionable guidance on how to resolve
    /// the issue or what the user should do next.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::EnvironmentNotFound { name } => {
                format!(
                    "The environment '{name}' does not exist.\n\n\
                     To see available environments:\n  \
                     torrust-tracker-deployer list\n\n\
                     To create this environment:\n  \
                     torrust-tracker-deployer create environment --env-file <path>"
                )
            }
            Self::EnvironmentAlreadyProvisioned {
                name,
                current_state,
                artifacts_path,
            } => {
                format!(
                    "Environment '{name}' is in '{current_state}' state.\n\n\
                     Deployment artifacts have already been generated during provisioning.\n\
                     You can find them at:\n  {}\n\n\
                     The 'render' command is only for environments in 'Created' state\n\
                     (before provisioning infrastructure).\n\n\
                     If you need to regenerate artifacts, you can:\n\
                     1. Destroy the environment: torrust-tracker-deployer destroy {name}\n\
                     2. Purge local data: torrust-tracker-deployer purge {name}\n\
                     3. Create a new environment: torrust-tracker-deployer create environment --env-file <path>\n\
                     4. Generate artifacts: torrust-tracker-deployer render --env-name {name} --instance-ip <ip>",
                    artifacts_path.display()
                )
            }
            Self::ConfigFileNotFound { path } => {
                format!(
                    "Configuration file not found: {}\n\n\
                     Make sure the file exists and the path is correct.\n\n\
                     To generate a configuration template:\n  \
                     torrust-tracker-deployer create template --provider lxd",
                    path.display()
                )
            }
            Self::ConfigParsingFailed { path, source } => {
                format!(
                    "Failed to parse configuration file: {}\n\n\
                     JSON Error: {source}\n\n\
                     Make sure the file is valid JSON and follows the configuration schema.\n\n\
                     To validate your configuration:\n  \
                     torrust-tracker-deployer validate --env-file {}",
                    path.display(),
                    path.display()
                )
            }
            Self::DomainValidationFailed { reason } => {
                format!(
                    "Configuration validation failed: {reason}\n\n\
                     Fix the configuration issues and try again.\n\n\
                     To validate your configuration:\n  \
                     torrust-tracker-deployer validate --env-file <path>"
                )
            }
            Self::InvalidIpAddress { value } => {
                format!(
                    "Invalid IP address format: {value}\n\n\
                     Please provide a valid IPv4 or IPv6 address.\n\n\
                     Examples:\n  \
                     IPv4: 10.0.0.1, 192.168.1.100\n  \
                     IPv6: 2001:db8::1, ::1"
                )
            }
            Self::TemplateRenderingFailed { reason } => {
                format!(
                    "Failed to render deployment artifacts: {reason}\n\n\
                     This is an internal error. Please report this issue with:\n\
                     - Your configuration file (redact sensitive data)\n\
                     - The full error message above\n\
                     - Environment details (OS, provider)"
                )
            }
            Self::RepositoryLoad(e) => {
                format!(
                    "Failed to load environment from repository.\n\n\
                     Repository error: {e}\n\n\
                     This may indicate data corruption or permission issues.\n\
                     Check that the data/ directory is readable and not corrupted."
                )
            }
            Self::NoInputMode => {
                "No input mode specified.\n\n\
                 You must provide either --env-name or --env-file.\n\n\
                 Examples:\n  \
                 # From existing environment\n  \
                 torrust-tracker-deployer render --env-name my-env --instance-ip 10.0.0.1\n\n  \
                 # From configuration file\n  \
                 torrust-tracker-deployer render --env-file envs/my-config.json --instance-ip 10.0.0.1"
                    .to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_implement_error_trait() {
        let error = RenderCommandHandlerError::NoInputMode;
        let _error_string: String = error.to_string();
    }

    #[test]
    fn it_should_provide_help_text_for_all_variants() {
        let errors = [
            RenderCommandHandlerError::EnvironmentNotFound {
                name: EnvironmentName::new("test").unwrap(),
            },
            RenderCommandHandlerError::EnvironmentAlreadyProvisioned {
                name: EnvironmentName::new("test").unwrap(),
                current_state: "Provisioned".to_string(),
                artifacts_path: PathBuf::from("build/test"),
            },
            RenderCommandHandlerError::ConfigFileNotFound {
                path: PathBuf::from("test.json"),
            },
            RenderCommandHandlerError::InvalidIpAddress {
                value: "not-an-ip".to_string(),
            },
            RenderCommandHandlerError::NoInputMode,
        ];

        for error in &errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
        }
    }
}
