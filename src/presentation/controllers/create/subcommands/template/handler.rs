//! Template Creation Command Handler
//!
//! This module handles the template creation command execution at the presentation layer,
//! including output path validation, template generation, and user guidance.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::CreateEnvironmentTemplateCommandError;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Number of main steps in the template creation workflow
const TEMPLATE_CREATION_WORKFLOW_STEPS: usize = 2;

// ============================================================================
// HIGH-LEVEL API (EXECUTION CONTEXT PATTERN)
// ============================================================================

/// Handle template creation command using `ExecutionContext` pattern
///
/// This function provides a clean interface for generating configuration templates,
/// integrating with the `ExecutionContext` pattern for dependency injection.
///
/// # Arguments
///
/// * `output_path` - Path where the template file should be created
/// * `context` - Execution context providing access to services
///
/// # Returns
///
/// * `Ok(())` - Template generated successfully
/// * `Err(CreateEnvironmentTemplateCommandError)` - Template generation failed
///
/// # Errors
///
/// Returns `CreateEnvironmentTemplateCommandError` when:
/// * Output path is not accessible or parent directory doesn't exist
/// * File system operations fail (permission errors, disk space)
/// * Template processing encounters errors
/// * User output system fails (mutex poisoning)
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::presentation::controllers::create::subcommands::template;
/// use torrust_tracker_deployer_lib::presentation::dispatch::context::ExecutionContext;
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let container = Arc::new(Container::new(VerbosityLevel::Normal));
/// let context = ExecutionContext::new(container);
/// let output_path = Path::new("./environment-template.json");
///
/// template::handle(output_path, &context)?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle(
    output_path: &Path,
    context: &crate::presentation::dispatch::context::ExecutionContext,
) -> Result<(), CreateEnvironmentTemplateCommandError> {
    handle_template_creation_command(output_path, &context.user_output())
}

// ============================================================================
// INTERMEDIATE API (DIRECT DEPENDENCY INJECTION)
// ============================================================================

/// Handle the template creation command
///
/// This is a thin wrapper over `CreateTemplateCommandController` that serves as
/// the public entry point for the template creation command.
///
/// # Arguments
///
/// * `output_path` - Path where the template file should be created
/// * `user_output` - Shared user output service for consistent output formatting
///
/// # Errors
///
/// Returns an error if:
/// - Output path validation fails
/// - Template generation fails  
/// - Progress reporting encounters a poisoned mutex
///
/// All errors include detailed context and actionable troubleshooting guidance.
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateEnvironmentTemplateCommandError` on failure.
///
/// # Example
///
/// Using with Container and `ExecutionContext` (recommended):
///
/// ```rust,no_run
/// use std::path::Path;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::bootstrap::Container;
/// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
/// use torrust_tracker_deployer_lib::presentation::controllers::create::subcommands::template;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// let container = Container::new(VerbosityLevel::Normal);
/// let context = ExecutionContext::new(Arc::new(container));
///
/// if let Err(e) = template::handle(Path::new("template.json"), &context) {
///     eprintln!("Template creation failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
///
/// Direct usage (for testing or specialized scenarios):
///
/// ```rust,no_run
/// use std::path::Path;
/// use std::sync::Arc;
/// use parking_lot::RawMutex;
/// use parking_lot::ReentrantMutex;
/// use std::cell::RefCell;
/// use torrust_tracker_deployer_lib::presentation::controllers::create::subcommands::template::handler::handle_template_creation_command;
/// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
///
/// let user_output = Arc::new(ReentrantMutex::<RefCell<UserOutput>>::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
///
/// if let Err(e) = handle_template_creation_command(Path::new("template.json"), &user_output) {
///     eprintln!("Template creation failed: {e}");
///     eprintln!("Help: {}", e.help());
/// }
/// ```
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_template_creation_command(
    output_path: &Path,
    user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>,
) -> Result<(), CreateEnvironmentTemplateCommandError> {
    CreateTemplateCommandController::new(user_output).execute(output_path)
}

// ============================================================================
// PRESENTATION LAYER CONTROLLER (IMPLEMENTATION DETAILS)
// ============================================================================

/// Presentation layer controller for template creation command workflow
///
/// Coordinates user interaction, progress reporting, and output formatting
/// before delegating to the application layer template generation logic.
///
/// # Responsibilities
///
/// - Validate output path accessibility  
/// - Show progress updates to the user
/// - Format success/error messages for display
/// - Generate user guidance for next steps
/// - Delegate template generation to application layer
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. It delegates actual template generation to the application layer's
/// `EnvironmentCreationConfig::generate_template_file`, maintaining clear separation of concerns.
pub struct CreateTemplateCommandController {
    progress: ProgressReporter,
}

impl CreateTemplateCommandController {
    /// Create a new template creation command controller  
    ///
    /// Creates a `CreateTemplateCommandController` with user output service injection.
    /// This follows the single container architecture pattern.
    pub fn new(user_output: &Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        let progress = ProgressReporter::new(user_output.clone(), TEMPLATE_CREATION_WORKFLOW_STEPS);

        Self { progress }
    }

    /// Execute the complete template creation workflow
    ///
    /// Orchestrates all steps of the template creation command:
    /// 1. Validate output path
    /// 2. Generate template file
    /// 3. Display success message and next steps guidance
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path where the template file should be created
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template generation fails
    /// - Progress reporting encounters a poisoned mutex
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `CreateEnvironmentTemplateCommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    pub fn execute(
        &mut self,
        output_path: &Path,
    ) -> Result<(), CreateEnvironmentTemplateCommandError> {
        self.generate_template_file(output_path)?;
        self.display_success_and_guidance(output_path)?;
        Ok(())
    }

    /// Generate the template file via application layer
    ///
    /// Delegates to the application layer template generation logic to
    /// create the configuration template file.
    #[allow(clippy::result_large_err)]
    fn generate_template_file(
        &mut self,
        output_path: &Path,
    ) -> Result<(), CreateEnvironmentTemplateCommandError> {
        self.progress
            .start_step("Generating configuration template")?;

        // Use synchronous version to avoid creating a tokio runtime
        // This prevents blocking and performance issues in test environments
        EnvironmentCreationConfig::generate_template_file(output_path).map_err(|source| {
            CreateEnvironmentTemplateCommandError::TemplateGenerationFailed {
                path: output_path.to_path_buf(),
                source: Box::new(source),
            }
        })?;

        self.progress.complete_step(Some(&format!(
            "Template generated: {}",
            output_path.display()
        )))?;

        Ok(())
    }

    /// Display success message and user guidance
    ///
    /// Shows the final success message and provides detailed guidance
    /// for next steps in the deployment process.
    #[allow(clippy::result_large_err)]
    fn display_success_and_guidance(
        &mut self,
        output_path: &Path,
    ) -> Result<(), CreateEnvironmentTemplateCommandError> {
        self.progress.start_step("Preparing user guidance")?;

        // Use ProgressReporter wrapper methods to avoid dual mutex acquisition
        self.progress.blank_line()?;
        self.progress.steps(
            "Next steps:",
            &[
                "Edit the template file and replace placeholder values:\n   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')\n   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key\n   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key",
                "Review default values:\n   - username: 'torrust' (can be changed if needed)\n   - port: 22 (standard SSH port)",
                &format!(
                    "Create the environment:\n   torrust-tracker-deployer create environment --env-file {}",
                    output_path.display()
                ),
            ],
        )?;

        self.progress.complete(&format!(
            "Configuration template ready: {}",
            output_path.display()
        ))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::views::test_support::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_generate_template_without_progress() {
        use crate::application::command_handlers::create::config::EnvironmentCreationConfig;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");

        // Test just the template generation without progress reporter
        let result = EnvironmentCreationConfig::generate_template_file(&output_path);

        assert!(result.is_ok(), "Template generation should work");
        assert!(output_path.exists(), "File should be created");

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
    }

    #[test]
    fn it_should_create_template_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Silent).into_reentrant_wrapped(); // Use Silent to avoid output issues

        let result = handle_template_creation_command(&output_path, &user_output);

        // Should succeed in creating template
        assert!(
            result.is_ok(),
            "Template creation should succeed: {result:?}"
        );

        // Template file should exist
        assert!(output_path.exists(), "Template file should be created");

        // File should contain template content
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(
            content.contains("REPLACE_WITH_ENVIRONMENT_NAME"),
            "Should contain placeholder"
        );
    }

    #[test]
    fn it_should_execute_controller_directly() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Silent).into_reentrant_wrapped();

        // Test controller directly
        let mut controller = CreateTemplateCommandController::new(&user_output);
        let result = controller.execute(&output_path);

        assert!(result.is_ok(), "Controller should succeed: {result:?}");
        assert!(output_path.exists(), "File should be created");
    }

    #[test]
    fn it_should_call_sync_method_directly() {
        use crate::application::command_handlers::create::config::EnvironmentCreationConfig;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");

        // Test the synchronous method directly
        let result = EnvironmentCreationConfig::generate_template_file(&output_path);

        assert!(result.is_ok(), "Sync method should work: {result:?}");
        assert!(output_path.exists(), "File should be created");

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
    }

    #[test]
    fn it_should_handle_invalid_output_path() {
        // Try to create template in a non-existent directory
        let invalid_path = std::path::Path::new("/non/existent/directory/template.json");
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();

        let result = handle_template_creation_command(invalid_path, &user_output);

        assert!(result.is_err(), "Should fail for invalid path");
        match result.unwrap_err() {
            CreateEnvironmentTemplateCommandError::TemplateGenerationFailed { path, .. } => {
                assert_eq!(path, invalid_path);
            }
            other => panic!("Expected TemplateGenerationFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_create_controller_successfully() {
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let _controller = CreateTemplateCommandController::new(&user_output);

        // Controller should be created successfully
        // (Just testing that constructor works without panicking)
    }
}
