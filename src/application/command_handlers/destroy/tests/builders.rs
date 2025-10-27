//! Test builders for Destroy Command
//!
//! This module provides test builders that simplify test setup by managing
//! dependencies and lifecycle for `DestroyCommandHandler` tests.

use std::sync::Arc;

use tempfile::TempDir;

use crate::application::command_handlers::destroy::DestroyCommandHandler;

/// Test builder for `DestroyCommandHandler` that manages dependencies and lifecycle
///
/// This builder simplifies test setup by:
/// - Managing `TempDir` lifecycle
/// - Providing sensible defaults for all dependencies
/// - Allowing selective customization of dependencies
/// - Returning only the command handler and necessary test artifacts
pub struct DestroyCommandHandlerTestBuilder {
    temp_dir: TempDir,
}

impl DestroyCommandHandlerTestBuilder {
    /// Create a new test builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        Self { temp_dir }
    }

    /// Build the `DestroyCommandHandler` with all dependencies
    ///
    /// Returns: (`command_handler`, `temp_dir`)
    /// The `temp_dir` must be kept alive for the duration of the test.
    pub fn build(self) -> (DestroyCommandHandler, TempDir) {
        let repository_factory =
            crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                std::time::Duration::from_secs(30),
            );
        let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

        // Create a system clock for testing
        let clock = Arc::new(crate::shared::SystemClock);

        let command_handler = DestroyCommandHandler::new(repository, clock);

        (command_handler, self.temp_dir)
    }
}

impl Default for DestroyCommandHandlerTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
