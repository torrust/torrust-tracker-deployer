//! Progress reporting interface for command workflows
//!
//! This module defines the `CommandProgressListener` trait that enables
//! application-layer command handlers to report progress to the user interface
//! without depending on presentation-layer types.
//!
//! # DDD Layer Placement
//!
//! - **Defined in**: Application layer (`src/application/traits/`)
//! - **Implemented in**: Presentation layer (`src/presentation/`)
//! - **Dependency direction**: Presentation → Application (correct)
//!
//! The trait lives here because it's a use-case concern: progress reporting
//! is about orchestrating steps in a command workflow, which is the application
//! layer's responsibility. The presentation layer implements the trait to
//! translate progress events into user-facing output.
//!
//! # Verbosity Mapping
//!
//! The listener methods map to verbosity levels (but the listener itself
//! does not know about verbosity — that's the implementation's concern):
//!
//! - `on_step_started` / `on_step_completed` → Verbose (`-v`)
//! - `on_detail` → `VeryVerbose` (`-vv`)
//! - `on_debug` → Debug (`-vvv`)
//!
//! The application layer reports everything; the presentation layer filters
//! based on the user's chosen verbosity level.
//!
//! # Example
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::application::traits::CommandProgressListener;
//!
//! async fn execute(
//!     &self,
//!     env_name: &EnvironmentName,
//!     listener: Option<&dyn CommandProgressListener>,
//! ) -> Result<(), Error> {
//!     if let Some(l) = listener {
//!         l.on_step_started(1, 3, "Rendering templates");
//!     }
//!     // ... perform step ...
//!     if let Some(l) = listener {
//!         l.on_step_completed(1, "Rendering templates");
//!     }
//!     Ok(())
//! }
//! ```

/// A listener for reporting command progress to the user interface.
///
/// This trait is defined in the application layer and implemented in the
/// presentation layer, following the Dependency Inversion Principle.
/// The application layer depends on this abstraction, not on concrete
/// UI implementations.
///
/// # Design Rationale
///
/// - **Generic**: One trait serves all commands (provision, configure, etc.)
/// - **String-based**: Receives human-readable descriptions, not command-specific enums
/// - **Optional**: Handlers accept `Option<&dyn CommandProgressListener>` for backward compatibility
/// - **Filtering-agnostic**: Reports everything; the implementation decides what to display
pub trait CommandProgressListener: Send + Sync {
    /// Called when a step begins execution.
    ///
    /// # Arguments
    ///
    /// * `step_number` - 1-based step index within the current workflow
    /// * `total_steps` - Total number of steps in the workflow
    /// * `description` - Human-readable step description
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str);

    /// Called when a step completes successfully.
    ///
    /// # Arguments
    ///
    /// * `step_number` - 1-based step index within the current workflow
    /// * `description` - Human-readable step description
    fn on_step_completed(&self, step_number: usize, description: &str);

    /// Reports a contextual detail about the current operation.
    ///
    /// Intended for intermediate results, file paths, counts, retry attempts, etc.
    /// Maps to `VeryVerbose` (`-vv`) level in the presentation implementation.
    ///
    /// # Arguments
    ///
    /// * `message` - Human-readable detail message
    fn on_detail(&self, message: &str);

    /// Reports a technical/debug detail about the current operation.
    ///
    /// Intended for commands executed, exit codes, raw output, etc.
    /// Maps to Debug (`-vvv`) level in the presentation implementation.
    ///
    /// # Arguments
    ///
    /// * `message` - Technical detail message
    fn on_debug(&self, message: &str);
}

/// A no-op listener that discards all progress events.
///
/// Used when progress reporting is not needed, such as in tests
/// or when the caller does not provide a listener.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::application::traits::NullProgressListener;
/// use torrust_tracker_deployer_lib::application::traits::CommandProgressListener;
///
/// let listener = NullProgressListener;
/// listener.on_step_started(1, 3, "Step one");
/// listener.on_step_completed(1, "Step one");
/// listener.on_detail("some detail");
/// listener.on_debug("some debug info");
/// // All calls are no-ops
/// ```
pub struct NullProgressListener;

impl CommandProgressListener for NullProgressListener {
    fn on_step_started(&self, _step_number: usize, _total_steps: usize, _description: &str) {}
    fn on_step_completed(&self, _step_number: usize, _description: &str) {}
    fn on_detail(&self, _message: &str) {}
    fn on_debug(&self, _message: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_accept_null_listener_without_panicking() {
        let listener = NullProgressListener;
        listener.on_step_started(1, 9, "Rendering templates");
        listener.on_step_completed(1, "Rendering templates");
        listener.on_detail("Template directory: build/test/tofu");
        listener.on_debug("Command: tofu init");
    }

    #[test]
    fn it_should_work_as_trait_object() {
        let listener: &dyn CommandProgressListener = &NullProgressListener;
        listener.on_step_started(1, 3, "First step");
        listener.on_step_completed(1, "First step");
        listener.on_detail("detail");
        listener.on_debug("debug");
    }

    #[test]
    fn it_should_work_as_optional_trait_object() {
        let listener: Option<&dyn CommandProgressListener> = Some(&NullProgressListener);
        if let Some(l) = listener {
            l.on_step_started(1, 3, "First step");
        }

        let no_listener: Option<&dyn CommandProgressListener> = None;
        if let Some(l) = no_listener {
            l.on_step_started(1, 3, "Should not reach here");
        }
    }
}
