//! Template Command Errors
//!
//! This module defines error types specific to template generation commands.
//! These errors provide detailed context and actionable guidance for template-related failures.

use std::path::PathBuf;
use thiserror::Error;

use crate::presentation::cli::views::progress::ProgressReporterError;

/// Errors that can occur during template generation commands
///
/// This error type covers all failure scenarios that can occur during template
/// generation operations, including file I/O errors, validation failures, and
/// system-level issues.
///
/// All variants include structured context to aid in debugging and provide
/// actionable guidance to users through the `.help()` method.
#[derive(Debug, Error)]
pub enum CreateEnvironmentTemplateCommandError {
    /// Failed to acquire lock on `UserOutput` for displaying progress/results
    ///
    /// This error occurs when the `UserOutput` mutex is poisoned, typically indicating
    /// that another thread panicked while holding the output lock. This is a system-level
    /// error that should be rare in normal operation.
    #[error(
        "Failed to acquire output lock for displaying template generation progress
Tip: This indicates a system error - try restarting the command"
    )]
    UserOutputLockFailed,

    /// Template file generation failed
    ///
    /// This error occurs when the underlying template generation operation fails,
    /// which could be due to file system permissions, disk space, or template
    /// processing errors.
    #[error(
        "Failed to generate configuration template at '{path}': {source}
Tip: Check directory permissions and available disk space"
    )]
    TemplateGenerationFailed {
        /// Path where template generation was attempted
        path: PathBuf,
        /// The underlying error that caused template generation to fail
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Progress reporting failed
    ///
    /// This error occurs when the progress reporting system fails, typically
    /// due to mutex poisoning or other critical system-level issues.
    #[error(
        "Failed to report progress: {source}
Tip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    ProgressReportingFailed {
        #[source]
        source: ProgressReporterError,
    },
}

impl CreateEnvironmentTemplateCommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::cli::controllers::create::subcommands::template::CreateEnvironmentTemplateCommandError;
    ///
    /// fn handle_error(error: CreateEnvironmentTemplateCommandError) {
    ///     eprintln!("Error: {error}");
    ///     eprintln!("\nTroubleshooting:\n{}", error.help());
    /// }
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::UserOutputLockFailed => {
                "User Output Lock Failed - Detailed Troubleshooting:

1. This is a rare system-level error that occurs when another thread panics
   while displaying output to the user.

2. Try the following steps:
   - Restart the command
   - If the error persists, restart your terminal
   - Check for any background processes that might interfere

3. If the problem continues:
   - Check system resources (memory, disk space)
   - Consider running with --verbose for more detailed error information
   - Report the issue if it's reproducible

For more information, see the troubleshooting guide."
            }

            Self::TemplateGenerationFailed { .. } => {
                "Template Generation Failed - Detailed Troubleshooting:

1. Check directory permissions:
   - Ensure write permissions for the target directory
   - Verify parent directory exists
   - Check ownership of the target path

2. Verify available resources:
   - Check available disk space: df -h (Unix) or dir (Windows)
   - Ensure sufficient memory is available

3. File system issues:
   - Check if the path contains invalid characters
   - Verify the path length is within system limits
   - Ensure no other process is using the target file

4. Template processing issues:
   - Check if required template data is available
   - Verify template format is valid
   - Look for any corrupted template files

If the problem persists, run with --verbose for detailed logs and report
the issue with system details."
            }

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Detailed Troubleshooting:

1. This is a critical system-level error that occurs when the progress
   reporting system fails, typically due to mutex poisoning.

2. Try the following steps:
   - Restart the command
   - If the error persists, restart your terminal
   - Check for any background processes that might interfere

3. If the problem continues:
   - Check system resources (memory, CPU usage)
   - Consider running with --verbose for more detailed error information
   - Report the issue with full logs as this indicates a bug

For more information, see the troubleshooting guide."
            }
        }
    }
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for CreateEnvironmentTemplateCommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::path::PathBuf;

    #[test]
    fn it_should_display_user_output_lock_failed_error() {
        let error = CreateEnvironmentTemplateCommandError::UserOutputLockFailed;
        let message = error.to_string();

        assert!(message.contains("Failed to acquire output lock"));
        assert!(message.contains("Tip: This indicates a system error"));
    }

    #[test]
    fn it_should_display_template_generation_failed_error() {
        let path = PathBuf::from("/test/template.json");
        let source: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Access denied",
        ));

        let error = CreateEnvironmentTemplateCommandError::TemplateGenerationFailed {
            path: path.clone(),
            source,
        };
        let message = error.to_string();

        assert!(message.contains("Failed to generate configuration template"));
        assert!(message.contains("/test/template.json"));
        assert!(message.contains("Tip: Check directory permissions"));
    }

    #[test]
    fn it_should_have_help_for_all_variants() {
        let errors = vec![
            CreateEnvironmentTemplateCommandError::UserOutputLockFailed,
            CreateEnvironmentTemplateCommandError::TemplateGenerationFailed {
                path: PathBuf::from("/test"),
                source: Box::new(std::io::Error::other("test")),
            },
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Troubleshooting") || help.len() > 50,
                "Help should contain actionable guidance"
            );
        }
    }

    #[test]
    fn it_should_preserve_error_chain() {
        let source_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = CreateEnvironmentTemplateCommandError::TemplateGenerationFailed {
            path: PathBuf::from("/test"),
            source: Box::new(source_error),
        };

        // Check that source error is preserved
        assert!(error.source().is_some(), "Source error should be preserved");

        // Check error chain
        let chain = std::error::Error::source(&error);
        assert!(chain.is_some(), "Error chain should exist");

        if let Some(source) = chain {
            assert!(source.to_string().contains("File not found"));
        }
    }
}
