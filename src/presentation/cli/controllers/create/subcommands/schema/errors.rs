//! Errors for Create Schema Command (Presentation Layer)

use thiserror::Error;

use crate::application::command_handlers::create::schema::CreateSchemaCommandHandlerError;
use crate::presentation::cli::views::progress::ProgressReporterError;

/// Errors that can occur during schema creation command execution
#[derive(Debug, Error)]
pub enum CreateSchemaCommandError {
    /// Failed to acquire user output lock
    #[error("Failed to acquire user output lock")]
    UserOutputLockFailed,

    /// Command handler execution failed
    #[error("Schema generation command failed")]
    CommandFailed {
        /// The underlying handler error
        #[source]
        source: CreateSchemaCommandHandlerError,
    },

    /// Progress reporter error
    #[error("Progress reporter error")]
    ProgressReporterFailed {
        /// The underlying progress reporter error
        #[source]
        source: ProgressReporterError,
    },
}

impl From<ProgressReporterError> for CreateSchemaCommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReporterFailed { source }
    }
}

impl CreateSchemaCommandError {
    /// Returns actionable help text for resolving this error
    ///
    /// Following the project's tiered help system pattern.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::UserOutputLockFailed => "Failed to acquire user output lock.\n\
                 \n\
                 This is typically caused by internal concurrency issues.\n\
                 \n\
                 What to do:\n\
                 1. Try running the command again\n\
                 2. If the problem persists, report it as a bug"
                .to_string(),
            Self::CommandFailed { source } => {
                format!(
                    "Schema generation command failed.\n\
                     \n\
                     {}\n\
                     \n\
                     If you need further assistance, check the documentation or report an issue.",
                    source.help()
                )
            }
            Self::ProgressReporterFailed { .. } => "Progress reporting failed.\n\
                 \n\
                 This is an internal error with the progress display system.\n\
                 \n\
                 What to do:\n\
                 1. The command may have still succeeded - check the output\n\
                 2. If the problem persists, report it as a bug"
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_text_for_user_output_lock_failed() {
        let error = CreateSchemaCommandError::UserOutputLockFailed;
        let help = error.help();
        assert!(help.contains("What to do:"));
        assert!(help.contains("concurrency"));
    }

    #[test]
    fn it_should_provide_help_text_for_progress_reporter_failed() {
        let error = CreateSchemaCommandError::ProgressReporterFailed {
            source: ProgressReporterError::UserOutputMutexPoisoned,
        };
        let help = error.help();
        assert!(help.contains("What to do:"));
    }
}
