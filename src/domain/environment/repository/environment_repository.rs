use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::state::AnyEnvironmentState;

use super::repository_error::RepositoryError;

/// Repository trait for persisting environments
///
/// This trait defines the contract for environment persistence operations.
/// Implementations can use different storage backends (files, databases, in-memory, etc.)
/// while maintaining consistent error handling and operation semantics.
///
/// # Concurrency
///
/// Implementations must handle concurrent access safely. File-based implementations
/// typically use locking mechanisms, while in-memory implementations might use
/// interior mutability patterns.
///
/// # Atomicity
///
/// Save operations should be atomic - either the entire environment is saved
/// successfully, or no changes are made to the storage.
///
/// # Error Handling
///
/// The trait uses `RepositoryError` as a generic error type. Implementation-specific
/// errors are wrapped in `RepositoryError::Internal(anyhow::Error)`, allowing
/// callers to handle errors generically while still supporting downcasting for
/// advanced debugging scenarios.
pub trait EnvironmentRepository {
    /// Save environment
    ///
    /// Persists the complete environment to storage. This operation should be atomic -
    /// either the entire environment is saved successfully, or no changes are made.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::Conflict` if another process is currently modifying
    /// the same environment.
    ///
    /// Returns `RepositoryError::Internal` for implementation-specific errors such as:
    /// - Serialization failures
    /// - Storage access issues (permissions, disk full, network errors)
    /// - Lock acquisition timeouts
    fn save(&self, env: &AnyEnvironmentState) -> Result<(), RepositoryError>;

    /// Load environment by name
    ///
    /// Retrieves the environment from storage. Returns `None` if the environment
    /// has never been saved.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::NotFound` if the environment does not exist.
    ///
    /// Returns `RepositoryError::Internal` for implementation-specific errors such as:
    /// - Deserialization failures (corrupted data)
    /// - Storage access issues
    /// - Lock acquisition timeouts
    fn load(&self, name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>, RepositoryError>;

    /// Check if environment exists
    ///
    /// Returns `true` if an environment with the given name exists in storage.
    /// Does not validate that the stored data is readable or well-formed.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::Internal` if there are storage access issues.
    fn exists(&self, name: &EnvironmentName) -> Result<bool, RepositoryError>;

    /// Delete environment
    ///
    /// Removes the persisted environment from storage. This is typically used
    /// when cleaning up after environment destruction.
    ///
    /// This operation is idempotent - deleting a non-existent environment is not an error.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::Conflict` if another process is currently accessing
    /// the environment.
    ///
    /// Returns `RepositoryError::Internal` for implementation-specific errors such as:
    /// - Storage access issues
    /// - Lock acquisition timeouts
    fn delete(&self, name: &EnvironmentName) -> Result<(), RepositoryError>;
}
