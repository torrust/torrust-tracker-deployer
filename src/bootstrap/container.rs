//! Application Service Container
//!
//! This module provides centralized initialization of application-wide services
//! that need consistent configuration across the entire application.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::domain::environment::repository::EnvironmentRepository;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::controllers::configure::ConfigureCommandController;
use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
use crate::presentation::controllers::create::subcommands::environment::CreateEnvironmentCommandController;
use crate::presentation::controllers::create::subcommands::template::CreateTemplateCommandController;
use crate::presentation::controllers::destroy::DestroyCommandController;
use crate::presentation::controllers::provision::ProvisionCommandController;
use crate::presentation::controllers::register::RegisterCommandController;
use crate::presentation::controllers::test::handler::TestCommandController;
use crate::presentation::views::{UserOutput, VerbosityLevel};
use crate::shared::clock::Clock;
use crate::shared::SystemClock;

/// Application service container
///
/// Holds shared services initialized during application bootstrap.
/// Services are wrapped in `Arc<T>` for thread-safe shared ownership
/// across the application.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// let working_dir = Path::new(".");
/// let container = Container::new(VerbosityLevel::Normal, working_dir);
/// let user_output = container.user_output();
/// user_output.lock().borrow_mut().success("Operation completed");
/// ```
#[derive(Clone)]
pub struct Container {
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    repository_factory: Arc<RepositoryFactory>,
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
}

impl Container {
    /// Create a new container with initialized services
    ///
    /// Initializes all services with specified verbosity level and working directory:
    /// - `UserOutput` with provided `verbosity_level`
    /// - `RepositoryFactory` with `DEFAULT_LOCK_TIMEOUT`
    /// - `EnvironmentRepository` using `working_dir/data` as base directory
    /// - `SystemClock` for time operations
    ///
    /// # Arguments
    ///
    /// * `verbosity_level` - Controls how verbose the user output will be
    /// * `working_dir` - Base working directory for the application (repository uses `working_dir/data`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// // For normal application use
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    ///
    /// // For completely silent testing
    /// let container = Container::new(VerbosityLevel::Silent, Path::new("/tmp/test"));
    /// ```
    #[must_use]
    pub fn new(verbosity_level: VerbosityLevel, working_dir: &Path) -> Self {
        let user_output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(
            verbosity_level,
        ))));
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));

        // Create repository once for the entire application
        let data_dir = working_dir.join("data");
        let repository = repository_factory.create(data_dir);

        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        Self {
            user_output,
            repository_factory,
            repository,
            clock,
        }
    }

    /// Get shared reference to user output service
    ///
    /// Returns an `Arc<ReentrantMutex<RefCell<UserOutput>>>` that can be safely cloned and shared
    /// across threads and function calls. Use the reentrant lock to acquire access, then `borrow_mut()`
    /// to get mutable access to the user output. The reentrant mutex prevents deadlocks when the
    /// same thread needs to acquire the lock multiple times.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let user_output = container.user_output();
    /// user_output.lock().borrow_mut().success("Operation completed");
    /// ```
    #[must_use]
    pub fn user_output(&self) -> Arc<ReentrantMutex<RefCell<UserOutput>>> {
        Arc::clone(&self.user_output)
    }

    /// Get shared reference to repository factory service
    ///
    /// Returns an `Arc<RepositoryFactory>` that can be cheaply cloned and shared
    /// across threads and function calls.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let repository_factory = container.repository_factory();
    /// // Use repository_factory to create repositories
    /// ```
    #[must_use]
    pub fn repository_factory(&self) -> Arc<RepositoryFactory> {
        Arc::clone(&self.repository_factory)
    }

    /// Get shared reference to environment repository
    ///
    /// Returns an `Arc<dyn EnvironmentRepository>` that can be cheaply cloned and shared
    /// across threads and function calls. The repository is initialized with the
    /// application's base data directory during container creation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let repository = container.repository();
    /// // Use repository to load/save environment state
    /// ```
    #[must_use]
    pub fn repository(&self) -> Arc<dyn EnvironmentRepository + Send + Sync> {
        Arc::clone(&self.repository)
    }

    /// Get shared reference to clock service
    ///
    /// Returns an `Arc<dyn Clock>` that can be cheaply cloned and shared
    /// across threads and function calls.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let clock = container.clock();
    /// // Use clock for time operations
    /// ```
    #[must_use]
    pub fn clock(&self) -> Arc<dyn Clock> {
        Arc::clone(&self.clock)
    }

    /// Create a new `CreateEnvironmentCommandController`
    #[must_use]
    pub fn create_environment_controller(&self) -> CreateEnvironmentCommandController {
        CreateEnvironmentCommandController::new(
            self.repository(),
            self.clock(),
            &self.user_output(),
        )
    }

    /// Create a new `CreateTemplateCommandController`
    #[must_use]
    pub fn create_template_controller(&self) -> CreateTemplateCommandController {
        CreateTemplateCommandController::new(&self.user_output())
    }

    /// Create a new `ProvisionCommandController`
    #[must_use]
    pub fn create_provision_controller(&self) -> ProvisionCommandController {
        ProvisionCommandController::new(self.repository(), self.clock(), self.user_output())
    }

    /// Create a new `DestroyCommandController`
    #[must_use]
    pub fn create_destroy_controller(&self) -> DestroyCommandController {
        DestroyCommandController::new(self.repository(), self.clock(), self.user_output())
    }

    /// Create a new `ConfigureCommandController`
    #[must_use]
    pub fn create_configure_controller(&self) -> ConfigureCommandController {
        ConfigureCommandController::new(self.repository(), self.clock(), self.user_output())
    }

    /// Create a new `TestCommandController`
    #[must_use]
    pub fn create_test_controller(&self) -> TestCommandController {
        TestCommandController::new(self.repository(), self.user_output())
    }

    /// Create a new `RegisterCommandController`
    #[must_use]
    pub fn create_register_controller(&self) -> RegisterCommandController {
        RegisterCommandController::new(self.repository(), self.user_output())
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new(VerbosityLevel::Normal, Path::new("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_container_with_all_services() {
        let temp_dir = TempDir::new().unwrap();
        let container = Container::new(VerbosityLevel::Normal, temp_dir.path());

        // Verify we can get all services
        let user_output = container.user_output();
        let repository_factory = container.repository_factory();
        let repository = container.repository();
        let clock = container.clock();

        assert!(Arc::strong_count(&user_output) >= 1);
        assert!(Arc::strong_count(&repository_factory) >= 1);
        assert!(Arc::strong_count(&repository) >= 1);
        assert!(Arc::strong_count(&clock) >= 1);
    }

    #[test]
    fn it_should_return_cloned_arc_on_repository_factory_access() {
        let temp_dir = TempDir::new().unwrap();
        let container = Container::new(VerbosityLevel::Normal, temp_dir.path());
        let factory1 = container.repository_factory();
        let factory2 = container.repository_factory();

        // Both should point to the same RepositoryFactory instance
        assert!(Arc::ptr_eq(&factory1, &factory2));
    }

    #[test]
    fn it_should_return_cloned_arc_on_repository_access() {
        let temp_dir = TempDir::new().unwrap();
        let container = Container::new(VerbosityLevel::Normal, temp_dir.path());
        let repo1 = container.repository();
        let repo2 = container.repository();

        // Both should point to the same Repository instance
        assert!(Arc::ptr_eq(&repo1, &repo2));
    }

    #[test]
    fn it_should_return_cloned_arc_on_clock_access() {
        let temp_dir = TempDir::new().unwrap();
        let container = Container::new(VerbosityLevel::Normal, temp_dir.path());
        let clock1 = container.clock();
        let clock2 = container.clock();

        // Both should point to the same Clock instance
        assert!(Arc::ptr_eq(&clock1, &clock2));
    }

    #[test]
    fn it_should_return_cloned_arc_on_user_output_access() {
        let temp_dir = TempDir::new().unwrap();
        let container = Container::new(VerbosityLevel::Normal, temp_dir.path());
        let user_output1 = container.user_output();
        let user_output2 = container.user_output();

        // Both should point to the same UserOutput instance
        assert!(Arc::ptr_eq(&user_output1, &user_output2));
    }

    #[test]
    fn it_should_be_clonable() {
        let temp_dir = TempDir::new().unwrap();
        let container1 = Container::new(VerbosityLevel::Normal, temp_dir.path());
        let container2 = container1.clone();

        // Cloned containers should share all services
        let user_output1 = container1.user_output();
        let user_output2 = container2.user_output();
        assert!(Arc::ptr_eq(&user_output1, &user_output2));

        let factory1 = container1.repository_factory();
        let factory2 = container2.repository_factory();
        assert!(Arc::ptr_eq(&factory1, &factory2));

        let repo1 = container1.repository();
        let repo2 = container2.repository();
        assert!(Arc::ptr_eq(&repo1, &repo2));

        let clock1 = container1.clock();
        let clock2 = container2.clock();
        assert!(Arc::ptr_eq(&clock1, &clock2));
    }

    #[test]
    fn it_should_create_container_with_silent_verbosity_for_tests() {
        let temp_dir = TempDir::new().unwrap();
        let container = Container::new(VerbosityLevel::Silent, temp_dir.path());

        // All services should be available
        let user_output = container.user_output();
        let repository_factory = container.repository_factory();
        let repository = container.repository();
        let clock = container.clock();

        assert!(Arc::strong_count(&user_output) >= 1);
        assert!(Arc::strong_count(&repository_factory) >= 1);
        assert!(Arc::strong_count(&repository) >= 1);
        assert!(Arc::strong_count(&clock) >= 1);

        // The container should work with any verbosity level, including Silent for tests
        // Silent mode will suppress all output, making tests clean
    }

    #[test]
    fn it_should_create_container_with_different_verbosity_levels() {
        let temp_dir = TempDir::new().unwrap();

        // Test all available verbosity levels
        let levels = [
            VerbosityLevel::Silent,
            VerbosityLevel::Quiet,
            VerbosityLevel::Normal,
            VerbosityLevel::Verbose,
            VerbosityLevel::VeryVerbose,
            VerbosityLevel::Debug,
        ];

        for level in &levels {
            let container = Container::new(*level, temp_dir.path());

            // All services should be available regardless of verbosity level
            let user_output = container.user_output();
            let repository_factory = container.repository_factory();
            let repository = container.repository();
            let clock = container.clock();

            assert!(Arc::strong_count(&user_output) >= 1);
            assert!(Arc::strong_count(&repository_factory) >= 1);
            assert!(Arc::strong_count(&repository) >= 1);
            assert!(Arc::strong_count(&clock) >= 1);
        }
    }
}
