//! Run Command Handler
//!
//! This module handles the run command execution at the presentation layer,
//! including environment validation, state validation, and user interaction.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::run::RunCommandHandler;
use crate::application::command_handlers::show::info::{GrafanaInfo, ServiceInfo};
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::AnyEnvironmentState;
use crate::presentation::views::commands::shared::service_urls::{
    CompactServiceUrlsView, DnsHintView,
};
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::RunSubcommandError;

/// Steps in the run workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunStep {
    ValidateEnvironment,
    RunServices,
}

impl RunStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[Self::ValidateEnvironment, Self::RunServices];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment",
            Self::RunServices => "Running application services",
        }
    }
}

/// Presentation layer controller for run command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `RunCommandHandler`.
///
/// # Responsibilities
///
/// - Validate user input (environment name format)
/// - Validate environment state (must be Released)
/// - Show progress updates to the user
/// - Format success/error messages for display
/// - Delegate business logic to application layer
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. It delegates actual business logic to the application layer's
/// `RunCommandHandler`.
pub struct RunCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl RunCommandController {
    /// Create a new run command controller
    ///
    /// Creates a `RunCommandController` with direct repository injection.
    /// This follows the single container architecture pattern.
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, RunStep::count());

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete run workflow
    ///
    /// Orchestrates all steps of the run command:
    /// 1. Validate environment name
    /// 2. Run application services via `RunCommandHandler`
    /// 3. Complete with success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to run services in
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid (format validation fails)
    /// - Environment is not in the Released state
    /// - Service start fails
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `RunSubcommandError` if any step fails.
    #[allow(clippy::result_large_err)]
    #[allow(clippy::unused_async)] // Part of uniform async presentation layer interface
    pub async fn execute(&mut self, environment_name: &str) -> Result<(), RunSubcommandError> {
        let env_name = self.validate_environment_name(environment_name)?;

        self.run_services(&env_name)?;

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
    ) -> Result<EnvironmentName, RunSubcommandError> {
        self.progress
            .start_step(RunStep::ValidateEnvironment.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            RunSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        self.progress
            .complete_step(Some(&format!("Environment name validated: {name}")))?;

        Ok(env_name)
    }

    /// Run services via the application layer handler
    ///
    /// Delegates to `RunCommandHandler` to execute the run workflow:
    /// 1. Load environment from repository
    /// 2. Validate environment is in Released state
    /// 3. Start Docker Compose services via Ansible
    /// 4. Update environment state to Running
    #[allow(clippy::result_large_err)]
    fn run_services(&mut self, env_name: &EnvironmentName) -> Result<(), RunSubcommandError> {
        self.progress
            .start_step(RunStep::RunServices.description())?;

        // Cast the repository to the base trait type that RunCommandHandler expects
        let repository: Arc<dyn crate::domain::environment::repository::EnvironmentRepository> =
            Arc::clone(&self.repository)
                as Arc<dyn crate::domain::environment::repository::EnvironmentRepository>;

        let handler = RunCommandHandler::new(repository, Arc::clone(&self.clock));

        handler.execute(env_name)?;

        self.progress.complete_step(Some("Services started"))?;

        Ok(())
    }

    /// Complete the workflow with success message and service URLs
    ///
    /// Loads environment info and displays:
    /// 1. Service URLs (excluding localhost-only services)
    /// 2. DNS hint for HTTPS/TLS services
    /// 3. Tip to use `show` command for full details
    ///
    /// Follows the same pattern as the show command for loading environment
    /// and extracting service information.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(&mut self, name: &str) -> Result<(), RunSubcommandError> {
        // Load environment to get service information
        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            RunSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        let any_env = self.load_environment(&env_name)?;

        // Display success message
        self.progress
            .complete(&format!("Run command completed for '{name}'"))?;

        // Display service URLs and hints
        self.display_service_urls(&any_env)?;

        Ok(())
    }

    /// Load environment from repository
    ///
    /// Reuses the same loading logic as the show command.
    #[allow(clippy::result_large_err)]
    fn load_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<AnyEnvironmentState, RunSubcommandError> {
        if !self.repository.exists(env_name)? {
            return Err(RunSubcommandError::EnvironmentNotAccessible {
                name: env_name.to_string(),
                data_dir: "data".to_string(),
            });
        }

        self.repository.load(env_name)?.ok_or_else(|| {
            RunSubcommandError::EnvironmentNotAccessible {
                name: env_name.to_string(),
                data_dir: "data".to_string(),
            }
        })
    }

    /// Display service URLs and DNS hints
    ///
    /// Renders service URLs using the shared views:
    /// - `CompactServiceUrlsView` - Only shows publicly accessible services
    /// - `DnsHintView` - Shows DNS configuration hint for HTTPS services
    ///
    /// Also displays a tip about using the `show` command for full details.
    #[allow(clippy::result_large_err)]
    fn display_service_urls(
        &mut self,
        any_env: &AnyEnvironmentState,
    ) -> Result<(), RunSubcommandError> {
        if let Some(instance_ip) = any_env.instance_ip() {
            let tracker_config = any_env.tracker_config();
            let grafana_config = any_env.grafana_config();

            let services =
                ServiceInfo::from_tracker_config(tracker_config, instance_ip, grafana_config);

            let grafana =
                grafana_config.map(|config| GrafanaInfo::from_config(config, instance_ip));

            // Render service URLs (only public services)
            let service_urls_output = CompactServiceUrlsView::render(&services, grafana.as_ref());
            if !service_urls_output.is_empty() {
                self.progress.result(&format!("\n{service_urls_output}"))?;
            }

            // Show DNS hint if HTTPS services are configured
            if let Some(dns_hint) = DnsHintView::render(&services) {
                self.progress.result(&format!("\n{dns_hint}"))?;
            }

            // Show tip about show command
            self.progress.result(&format!(
                "\nTip: Run 'torrust-tracker-deployer show {}' for full details\n",
                any_env.name()
            ))?;
        }

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

    /// Create test dependencies for run command handler tests
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
        let result = RunCommandController::new(repository, clock, user_output.clone())
            .execute("invalid_name")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RunSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "invalid_name");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_for_empty_environment_name() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = RunCommandController::new(repository, clock, user_output.clone())
            .execute("")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RunSubcommandError::InvalidEnvironmentName { name, .. } => {
                assert_eq!(name, "");
            }
            other => panic!("Expected InvalidEnvironmentName, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn it_should_return_error_when_environment_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        // Valid environment name but doesn't exist
        let result = RunCommandController::new(repository, clock, user_output.clone())
            .execute("test-env")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RunSubcommandError::EnvironmentNotAccessible { name, .. } => {
                assert_eq!(name, "test-env");
            }
            other => panic!("Expected EnvironmentNotAccessible, got: {other:?}"),
        }
    }
}
