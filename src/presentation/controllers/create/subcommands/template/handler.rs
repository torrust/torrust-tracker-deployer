//! Template Creation Command Handler
//!
//! This module handles the template creation command execution at the presentation layer,
//! including output path validation, template generation, and user guidance.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::domain::provider::Provider;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::CreateEnvironmentTemplateCommandError;

/// Steps in the template creation workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreateTemplateStep {
    GenerateTemplate,
    PrepareGuidance,
}

impl CreateTemplateStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[Self::GenerateTemplate, Self::PrepareGuidance];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::GenerateTemplate => "Generating configuration template",
            Self::PrepareGuidance => "Preparing user guidance",
        }
    }
}

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
        let progress = ProgressReporter::new(user_output.clone(), CreateTemplateStep::count());

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
    /// * `provider` - Provider to generate template for (lxd or hetzner)
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
    #[allow(clippy::unused_async)] // Part of uniform async presentation layer interface
    pub async fn execute(
        &mut self,
        output_path: &Path,
        provider: Provider,
    ) -> Result<(), CreateEnvironmentTemplateCommandError> {
        self.generate_template_file(output_path, provider)?;
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
        provider: Provider,
    ) -> Result<(), CreateEnvironmentTemplateCommandError> {
        self.progress
            .start_step(CreateTemplateStep::GenerateTemplate.description())?;

        // Use synchronous version to avoid creating a tokio runtime
        // This prevents blocking and performance issues in test environments
        EnvironmentCreationConfig::generate_template_file(output_path, provider).map_err(
            |source| CreateEnvironmentTemplateCommandError::TemplateGenerationFailed {
                path: output_path.to_path_buf(),
                source: Box::new(source),
            },
        )?;

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
        self.progress
            .start_step(CreateTemplateStep::PrepareGuidance.description())?;

        // Use ProgressReporter wrapper methods to avoid dual mutex acquisition
        self.progress.blank_line()?;
        self.progress.steps(
            "Next steps:",
            &[
                "Edit the template file and replace placeholder values:\n   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')\n   - REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH: Path to your SSH private key\n   - REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH: Path to your SSH public key",
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
    use crate::domain::provider::Provider;
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_generate_template_without_progress() {
        use crate::application::command_handlers::create::config::EnvironmentCreationConfig;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");

        // Test just the template generation without progress reporter
        let result = EnvironmentCreationConfig::generate_template_file(&output_path, Provider::Lxd);

        assert!(result.is_ok(), "Template generation should work");
        assert!(output_path.exists(), "File should be created");

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
    }

    #[tokio::test]
    async fn it_should_create_template_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Silent).into_reentrant_wrapped(); // Use Silent to avoid output issues

        let result = CreateTemplateCommandController::new(&user_output)
            .execute(&output_path, Provider::Lxd)
            .await;

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

    #[tokio::test]
    async fn it_should_execute_controller_directly() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Silent).into_reentrant_wrapped();

        // Test controller directly
        let mut controller = CreateTemplateCommandController::new(&user_output);
        let result = controller.execute(&output_path, Provider::Lxd).await;

        assert!(result.is_ok(), "Controller should succeed: {result:?}");
        assert!(output_path.exists(), "File should be created");
    }

    #[test]
    fn it_should_call_sync_method_directly() {
        use crate::application::command_handlers::create::config::EnvironmentCreationConfig;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-template.json");

        // Test the synchronous method directly
        let result = EnvironmentCreationConfig::generate_template_file(&output_path, Provider::Lxd);

        assert!(result.is_ok(), "Sync method should work: {result:?}");
        assert!(output_path.exists(), "File should be created");

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("REPLACE_WITH_ENVIRONMENT_NAME"));
    }

    #[tokio::test]
    async fn it_should_handle_invalid_output_path() {
        // Try to create template in a non-existent directory
        let invalid_path = std::path::Path::new("/non/existent/directory/template.json");
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();

        let result = CreateTemplateCommandController::new(&user_output)
            .execute(invalid_path, Provider::Lxd)
            .await;

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
