//! Application Service Container
//!
//! This module provides centralized initialization of application-wide services
//! that need consistent configuration across the entire application.

use std::sync::{Arc, Mutex};

use crate::presentation::commands::constants::DEFAULT_VERBOSITY;
use crate::presentation::user_output::UserOutput;

/// Application service container
///
/// Holds shared services initialized during application bootstrap.
/// Services are wrapped in `Arc<Mutex<T>>` for thread-safe shared ownership
/// with interior mutability across the application.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
///
/// let container = Container::new();
/// let user_output = container.user_output();
/// user_output.lock().unwrap().success("Operation completed");
/// ```
#[derive(Clone)]
pub struct Container {
    user_output: Arc<Mutex<UserOutput>>,
}

impl Container {
    /// Create a new container with initialized services
    ///
    /// Uses `DEFAULT_VERBOSITY` for user output. In the future, this may
    /// accept a verbosity parameter from CLI flags.
    #[must_use]
    pub fn new() -> Self {
        let user_output = Arc::new(Mutex::new(UserOutput::new(DEFAULT_VERBOSITY)));

        Self { user_output }
    }

    /// Get shared reference to user output service
    ///
    /// Returns an `Arc<Mutex<UserOutput>>` that can be cheaply cloned and shared
    /// across threads and function calls. Lock the mutex to access the user output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::bootstrap::container::Container;
    ///
    /// let container = Container::new();
    /// let user_output = container.user_output();
    /// user_output.lock().unwrap().success("Operation completed");
    /// ```
    #[must_use]
    pub fn user_output(&self) -> Arc<Mutex<UserOutput>> {
        Arc::clone(&self.user_output)
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_container_with_user_output() {
        let container = Container::new();
        let user_output = container.user_output();

        // Verify we can get the user_output service
        assert!(Arc::strong_count(&user_output) >= 1);
    }

    #[test]
    fn it_should_return_cloned_arc_on_user_output_access() {
        let container = Container::new();
        let user_output1 = container.user_output();
        let user_output2 = container.user_output();

        // Both should point to the same UserOutput instance
        assert!(Arc::ptr_eq(&user_output1, &user_output2));
    }

    #[test]
    fn it_should_be_clonable() {
        let container1 = Container::new();
        let container2 = container1.clone();

        let user_output1 = container1.user_output();
        let user_output2 = container2.user_output();

        // Cloned containers should share the same UserOutput
        assert!(Arc::ptr_eq(&user_output1, &user_output2));
    }
}
