//! Docs Command Controller (Presentation Layer)
//!
//! Handles the presentation layer concerns for CLI JSON documentation generation,
//! including user output and progress reporting.
//!
//! ## Architecture Note
//!
//! This controller directly uses infrastructure-layer services without going
//! through the application layer. This is architecturally correct because:
//!
//! - CLI documentation generation is a **presentation concern** (self-documentation)
//! - There is no business logic or orchestration (not a use case)
//! - Application layer would be unnecessary indirection
//!
//! Compare with `create schema` which generates business DTOs - that correctly
//! goes through application layer because it documents business configuration.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::infrastructure::cli_docs::CliDocsGenerator;
use crate::presentation::input::cli::Cli;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::DocsCommandError;

/// Steps for CLI documentation generation workflow
enum DocsStep {
    GenerateDocs,
}

impl DocsStep {
    fn description(&self) -> &str {
        match self {
            Self::GenerateDocs => "Generating CLI JSON documentation",
        }
    }

    fn count() -> usize {
        1
    }
}

/// Controller for docs command
///
/// Handles the presentation layer for CLI JSON documentation generation,
/// coordinating between the command handler and user output.
pub struct DocsCommandController {
    progress: ProgressReporter,
}

impl DocsCommandController {
    /// Create a new CLI documentation generation command controller
    pub fn new(user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        let progress = ProgressReporter::new(user_output.clone(), DocsStep::count());

        Self { progress }
    }

    /// Execute the CLI documentation generation command
    ///
    /// Generates CLI JSON documentation and either writes to file or outputs to stdout.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Optional path to write schema file. If `None`, outputs to stdout.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or error if generation or output fails.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - CLI documentation generation fails
    /// - File write fails (when path provided)
    /// - Parent directory creation fails (when path provided)
    /// - Stdout write fails (when no path provided)
    pub fn execute(&mut self, output_path: Option<&PathBuf>) -> Result<(), DocsCommandError> {
        // Generate CLI documentation using infrastructure layer directly
        let docs = CliDocsGenerator::generate::<Cli>()
            .map_err(|source| DocsCommandError::SchemaGenerationFailed { source })?;

        // Handle output based on destination
        if let Some(path) = output_path {
            // When writing to file, show progress to user
            self.progress
                .start_step(DocsStep::GenerateDocs.description())?;

            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|source| {
                    DocsCommandError::DirectoryCreationFailed {
                        path: parent.to_path_buf(),
                        source,
                    }
                })?;
            }

            // Write documentation to file
            std::fs::write(path, &docs).map_err(|source| DocsCommandError::FileWriteFailed {
                path: path.clone(),
                source,
            })?;

            self.progress
                .complete_step(Some("CLI documentation written to file successfully"))?;
            self.progress
                .complete("CLI documentation generation completed successfully")?;
        } else {
            // When writing to stdout, only output the documentation (no progress messages)
            // This enables clean piping: `cmd docs > file.json`
            self.progress.result(&docs)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::views::testing::test_user_output::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;
    use tempfile::TempDir;

    #[test]
    fn it_should_generate_cli_schema_to_file_when_path_provided() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("docs.json");

        let (user_output, _capture, _capture_stderr) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let mut controller = DocsCommandController::new(&user_output);

        let result = controller.execute(Some(&schema_path));
        assert!(result.is_ok());

        // Verify file was created
        assert!(schema_path.exists());

        // Verify file contains valid CLI documentation
        let content = std::fs::read_to_string(&schema_path).unwrap();
        assert!(content.contains("\"format\""));
        assert!(content.contains("\"format_version\""));
        assert!(content.contains("\"cli\""));
    }

    #[test]
    fn it_should_complete_progress_when_generating_cli_schema() {
        let (user_output, _capture, _capture_stderr) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let mut controller = DocsCommandController::new(&user_output);

        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("docs.json");

        let result = controller.execute(Some(&schema_path));
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_output_to_stdout_when_no_path_provided() {
        let (user_output, capture, _capture_stderr) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let mut controller = DocsCommandController::new(&user_output);

        let result = controller.execute(None);
        assert!(result.is_ok());

        // Verify documentation was written to stdout
        let output = String::from_utf8(capture.lock().clone()).unwrap();
        assert!(output.contains("\"format\""));
        assert!(output.contains("\"format_version\""));
        assert!(output.contains("\"cli\""));
    }

    #[test]
    fn it_should_generate_valid_cli_schema_structure() {
        let (user_output, capture, _capture_stderr) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let mut controller = DocsCommandController::new(&user_output);

        let result = controller.execute(None);
        assert!(result.is_ok());

        let output = String::from_utf8(capture.lock().clone()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Verify CLI documentation structure
        assert!(json.get("format").is_some());
        assert_eq!(json.get("format").unwrap(), "cli-documentation");
        assert!(json.get("format_version").is_some());
        assert!(json.get("cli").is_some());

        let cli = json.get("cli").unwrap();
        assert!(cli.get("name").is_some());
        assert!(cli.get("version").is_some());
        assert!(cli.get("commands").is_some());
    }
}
