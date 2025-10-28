//! Create Command Implementation
//!
//! This module implements the `CreateCommandHandler` that orchestrates environment
//! creation business logic. It follows the Command Pattern with dependency
//! injection and is delivery-agnostic.

use std::sync::Arc;
use tracing::{info, instrument};

use crate::domain::config::EnvironmentCreationConfig;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::{Created, Environment};
use crate::shared::Clock;

use super::errors::CreateCommandHandlerError;

/// Command to create a new deployment environment
///
/// This command is delivery-agnostic and can be used from CLI, REST API,
/// GraphQL, or any other delivery mechanism. It orchestrates the business
/// logic for environment creation without knowledge of how the configuration
/// was obtained.
///
/// # Architecture
///
/// The command follows these design principles:
///
/// - **Synchronous**: No async/await, following existing patterns
/// - **Dependency Injection**: Uses `Arc<dyn Trait>` for testability
/// - **Repository Pattern**: Delegates persistence to repository
/// - **Explicit Errors**: All failures return structured errors with `.help()`
///
/// # Business Logic Flow
///
/// 1. Convert configuration to domain objects
/// 2. Check if environment already exists (prevent duplicates)
/// 3. Create environment entity using `Environment::new()`
/// 4. Persist via repository (repository handles directory creation)
///
/// # Examples
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::application::command_handlers::create::CreateCommandHandler;
/// use torrust_tracker_deployer_lib::domain::config::{
///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig
/// };
/// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
/// use torrust_tracker_deployer_lib::shared::{SystemClock, Clock};
///
/// // Setup dependencies
/// let repository_factory = RepositoryFactory::new(std::time::Duration::from_secs(30));
/// let repository = repository_factory.create(std::path::PathBuf::from("."));
/// let clock: Arc<dyn Clock> = Arc::new(SystemClock);
///
/// // Create command
/// let command = CreateCommandHandler::new(repository, clock);
///
/// // Prepare configuration
/// let config = EnvironmentCreationConfig::new(
///     EnvironmentSection {
///         name: "dev".to_string(),
///     },
///     SshCredentialsConfig::new(
///         "fixtures/testing_rsa".to_string(),
///         "fixtures/testing_rsa.pub".to_string(),
///         "torrust".to_string(),
///         22,
///     ),
/// );
///
/// // Execute command
/// let environment = command.execute(config)?;
/// println!("Created environment: {}", environment.name());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CreateCommandHandler {
    /// Repository for persisting environment state
    pub(crate) environment_repository: Arc<dyn EnvironmentRepository>,

    /// Clock for timestamp generation (injected for testability)
    #[allow(dead_code)] // Will be used in future enhancements
    pub(crate) clock: Arc<dyn Clock>,
}

impl CreateCommandHandler {
    /// Create a new `CreateCommandHandler` with required dependencies
    ///
    /// # Arguments
    ///
    /// * `environment_repository` - Repository for persisting environment state
    /// * `clock` - Clock for timestamp generation (for future use)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::CreateCommandHandler;
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    /// use torrust_tracker_deployer_lib::shared::{SystemClock, Clock};
    ///
    /// let repository_factory = RepositoryFactory::new(std::time::Duration::from_secs(30));
    /// let repository = repository_factory.create(std::path::PathBuf::from("."));
    /// let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    ///
    /// let command = CreateCommandHandler::new(repository, clock);
    /// ```
    #[must_use]
    pub fn new(
        environment_repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            environment_repository,
            clock,
        }
    }

    /// Execute the create command with validated configuration
    ///
    /// This method orchestrates the complete environment creation workflow:
    /// 1. Converts configuration to domain objects
    /// 2. Validates environment uniqueness
    /// 3. Creates the environment entity
    /// 4. Persists the environment state
    ///
    /// # Arguments
    ///
    /// * `config` - Validated environment configuration from domain layer
    ///
    /// # Returns
    ///
    /// * `Ok(Environment<Created>)` - Successfully created environment
    /// * `Err(CreateCommandHandlerError)` - Business logic or persistence failure
    ///
    /// # Business Rules
    ///
    /// 1. Configuration must convert to valid domain objects
    /// 2. Environment name must be unique (no duplicates)
    /// 3. Repository handles directory creation atomically during save
    /// 4. Environment state must be persisted successfully
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration validation fails
    /// - Environment with the same name already exists
    /// - Repository persistence fails
    ///
    /// All errors implement `.help()` with detailed troubleshooting guidance.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::CreateCommandHandler;
    /// use torrust_tracker_deployer_lib::domain::config::{
    ///     EnvironmentCreationConfig, EnvironmentSection, SshCredentialsConfig
    /// };
    ///
    /// # fn example(command: CreateCommandHandler) -> Result<(), Box<dyn std::error::Error>> {
    /// let config = EnvironmentCreationConfig::new(
    ///     EnvironmentSection {
    ///         name: "staging".to_string(),
    ///     },
    ///     SshCredentialsConfig::new(
    ///         "keys/stage_key".to_string(),
    ///         "keys/stage_key.pub".to_string(),
    ///         "torrust".to_string(),
    ///         22,
    ///     ),
    /// );
    ///
    /// let environment = command.execute(config)?;
    /// println!("Created: {}", environment.name());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        name = "create_command",
        skip_all,
        fields(
            command_type = "create",
            environment = %config.environment.name
        )
    )]
    pub fn execute(
        &self,
        config: EnvironmentCreationConfig,
    ) -> Result<Environment<Created>, CreateCommandHandlerError> {
        info!(
            command = "create",
            environment = %config.environment.name,
            "Starting environment creation"
        );

        // Step 1: Convert configuration to domain objects
        // This validates environment name, SSH username, and file existence
        let (environment_name, ssh_credentials, ssh_port) = config
            .to_environment_params()
            .map_err(CreateCommandHandlerError::InvalidConfiguration)?;

        // Step 2: Check if environment already exists
        // This prevents duplicate environments and provides clear feedback
        if self
            .environment_repository
            .exists(&environment_name)
            .map_err(CreateCommandHandlerError::RepositoryError)?
        {
            return Err(CreateCommandHandlerError::EnvironmentAlreadyExists {
                name: environment_name.as_str().to_string(),
            });
        }

        // Step 3: Create environment entity using existing Environment::new()
        // No need for create_from_config() - use existing constructor
        let environment = Environment::new(environment_name, ssh_credentials, ssh_port);

        // Step 4: Persist environment state
        // Repository handles directory creation atomically during save
        self.environment_repository
            .save(&environment.clone().into_any())
            .map_err(CreateCommandHandlerError::RepositoryError)?;

        info!(
            command = "create",
            environment = %environment.name(),
            "Environment created successfully"
        );

        Ok(environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_create_command_with_dependencies() {
        use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
        use crate::shared::SystemClock;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let repository_factory = RepositoryFactory::new(std::time::Duration::from_secs(30));
        let repository = repository_factory.create(temp_dir.path().to_path_buf());
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let command = CreateCommandHandler::new(repository, clock);

        // Verify the command was created (basic structure test)
        assert_eq!(Arc::strong_count(&command.environment_repository), 1);
        assert_eq!(Arc::strong_count(&command.clock), 1);
    }
}
