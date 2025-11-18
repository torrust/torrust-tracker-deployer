//! Integration tests for configure command presentation layer
//!
//! These tests verify the complete workflow of the configure command
//! from CLI interface to application layer integration.

#[cfg(test)]
mod integration_tests {
    use std::cell::RefCell;
    use std::sync::Arc;

    use parking_lot::ReentrantMutex;

    use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
    use crate::presentation::controllers::configure;
    use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::{UserOutput, VerbosityLevel};
    use crate::shared::clock::Clock;
    use crate::shared::SystemClock;

    /// Create test dependencies for configure command integration tests
    #[allow(clippy::type_complexity)]
    fn create_test_dependencies() -> (
        Arc<ReentrantMutex<RefCell<UserOutput>>>,
        Arc<RepositoryFactory>,
        Arc<dyn Clock>,
    ) {
        let (user_output, _, _) =
            TestUserOutput::new(VerbosityLevel::Normal).into_reentrant_wrapped();
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let clock = Arc::new(SystemClock);

        (user_output, repository_factory, clock)
    }

    #[tokio::test]
    async fn configure_command_validates_environment_name() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository_factory, clock) = create_test_dependencies();

        let result = configure::handle_configure_command(
            "invalid_name_with_underscore",
            temp_dir.path(),
            repository_factory,
            clock,
            &user_output,
        )
        .await;

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
        let (user_output, repository_factory, clock) = create_test_dependencies();

        let result = configure::handle_configure_command(
            "bad_name",
            temp_dir.path(),
            repository_factory,
            clock,
            &user_output,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        let help = error.help();
        assert!(help.contains("Invalid Environment Name"));
        assert!(help.contains("1-63 characters"));
    }

    #[tokio::test]
    async fn configure_command_handles_nonexistent_environment() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let (user_output, repository_factory, clock) = create_test_dependencies();

        let result = configure::handle_configure_command(
            "nonexistent-env",
            temp_dir.path(),
            repository_factory,
            clock,
            &user_output,
        )
        .await;

        assert!(result.is_err());
        // Repository will return NotFound error, wrapped in ConfigureOperationFailed
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            configure::ConfigureSubcommandError::ConfigureOperationFailed { .. }
        ));
    }
}
