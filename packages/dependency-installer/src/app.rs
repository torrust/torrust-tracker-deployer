//! Application logic for the dependency installer CLI
//!
//! This module contains the core application logic for running the CLI.

use clap::Parser;
use thiserror::Error;

use crate::cli::{Cli, Commands};
use crate::handlers::check::CheckError;
use crate::handlers::list::ListError;
use crate::DependencyManager;

/// Exit codes for the CLI application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success - all checks passed
    Success = 0,
    /// Missing dependencies (tool not installed or missing dependencies)
    MissingDependencies = 1,
    /// Invalid arguments (unknown tool name)
    InvalidArguments = 2,
    /// Internal error (detection failures or other errors)
    InternalError = 3,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

/// Errors that can occur when running the application
#[derive(Debug, Error)]
pub enum AppError {
    /// Failed to execute the check command
    ///
    /// This occurs when the check command fails to verify dependencies.
    #[error("Check command failed: {source}")]
    CheckFailed {
        #[source]
        source: CheckError,
    },

    /// Failed to execute the list command
    ///
    /// This occurs when the list command fails to list dependencies.
    #[error("List command failed: {source}")]
    ListFailed {
        #[source]
        source: ListError,
    },
}

impl AppError {
    /// Convert the error to an appropriate exit code for the CLI
    ///
    /// # Exit Codes
    ///
    /// - `ExitCode::MissingDependencies`: Tool not installed or missing dependencies
    /// - `ExitCode::InvalidArguments`: Unknown tool name
    /// - `ExitCode::InternalError`: Detection failures or other errors
    #[must_use]
    pub fn to_exit_code(&self) -> ExitCode {
        use crate::handlers::check::{
            CheckAllToolsError, CheckError, CheckSpecificToolError, ParseToolNameError,
        };

        match self {
            Self::CheckFailed { source } => match source {
                CheckError::CheckAllFailed { source } => match source {
                    CheckAllToolsError::MissingDependencies { .. } => ExitCode::MissingDependencies,
                    CheckAllToolsError::DependencyCheckFailed { .. } => ExitCode::InternalError,
                },
                CheckError::CheckSpecificFailed { source } => match source {
                    CheckSpecificToolError::ParseFailed {
                        source: ParseToolNameError::UnknownTool { .. },
                    } => ExitCode::InvalidArguments,
                    CheckSpecificToolError::ToolNotInstalled { .. } => {
                        ExitCode::MissingDependencies
                    }
                    CheckSpecificToolError::DetectionFailed { .. } => ExitCode::InternalError,
                },
            },
            Self::ListFailed { .. } => ExitCode::InternalError,
        }
    }
}

impl From<CheckError> for AppError {
    fn from(source: CheckError) -> Self {
        Self::CheckFailed { source }
    }
}

impl From<ListError> for AppError {
    fn from(source: ListError) -> Self {
        Self::ListFailed { source }
    }
}

/// Run the CLI application
///
/// # Errors
///
/// Returns an error if:
/// - Dependencies are missing
/// - Invalid tool name is provided
/// - Internal error occurs during dependency checking
pub fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    // Initialize tracing based on verbose flag
    // Must set environment variable before calling init_tracing()
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    crate::init_tracing();

    let manager = DependencyManager::new();

    match cli.command {
        Commands::Check { tool } => crate::handlers::check::handle_check(&manager, tool)?,
        Commands::List => crate::handlers::list::handle_list(&manager)?,
    }

    Ok(())
}
