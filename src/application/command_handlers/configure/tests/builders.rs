//! Test builders for Configure Command
//!
//! This module provides test builders that simplify test setup by managing
//! dependencies and lifecycle for `ConfigureCommandHandler` tests.

use std::sync::Arc;

use tempfile::TempDir;

use crate::adapters::ansible::AnsibleClient;
use crate::application::command_handlers::configure::ConfigureCommandHandler;

/// Test builder for `ConfigureCommandHandler` that manages dependencies and lifecycle
///
/// This builder simplifies test setup by:
/// - Managing `TempDir` lifecycle
/// - Providing sensible defaults for all dependencies
/// - Returning only the command and necessary test artifacts
pub struct ConfigureCommandHandlerTestBuilder {
    temp_dir: TempDir,
}

impl ConfigureCommandHandlerTestBuilder {
    /// Create a new test builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        Self { temp_dir }
    }

    /// Build the `ConfigureCommandHandler` with all dependencies
    ///
    /// Returns: (`command`, `temp_dir`)
    /// The `temp_dir` must be kept alive for the duration of the test.
    pub fn build(self) -> (ConfigureCommandHandler, TempDir) {
        let ansible_client = Arc::new(AnsibleClient::new(self.temp_dir.path()));
        let clock: Arc<dyn crate::shared::Clock> = Arc::new(crate::shared::SystemClock);

        let repository_factory =
            crate::infrastructure::persistence::repository_factory::RepositoryFactory::new(
                std::time::Duration::from_secs(30),
            );
        let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

        let command_handler = ConfigureCommandHandler::new(ansible_client, clock, repository);

        (command_handler, self.temp_dir)
    }
}

impl Default for ConfigureCommandHandlerTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
