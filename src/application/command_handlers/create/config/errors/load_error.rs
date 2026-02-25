//! Error type for loading `EnvironmentCreationConfig` from files or JSON strings.

use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur when loading an [`EnvironmentCreationConfig`]
/// from a JSON string or a file.
///
/// This is distinct from [`super::CreateConfigError`] which covers
/// domain validation failures *after* parsing.
///
/// [`EnvironmentCreationConfig`]: crate::application::command_handlers::create::config::EnvironmentCreationConfig
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    /// The configuration file does not exist.
    #[error("Configuration file not found: {path}")]
    FileNotFound {
        /// Path that was not found.
        path: PathBuf,
    },

    /// An I/O error occurred while reading the file.
    #[error("Failed to read configuration file '{path}': {source}")]
    FileReadFailed {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The JSON content could not be parsed into `EnvironmentCreationConfig`.
    #[error("Failed to parse configuration: {source}")]
    JsonParseFailed {
        /// Underlying `serde_json` error.
        source: serde_json::Error,
    },
}
