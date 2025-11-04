//! Application logic for the dependency installer CLI
//!
//! This module contains the core application logic for running the CLI.

// External crates
use clap::Parser;
use thiserror::Error;

// Internal crate
use crate::cli::{Cli, Commands};
use crate::handlers::check::CheckError;
use crate::handlers::list::ListError;
use crate::DependencyManager;

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

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

// ============================================================================
// PUBLIC API - Implementations
// ============================================================================

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

// ============================================================================
// PUBLIC API - Functions
// ============================================================================

/// Run the CLI application
///
/// Returns the appropriate exit code based on the operation result.
/// Errors are logged using tracing and do not propagate to stderr.
pub fn run() -> ExitCode {
    let cli = Cli::parse();

    // Determine log level: verbose flag overrides log-level argument
    let log_level = if cli.verbose {
        Some(tracing::Level::DEBUG)
    } else {
        cli.log_level.to_tracing_level()
    };

    // Initialize tracing with the determined log level
    crate::init_tracing(log_level);

    let manager = DependencyManager::new();

    let result: Result<(), AppError> = match cli.command {
        Commands::Check { dependency } => {
            crate::handlers::check::handle_check(&manager, dependency).map_err(AppError::from)
        }
        Commands::List => crate::handlers::list::handle_list(&manager).map_err(AppError::from),
    };

    match result {
        Ok(()) => ExitCode::Success,
        Err(e) => {
            // Log the error using tracing instead of eprintln
            tracing::error!(error = %e, "Command failed");
            e.to_exit_code()
        }
    }
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

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
    /// - `ExitCode::InvalidArguments`: Unknown dependency name
    /// - `ExitCode::InternalError`: Detection failures or other errors
    #[must_use]
    pub fn to_exit_code(&self) -> ExitCode {
        use crate::handlers::check::{
            CheckAllDependenciesError, CheckError, CheckSpecificDependencyError,
        };

        match self {
            Self::CheckFailed { source } => match source {
                CheckError::CheckAllFailed { source } => match source {
                    CheckAllDependenciesError::MissingDependencies { .. } => {
                        ExitCode::MissingDependencies
                    }
                    CheckAllDependenciesError::DependencyCheckFailed { .. } => {
                        ExitCode::InternalError
                    }
                },
                CheckError::CheckSpecificFailed { source } => match source {
                    CheckSpecificDependencyError::DependencyNotInstalled { .. } => {
                        ExitCode::MissingDependencies
                    }
                    CheckSpecificDependencyError::DetectionFailed { .. } => ExitCode::InternalError,
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
