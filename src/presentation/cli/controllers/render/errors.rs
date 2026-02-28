//! Error types for Render Command Controller
//!
//! This module defines presentation layer errors for the render command.

use std::path::PathBuf;

use thiserror::Error;

use crate::application::command_handlers::render::RenderCommandHandlerError;
use crate::presentation::cli::views::progress::ProgressReporterError;
use crate::presentation::cli::views::ViewRenderError;

/// Errors that can occur in the render command controller
///
/// These are presentation-layer errors that occur during:
/// - Input validation
/// - IP address parsing
/// - Mode selection (env-name vs env-file)
/// - Delegation to application handler
#[derive(Debug, Error)]
pub enum RenderCommandError {
    /// No input mode specified
    ///
    /// User must provide either --env-name OR --env-file
    #[error("No input mode specified: must provide either --env-name or --env-file")]
    NoInputMode,

    /// Invalid IP address format
    ///
    /// The IP address provided via --instance-ip flag is not a valid IPv4 address
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

    /// Invalid environment name format
    ///
    /// The environment name provided does not meet naming constraints
    #[error("Invalid environment name: {value}")]
    InvalidEnvironmentName {
        /// The invalid environment name
        value: String,
        /// Reason for rejection
        reason: String,
    },

    /// Working directory unavailable
    ///
    /// Cannot determine current working directory
    #[error("Cannot determine working directory: {reason}")]
    WorkingDirectoryUnavailable {
        /// Why the working directory is unavailable
        reason: String,
    },

    /// Application handler error
    ///
    /// Error from the application layer handler
    #[error("Render command failed: {0}")]
    Handler(#[from] RenderCommandHandlerError),

    /// Progress reporter error
    ///
    /// Error from displaying progress to the user
    #[error(transparent)]
    ProgressReporter(#[from] ProgressReporterError),
    /// Output formatting failed (JSON serialization error).
    /// This indicates an internal error in data serialization.
    #[error(
        "Failed to format output: {reason}\nTip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    OutputFormatting { reason: String },
}
impl From<ViewRenderError> for RenderCommandError {
    fn from(e: ViewRenderError) -> Self {
        Self::OutputFormatting {
            reason: e.to_string(),
        }
    }
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
                  torrust-tracker-deployer render --env-name my-env --instance-ip 10.0.0.1\n\n\
                Option 2: Use configuration file\n  \
                  torrust-tracker-deployer render --env-file envs/my-config.json --instance-ip 10.0.0.1\n\n\
                For more information, see: docs/user-guide/commands/render.md"
                    .to_string(),
            ),
            Self::InvalidIpAddress { ip } => Some(format!(
                "The IP address '{ip}' is not a valid IPv4 address.\n\n\
                Valid format: xxx.xxx.xxx.xxx (e.g., 10.0.0.1 or 192.168.1.100)\n\n\
                Examples:\n  \
                  torrust-tracker-deployer render --env-name my-env --instance-ip 10.0.0.1\n  \
                  torrust-tracker-deployer render --env-file envs/test.json --instance-ip 192.168.1.50\n\n\
                For more information, see: docs/user-guide/commands/render.md"
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
            Self::InvalidEnvironmentName { value, reason } => Some(format!(
                "Invalid environment name: {value}\n\n\
                Reason: {reason}\n\n\
                Environment names must follow these rules:\n\
                - Only lowercase alphanumeric and hyphens\n\
                - Start and end with alphanumeric\n\
                - Between 1 and 63 characters\n\n\
                For more information, see: docs/user-guide/commands/render.md"
            )),
            Self::WorkingDirectoryUnavailable { reason } => Some(format!(
                "Cannot determine current working directory: {reason}\n\n\
                This is unusual and may indicate filesystem or permission issues.\n\
                Try running from a different directory or check filesystem status."
            )),
            Self::Handler(e) => Some(e.help()),
            Self::ProgressReporter(_) => None,
            Self::OutputFormatting { reason } => Some(format!(
                "Output Formatting Failed - Critical Internal Error:\n\nThis is a critical internal error: {reason}\n\nPlease report this bug with full logs.",
            )),
        }
    }
}
