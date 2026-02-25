//! `RepositoryProvider` trait — application-layer abstraction for repository creation
//!
//! Defines the port that the application layer uses to create environment repositories
//! without depending on any concrete infrastructure type.
//!
//! The infrastructure layer implements this trait for `FileRepositoryFactory`.
//! The bootstrap layer creates the concrete implementation and injects it.

use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::environment::repository::EnvironmentRepository;

/// Application-layer trait for creating environment repositories.
///
/// This trait abstracts over the concrete `FileRepositoryFactory` from the
/// infrastructure layer, allowing the application layer and SDK to hold a
/// `Arc<dyn RepositoryProvider>` without a compile-time dependency on any
/// infrastructure type.
///
/// The bootstrap layer provides the default implementation via
/// `crate::infrastructure::persistence::file_repository_factory::FileRepositoryFactory`.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use std::sync::Arc;
/// use torrust_tracker_deployer_lib::application::traits::RepositoryProvider;
///
/// fn list_all_environments(
///     provider: Arc<dyn RepositoryProvider>,
///     data_dir: PathBuf,
/// ) {
///     let repo = provider.create(data_dir.clone());
///     // use repo ...
///     drop(repo);
/// }
/// ```
pub trait RepositoryProvider: Send + Sync {
    /// Create a new repository scoped to the given data directory.
    fn create(&self, data_dir: PathBuf) -> Arc<dyn EnvironmentRepository + Send + Sync>;
}
