//! Render Command Controller
//!
//! This module handles the render command execution at the presentation layer,
//! including input validation, mode selection, and user feedback.

use std::cell::RefCell;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::render::{RenderCommandHandler, RenderInputMode};
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::EnvironmentName;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::RenderCommandError;

/// Steps in the render workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderStep {
    ValidateInput,
    LoadConfiguration,
    GenerateArtifacts,
}

impl RenderStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateInput,
        Self::LoadConfiguration,
        Self::GenerateArtifacts,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateInput => "Validating input parameters",
            Self::LoadConfiguration => "Loading configuration",
            Self::GenerateArtifacts => "Generating deployment artifacts",
        }
    }
}

/// Presentation layer controller for render command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// for generating deployment artifacts without executing deployment.
///
/// # Responsibilities
///
/// - Validate input mode (env-name OR env-file)
/// - Parse and validate IP address
/// - Show progress updates to the user
/// - Format output for display
/// - Delegate artifact generation to application layer handler
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. Business logic is delegated to the application layer's
/// `RenderCommandHandler`.
pub struct RenderCommandController {
    handler: RenderCommandHandler,
    progress: ProgressReporter,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
}

impl RenderCommandController {
    /// Create a new render command controller
    ///
    /// Creates a `RenderCommandController` with repository and user output.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        Self {
            handler: RenderCommandHandler::new(repository),
            progress: ProgressReporter::new(Arc::clone(&user_output), RenderStep::count()),
            user_output,
        }
    }

    /// Execute the render command workflow
    ///
    /// This performs input validation and delegates to the application handler.
    ///
    /// # Arguments
    ///
    /// * `env_name` - Optional environment name (mutually exclusive with `env_file`)
    /// * `env_file` - Optional config file path (mutually exclusive with `env_name`)
    /// * `ip` - Target instance IP address (required)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Artifact generation succeeded
    /// * `Err(RenderCommandError)` - Validation or generation failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Neither `env_name` nor `env_file` is provided
    /// - IP address is invalid
    /// - Config file doesn't exist
    /// - Environment not found or wrong state
    /// - Template rendering fails
    pub async fn execute(
        &mut self,
        env_name: Option<&str>,
        env_file: Option<&Path>,
        ip: &str,
    ) -> Result<(), RenderCommandError> {
        // Step 1: Validate input
        self.progress
            .start_step(RenderStep::ValidateInput.description())?;

        // Validate IP address first (fail fast)
        let _target_ip: Ipv4Addr = ip
            .parse()
            .map_err(|_| RenderCommandError::InvalidIpAddress { ip: ip.to_string() })?;

        // Determine input mode and prepare handler parameters
        let (input_mode, source_desc) = match (env_name, env_file) {
            (Some(name), None) => {
                let env_name = EnvironmentName::new(name).map_err(|e| {
                    RenderCommandError::InvalidEnvironmentName {
                        value: name.to_string(),
                        reason: e.to_string(),
                    }
                })?;
                (
                    RenderInputMode::EnvironmentName(env_name.clone()),
                    format!("Environment: {env_name}"),
                )
            }
            (None, Some(path)) => {
                // Validate file exists
                if !path.exists() {
                    return Err(RenderCommandError::ConfigFileNotFound {
                        path: path.to_path_buf(),
                    });
                }
                (
                    RenderInputMode::ConfigFile(path.to_path_buf()),
                    format!("Config file: {}", path.display()),
                )
            }
            (None, None) => return Err(RenderCommandError::NoInputMode),
            (Some(_), Some(_)) => unreachable!("Clap ensures mutual exclusivity"),
        };

        self.progress.complete_step(None)?;

        // Step 2: Load configuration and validate
        self.progress
            .start_step(RenderStep::LoadConfiguration.description())?;

        // Get working directory
        let working_dir = std::env::current_dir().map_err(|e| {
            RenderCommandError::WorkingDirectoryUnavailable {
                reason: e.to_string(),
            }
        })?;

        self.progress.complete_step(None)?;

        // Step 3: Generate artifacts
        self.progress
            .start_step(RenderStep::GenerateArtifacts.description())?;

        // Call application handler
        let result = self
            .handler
            .execute(input_mode, ip, &working_dir)
            .await
            .map_err(RenderCommandError::from)?;

        self.progress.complete_step(None)?;

        // Show success message
        self.show_success(
            &source_desc,
            &result.target_ip.to_string(),
            &result.output_dir,
        );

        Ok(())
    }

    /// Show success message to user
    fn show_success(&mut self, source: &str, target_ip: &str, output_dir: &Path) {
        let output = self.user_output.lock();
        let mut output_ref = output.borrow_mut();

        output_ref.success(&format!(
            "\nDeployment artifacts generated successfully!\n\n  \
             Source: {source}\n  \
             Target IP: {target_ip}\n  \
             Output: {}\n\n\
             Next steps:\n  \
             - Review artifacts in the output directory\n  \
             - Use 'provision' command to deploy infrastructure\n  \
             - Or use artifacts manually with your deployment tools",
            output_dir.display()
        ));
    }
}
