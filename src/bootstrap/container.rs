//! Application Service Container
//!
//! This module provides centralized initialization of application-wide services
//! that need consistent configuration across the entire application.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::controllers::constants::DEFAULT_LOCK_TIMEOUT;
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
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
/// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
///
/// let container = Container::new(VerbosityLevel::Normal);
/// let user_output = container.user_output();
/// user_output.lock().borrow_mut().success("Operation completed");
/// ```
#[derive(Clone)]
pub struct Container {
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
}

impl Container {
    /// Create a new container with initialized services
    ///
    /// Initializes all services with specified verbosity level:
    /// - `UserOutput` with provided `verbosity_level`
    /// - `RepositoryFactory` with `DEFAULT_LOCK_TIMEOUT`
    /// - `SystemClock` for time operations
    ///
    /// # Arguments
    ///
    /// * `verbosity_level` - Controls how verbose the user output will be
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// // For normal application use
    /// let container = Container::new(VerbosityLevel::Normal);
    ///
    /// // For completely silent testing
    /// let container = Container::new(VerbosityLevel::Silent);
    /// ```
    #[must_use]
    pub fn new(verbosity_level: VerbosityLevel) -> Self {
        let user_output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(
            verbosity_level,
        ))));
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        Self {
            user_output,
            repository_factory,
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
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal);
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
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal);
    /// let repository_factory = container.repository_factory();
    /// // Use repository_factory to create repositories
    /// ```
    #[must_use]
    pub fn repository_factory(&self) -> Arc<RepositoryFactory> {
        Arc::clone(&self.repository_factory)
    }

    /// Get shared reference to clock service
    ///
    /// Returns an `Arc<dyn Clock>` that can be cheaply cloned and shared
    /// across threads and function calls.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal);
    /// let clock = container.clock();
    /// // Use clock for time operations
    /// ```
    #[must_use]
    pub fn clock(&self) -> Arc<dyn Clock> {
        Arc::clone(&self.clock)
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new(VerbosityLevel::Normal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_container_with_all_services() {
        let container = Container::new(VerbosityLevel::Normal);

        // Verify we can get all services
        let user_output = container.user_output();
        let repository_factory = container.repository_factory();
        let clock = container.clock();

        assert!(Arc::strong_count(&user_output) >= 1);
        assert!(Arc::strong_count(&repository_factory) >= 1);
        assert!(Arc::strong_count(&clock) >= 1);
    }

    #[test]
    fn it_should_return_cloned_arc_on_repository_factory_access() {
        let container = Container::new(VerbosityLevel::Normal);
        let factory1 = container.repository_factory();
        let factory2 = container.repository_factory();

        // Both should point to the same RepositoryFactory instance
        assert!(Arc::ptr_eq(&factory1, &factory2));
    }

    #[test]
    fn it_should_return_cloned_arc_on_clock_access() {
        let container = Container::new(VerbosityLevel::Normal);
        let clock1 = container.clock();
        let clock2 = container.clock();

        // Both should point to the same Clock instance
        assert!(Arc::ptr_eq(&clock1, &clock2));
    }

    #[test]
    fn it_should_return_cloned_arc_on_user_output_access() {
        let container = Container::new(VerbosityLevel::Normal);
        let user_output1 = container.user_output();
        let user_output2 = container.user_output();

        // Both should point to the same UserOutput instance
        assert!(Arc::ptr_eq(&user_output1, &user_output2));
    }

    #[test]
    fn it_should_be_clonable() {
        let container1 = Container::new(VerbosityLevel::Normal);
        let container2 = container1.clone();

        // Cloned containers should share all services
        let user_output1 = container1.user_output();
        let user_output2 = container2.user_output();
        assert!(Arc::ptr_eq(&user_output1, &user_output2));

        let factory1 = container1.repository_factory();
        let factory2 = container2.repository_factory();
        assert!(Arc::ptr_eq(&factory1, &factory2));

        let clock1 = container1.clock();
        let clock2 = container2.clock();
        assert!(Arc::ptr_eq(&clock1, &clock2));
    }

    #[test]
    fn it_should_create_container_with_silent_verbosity_for_tests() {
        let container = Container::new(VerbosityLevel::Silent);

        // All services should be available
        let user_output = container.user_output();
        let repository_factory = container.repository_factory();
        let clock = container.clock();

        assert!(Arc::strong_count(&user_output) >= 1);
        assert!(Arc::strong_count(&repository_factory) >= 1);
        assert!(Arc::strong_count(&clock) >= 1);

        // The container should work with any verbosity level, including Silent for tests
        // Silent mode will suppress all output, making tests clean
    }

    #[test]
    fn it_should_create_container_with_different_verbosity_levels() {
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
            let container = Container::new(*level);

            // All services should be available regardless of verbosity level
            let user_output = container.user_output();
            let repository_factory = container.repository_factory();
            let clock = container.clock();

            assert!(Arc::strong_count(&user_output) >= 1);
            assert!(Arc::strong_count(&repository_factory) >= 1);
            assert!(Arc::strong_count(&clock) >= 1);
        }
    }
}
