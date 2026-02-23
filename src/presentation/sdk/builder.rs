//! Builder for constructing a [`Deployer`] with sensible defaults.
//!
//! The builder pattern hides dependency wiring (repository, clock, etc.)
//! so SDK consumers only need to provide the workspace path.
//!
//! # Example
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::presentation::sdk::Deployer;
//!
//! let deployer = Deployer::builder()
//!     .working_dir("/home/user/deployer-workspace")
//!     .build()
//!     .unwrap();
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use super::deployer::Deployer;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::SystemClock;

/// Default timeout for file lock operations (matches CLI default).
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(30);

/// Builder for constructing a [`Deployer`] instance.
///
/// # Required
///
/// - [`working_dir`](DeployerBuilder::working_dir) — the workspace root
///   where `data/` and `build/` directories live
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::presentation::sdk::Deployer;
///
/// let deployer = Deployer::builder()
///     .working_dir("/path/to/workspace")
///     .build()
///     .expect("Failed to build deployer");
/// ```
pub struct DeployerBuilder {
    working_dir: Option<PathBuf>,
}

impl DeployerBuilder {
    /// Create a new builder with no configuration.
    #[must_use]
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    /// Set the workspace root directory.
    ///
    /// This is the directory containing `data/` and `build/` subdirectories.
    /// It is the only required setting.
    #[must_use]
    pub fn working_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(path.into());
        self
    }

    /// Build the [`Deployer`] instance.
    ///
    /// # Errors
    ///
    /// Returns [`DeployerBuildError::MissingWorkingDir`] if `working_dir`
    /// was not set.
    pub fn build(self) -> Result<Deployer, DeployerBuildError> {
        let working_dir = self
            .working_dir
            .ok_or(DeployerBuildError::MissingWorkingDir)?;

        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));
        let data_dir = working_dir.join("data");
        let data_directory: Arc<Path> = Arc::from(data_dir.as_path());
        let repository = repository_factory.create(data_dir);
        let clock = Arc::new(SystemClock);

        Ok(Deployer::new(
            working_dir,
            repository,
            repository_factory,
            clock,
            data_directory,
        ))
    }
}

impl Default for DeployerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur when building a [`Deployer`].
#[derive(Debug, Error)]
pub enum DeployerBuildError {
    /// The required `working_dir` was not provided.
    #[error("working_dir is required but was not set")]
    MissingWorkingDir,
}
