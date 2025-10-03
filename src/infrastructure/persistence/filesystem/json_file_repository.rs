//! Generic JSON file-based persistence layer
//!
//! This module provides a generic file-based repository that persists entities
//!
//! The repository is designed to be a reusable component that can be used by
//! domain-specific repositories as a collaborator for handling all file I/O,
//! atomic writes, and locking logic.
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
//! # Usage
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use std::time::Duration;
//! use torrust_tracker_deploy::infrastructure::persistence::filesystem::json_file_repository::JsonFileRepository;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyEntity {
//!     id: String,
//!     value: i32,
//! }
//!
//! let repo = JsonFileRepository::new(Duration::from_secs(10));
//! let entity = MyEntity { id: "test".to_string(), value: 42 };
//!
//! // Save entity
//! // repo.save(&PathBuf::from("./data/entity.json"), &entity)?;
//!
//! // Load entity
//! // let loaded: Option<MyEntity> = repo.load(&PathBuf::from("./data/entity.json"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

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
            .map_err(|e| Self::convert_lock_error(e, file_path))?;

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
            .map_err(|e| Self::convert_lock_error(e, file_path))?;

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
            .map_err(|e| Self::convert_lock_error(e, file_path))?;

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
        let temp_path = file_path.with_extension("json.tmp");

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

    /// Convert `FileLockError` to `JsonFileError`
    ///
    /// Lock acquisition timeouts are mapped to `Conflict` errors, while other
    /// lock errors are wrapped in `Internal`.
    fn convert_lock_error(error: FileLockError, file_path: &Path) -> JsonFileError {
        match error {
            FileLockError::AcquisitionTimeout { .. } | FileLockError::LockHeldByProcess { .. } => {
                JsonFileError::Conflict {
                    path: file_path.display().to_string(),
                }
            }
            _ => JsonFileError::Internal(anyhow::Error::from(error).context(format!(
                "Lock operation failed for: {}",
                file_path.display()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::fixtures::TestEntity;
    use std::error::Error as StdError;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_repository_with_custom_timeout() {
        let timeout = Duration::from_secs(30);
        let repo = JsonFileRepository::new(timeout);
        assert_eq!(repo.lock_timeout, timeout);
    }

    #[test]
    fn it_should_save_and_load_entity_successfully() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("entity.json");

        let entity = TestEntity::new("test-123", 42);

        // Save
        repo.save(&file_path, &entity).unwrap();

        // Load
        let loaded: Option<TestEntity> = repo.load(&file_path).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), entity);
    }

    #[test]
    fn it_should_return_none_when_loading_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("nonexistent.json");

        let result: Option<TestEntity> = repo.load(&file_path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn it_should_check_if_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("entity.json");

        let entity = TestEntity::new("test", 100);

        // Before save
        assert!(!repo.exists(&file_path));

        // Save
        repo.save(&file_path, &entity).unwrap();

        // After save
        assert!(repo.exists(&file_path));
    }

    #[test]
    fn it_should_delete_file_successfully() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("entity.json");

        let entity = TestEntity::new("test", 100);

        // Save then delete
        repo.save(&file_path, &entity).unwrap();
        assert!(repo.exists(&file_path));

        repo.delete(&file_path).unwrap();
        assert!(!repo.exists(&file_path));
    }

    #[test]
    fn it_should_delete_nonexistent_file_without_error() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("nonexistent.json");

        // Delete should succeed even if file doesn't exist (idempotent)
        repo.delete(&file_path).unwrap();
    }

    #[test]
    fn it_should_create_parent_directories_automatically() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir
            .path()
            .join("nested")
            .join("deep")
            .join("entity.json");

        let entity = TestEntity::new("test", 100);

        // Save should create nested directory structure
        repo.save(&file_path, &entity).unwrap();

        assert!(file_path.parent().unwrap().exists());
        assert!(file_path.exists());
    }

    #[test]
    fn it_should_overwrite_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("entity.json");

        let entity1 = TestEntity::new("first", 1);
        let entity2 = TestEntity::new("second", 2);

        // Save first version
        repo.save(&file_path, &entity1).unwrap();

        // Save second version (should overwrite)
        repo.save(&file_path, &entity2).unwrap();

        // Load should return latest version
        let loaded: TestEntity = repo.load(&file_path).unwrap().unwrap();
        assert_eq!(loaded, entity2);
    }

    #[test]
    fn it_should_use_atomic_writes() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("entity.json");

        let entity = TestEntity::new("test", 100);

        // Save
        repo.save(&file_path, &entity).unwrap();

        // Verify no temporary file exists after save
        let temp_file = file_path.with_extension("json.tmp");
        assert!(!temp_file.exists());

        // Verify target file exists
        assert!(file_path.exists());
    }

    #[test]
    fn it_should_preserve_json_structure() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_secs(10));
        let file_path = temp_dir.path().join("entity.json");

        let entity = TestEntity::new("test", 100);

        // Save
        repo.save(&file_path, &entity).unwrap();

        // Read raw JSON
        let json_content = fs::read_to_string(&file_path).unwrap();

        // Verify it's valid JSON and has expected structure
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        assert!(parsed.is_object());
        assert_eq!(parsed["id"], "test");
        assert_eq!(parsed["value"], 100);
    }

    #[test]
    fn it_should_handle_concurrent_access_with_locking() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_millis(100));
        let file_path = temp_dir.path().join("entity.json");

        let entity = TestEntity::new("test", 100);

        // Save to create the file
        repo.save(&file_path, &entity).unwrap();

        // Acquire lock manually
        let _lock = FileLock::acquire(&file_path, Duration::from_secs(5)).unwrap();

        // Try to load while lock is held - should timeout
        let result: Result<Option<TestEntity>, JsonFileError> = repo.load(&file_path);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, JsonFileError::Conflict { .. }));
    }

    #[test]
    fn it_should_return_conflict_error_on_lock_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonFileRepository::new(Duration::from_millis(50));
        let file_path = temp_dir.path().join("entity.json");

        let entity = TestEntity::new("test", 100);

        // Save to create the file
        repo.save(&file_path, &entity).unwrap();

        // Hold lock
        let _lock = FileLock::acquire(&file_path, Duration::from_secs(5)).unwrap();

        // Try to save while lock is held
        let result = repo.save(&file_path, &entity);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JsonFileError::Conflict { .. }
        ));
    }

    #[test]
    fn it_should_display_error_messages_correctly() {
        let not_found = JsonFileError::NotFound {
            path: "/path/to/file.json".to_string(),
        };
        assert!(not_found.to_string().contains("File not found"));
        assert!(not_found.to_string().contains("/path/to/file.json"));

        let conflict = JsonFileError::Conflict {
            path: "/path/to/file.json".to_string(),
        };
        assert!(conflict.to_string().contains("Lock conflict"));
        assert!(conflict.to_string().contains("/path/to/file.json"));

        let internal = JsonFileError::Internal(anyhow::anyhow!("test error"));
        assert!(internal.to_string().contains("Internal error"));
        assert!(internal.to_string().contains("test error"));
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
}
