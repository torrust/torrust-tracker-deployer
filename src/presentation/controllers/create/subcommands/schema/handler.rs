//! Create Schema Command Controller (Presentation Layer)
//!
//! Handles the presentation layer concerns for JSON Schema generation,
//! including user output and progress reporting.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::create::schema::CreateSchemaCommandHandler;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::CreateSchemaCommandError;

/// Steps for schema generation workflow
enum CreateSchemaStep {
    GenerateSchema,
}

impl CreateSchemaStep {
    fn description(&self) -> &str {
        match self {
            Self::GenerateSchema => "Generating JSON Schema",
        }
    }

    fn count() -> usize {
        1
    }
}

/// Controller for create schema command
///
/// Handles the presentation layer for JSON Schema generation,
/// coordinating between the command handler and user output.
pub struct CreateSchemaCommandController {
    progress: ProgressReporter,
}

impl CreateSchemaCommandController {
    /// Create a new schema generation command controller
    pub fn new(user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        let progress = ProgressReporter::new(user_output.clone(), CreateSchemaStep::count());

        Self { progress }
    }

    /// Execute the schema generation command
    ///
    /// Generates JSON Schema and either writes to file or outputs to stdout.
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
    /// - Schema generation fails
    /// - File write fails (when path provided)
    /// - Stdout write fails (when no path provided)
    pub fn execute(
        &mut self,
        output_path: Option<&PathBuf>,
    ) -> Result<(), CreateSchemaCommandError> {
        self.progress
            .start_step(CreateSchemaStep::GenerateSchema.description())?;

        // Generate schema using application layer handler
        let schema = CreateSchemaCommandHandler::execute(output_path.cloned())
            .map_err(|source| CreateSchemaCommandError::CommandFailed { source })?;

        // Handle output
        if output_path.is_some() {
            self.progress
                .complete_step(Some("Schema written to file successfully"))?;
        } else {
            // Output to stdout using ProgressReporter abstraction
            self.progress.complete_step(Some("Schema generated"))?;

            // Write schema to stdout (result data goes to stdout, not stderr)
            self.progress.result(&schema)?;
        }

        self.progress
            .complete("Schema generation completed successfully")?;

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
    fn it_should_generate_schema_to_file_when_path_provided() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("schema.json");

        let (user_output, _capture, _capture_stderr) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let mut controller = CreateSchemaCommandController::new(&user_output);

        let result = controller.execute(Some(&schema_path));
        assert!(result.is_ok());

        // Verify file was created
        assert!(schema_path.exists());

        // Verify file contains valid JSON schema
        let content = std::fs::read_to_string(&schema_path).unwrap();
        assert!(content.contains("\"$schema\""));
    }

    #[test]
    fn it_should_complete_progress_when_generating_schema() {
        let (user_output, _capture, _capture_stderr) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let mut controller = CreateSchemaCommandController::new(&user_output);

        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("test.json");

        let result = controller.execute(Some(&schema_path));
        assert!(result.is_ok());
    }
}
