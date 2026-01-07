//! List command handler implementation
//!
//! **Purpose**: List all environments in the deployment workspace
//!
//! This handler scans the data directory for environments and extracts
//! summary information for display. It is a read-only operation that
//! does not modify any state or make any network calls.
//!
//! ## Design Strategy
//!
//! The list command scans local storage for environments:
//!
//! 1. **Directory Scan**: Find all environment directories in data/
//! 2. **Load Summaries**: Extract lightweight info from each environment
//! 3. **Graceful Degradation**: Continue on per-environment errors
//! 4. **Report Failures**: Include failed environments in the result
//!
//! ## Design Rationale
//!
//! This command works directly with the data directory rather than through
//! the repository abstraction because:
//!
//! - Need to enumerate all environments (repository has no list method)
//! - Must handle partially corrupted data gracefully
//! - Performance: lightweight scanning without full deserialization where possible

use std::fs;
use std::path::Path;
use std::sync::Arc;

use tracing::{instrument, warn};

use super::errors::ListCommandHandlerError;
use super::info::{EnvironmentList, EnvironmentSummary};
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::AnyEnvironmentState;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;

/// `ListCommandHandler` scans and lists all environments
///
/// **Purpose**: Read-only enumeration of environments in the workspace
///
/// This handler scans the data directory and extracts summary information
/// for each environment found. It handles partial failures gracefully,
/// continuing to list valid environments even when some fail to load.
///
/// ## Error Handling
///
/// - **Empty directory**: Returns empty list (not an error)
/// - **Per-environment errors**: Collected and reported, don't stop listing
/// - **Fatal errors**: Directory not found, permission denied
pub struct ListCommandHandler {
    repository_factory: Arc<RepositoryFactory>,
    data_directory: Arc<Path>,
}

impl ListCommandHandler {
    /// Create a new `ListCommandHandler`
    #[must_use]
    pub fn new(repository_factory: Arc<RepositoryFactory>, data_directory: Arc<Path>) -> Self {
        Self {
            repository_factory,
            data_directory,
        }
    }

    /// Execute the list command workflow
    ///
    /// Scans the data directory and extracts summary information for all
    /// environments found.
    ///
    /// # Returns
    ///
    /// * `Ok(EnvironmentList)` - List of environment summaries (may include failures)
    /// * `Err(ListCommandHandlerError)` - If the data directory cannot be accessed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Data directory does not exist
    /// * Permission denied accessing data directory
    #[instrument(
        name = "list_command",
        skip_all,
        fields(
            command_type = "list",
            data_directory = %self.data_directory.display()
        )
    )]
    pub fn execute(&self) -> Result<EnvironmentList, ListCommandHandlerError> {
        // Verify data directory exists
        if !self.data_directory.exists() {
            return Err(ListCommandHandlerError::DataDirectoryNotFound {
                path: self.data_directory.to_path_buf(),
            });
        }

        // Scan for environment directories
        let env_dirs = self.scan_environment_directories()?;

        // Load summaries for each environment
        let (summaries, failures) = self.load_environment_summaries(&env_dirs);

        Ok(EnvironmentList::new(
            summaries,
            failures,
            self.data_directory.to_string_lossy().to_string(),
        ))
    }

    /// Scan the data directory for environment subdirectories
    fn scan_environment_directories(&self) -> Result<Vec<String>, ListCommandHandlerError> {
        let entries = fs::read_dir(&self.data_directory).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                ListCommandHandlerError::PermissionDenied {
                    path: self.data_directory.to_path_buf(),
                }
            } else {
                ListCommandHandlerError::ScanError {
                    message: e.to_string(),
                }
            }
        })?;

        let mut env_names = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {e}");
                    continue;
                }
            };

            // Only consider directories (environments are stored in subdirectories)
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Check if this directory contains an environment.json file
            let env_file = path.join("environment.json");
            if !env_file.exists() {
                continue;
            }

            // Extract directory name as environment name
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                env_names.push(name.to_string());
            }
        }

        Ok(env_names)
    }

    /// Load summaries for all discovered environments
    ///
    /// Returns a tuple of (successful summaries, failed environments)
    fn load_environment_summaries(
        &self,
        env_names: &[String],
    ) -> (Vec<EnvironmentSummary>, Vec<(String, String)>) {
        let mut summaries = Vec::new();
        let mut failures = Vec::new();

        for name in env_names {
            match self.load_environment_summary(name) {
                Ok(summary) => summaries.push(summary),
                Err(error) => {
                    warn!(
                        environment = %name,
                        error = %error,
                        "Failed to load environment"
                    );
                    failures.push((name.clone(), error));
                }
            }
        }

        (summaries, failures)
    }

    /// Load summary for a single environment
    fn load_environment_summary(&self, name: &str) -> Result<EnvironmentSummary, String> {
        // Validate environment name
        let env_name = EnvironmentName::new(name.to_string())
            .map_err(|e| format!("Invalid environment name: {e}"))?;

        // Create repository for the base data directory
        // (repository internally handles {base_dir}/{env_name}/environment.json)
        let repository = self
            .repository_factory
            .create(self.data_directory.to_path_buf());

        // Load environment from repository
        let any_env = Self::load_environment(&repository, &env_name)?;

        // Extract summary
        Ok(Self::extract_summary(&any_env))
    }

    /// Load environment from repository
    fn load_environment(
        repository: &Arc<dyn EnvironmentRepository + Send + Sync>,
        env_name: &EnvironmentName,
    ) -> Result<AnyEnvironmentState, String> {
        repository
            .load(env_name)
            .map_err(|e| format!("Failed to load environment: {e}"))?
            .ok_or_else(|| format!("Environment '{env_name}' not found in repository"))
    }

    /// Extract summary information from an environment
    fn extract_summary(any_env: &AnyEnvironmentState) -> EnvironmentSummary {
        let name = any_env.name().to_string();
        let state = any_env.state_display_name().to_string();
        let provider = any_env.provider_display_name().to_string();
        let created_at = any_env.created_at().to_rfc3339();

        EnvironmentSummary::new(name, state, provider, created_at)
    }
}
