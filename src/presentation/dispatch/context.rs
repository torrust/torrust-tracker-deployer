//! Execution Context
//!
//! This module provides the ExecutionContext wrapper around the Container for
//! dependency injection in command handlers. It offers a clean interface for
//! accessing services needed during command execution.
//!
//! ## Purpose
//!
//! The ExecutionContext serves as an abstraction layer between the Container
//! (which holds raw services) and command handlers (which need typed access).
//! This separation provides:
//!
//! - **Clean Interface**: Command handlers get strongly-typed service access
//! - **Thread Safety**: All services are properly wrapped for concurrent access
//! - **Future-Proofing**: Easy to add new services without changing handler signatures
//! - **Testing Support**: Easy to inject test doubles through Container
//!
//! ## Design
//!
//! ```text
//! Container (bootstrap) → ExecutionContext (dispatch) → Command Handlers
//!
//! Raw services        Clean typed access      Business logic
//! ```
//!
//! ## Usage Example
//!
//! ```rust
//! use std::sync::Arc;
//! use crate::bootstrap::Container;
//! use crate::presentation::dispatch::ExecutionContext;
//!
//! // Create execution context from container
//! let container = Arc::new(Container::new());
//! let context = ExecutionContext::new(container);
//!
//! // Command handlers access services through context
//! let user_output = context.user_output();
//! user_output.lock().unwrap().progress("Processing...");
//! ```

use std::sync::{Arc, Mutex};

use crate::bootstrap::Container;
use crate::presentation::user_output::UserOutput;

/// Execution context for command handlers
///
/// Wraps the Container to provide clean, typed access to application services
/// needed during command execution. Acts as a bridge between the dependency
/// injection container and command handlers.
///
/// # Thread Safety
///
/// All services accessed through ExecutionContext are thread-safe and can be
/// safely shared across async operations and threads.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::bootstrap::Container;
/// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
///
/// let container = Arc::new(Container::new());
/// let context = ExecutionContext::new(container);
///
/// // Access user output service
/// let user_output = context.user_output();
/// user_output.lock().unwrap().success("Operation completed");
/// ```
#[derive(Clone)]
pub struct ExecutionContext {
    container: Arc<Container>,
}

impl ExecutionContext {
    /// Create a new execution context from a container
    ///
    /// # Arguments
    ///
    /// * `container` - Application service container with initialized services
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    ///
    /// let container = Arc::new(Container::new());
    /// let context = ExecutionContext::new(container);
    /// ```
    #[must_use]
    pub fn new(container: Arc<Container>) -> Self {
        Self { container }
    }

    /// Get reference to the underlying container
    ///
    /// Provides access to the raw container for cases where direct access
    /// to container methods is needed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    ///
    /// let container = Arc::new(Container::new());
    /// let context = ExecutionContext::new(container.clone());
    ///
    /// let container_ref = context.container();
    /// assert!(Arc::ptr_eq(&container, container_ref));
    /// ```
    #[must_use]
    pub fn container(&self) -> &Arc<Container> {
        &self.container
    }

    /// Get shared reference to user output service
    ///
    /// Returns the user output service for displaying messages, progress,
    /// and results to users. The service is wrapped in Arc<Mutex<T>> for
    /// thread-safe shared access.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    ///
    /// let container = Arc::new(Container::new());
    /// let context = ExecutionContext::new(container);
    ///
    /// let user_output = context.user_output();
    /// user_output.lock().unwrap().success("Operation completed");
    /// ```
    #[must_use]
    pub fn user_output(&self) -> Arc<Mutex<UserOutput>> {
        self.container.user_output()
    }

    // TODO: Add more service accessors as Container expands
    //
    // Future services that will be added to Container and accessed here:
    // - opentofu_client() -> Arc<dyn OpenTofuClient>
    // - ansible_client() -> Arc<dyn AnsibleClient>
    // - environment_repository() -> Arc<dyn EnvironmentRepository>
    // - clock() -> Arc<dyn Clock>
    //
    // These will be added as the Container is expanded in future proposals
    // to support the full dependency injection pattern.
}
