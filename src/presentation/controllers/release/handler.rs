//! Release Command Handler
//!
//! This module handles the release command execution at the presentation layer,
//! including environment validation, state validation, and user interaction.
//!
//! ## Current Status: Scaffolding (No-op)
//!
//! This handler is part of Issue #217 (Demo Slice - Release and Run Commands Scaffolding).
//! It validates the environment name and logs intent, but does not execute actual
//! release operations yet. The application layer handler will be implemented in Phase 4.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;
use tracing::info;

use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::ReleaseSubcommandError;

/// Steps in the release workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReleaseStep {
    ValidateEnvironment,
    ValidateState,
    ReleaseApplication,
}

impl ReleaseStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateEnvironment,
        Self::ValidateState,
        Self::ReleaseApplication,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment",
            Self::ValidateState => "Validating environment state",
            Self::ReleaseApplication => "Releasing application",
        }
    }
}

/// Presentation layer controller for release command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `ReleaseCommandHandler`.
///
/// # Current Status
///
/// This controller is scaffolding for Issue #217. It validates the environment
/// name and logs intent, but does not execute actual release operations yet.
///
/// # Responsibilities
///
/// - Validate user input (environment name format)
/// - Validate environment state (must be Configured)
/// - Show progress updates to the user
/// - Format success/error messages for display
/// - Delegate business logic to application layer (future)
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. It will delegate actual business logic to the application layer's
/// `ReleaseCommandHandler` once implemented.
pub struct ReleaseCommandController {
    #[allow(dead_code)] // Will be used in Phase 4
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    #[allow(dead_code)] // Will be used in Phase 4
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl ReleaseCommandController {
    /// Create a new release command controller
    ///
    /// Creates a `ReleaseCommandController` with direct repository injection.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, ReleaseStep::count());

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete release workflow
    ///
    /// Orchestrates all steps of the release command:
    /// 1. Validate environment name
    /// 2. Validate environment state (must be Configured)
    /// 3. Release application (currently no-op)
    /// 4. Complete with success message
    ///
    /// # Current Status
    ///
    /// This is scaffolding for Issue #217. Steps 1-2 validate inputs,
    /// step 3 logs intent but does not execute actual release operations.
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to release to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid (format validation fails)
    /// - Environment is not in the Configured state (future)
    /// - Release operation fails (future)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ReleaseSubcommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    #[allow(clippy::unused_async)] // Part of uniform async presentation layer interface
    pub async fn execute(&mut self, environment_name: &str) -> Result<(), ReleaseSubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        self.validate_state(&env_name)?;

        self.release_application(&env_name)?;

        self.complete_workflow(environment_name)?;

        Ok(())
    }

    /// Validate the environment name format
    ///
    /// Shows progress to user and validates that the environment name
    /// meets domain requirements (1-63 chars, alphanumeric + hyphens).
    #[allow(clippy::result_large_err)]
    fn validate_environment_name(
        &mut self,
        name: &str,
    ) -> Result<EnvironmentName, ReleaseSubcommandError> {
        self.progress
            .start_step(ReleaseStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            ReleaseSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    /// Validate environment state
    ///
    /// The environment must be in the Configured state before release.
    /// Currently this is a no-op that logs intent (scaffolding for Issue #217).
    #[allow(clippy::result_large_err)]
    fn validate_state(&mut self, env_name: &EnvironmentName) -> Result<(), ReleaseSubcommandError> {
        self.progress
            .start_step(ReleaseStep::ValidateState.description())?;

        // TODO: Phase 4 - Actually validate state from repository
        // let environment = self.repository.load(env_name)?;
        // if environment.state() != State::Configured {
        //     return Err(ReleaseSubcommandError::InvalidEnvironmentState { ... });
        // }

        info!(
            environment = %env_name,
            action = "validate_state",
            status = "scaffolding",
            "Would validate environment is in Configured state"
        );

        self.progress
            .complete_step(Some("State validation passed (scaffolding)"))?;

        Ok(())
    }

    /// Release application to the environment
    ///
    /// Currently this is a no-op that logs intent (scaffolding for Issue #217).
    /// The actual release logic will be implemented in Phase 4+.
    #[allow(clippy::result_large_err)]
    fn release_application(
        &mut self,
        env_name: &EnvironmentName,
    ) -> Result<(), ReleaseSubcommandError> {
        self.progress
            .start_step(ReleaseStep::ReleaseApplication.description())?;

        // TODO: Phase 4+ - Implement actual release logic
        // 1. Load environment from repository
        // 2. Create ReleaseCommandHandler with dependencies
        // 3. Execute release workflow (copy files, deploy Docker Compose)
        // 4. Update environment state to Released

        info!(
            environment = %env_name,
            action = "release_application",
            status = "scaffolding",
            "Would release application to environment (not implemented yet)"
        );

        self.progress
            .complete_step(Some("Application released (scaffolding - no-op)"))?;

        Ok(())
    }

    /// Complete the workflow with success message
    ///
    /// Shows final success message to the user with workflow summary.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(&mut self, name: &str) -> Result<(), ReleaseSubcommandError> {
        self.progress.complete(&format!(
            "Release command completed for '{name}' (scaffolding - no actual release performed)"
        ))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
    use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;
    use crate::shared::SystemClock;
    use tempfile::TempDir;

    /// Create test dependencies for release command handler tests
    #[allow(clippy::type_complexity)]
    fn create_test_dependencies(
        temp_dir: &TempDir,
    ) -> (
        Arc<ReentrantMutex<RefCell<UserOutput>>>,
        Arc<dyn EnvironmentRepository + Send + Sync>,
        Arc<dyn Clock>,
    ) {
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let data_dir = temp_dir.path().join("data");
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(data_dir);
        let clock = Arc::new(SystemClock);

        (user_output, repository, clock)
    }

    #[tokio::test]
    async fn it_should_return_error_for_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Test with invalid environment name (contains underscore)
        let result = ReleaseCommandController::new(repository, clock, user_output.clone())
            .execute("invalid_name")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ReleaseSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_empty_environment_name() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ReleaseCommandController::new(repository, clock, user_output.clone())
            .execute("")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ReleaseSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_accept_valid_environment_name_and_complete_scaffolding() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Valid environment name should pass validation and complete (scaffolding)
        let result = ReleaseCommandController::new(repository, clock, user_output.clone())
            .execute("test-env")
            .await;

        // Should succeed since this is scaffolding (no actual release)
        assert!(
            result.is_ok(),
            "Expected success for valid environment name in scaffolding mode"
        );
    }
}
