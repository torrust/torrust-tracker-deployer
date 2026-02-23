//! Integration tests for configure command presentation layer
//!
//! These tests verify the complete workflow of the configure command
//! from CLI interface to application layer integration.

#[cfg(test)]
mod integration_tests {
    use std::cell::RefCell;
    use std::sync::Arc;

    use parking_lot::ReentrantMutex;

    use crate::domain::environment::repository::EnvironmentRepository;
    use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
    use crate::presentation::controllers::configure;
    use crate::presentation::controllers::configure::handler::ConfigureCommandController;
    use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::input::cli::OutputFormat;
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::{UserOutput, VerbosityLevel};
    use crate::shared::clock::Clock;
    use crate::shared::SystemClock;

    /// Create test dependencies for configure command integration tests
    #[allow(clippy::type_complexity)]
    fn create_test_dependencies(
        temp_dir: &tempfile::TempDir,
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
    async fn configure_command_validates_environment_name() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ConfigureCommandController::new(repository, clock, user_output.clone())
            .execute("invalid_name_with_underscore", OutputFormat::Text);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            configure::ConfigureSubcommandError::InvalidEnvironmentName { .. }
        ));
    }

    #[tokio::test]
    async fn configure_command_provides_help_for_invalid_name() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ConfigureCommandController::new(repository, clock, user_output.clone())
            .execute("bad_name", OutputFormat::Text);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let help = error.help();
        assert!(help.contains("Invalid Environment Name"));
        assert!(help.contains("1-63 characters"));
    }

    #[tokio::test]
    async fn configure_command_propagates_repository_errors() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository, clock) = create_test_dependencies(&temp_dir);

        let result = ConfigureCommandController::new(repository, clock, user_output.clone())
            .execute("nonexistent-environment", OutputFormat::Text);

        assert!(result.is_err());
        // Repository will return NotFound error, wrapped in ConfigureOperationFailed
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            configure::ConfigureSubcommandError::ConfigureOperationFailed { .. }
        ));
    }
}
