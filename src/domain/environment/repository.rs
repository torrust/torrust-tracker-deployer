use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::state::AnyEnvironmentState;
use thiserror::Error;

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

/// Errors that can occur during repository operations
///
/// This enum provides a generic error interface that doesn't expose implementation
/// details. Concrete repository implementations wrap their specific errors in
/// `Internal(anyhow::Error)`, allowing callers to:
/// - Handle errors generically in most cases
/// - Downcast to specific error types for advanced debugging
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::repository::{EnvironmentRepository, RepositoryError};
///
/// fn handle_repository_error(err: RepositoryError) {
///     match err {
///         RepositoryError::NotFound => {
///             println!("Environment not found - might be first run");
///         }
///         RepositoryError::Conflict => {
///             println!("Another process is accessing this environment");
///         }
///         RepositoryError::Internal(inner) => {
///             eprintln!("Internal error: {}", inner);
///             
///             // Advanced: downcast to specific error type for debugging
///             if let Some(io_err) = inner.downcast_ref::<std::io::Error>() {
///                 eprintln!("Underlying IO error: {:?}", io_err.kind());
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// Environment not found in storage
    ///
    /// This typically means the environment has never been saved, or has been deleted.
    #[error("Environment not found")]
    NotFound,

    /// Conflict with concurrent operation
    ///
    /// Another process is currently accessing the same environment. This can occur when:
    /// - File-based repository: Another process holds the lock
    /// - Database repository: Transaction conflict or row lock
    /// - In-memory repository: Concurrent modification detected
    #[error("Conflict: another process is accessing this environment")]
    Conflict,

    /// Internal implementation-specific error
    ///
    /// This wraps errors specific to the repository implementation:
    /// - File repository: I/O errors, serialization failures, permission issues
    /// - Database repository: Connection errors, query failures
    /// - In-memory repository: Usually not used (in-memory ops rarely fail)
    ///
    /// Advanced callers can downcast the inner `anyhow::Error` to recover the
    /// original error type for detailed debugging.
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_should_derive_debug_for_repository_error() {
        let error = RepositoryError::NotFound;
        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("NotFound"));
    }

    #[test]
    fn it_should_display_not_found_error_message() {
        let error = RepositoryError::NotFound;
        let message = error.to_string();
        assert_eq!(message, "Environment not found");
    }

    #[test]
    fn it_should_display_conflict_error_message() {
        let error = RepositoryError::Conflict;
        let message = error.to_string();
        assert!(message.contains("Conflict"));
        assert!(message.contains("another process"));
    }

    #[test]
    fn it_should_display_internal_error_message() {
        let inner = anyhow::Error::from(io::Error::other("test error"));
        let error = RepositoryError::Internal(inner);
        let message = error.to_string();
        assert!(message.contains("Internal error"));
        assert!(message.contains("test error"));
    }

    #[test]
    fn it_should_convert_from_anyhow_error() {
        let io_error = io::Error::other("test");
        let anyhow_error = anyhow::Error::from(io_error);
        let repo_error: RepositoryError = anyhow_error.into();

        match repo_error {
            RepositoryError::Internal(_) => {
                // Success - converted to Internal variant
            }
            _ => panic!("Expected Internal variant"),
        }
    }

    #[test]
    fn it_should_preserve_error_source_in_internal_variant() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let anyhow_error = anyhow::Error::from(io_error);
        let repo_error = RepositoryError::Internal(anyhow_error);

        // Verify we can downcast back to original error type
        if let RepositoryError::Internal(inner) = repo_error {
            let downcasted = inner.downcast_ref::<io::Error>();
            assert!(downcasted.is_some());
            assert_eq!(downcasted.unwrap().kind(), io::ErrorKind::PermissionDenied);
        } else {
            panic!("Expected Internal variant");
        }
    }

    #[test]
    fn it_should_allow_downcasting_to_multiple_error_types() {
        // Test with I/O error
        let io_error = io::Error::other("io test");
        let repo_error = RepositoryError::Internal(anyhow::Error::from(io_error));

        if let RepositoryError::Internal(inner) = repo_error {
            assert!(inner.downcast_ref::<io::Error>().is_some());
        }

        // Test with different error type (using serde_json as example)
        let json_error = serde_json::Error::io(io::Error::other("json test"));
        let repo_error = RepositoryError::Internal(anyhow::Error::from(json_error));

        if let RepositoryError::Internal(inner) = repo_error {
            assert!(inner.downcast_ref::<serde_json::Error>().is_some());
        }
    }

    #[test]
    fn it_should_support_error_equality_for_variants() {
        // NotFound and Conflict don't have data, so we can compare them directly
        let error1 = RepositoryError::NotFound;
        let error2 = RepositoryError::NotFound;

        assert_eq!(
            format!("{error1:?}"),
            format!("{error2:?}"),
            "NotFound errors should format identically"
        );
    }

    #[test]
    fn it_should_provide_clear_error_messages_for_user_facing_display() {
        // Test that error messages are suitable for end users
        let not_found = RepositoryError::NotFound;
        assert!(
            !not_found.to_string().contains("implementation"),
            "NotFound message should not expose implementation details"
        );

        let conflict = RepositoryError::Conflict;
        assert!(
            !conflict.to_string().contains("lock") && !conflict.to_string().contains("file"),
            "Conflict message should not expose implementation details"
        );
    }

    #[test]
    fn it_should_wrap_complex_error_chains_in_internal() {
        // Simulate a chain of errors (e.g., I/O error -> serde error -> our error)
        let root_cause = io::Error::other("disk full");
        let wrapped = anyhow::Error::from(root_cause)
            .context("Failed to write JSON")
            .context("Failed to save environment");

        let repo_error = RepositoryError::Internal(wrapped);

        // The full error chain should be preserved
        let error_string = format!(
            "{:#}",
            match repo_error {
                RepositoryError::Internal(ref e) => e,
                _ => panic!("Expected Internal"),
            }
        );

        assert!(error_string.contains("Failed to save environment"));
        assert!(error_string.contains("Failed to write JSON"));
    }
}
