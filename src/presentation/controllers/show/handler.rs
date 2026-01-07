//! Show Command Handler
//!
//! This module handles the show command execution at the presentation layer,
//! displaying environment information with state-aware details.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

use super::errors::ShowSubcommandError;

/// Steps in the show workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShowStep {
    ValidateEnvironment,
    LoadEnvironment,
    DisplayInformation,
}

impl ShowStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateEnvironment,
        Self::LoadEnvironment,
        Self::DisplayInformation,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment name",
            Self::LoadEnvironment => "Loading environment",
            Self::DisplayInformation => "Displaying information",
        }
    }
}

/// Presentation layer controller for show command workflow
///
/// Displays environment information with state-aware details.
/// This is a read-only command that shows stored data without remote verification.
///
/// ## Responsibilities
///
/// - Validate environment name format
/// - Load environment from repository
/// - Display state-aware information to the user
/// - Provide next-step guidance based on current state
///
/// ## Architecture
///
/// This controller implements the Presentation Layer pattern, handling
/// user interaction while delegating data access to the repository.
pub struct ShowCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}

impl ShowCommandController {
    /// Create a new `ShowCommandController` with dependencies
    ///
    /// # Arguments
    ///
    /// * `repository` - Environment repository for loading environment data
    /// * `user_output` - Shared output service for user feedback
    #[allow(clippy::needless_pass_by_value)] // Arc parameters are moved to constructor for ownership
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(Arc::clone(&user_output), ShowStep::count());

        Self {
            repository,
            user_output,
            progress,
        }
    }

    /// Execute the show command workflow
    ///
    /// This method orchestrates the three-step workflow:
    /// 1. Validate environment name
    /// 2. Load environment from repository
    /// 3. Display information to user
    ///
    /// # Arguments
    ///
    /// * `environment_name` - Name of the environment to show
    ///
    /// # Errors
    ///
    /// Returns `ShowSubcommandError` if any step fails
    pub fn execute(&mut self, environment_name: &str) -> Result<(), ShowSubcommandError> {
        // Step 1: Validate environment name
        let env_name = self.validate_environment_name(environment_name)?;

        // Step 2: Load environment
        let any_env = self.load_environment(&env_name)?;

        // Step 3: Display information
        self.display_information(&any_env)?;

        Ok(())
    }

    /// Step 1: Validate environment name format
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, ShowSubcommandError> {
        self.progress
            .start_step(ShowStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            ShowSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    /// Step 2: Load environment from repository
    fn load_environment(
        &mut self,
        env_name: &EnvironmentName,
    ) -> Result<crate::domain::environment::state::AnyEnvironmentState, ShowSubcommandError> {
        self.progress
            .start_step(ShowStep::LoadEnvironment.description())?;

        let any_env = self
            .repository
            .load(env_name)
            .map_err(|e| ShowSubcommandError::LoadError {
                name: env_name.to_string(),
                message: e.to_string(),
            })?
            .ok_or_else(|| ShowSubcommandError::EnvironmentNotFound {
                name: env_name.to_string(),
            })?;

        self.progress
            .complete_step(Some(&format!("Environment loaded: {env_name}")))?;

        Ok(any_env)
    }

    /// Step 3: Display environment information
    fn display_information(
        &mut self,
        any_env: &crate::domain::environment::state::AnyEnvironmentState,
    ) -> Result<(), ShowSubcommandError> {
        self.progress
            .start_step(ShowStep::DisplayInformation.description())?;

        // Display basic information
        let output = self.user_output.lock();
        let mut output = output.borrow_mut();

        output.blank_line();
        output.result(&format!("Environment: {}", any_env.name()));
        output.result(&format!("State: {}", Self::format_state_name(any_env)));
        output.result(&format!("Provider: {}", any_env.provider_name()));

        // Display next step guidance
        output.blank_line();
        output.result(&Self::get_next_step_guidance(any_env));

        drop(output);

        self.progress.complete_step(Some("Information displayed"))?;

        Ok(())
    }

    /// Format the state name in a user-friendly way
    fn format_state_name(
        any_env: &crate::domain::environment::state::AnyEnvironmentState,
    ) -> String {
        // Capitalize first letter of state name
        let state_name = any_env.state_name();
        let mut chars = state_name.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }

    /// Get next step guidance based on the current state
    fn get_next_step_guidance(
        any_env: &crate::domain::environment::state::AnyEnvironmentState,
    ) -> String {
        match any_env.state_name() {
            "created" => {
                "The environment configuration is ready. Run 'provision' to create infrastructure."
                    .to_string()
            }
            "provisioned" => "Next step: Run 'configure' to set up the system.".to_string(),
            "configured" => {
                "Next step: Run 'release' to deploy application files.".to_string()
            }
            "released" => "Next step: Run 'run' to start the services.".to_string(),
            "running" => "Status: ✓ All services running".to_string(),
            "destroyed" => "The environment has been destroyed.".to_string(),
            state if state.ends_with("_failed") => {
                "The environment is in a failed state. Check error details and retry or destroy the environment.".to_string()
            }
            state if state.ends_with("ing") => {
                format!("The environment is currently in a transitional state ({state}). Wait for the operation to complete.")
            }
            _ => "Check environment state and take appropriate action.".to_string(),
        }
    }
}
