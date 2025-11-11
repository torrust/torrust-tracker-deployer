//! Unified Create Command Errors
//!
//! This module defines a unified error type that encompasses all create subcommand errors,
//! providing a single interface for environment and template command errors.

use thiserror::Error;

use super::subcommands::{
    environment::CreateEnvironmentCommandError, template::CreateEnvironmentTemplateCommandError,
};

/// Unified error type for all create subcommands
///
/// This error type provides a unified interface for errors that can occur during
/// any create subcommand execution (environment creation or template generation).
/// It wraps the specific command errors while preserving their context and help methods.
#[derive(Debug, Error)]
pub enum CreateCommandError {
    /// Environment creation errors
    #[error(transparent)]
    Environment(#[from] CreateEnvironmentCommandError),

    /// Template generation errors
    #[error(transparent)]
    Template(#[from] CreateEnvironmentTemplateCommandError),
}

impl CreateCommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method delegates to the specific command error's help method,
    /// providing context-appropriate troubleshooting guidance.
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::Environment(err) => err.help(),
            Self::Template(err) => err.help(),
        }
    }
}
