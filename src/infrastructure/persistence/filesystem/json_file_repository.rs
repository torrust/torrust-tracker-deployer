//! Generic JSON file-based persistence layer
//!
//! This module provides a generic file-based repository that persists entities
//! as JSON files with atomic writes and file locking for concurrency control.
//!
//! # Design Philosophy
//!
//! The repository is designed as a **reusable infrastructure component** that
//! domain-specific repositories can use as a collaborator. This separation of
//! concerns allows domain repositories to focus on business logic while
//! delegating file I/O, serialization, and locking to this generic component.
//!
//! # Usage Patterns
//!
//! ## Pattern 1: Direct Usage (Simple Cases)
//!
//! For simple persistence needs, use the repository directly:
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use std::time::Duration;
//! use torrust_tracker_deploy::infrastructure::persistence::filesystem::json_file_repository::JsonFileRepository;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct AppConfig {
//!     host: String,
//!     port: u16,
//! }
//!
//! let repo = JsonFileRepository::new(Duration::from_secs(10));
//! let config = AppConfig {
//!     host: "localhost".to_string(),
//!     port: 8080,
//! };
//!
//! // Save configuration
//! repo.save(&PathBuf::from("./config/app.json"), &config)?;
//!
//! // Load configuration
//! let loaded_config: Option<AppConfig> = repo.load(&PathBuf::from("./config/app.json"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Pattern 2: Collaborator in Domain Repository (Recommended)
//!
//! For complex domain logic, wrap this repository in a domain-specific repository:
//!
//! ```rust,no_run
//! use std::path::{Path, PathBuf};
//! use std::time::Duration;
//! use torrust_tracker_deploy::infrastructure::persistence::filesystem::json_file_repository::{
//!     JsonFileRepository, JsonFileError
//! };
//! use serde::{Deserialize, Serialize};
//!
//! // Domain entity
//! #[derive(Serialize, Deserialize)]
//! struct UserProfile {
//!     username: String,
//!     email: String,
//! }
//!
//! // Domain-specific error type
//! #[derive(Debug, thiserror::Error)]
//! enum UserProfileError {
//!     #[error("User profile not found: {username}")]
//!     NotFound { username: String },
//!
//!     #[error("User profile is locked by another process: {username}")]
//!     Locked { username: String },
//!
//!     #[error("Failed to persist user profile: {0}")]
//!     PersistenceError(#[from] JsonFileError),
//! }
//!
//! // Domain repository wrapping generic repository
//! struct UserProfileRepository {
//!     file_repo: JsonFileRepository,
//!     base_path: PathBuf,
//! }
//!
//! impl UserProfileRepository {
//!     pub fn new(base_path: PathBuf) -> Self {
//!         Self {
//!             file_repo: JsonFileRepository::new(Duration::from_secs(5)),
//!             base_path,
//!         }
//!     }
//!
//!     fn user_file_path(&self, username: &str) -> PathBuf {
//!         self.base_path.join(format!("{username}.json"))
//!     }
//!
//!     pub fn save_profile(&self, profile: &UserProfile) -> Result<(), UserProfileError> {
//!         let file_path = self.user_file_path(&profile.username);
//!
//!         // Delegate to generic repository
//!         self.file_repo.save(&file_path, profile)
//!             .map_err(|e| match e {
//!                 JsonFileError::Conflict { .. } => UserProfileError::Locked {
//!                     username: profile.username.clone(),
//!                 },
//!                 _ => UserProfileError::from(e),
//!             })
//!     }
//!
//!     pub fn load_profile(&self, username: &str) -> Result<UserProfile, UserProfileError> {
//!         let file_path = self.user_file_path(username);
//!
//!         // Delegate to generic repository
//!         self.file_repo.load(&file_path)
//!             .map_err(UserProfileError::from)?
//!             .ok_or_else(|| UserProfileError::NotFound {
//!                 username: username.to_string(),
//!             })
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # File Structure
//!
//! Files are organized as follows:
//! ```text
//! {file_path}       # Entity data in JSON format
//! {file_path}.lock  # Lock file (contains process ID)
//! ```
//!
//! # Atomic Writes
//!
//! Uses the temp file + rename pattern for atomic writes:
//! 1. Write data to `{file}.tmp`
//! 2. Fsync to ensure data is on disk (Unix only)
//! 3. Rename to final location (atomic operation)
//!
//! This ensures that files are never partially written, even if a crash
//! occurs during the write operation.
//!
//! # Concurrency Control
//!
//! Uses `FileLock` to prevent concurrent access. Lock files contain the process
//! ID of the lock holder for debugging and stale lock detection.
//!
//! All operations (read and write) acquire locks to ensure consistency.
//!
//! # Error Handling
//!
//! The repository uses a simplified error type with three categories:
//!
//! - `NotFound`: File doesn't exist (only used internally)
//! - `Conflict`: File is locked by another process (timeout or held by another PID)
//! - `Internal`: I/O errors, serialization errors, or unexpected failures
//!
//! Domain-specific repositories should map these to their own error types
//! with more context (see Pattern 2 example above).
//!
//! # Thread Safety
//!
//! The repository itself is thread-safe and can be shared across threads using `Arc`.
//! File locking ensures that concurrent access from multiple threads or processes
//! is handled correctly.

use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::infrastructure::persistence::filesystem::file_lock::{FileLock, FileLockError};

/// Error type for JSON file operations
///
/// This is a simplified error type for the generic repository. Domain-specific
/// repositories should map these errors to their own error types.
#[derive(Debug, thiserror::Error)]
pub enum JsonFileError {
    /// File not found
    #[error("File not found: {path}")]
    NotFound { path: String },

    /// Lock conflict - another process is accessing the file
    #[error("Lock conflict: another process is accessing {path}")]
    Conflict { path: String },

    /// Internal error with context
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Generic JSON file-based repository
///
/// Provides atomic write operations and file locking for any serializable entity.
/// Designed to be used as a collaborator by domain-specific repositories.
///
/// # Type Parameters
///
/// This repository can persist any type that implements `Serialize` and `Deserialize`.
///
/// # Atomic Writes
///
/// All write operations use atomic rename to prevent partial writes:
/// 1. Write to temporary file
/// 2. Fsync (Unix only)
/// 3. Atomic rename to final location
///
/// # Concurrency
///
/// File locks are acquired for all operations (read and write) to prevent
/// concurrent access conflicts.
pub struct JsonFileRepository {
    /// Timeout for acquiring file locks (public for testing)
    pub lock_timeout: Duration,
}

impl JsonFileRepository {
    /// Temporary file extension used during atomic writes
    ///
    /// This extension is appended to the target file path when creating
    /// temporary files for atomic write operations. After the write is
    /// complete, the temporary file is atomically renamed to the target path.
    const TEMP_FILE_EXTENSION: &'static str = "json.tmp";

    /// Create a new JSON file repository
    ///
    /// # Arguments
    ///
    /// * `lock_timeout` - Maximum time to wait for lock acquisition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use torrust_tracker_deploy::infrastructure::persistence::filesystem::json_file_repository::JsonFileRepository;
    ///
    /// let repo = JsonFileRepository::new(Duration::from_secs(10));
    /// ```
    #[must_use]
    pub fn new(lock_timeout: Duration) -> Self {
        Self { lock_timeout }
    }

    /// Save an entity to a JSON file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path where the entity will be saved
    /// * `entity` - Entity to serialize and save
    ///
    /// # Errors
    ///
    /// Returns `JsonFileError::Conflict` if the file is locked by another process.
    /// Returns `JsonFileError::Internal` for I/O or serialization errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    /// use torrust_tracker_deploy::infrastructure::persistence::filesystem::json_file_repository::JsonFileRepository;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct MyEntity { value: i32 }
    ///
    /// let repo = JsonFileRepository::new(Duration::from_secs(10));
    /// let entity = MyEntity { value: 42 };
    ///
    /// // repo.save(&PathBuf::from("./data/entity.json"), &entity)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn save<T: Serialize>(&self, file_path: &Path, entity: &T) -> Result<(), JsonFileError> {
        // Ensure parent directory exists
        Self::ensure_parent_dir(file_path)?;

        // Acquire lock
        let _lock = FileLock::acquire(file_path, self.lock_timeout)
            .map_err(|e| Self::convert_lock_error(e, file_path, "save"))?;

        // Serialize to JSON
        let json_content = serde_json::to_string_pretty(entity)
            .context("Failed to serialize entity to JSON")
            .map_err(JsonFileError::Internal)?;

        // Write atomically
        Self::write_atomic(file_path, &json_content)?;

        Ok(())
        // Lock is automatically released when _lock goes out of scope
    }

    /// Load an entity from a JSON file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to load
    ///
    /// # Returns
    ///
    /// * `Ok(Some(entity))` - Entity was successfully loaded
    /// * `Ok(None)` - File doesn't exist
    /// * `Err(_)` - An error occurred
    ///
    /// # Errors
    ///
    /// Returns `JsonFileError::Conflict` if the file is locked by another process.
    /// Returns `JsonFileError::Internal` for I/O or deserialization errors.
    pub fn load<T: for<'de> Deserialize<'de>>(
        &self,
        file_path: &Path,
    ) -> Result<Option<T>, JsonFileError> {
        // Check if file exists
        if !file_path.exists() {
            return Ok(None);
        }

        // Acquire lock for reading
        let _lock = FileLock::acquire(file_path, self.lock_timeout)
            .map_err(|e| Self::convert_lock_error(e, file_path, "load"))?;

        // Read file content
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))
            .map_err(JsonFileError::Internal)?;

        // Deserialize from JSON
        let entity: T = serde_json::from_str(&content)
            .with_context(|| format!("Failed to deserialize JSON from: {}", file_path.display()))
            .map_err(JsonFileError::Internal)?;

        Ok(Some(entity))
        // Lock is automatically released when _lock goes out of scope
    }

    /// Check if a file exists
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to check
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise.
    #[must_use]
    pub fn exists(&self, file_path: &Path) -> bool {
        file_path.exists()
    }

    /// Delete a file
    ///
    /// This operation is idempotent - deleting a non-existent file is not an error.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to delete
    ///
    /// # Errors
    ///
    /// Returns `JsonFileError::Conflict` if the file is locked by another process.
    /// Returns `JsonFileError::Internal` for I/O errors.
    pub fn delete(&self, file_path: &Path) -> Result<(), JsonFileError> {
        // If file doesn't exist, operation is successful (idempotent)
        if !file_path.exists() {
            return Ok(());
        }

        // Acquire lock before deletion
        let _lock = FileLock::acquire(file_path, self.lock_timeout)
            .map_err(|e| Self::convert_lock_error(e, file_path, "delete"))?;

        // Delete the file
        fs::remove_file(file_path)
            .with_context(|| format!("Failed to delete file: {}", file_path.display()))
            .map_err(JsonFileError::Internal)?;

        Ok(())
        // Lock is automatically released when _lock goes out of scope
    }

    /// Ensure the parent directory of a file exists
    ///
    /// Creates the directory and all parent directories if they don't exist.
    fn ensure_parent_dir(file_path: &Path) -> Result<(), JsonFileError> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))
                .map_err(JsonFileError::Internal)?;
        }
        Ok(())
    }

    /// Write content to file atomically using temp file + rename pattern
    ///
    /// This ensures that the file is never in a partially written state, even if
    /// a crash occurs during writing.
    fn write_atomic(file_path: &Path, content: &str) -> Result<(), JsonFileError> {
        let temp_path = file_path.with_extension(Self::TEMP_FILE_EXTENSION);

        // Write to temporary file
        fs::write(&temp_path, content)
            .with_context(|| format!("Failed to write temporary file: {}", temp_path.display()))
            .map_err(JsonFileError::Internal)?;

        // Fsync to ensure data is on disk (Unix only)
        #[cfg(unix)]
        {
            use std::fs::OpenOptions;
            let file = OpenOptions::new()
                .write(true)
                .open(&temp_path)
                .with_context(|| {
                    format!(
                        "Failed to open temporary file for fsync: {}",
                        temp_path.display()
                    )
                })
                .map_err(JsonFileError::Internal)?;

            file.sync_all()
                .with_context(|| format!("Failed to fsync temporary file: {}", temp_path.display()))
                .map_err(JsonFileError::Internal)?;
        }

        // Atomic rename
        fs::rename(&temp_path, file_path)
            .with_context(|| {
                format!(
                    "Failed to rename {} to {}",
                    temp_path.display(),
                    file_path.display()
                )
            })
            .map_err(JsonFileError::Internal)
    }

    /// Convert `FileLockError` to `JsonFileError` with operation context
    ///
    /// Lock acquisition timeouts are mapped to `Conflict` errors, while other
    /// lock errors are wrapped in `Internal` with operation context.
    ///
    /// # Arguments
    ///
    /// * `error` - The lock error to convert
    /// * `file_path` - Path to the file being locked
    /// * `operation` - Description of the operation being performed (e.g., "save", "load", "delete")
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let _lock = FileLock::acquire(file_path, self.lock_timeout)
    ///     .map_err(|e| Self::convert_lock_error(e, file_path, "save"))?;
    /// ```
    fn convert_lock_error(
        error: FileLockError,
        file_path: &Path,
        operation: &str,
    ) -> JsonFileError {
        match error {
            FileLockError::AcquisitionTimeout { .. } | FileLockError::LockHeldByProcess { .. } => {
                JsonFileError::Conflict {
                    path: file_path.display().to_string(),
                }
            }
            _ => JsonFileError::Internal(anyhow::Error::from(error).context(format!(
                "Lock operation failed during '{}' for: {}",
                operation,
                file_path.display()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::fixtures::TestEntity;
    use rstest::rstest;
    use std::error::Error as StdError;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test scenario builder for JSON file repository tests
    ///
    /// Provides a fluent interface for setting up common test scenarios,
    /// reducing boilerplate and improving test readability.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Basic scenario with defaults
    /// let scenario = TestRepositoryScenario::new();
    /// let entity = TestEntity::default();
    /// scenario.save(&entity).expect("Failed to save");
    ///
    /// // Scenario optimized for timeout tests
    /// let scenario = TestRepositoryScenario::for_timeout_test();
    /// // ... test timeout behavior
    ///
    /// // Custom file name
    /// let scenario = TestRepositoryScenario::new()
    ///     .with_file_name("custom.json");
    /// ```
    struct TestRepositoryScenario {
        temp_dir: TempDir,
        repo: JsonFileRepository,
        file_name: String,
    }

    impl TestRepositoryScenario {
        /// Create a new test scenario with default settings
        ///
        /// Default timeout: 10 seconds
        /// Default file name: "test.json"
        fn new() -> Self {
            Self {
                temp_dir: TempDir::new().expect("Failed to create temporary directory for test"),
                repo: JsonFileRepository::new(Duration::from_secs(10)),
                file_name: "test.json".to_string(),
            }
        }

        /// Create scenario with custom timeout
        fn with_timeout(timeout: Duration) -> Self {
            Self {
                temp_dir: TempDir::new().expect("Failed to create temporary directory for test"),
                repo: JsonFileRepository::new(timeout),
                file_name: "test.json".to_string(),
            }
        }

        /// Create scenario optimized for timeout tests (short timeout)
        fn for_timeout_test() -> Self {
            Self::with_timeout(Duration::from_millis(100))
        }

        /// Create scenario optimized for success tests (longer timeout)
        #[allow(dead_code)]
        fn for_success_test() -> Self {
            Self::with_timeout(Duration::from_secs(10))
        }

        /// Set custom file name
        fn with_file_name(mut self, name: impl Into<String>) -> Self {
            self.file_name = name.into();
            self
        }

        /// Get the repository instance
        #[allow(dead_code)]
        fn repo(&self) -> &JsonFileRepository {
            &self.repo
        }

        /// Get the file path for the scenario
        fn file_path(&self) -> PathBuf {
            self.temp_dir.path().join(&self.file_name)
        }

        /// Save an entity using this scenario's repository
        fn save<T: Serialize>(&self, entity: &T) -> Result<(), JsonFileError> {
            self.repo.save(&self.file_path(), entity)
        }

        /// Load an entity using this scenario's repository
        fn load<T: for<'de> Deserialize<'de>>(&self) -> Result<Option<T>, JsonFileError> {
            self.repo.load(&self.file_path())
        }

        /// Check if file exists using this scenario's repository
        fn exists(&self) -> bool {
            self.repo.exists(&self.file_path())
        }

        /// Delete file using this scenario's repository
        fn delete(&self) -> Result<(), JsonFileError> {
            self.repo.delete(&self.file_path())
        }
    }

    // Test Assertion Helpers
    // These functions provide reusable assertions for common test patterns,
    // improving test readability and reducing duplication.

    /// Assert that the temporary file was cleaned up and target file exists after atomic write
    ///
    /// This verifies the atomic write operation completed successfully by checking:
    /// - Temporary file (*.json.tmp) no longer exists
    /// - Target file exists at the expected location
    ///
    /// # Panics
    ///
    /// Panics if either assertion fails, indicating incomplete atomic write
    fn assert_atomic_write_completed(file_path: &Path) {
        let temp_file = file_path.with_extension(JsonFileRepository::TEMP_FILE_EXTENSION);
        assert!(
            !temp_file.exists(),
            "Temporary file should be cleaned up after atomic write: {temp_file:?}"
        );
        assert!(
            file_path.exists(),
            "Target file should exist after atomic write: {file_path:?}"
        );
    }

    /// Assert that file contains valid JSON with expected structure
    ///
    /// This function:
    /// 1. Reads the file content as a string
    /// 2. Parses it as JSON to verify syntax
    /// 3. Deserializes to the expected type to verify structure
    /// 4. Returns the parsed JSON value for further assertions
    ///
    /// # Type Parameters
    ///
    /// * `T` - The expected entity type that the JSON should deserialize to
    ///
    /// # Returns
    ///
    /// The parsed JSON value for additional field-level assertions
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - File cannot be read
    /// - Content is not valid JSON
    /// - JSON cannot be deserialized to type `T`
    fn assert_json_structure_valid<T: for<'de> Deserialize<'de>>(
        file_path: &Path,
    ) -> serde_json::Value {
        let json_content = fs::read_to_string(file_path).expect("Should be able to read JSON file");

        // Verify it's valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json_content).expect("File should contain valid JSON");

        // Verify it can be deserialized to expected type
        let _typed: T = serde_json::from_value(parsed.clone())
            .expect("JSON should deserialize to expected type");

        parsed
    }

    /// Assert that the result is a conflict error (lock timeout or held by another process)
    ///
    /// This is used to verify that file locking is working correctly when
    /// multiple processes or threads attempt to access the same file.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Result is Ok (expected an error)
    /// - Error is not a `JsonFileError::Conflict` variant
    fn assert_is_conflict_error<T: std::fmt::Debug>(result: Result<T, JsonFileError>) {
        assert!(result.is_err(), "Expected conflict error, got Ok result");
        let err = result.expect_err("Already verified result is Err");
        assert!(
            matches!(err, JsonFileError::Conflict { .. }),
            "Expected Conflict error, got: {err:?}"
        );
    }

    /// Assert that the result is an internal error
    ///
    /// This is used to verify error handling for unexpected failures like
    /// I/O errors, serialization errors, or other system-level issues.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Result is Ok (expected an error)
    /// - Error is not a `JsonFileError::Internal` variant
    #[allow(dead_code)]
    fn assert_is_internal_error<T: std::fmt::Debug>(result: Result<T, JsonFileError>) {
        assert!(result.is_err(), "Expected internal error, got Ok result");
        let err = result.expect_err("Already verified result is Err");
        assert!(
            matches!(err, JsonFileError::Internal(_)),
            "Expected Internal error, got: {err:?}"
        );
    }

    #[test]
    fn it_should_create_repository_with_custom_timeout() {
        let timeout = Duration::from_secs(30);
        let repo = JsonFileRepository::new(timeout);
        assert_eq!(repo.lock_timeout, timeout);
    }

    #[test]
    fn it_should_save_and_load_entity_successfully() {
        // Arrange
        let scenario = TestRepositoryScenario::new();
        let entity = TestEntity::new("test-123", 42);

        // Act
        scenario
            .save(&entity)
            .expect("Failed to save entity to file");

        let loaded: Option<TestEntity> = scenario.load().expect("Failed to load entity from file");

        // Assert
        assert!(loaded.is_some());
        assert_eq!(loaded.expect("Entity should exist in file"), entity);
    }

    #[test]
    fn it_should_return_none_when_loading_nonexistent_file() {
        // Arrange
        let scenario = TestRepositoryScenario::new().with_file_name("nonexistent.json");

        // Act
        let result: Option<TestEntity> = scenario.load().expect("Failed to load from file");

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn it_should_check_if_file_exists() {
        // Arrange
        let scenario = TestRepositoryScenario::new();
        let entity = TestEntity::new("test", 100);

        // Act & Assert - Before save
        assert!(!scenario.exists());

        // Act - Save
        scenario
            .save(&entity)
            .expect("Failed to save entity to file");

        // Assert - After save
        assert!(scenario.exists());
    }

    #[test]
    fn it_should_delete_file_successfully() {
        // Arrange
        let scenario = TestRepositoryScenario::new();
        let entity = TestEntity::new("test", 100);

        // Act - Save then delete
        scenario
            .save(&entity)
            .expect("Failed to save entity to file");
        assert!(scenario.exists());

        scenario.delete().expect("Failed to delete file");

        // Assert
        assert!(!scenario.exists());
    }

    #[test]
    fn it_should_delete_nonexistent_file_without_error() {
        // Arrange
        let scenario = TestRepositoryScenario::new().with_file_name("nonexistent.json");

        // Act & Assert - Delete should succeed even if file doesn't exist (idempotent)
        scenario
            .delete()
            .expect("Failed to delete nonexistent file");
    }

    #[rstest]
    #[case("entity.json", "root directory")]
    #[case("nested/entity.json", "single nested directory")]
    #[case("nested/deep/entity.json", "double nested directory")]
    #[case("very/deep/nested/path/entity.json", "deep nested path")]
    fn it_should_create_parent_directories_automatically(
        #[case] file_path: &str,
        #[case] description: &str,
    ) {
        // Arrange
        let scenario = TestRepositoryScenario::new().with_file_name(file_path);
        let entity = TestEntity::new("test", 100);

        // Act - Save should create nested directory structure
        let result = scenario.save(&entity);

        // Assert
        assert!(
            result.is_ok(),
            "Failed to save to {description}: {result:?}"
        );
        assert!(scenario.exists(), "File should exist in {description}");

        let file_path = scenario.file_path();
        assert!(
            file_path
                .parent()
                .expect("File path should have parent directory")
                .exists(),
            "Parent directory should exist for {description}"
        );
    }

    #[test]
    fn it_should_overwrite_existing_file() {
        // Arrange
        let scenario = TestRepositoryScenario::new();
        let entity1 = TestEntity::new("first", 1);
        let entity2 = TestEntity::new("second", 2);

        // Act - Save first version
        scenario
            .save(&entity1)
            .expect("Failed to save first entity version");

        // Act - Save second version (should overwrite)
        scenario
            .save(&entity2)
            .expect("Failed to save second entity version");

        // Assert - Load should return latest version
        let loaded: TestEntity = scenario
            .load()
            .expect("Failed to load entity from file")
            .expect("Entity should exist in file");
        assert_eq!(loaded, entity2);
    }

    #[test]
    fn it_should_use_atomic_writes() {
        // Arrange
        let scenario = TestRepositoryScenario::new();
        let entity = TestEntity::new("test", 100);

        // Act
        scenario
            .save(&entity)
            .expect("Failed to save entity to file");

        // Assert
        assert_atomic_write_completed(&scenario.file_path());
    }

    #[test]
    fn it_should_preserve_json_structure() {
        // Arrange
        let scenario = TestRepositoryScenario::new();
        let entity = TestEntity::new("test", 100);

        // Act
        scenario
            .save(&entity)
            .expect("Failed to save entity to file");

        // Assert
        let json = assert_json_structure_valid::<TestEntity>(&scenario.file_path());
        assert!(json.is_object());
        assert_eq!(json["id"], "test");
        assert_eq!(json["value"], 100);
    }

    #[test]
    fn it_should_handle_concurrent_access_with_locking() {
        // Arrange
        let scenario = TestRepositoryScenario::for_timeout_test();
        let entity = TestEntity::new("test", 100);

        scenario
            .save(&entity)
            .expect("Failed to save entity to file");

        // Hold lock manually
        let _lock = FileLock::acquire(&scenario.file_path(), Duration::from_secs(5))
            .expect("Failed to acquire lock for test");

        // Act - Try to load while lock is held - should timeout
        let result: Result<Option<TestEntity>, JsonFileError> = scenario.load();

        // Assert
        assert_is_conflict_error(result);
    }

    #[test]
    fn it_should_return_conflict_error_on_lock_timeout() {
        // Arrange
        let scenario = TestRepositoryScenario::with_timeout(Duration::from_millis(50));
        let entity = TestEntity::new("test", 100);

        scenario
            .save(&entity)
            .expect("Failed to save entity to file");

        // Hold lock
        let _lock = FileLock::acquire(&scenario.file_path(), Duration::from_secs(5))
            .expect("Failed to acquire lock for test");

        // Act - Try to save while lock is held
        let result = scenario.save(&entity);

        // Assert
        assert_is_conflict_error(result);
    }

    #[test]
    fn it_should_display_error_messages_correctly() {
        // Test NotFound error - should be clear and include path
        let not_found = JsonFileError::NotFound {
            path: "/path/to/file.json".to_string(),
        };
        let message = not_found.to_string();
        assert!(
            message.contains("File not found"),
            "Should clearly state the problem"
        );
        assert!(
            message.contains("/path/to/file.json"),
            "Should include the file path for context"
        );

        // Test Conflict error - should explain the conflict and include path
        let conflict = JsonFileError::Conflict {
            path: "/path/to/file.json".to_string(),
        };
        let message = conflict.to_string();
        assert!(
            message.contains("Lock conflict"),
            "Should clearly state lock issue"
        );
        assert!(
            message.contains("another process"),
            "Should explain the conflict source"
        );
        assert!(
            message.contains("/path/to/file.json"),
            "Should include the file path for context"
        );

        // Test Internal error - should preserve context
        let internal = JsonFileError::Internal(anyhow::anyhow!("test error"));
        let message = internal.to_string();
        assert!(
            message.contains("Internal error"),
            "Should indicate internal error category"
        );
        assert!(
            message.contains("test error"),
            "Should preserve the underlying error message"
        );
    }

    #[test]
    fn it_should_preserve_error_source_chain() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let anyhow_error = anyhow::Error::from(io_error).context("operation failed");
        let json_error = JsonFileError::Internal(anyhow_error);

        // Verify error chain is preserved
        let mut source = json_error.source();
        let mut chain_length = 0;

        while let Some(err) = source {
            chain_length += 1;
            source = err.source();
        }

        assert!(
            chain_length >= 2,
            "Error chain should have at least 2 levels"
        );
    }

    #[test]
    fn it_should_preserve_full_error_context_chain_with_operation_context() {
        // Create a realistic error chain simulating an atomic write failure
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let anyhow_error = anyhow::Error::from(io_error)
            .context("Failed to write to temporary file")
            .context("Lock operation failed during 'save' for: /data/entity.json");
        let json_error = JsonFileError::Internal(anyhow_error);

        // Verify error chain is preserved and accessible
        let mut source = json_error.source();
        let mut chain_messages = Vec::new();

        while let Some(err) = source {
            chain_messages.push(err.to_string());
            source = err.source();
        }

        // Should have multiple context levels
        assert!(
            chain_messages.len() >= 2,
            "Error chain should preserve multiple context levels, found: {}",
            chain_messages.len()
        );

        // Should preserve operation context (high-level)
        assert!(
            chain_messages
                .iter()
                .any(|m| m.contains("Lock operation failed during 'save'")),
            "Should preserve operation context in error chain"
        );

        // Should preserve intermediate context
        assert!(
            chain_messages
                .iter()
                .any(|m| m.contains("Failed to write to temporary file")),
            "Should preserve intermediate context in error chain"
        );

        // Should preserve root cause
        assert!(
            chain_messages.iter().any(|m| m.contains("access denied")),
            "Should preserve root cause in error chain"
        );
    }
}
