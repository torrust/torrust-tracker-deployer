//! Render Command Controller
//!
//! This module handles the render command execution at the presentation layer,
//! including input validation, mode selection, and user feedback.

use std::cell::RefCell;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

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
/// - Delegate artifact generation to application layer (Phase 2)
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. Business logic will be delegated to the application layer's
/// `RenderCommandHandler` in Phase 2.
pub struct RenderCommandController {
    progress: ProgressReporter,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
}

impl RenderCommandController {
    /// Create a new render command controller
    ///
    /// Creates a `RenderCommandController` with user output.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        Self {
            progress: ProgressReporter::new(Arc::clone(&user_output), RenderStep::count()),
            user_output,
        }
    }

    /// Execute the render command workflow
    ///
    /// This performs input validation and shows progress steps.
    /// Actual artifact generation will be implemented in Phase 2.
    ///
    /// # Arguments
    ///
    /// * `env_name` - Optional environment name (mutually exclusive with env_file)
    /// * `env_file` - Optional config file path (mutually exclusive with env_name)
    /// * `ip` - Target instance IP address (required)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Stub execution succeeded
    /// * `Err(RenderCommandError)` - Input validation failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Neither env_name nor env_file is provided
    /// - IP address is invalid
    /// - Config file doesn't exist
    pub fn execute(
        &mut self,
        env_name: Option<&str>,
        env_file: Option<&Path>,
        ip: &str,
    ) -> Result<(), RenderCommandError> {
        // Step 1: Validate input
        self.progress.start_step(RenderStep::ValidateInput.description())?;

        // Validate input mode
        let (mode_desc, source) = match (env_name, env_file) {
            (Some(name), None) => ("environment", format!("Environment: {name}")),
            (None, Some(path)) => {
                // Validate file exists
                if !path.exists() {
                    return Err(RenderCommandError::ConfigFileNotFound {
                        path: path.to_path_buf(),
                    });
                }
                ("config file", format!("Config file: {}", path.display()))
            }
            (None, None) => return Err(RenderCommandError::NoInputMode),
            (Some(_), Some(_)) => unreachable!("Clap ensures mutual exclusivity"),
        };

        // Parse and validate IP address
        let target_ip: Ipv4Addr = ip.parse().map_err(|_| RenderCommandError::InvalidIpAddress {
            ip: ip.to_string(),
        })?;

        self.progress.complete_step(None)?;

        // Step 2: Load configuration (stub)
        self.progress.start_step(RenderStep::LoadConfiguration.description())?;
        self.progress.complete_step(None)?;

        // Step 3: Generate artifacts (stub)
        self.progress.start_step(RenderStep::GenerateArtifacts.description())?;
        self.progress.complete_step(None)?;

        // Show success message
        self.show_success(&source, target_ip, mode_desc)?;

        // Indicate this is stub implementation
        self.show_stub_message()?;

        Ok(())
    }

    /// Show success message to user
    fn show_success(
        &mut self,
        source: &str,
        target_ip: Ipv4Addr,
        _mode: &str,
    ) -> Result<(), RenderCommandError> {
        let output = self.user_output.lock();
        let mut output_ref = output.borrow_mut();

        output_ref.success(&format!(
            "\nArtifacts would be generated for:\n  \
             Source: {source}\n  \
             Target IP: {target_ip}\n  \
             Output: build/<env-name>/"
        ));

        Ok(())
    }

    /// Show stub implementation message
    fn show_stub_message(&mut self) -> Result<(), RenderCommandError> {
        let output = self.user_output.lock();
        let mut output_ref = output.borrow_mut();

        output_ref.progress(
            "\n[Phase 1 Complete]\n\
             This is a presentation layer stub. Phase 2 will implement:\n\
             - Application handler with business logic\n\
             - Template rendering orchestration\n\
             - State validation (Created state only)\n\
             - Actual artifact generation to build/"
        );

        Ok(())
    }
}
