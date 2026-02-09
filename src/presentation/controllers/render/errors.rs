//! Error types for Render Command Controller
//!
//! This module defines presentation layer errors for the render command.

use std::path::PathBuf;

use thiserror::Error;

use crate::presentation::views::progress::ProgressReporterError;

/// Errors that can occur in the render command controller
///
/// These are presentation-layer errors that occur during:
/// - Input validation
/// - IP address parsing
/// - Mode selection (env-name vs env-file)
#[derive(Debug, Error)]
pub enum RenderCommandError {
    /// No input mode specified
    ///
    /// User must provide either --env-name OR --env-file
    #[error("No input mode specified: must provide either --env-name or --env-file")]
    NoInputMode,

    /// Invalid IP address format
    ///
    /// The IP address provided via --ip flag is not a valid IPv4 address
    #[error("Invalid IP address format: {ip}")]
    InvalidIpAddress {
        /// The invalid IP string provided by the user
        ip: String,
    },

    /// Config file does not exist
    ///
    /// The file path provided via --env-file does not exist
    #[error("Configuration file not found: {path}")]
    ConfigFileNotFound {
        /// The file path that doesn't exist
        path: PathBuf,
    },

    /// Placeholder for future application layer errors
    ///
    /// This will be replaced with actual application handler errors in Phase 2
    #[error("Not yet implemented: {message}")]
    NotImplemented {
        /// Description of what's not implemented
        message: String,
    },

    /// Progress reporter error
    ///
    /// Error from displaying progress to the user
    #[error(transparent)]
    ProgressReporter(#[from] ProgressReporterError),
}

impl RenderCommandError {
    /// Provides context-specific help for troubleshooting
    ///
    /// Returns detailed guidance based on the specific error type.
    #[must_use]
    pub fn help(&self) -> Option<String> {
        match self {
            Self::NoInputMode => Some(
                "You must specify an input mode:\n\n\
                Option 1: Use existing Created environment\n  \
                  torrust-tracker-deployer render --env-name my-env --ip 10.0.0.1\n\n\
                Option 2: Use configuration file\n  \
                  torrust-tracker-deployer render --env-file envs/my-config.json --ip 10.0.0.1\n\n\
                For more information, see: docs/user-guide/commands/render.md"
                    .to_string(),
            ),
            Self::InvalidIpAddress { ip } => Some(format!(
                "The IP address '{}' is not a valid IPv4 address.\n\n\
                Valid format: xxx.xxx.xxx.xxx (e.g., 10.0.0.1 or 192.168.1.100)\n\n\
                Examples:\n  \
                  torrust-tracker-deployer render --env-name my-env --ip 10.0.0.1\n  \
                  torrust-tracker-deployer render --env-file envs/test.json --ip 192.168.1.50\n\n\
                For more information, see: docs/user-guide/commands/render.md",
                ip
            )),
            Self::ConfigFileNotFound { path } => Some(format!(
                "Configuration file not found at: {}\n\n\
                Solutions:\n\
                - Check the file path is correct\n\
                - Generate a template: torrust-tracker-deployer create template --provider lxd\n\
                - List your config files: ls envs/\n\n\
                For more information, see: docs/user-guide/commands/render.md",
                path.display()
            )),
            Self::NotImplemented { .. } => Some(
                "This command is not fully implemented yet.\n\n\
                Phase 1 (presentation layer) is complete, but Phase 2 (application handler)\n\
                needs to be implemented next."
                    .to_string(),
            ),
            Self::ProgressReporter(_) => None,
        }
    }
}
