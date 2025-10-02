# Phase 4: Persistence - Implementation Plan

> **📋 Detailed Plan**  
> Breaking down Phase 4 into manageable, testable subtasks for state persistence with atomic writes and file locking.

## 🎯 Phase 4 Overview

**Goal**: Implement repository pattern for persistent state storage with atomic writes and file locking to prevent concurrent access issues.

**Why We Need This**: Enable environments to persist across command executions, survive crashes, and provide observability into deployment state without relying on memory-only storage.

**Approach**:

- Repository pattern with trait abstraction for multiple storage backends
- JSON file-based implementation as the primary storage backend
- Atomic write operations (temp file + rename) for data integrity
- File locking mechanism with process ID tracking to prevent race conditions
- Graceful handling of stale locks from crashed processes

## 📋 Implementation Subtasks

### Subtask 1: Define Repository Trait & Error Types ✅

**Purpose**: Define the contract for environment persistence operations with generic error handling that doesn't expose implementation details.

**Design Decision**: Renamed from `StateRepository` to `EnvironmentRepository` since we persist the entire `Environment<S>` (including name, credentials, instance name, state), not just the state marker. The error handling uses a generic pattern (NotFound, Conflict, Internal) to avoid exposing implementation-specific details (file paths, locks, etc.) in the trait interface.

**Changes**:

- Create new file `src/domain/environment/repository.rs`
- Define `EnvironmentRepository` trait with save/load operations
- Create `RepositoryError` enum with generic error variants
- Use `Internal(anyhow::Error)` to wrap implementation-specific errors
- Add module to `src/domain/environment/mod.rs`
- Document trait methods and error cases with examples

**Implementation Details**:

```rust
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
```

**Tests to Add**:

- Test that trait compiles and can be implemented
- Test error types derive Debug, Display correctly
- Test NotFound and Conflict variants display correct messages
- Test Internal variant wraps anyhow::Error correctly
- Test error conversion from anyhow::Error
- Test downcasting from Internal variant to specific error types
- Test error messages don't expose implementation details

**Success Criteria**:

- ✅ `EnvironmentRepository` trait compiles
- ✅ All trait methods documented with generic error descriptions
- ✅ `RepositoryError` enum uses generic pattern (NotFound, Conflict, Internal)
- ✅ Error messages are clear, actionable, and don't expose implementation details
- ✅ Downcasting works for debugging scenarios
- ✅ All linters pass
- ✅ All tests pass (615 tests)

**Commit**: `feat: [#24] add EnvironmentRepository trait with generic error handling`

**Status**: ✅ Complete

---

### Subtask 2: Implement File Locking Mechanism ⏳

**Purpose**: Implement robust file locking with process ID tracking to prevent concurrent access and handle stale locks from crashed processes.

**Changes**:

- Create new file `src/infrastructure/repository/file_lock.rs`
- Implement `FileLock` struct with process ID tracking
- Add lock acquisition with timeout
- Add stale lock detection and cleanup
- Add lock release with automatic cleanup
- Comprehensive tests for all locking scenarios

**Implementation Details**:

````rust
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, Instant};
use thiserror::Error;

/// File locking mechanism with process ID tracking
///
/// Provides exclusive access to state files by creating lock files that contain
/// the process ID of the lock holder. This prevents race conditions when multiple
/// processes attempt to read/write the same state file.
///
/// # Lock Files
///
/// Lock files are named `{state_file}.lock` and contain the process ID as text.
/// Example: `./data/my-env/state.json.lock` contains "12345"
///
/// # Stale Lock Detection
///
/// If a process crashes while holding a lock, the lock file remains but the
/// process is dead. This implementation detects stale locks by checking if
/// the process ID in the lock file is still running.
///
/// # Usage
///
/// ```rust
/// use std::path::Path;
///
/// let state_file = Path::new("./data/test-env/state.json");
/// let lock = FileLock::acquire(state_file, Duration::from_secs(5))?;
///
/// // Perform file operations...
///
/// lock.release()?; // Explicit release, also happens on drop
/// ```
pub struct FileLock {
    lock_file_path: PathBuf,
    acquired: bool,
}

impl FileLock {
    /// Attempt to acquire a lock for the given file path
    ///
    /// Creates a lock file at `{file_path}.lock` containing the current process ID.
    /// If the lock file already exists, checks if the holding process is still alive.
    /// If the process is dead, removes the stale lock and retries.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to lock (state file, not lock file)
    /// * `timeout` - Maximum time to wait for lock acquisition
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Another process holds the lock and timeout expires
    /// - Lock file cannot be created due to permissions
    /// - I/O error occurs
    pub fn acquire(file_path: &Path, timeout: Duration) -> Result<Self, FileLockError> {
        let lock_file_path = Self::lock_file_path(file_path);
        let start = Instant::now();
        let current_pid = process::id();

        loop {
            // Try to create lock file with our PID
            match Self::try_create_lock(&lock_file_path, current_pid) {
                Ok(()) => {
                    return Ok(Self {
                        lock_file_path,
                        acquired: true,
                    });
                }
                Err(FileLockError::LockHeldByProcess { pid }) => {
                    // Check if holding process is alive
                    if !Self::is_process_alive(pid) {
                        // Stale lock detected, clean it up and retry
                        let _ = fs::remove_file(&lock_file_path);
                        continue;
                    }

                    // Process is alive, check timeout
                    if start.elapsed() >= timeout {
                        return Err(FileLockError::AcquisitionTimeout {
                            path: lock_file_path,
                            holder_pid: Some(pid),
                            timeout,
                        });
                    }

                    // Wait a bit before retrying
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Release the lock by removing the lock file
    ///
    /// This is called automatically when the `FileLock` is dropped, but can
    /// also be called explicitly for better error handling.
    pub fn release(mut self) -> Result<(), FileLockError> {
        if self.acquired {
            fs::remove_file(&self.lock_file_path).map_err(|source| {
                FileLockError::ReleaseFailed {
                    path: self.lock_file_path.clone(),
                    source,
                }
            })?;
            self.acquired = false;
        }
        Ok(())
    }

    /// Get the lock file path for a given state file path
    fn lock_file_path(file_path: &Path) -> PathBuf {
        let mut lock_path = file_path.to_path_buf();
        lock_path.set_extension("json.lock");
        lock_path
    }

    /// Try to create lock file atomically with current process ID
    fn try_create_lock(lock_path: &Path, pid: u32) -> Result<(), FileLockError> {
        use std::fs::OpenOptions;
        use std::io::Write;

        // Try to create the file exclusively (fails if exists)
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(mut file) => {
                // Write our PID to the lock file
                write!(file, "{}", pid).map_err(|source| FileLockError::CreateFailed {
                    path: lock_path.to_path_buf(),
                    source,
                })?;
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Lock file exists, read the holder PID
                let content = fs::read_to_string(lock_path).map_err(|source| {
                    FileLockError::ReadFailed {
                        path: lock_path.to_path_buf(),
                        source,
                    }
                })?;

                let holder_pid = content.trim().parse::<u32>().map_err(|_| {
                    FileLockError::InvalidLockFile {
                        path: lock_path.to_path_buf(),
                        content,
                    }
                })?;

                Err(FileLockError::LockHeldByProcess { pid: holder_pid })
            }
            Err(source) => Err(FileLockError::CreateFailed {
                path: lock_path.to_path_buf(),
                source,
            }),
        }
    }

    /// Check if a process with the given PID is currently running
    #[cfg(unix)]
    fn is_process_alive(pid: u32) -> bool {
        use std::os::unix::process::ExitStatusExt;

        // On Unix, we can send signal 0 to check if process exists
        // This doesn't actually send a signal, just checks permissions
        match std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    #[cfg(windows)]
    fn is_process_alive(pid: u32) -> bool {
        // On Windows, try to open the process handle
        // This is a simplified check; production code might use Windows APIs
        std::process::Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if self.acquired {
            // Best effort cleanup, ignore errors on drop
            let _ = fs::remove_file(&self.lock_file_path);
        }
    }
}

/// Errors related to file locking operations
#[derive(Debug, Error)]
pub enum FileLockError {
    /// Lock is held by another process
    #[error("Lock held by process {pid}")]
    LockHeldByProcess { pid: u32 },

    /// Failed to acquire lock within timeout period
    #[error("Failed to acquire lock for {path:?} within {timeout:?} (held by process {holder_pid:?})")]
    AcquisitionTimeout {
        path: PathBuf,
        holder_pid: Option<u32>,
        timeout: Duration,
    },

    /// Failed to create lock file
    #[error("Failed to create lock file: {path:?}")]
    CreateFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to read lock file
    #[error("Failed to read lock file: {path:?}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Lock file contains invalid content
    #[error("Invalid lock file content at {path:?}: {content}")]
    InvalidLockFile { path: PathBuf, content: String },

    /// Failed to release lock
    #[error("Failed to release lock file: {path:?}")]
    ReleaseFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
````

**Tests to Add**:

- Test successful lock acquisition
- Test lock prevents concurrent access
- Test stale lock detection and cleanup
- Test lock timeout behavior
- Test lock release (explicit and on drop)
- Test process alive detection (mock or integration test)
- Test lock file contains correct PID
- Test concurrent lock attempts from multiple threads
- Test lock file cleanup on normal exit
- Test lock file remains on crash (simulated)

**Success Criteria**:

- ✅ `FileLock` prevents concurrent access
- ✅ Stale locks are detected and cleaned up
- ✅ Process ID is correctly written and read
- ✅ Timeout works as expected
- ✅ Automatic cleanup on drop works
- ✅ All linters pass
- ✅ All tests pass (including concurrent scenarios)

**Commit**: `feat: implement file locking with process ID tracking and stale lock detection`

**Status**: ⏳ Not started

---

### Subtask 3: Implement JSON File Repository ⏳

**Purpose**: Implement the `EnvironmentRepository` trait using JSON files with atomic writes and file locking. Implementation-specific errors (file paths, locks, I/O) will be wrapped in `RepositoryError::Internal(anyhow::Error)`.

**Changes**:

- Create new file `src/infrastructure/repository/json_file_repository.rs`
- Implement `JsonFileRepository` struct
- Implement `EnvironmentRepository` trait for `JsonFileRepository`
- Use atomic write pattern (write to temp file, then rename)
- Integrate `FileLock` for all operations
- Convert implementation-specific errors to `RepositoryError::Conflict` or `RepositoryError::Internal`
- Add comprehensive tests

**Implementation Details**:

````rust
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json;
use anyhow::Context;

use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::state::AnyEnvironmentState;
use crate::domain::environment::repository::{RepositoryError, EnvironmentRepository};
use crate::infrastructure::repository::file_lock::FileLock;

/// JSON file-based implementation of state repository
///
/// Persists environment state to JSON files in the following structure:
/// ```text
/// ./data/{env_name}/state.json
/// ./data/{env_name}/state.json.lock
/// ```
///
/// # Atomic Writes
///
/// Uses temp file + rename pattern for atomic writes:
/// 1. Write to `state.json.tmp`
/// 2. Fsync to ensure data is on disk
/// 3. Rename to `state.json` (atomic operation)
///
/// # Concurrency
///
/// Uses file locks to prevent concurrent access. Lock files contain the
/// process ID of the lock holder for debugging and stale lock detection.
///
/// # File Structure
///
/// State files contain the complete `AnyEnvironmentState` serialized as JSON:
/// ```json
/// {
///   "Created": {
///     "name": "my-env",
///     "instance_name": "torrust-deploy-my-env",
///     "ssh_credentials": { ... },
///     "state": {}
///   }
/// }
/// ```
pub struct JsonFileRepository {
    /// Base directory for state files (typically "./data")
    base_dir: PathBuf,
    /// Lock acquisition timeout
    lock_timeout: Duration,
}

impl JsonFileRepository {
    /// Create a new JSON file repository
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory where state files will be stored
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    ///
    /// let repo = JsonFileRepository::new(PathBuf::from("./data"));
    /// ```
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            lock_timeout: Duration::from_secs(10),
        }
    }

    /// Create repository with custom lock timeout
    pub fn with_lock_timeout(mut self, timeout: Duration) -> Self {
        self.lock_timeout = timeout;
        self
    }

    /// Get the state file path for an environment
    fn state_file_path(&self, name: &EnvironmentName) -> PathBuf {
        self.base_dir.join(name.as_str()).join("state.json")
    }

    /// Get the directory path for an environment
    fn env_dir_path(&self, name: &EnvironmentName) -> PathBuf {
        self.base_dir.join(name.as_str())
    }

    /// Ensure the environment directory exists
    fn ensure_env_dir(&self, name: &EnvironmentName) -> Result<(), RepositoryError> {
        let dir_path = self.env_dir_path(name);
        fs::create_dir_all(&dir_path).map_err(|source| {
            RepositoryError::DirectoryCreationFailed {
                path: dir_path,
                source,
            }
        })
    }

    /// Write state to file atomically using temp file + rename
    fn write_atomic(
        &self,
        state_path: &Path,
        content: &str,
    ) -> Result<(), RepositoryError> {
        // Write to temporary file
        let temp_path = state_path.with_extension("json.tmp");
        fs::write(&temp_path, content).map_err(|source| {
            RepositoryError::WriteStateFailed {
                path: temp_path.clone(),
                source,
            }
        })?;

        // Fsync to ensure data is on disk (optional but recommended)
        // Note: This requires opening the file, flushing, and syncing
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let file = fs::OpenOptions::new()
                .write(true)
                .open(&temp_path)
                .map_err(|source| RepositoryError::WriteStateFailed {
                    path: temp_path.clone(),
                    source,
                })?;
            file.sync_all().map_err(|source| {
                RepositoryError::WriteStateFailed {
                    path: temp_path.clone(),
                    source,
                }
            })?;
        }

        // Rename (atomic operation)
        fs::rename(&temp_path, state_path).map_err(|source| {
            RepositoryError::WriteStateFailed {
                path: state_path.to_path_buf(),
                source,
            }
        })
    }
}

impl EnvironmentRepository for JsonFileRepository {
    fn save(&self, env: &AnyEnvironmentState) -> Result<(), RepositoryError> {
        let env_name = env.name();

        // Ensure directory exists
        self.ensure_env_dir(env_name).map_err(|e| anyhow::Error::from(e))?;

        let state_path = self.state_file_path(env_name);

        // Acquire lock - convert lock errors to Conflict
        let _lock = FileLock::acquire(&state_path, self.lock_timeout).map_err(|e| {
            match e {
                file_lock::FileLockError::AcquisitionTimeout { holder_pid, .. } => {
                    RepositoryError::LockAcquisitionFailed {
                        path: state_path.clone(),
                        holder_pid,
                    }
                }
                _ => RepositoryError::LockAcquisitionFailed {
                    path: state_path.clone(),
                    holder_pid: None,
                },
            }
        })?;

        // Serialize to JSON
        let json_content = serde_json::to_string_pretty(env)
            .map_err(|source| RepositoryError::SerializationFailed { source })?;

        // Write atomically
        self.write_atomic(&state_path, &json_content)?;

        Ok(())
        // Lock is automatically released when _lock goes out of scope
    }

    fn load(&self, name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>, RepositoryError> {
        let state_path = self.state_file_path(name);

        // Check if file exists
        if !state_path.exists() {
            return Ok(None);
        }

        // Acquire lock for reading
        let _lock = FileLock::acquire(&state_path, self.lock_timeout).map_err(|e| {
            match e {
                file_lock::FileLockError::AcquisitionTimeout { holder_pid, .. } => {
                    RepositoryError::LockAcquisitionFailed {
                        path: state_path.clone(),
                        holder_pid,
                    }
                }
                _ => RepositoryError::LockAcquisitionFailed {
                    path: state_path.clone(),
                    holder_pid: None,
                },
            }
        })?;

        // Read file
        let content = fs::read_to_string(&state_path).map_err(|source| {
            RepositoryError::ReadStateFailed {
                path: state_path.clone(),
                source,
            }
        })?;

        // Deserialize
        let env = serde_json::from_str(&content).map_err(|source| {
            RepositoryError::DeserializationFailed {
                path: state_path,
                source,
            }
        })?;

        Ok(Some(env))
        // Lock is automatically released
    }

    fn exists(&self, name: &EnvironmentName) -> Result<bool, RepositoryError> {
        let state_path = self.state_file_path(name);
        Ok(state_path.exists())
    }

    fn delete(&self, name: &EnvironmentName) -> Result<(), RepositoryError> {
        let state_path = self.state_file_path(name);

        if !state_path.exists() {
            return Ok(()); // Already deleted, idempotent
        }

        // Acquire lock before deletion
        let _lock = FileLock::acquire(&state_path, self.lock_timeout).map_err(|e| {
            match e {
                file_lock::FileLockError::AcquisitionTimeout { holder_pid, .. } => {
                    RepositoryError::LockAcquisitionFailed {
                        path: state_path.clone(),
                        holder_pid,
                    }
                }
                _ => RepositoryError::LockAcquisitionFailed {
                    path: state_path.clone(),
                    holder_pid: None,
                },
            }
        })?;

        // Delete state file
        fs::remove_file(&state_path).map_err(|source| {
            RepositoryError::DeleteStateFailed {
                path: state_path,
                source,
            }
        })?;

        Ok(())
        // Lock is automatically released
    }
}
````

**Tests to Add**:

- Test save creates directory if not exists
- Test save writes valid JSON
- Test save is atomic (simulate crash during write)
- Test load returns None for non-existent environment
- Test load deserializes saved state correctly
- Test round-trip save/load preserves all data
- Test exists returns true/false correctly
- Test delete removes state file
- Test delete is idempotent
- Test concurrent save attempts are serialized
- Test concurrent load attempts work correctly
- Test lock prevents corruption during concurrent access
- Test repository with custom lock timeout
- Test error cases (permission denied, disk full, corrupted JSON)

**Success Criteria**:

- ✅ `EnvironmentRepository` trait fully implemented
- ✅ Atomic writes prevent partial state corruption
- ✅ File locking prevents concurrent access issues
- ✅ All state data preserved in round-trip
- ✅ Implementation-specific errors wrapped in `Internal(anyhow::Error)`
- ✅ Lock errors converted to `Conflict` variant
- ✅ Error handling is comprehensive and clear
- ✅ All linters pass
- ✅ All tests pass (including concurrent scenarios)

**Commit**: `feat: [#24] implement JSON file repository with atomic writes and locking`

**Status**: ⏳ Not started

---

## 🎯 Phase 4 Completion Criteria

When all three subtasks are complete, we should have:

- [x] `EnvironmentRepository` trait defining persistence contract with generic errors
- [x] `RepositoryError` with generic variants (NotFound, Conflict, Internal) that don't expose implementation details
- [ ] `FileLock` mechanism with process ID tracking
- [ ] Stale lock detection and automatic cleanup
- [ ] `JsonFileRepository` implementation with atomic writes
- [ ] File locking integrated into all repository operations
- [ ] Implementation-specific errors wrapped appropriately
- [ ] Comprehensive test coverage (~50 tests total)
- [ ] All existing functionality preserved
- [ ] All linters passing
- [ ] All tests passing (665+ tests total)

## 📊 Expected Test Coverage After Phase 4

- **Subtask 1**: +10 tests (trait, error types)
- **Subtask 2**: +20 tests (file locking, concurrency)
- **Subtask 3**: +20 tests (repository operations, integration)
- **Total New Tests**: ~50 tests
- **Total Project Tests**: ~655 tests

## 🔄 Integration with Previous Phases

Phase 4 builds on all previous phases:

- Uses `AnyEnvironmentState` from Phase 3 for type erasure
- Serializes/deserializes using Serde derives from Phase 3
- Benefits from Phase 2 logging (can log state persistence events)
- Operates on generic `Environment<S>` from Phase 1
- Maintains all type-safe transitions from Phase 1

## 🚀 What Comes After Phase 4

Once Phase 4 is complete, Phase 5 will integrate persistence into commands:

- Commands will save state after each transition
- Commands will load state before execution to verify preconditions
- Error states will be persisted for user visibility
- Status command will read persisted state
- Environment cleanup will delete persisted state

## 🔍 Design Decisions & Rationale

### Why Repository Pattern?

**Chosen**: Trait-based repository abstraction  
**Alternative**: Concrete implementation only

**Rationale**:

1. ✅ **Flexibility**: Easy to add database backend later
2. ✅ **Testing**: Can create in-memory implementation for tests
3. ✅ **Separation**: Domain logic separated from storage details
4. ✅ **Future-Proof**: Easy to add caching, remote storage, etc.

### Why EnvironmentRepository (Not StateRepository)?

**Chosen**: `EnvironmentRepository` trait name  
**Alternative**: `StateRepository` trait name

**Rationale**:

1. ✅ **Accuracy**: We persist the entire `Environment<S>` object, not just the state
2. ✅ **Clarity**: The name reflects what is actually stored in the repository
3. ✅ **Domain Alignment**: Matches domain terminology - environments have states, not states have environments
4. ✅ **API Intent**: Methods like `save(environment)` and `load() -> Environment` make the purpose clear

### Why Generic Error Types (Not Implementation-Specific)?

**Chosen**: Generic `RepositoryError` with `NotFound`, `Conflict`, `Internal(anyhow::Error)` variants  
**Alternative**: File-system-specific errors (e.g., `FileLocked { path, pid }`, `WriteError { path }`)

**Rationale**:

1. ✅ **Abstraction**: Trait interface doesn't expose storage implementation details
2. ✅ **Flexibility**: Works equally well for file, database, or in-memory implementations
3. ✅ **Simplicity**: Callers handle 3 generic cases, not N file-specific cases
4. ✅ **Future-Proof**: Can add new implementations without changing error API
5. ✅ **Debugging**: Advanced users can still downcast `Internal` variant to access original errors
6. ✅ **Error Handling Guide**: Aligns with project principle: "errors should be actionable without exposing implementation details"

**Example - Why this matters**:

```rust
// ❌ BAD: Exposes file-system implementation details in trait
pub enum RepositoryError {
    FileLocked { path: PathBuf, pid: u32 },  // Only makes sense for file storage
    WriteError { path: PathBuf, error: std::io::Error },  // File-specific
}

// ✅ GOOD: Generic errors work for any storage backend
pub enum RepositoryError {
    NotFound,  // Works for files, databases, memory, etc.
    Conflict,  // Generic: concurrent access detected
    Internal(#[from] anyhow::Error),  // Wraps implementation-specific details
}

// Implementation can wrap specific errors:
// File implementation: io::Error, lock acquisition → Internal
// Database implementation: sqlx::Error → Internal
// In-memory implementation: May not even need Internal variant
```

### Why Atomic Writes (Temp File + Rename)?

**Chosen**: Write to temp file, fsync, then rename  
**Alternative**: Direct file write

**Rationale**:

1. ✅ **Atomicity**: Rename is atomic on Unix and Windows
2. ✅ **Crash Safety**: Never see partial state
3. ✅ **Consistency**: State file is always valid or absent
4. ✅ **Industry Standard**: Used by databases, editors, etc.

### Why File Locking with Process IDs?

**Chosen**: Lock files with PID, stale lock detection  
**Alternative**: OS-level file locks (flock/LockFile)

**Rationale**:

1. ✅ **Portability**: Works on Unix and Windows
2. ✅ **Debugging**: Can see which process holds lock
3. ✅ **Crash Recovery**: Can detect and clean stale locks
4. ✅ **Simplicity**: No need for OS-specific locking APIs
5. ⚠️ **Trade-off**: Slightly more complex than OS locks

### Why 10-Second Default Lock Timeout?

**Chosen**: 10 seconds  
**Alternative**: Shorter (1s) or longer (30s)

**Rationale**:

1. ✅ **Balance**: Long enough for normal operations
2. ✅ **Responsive**: Short enough for good UX on errors
3. ✅ **Typical Operations**: State save/load completes in <100ms
4. ✅ **Network Filesystems**: Allows for slightly slower I/O

## 📚 Related Documentation

- [Development Principles](../../../development-principles.md) - Observability, testability, user-friendliness
- [Error Handling Guide](../../../contributing/error-handling.md) - Error handling principles and patterns
- [Phase 3 Plan](./phase-3-serialization.md) - Type erasure and serialization foundation
- [Requirements Analysis](../requirements-analysis.md) - Original requirements and Q&A

## 🛡️ Error Handling Strategy

All errors in Phase 4 follow the project's error handling principles:

### Clarity

- Error messages clearly state what went wrong
- Include file paths, process IDs, and other context
- Distinguish between expected failures and bugs

### Traceability

- All errors preserve source error chains with `#[source]`
- Log error context at appropriate tracing levels
- Include enough information to diagnose issues from logs

### Actionability

- Error messages suggest how to fix the problem
- Lock errors show which process holds the lock
- Permission errors point to permission issues
- Corruption errors suggest file deletion/recreation

### Examples

```rust
// ✅ Good: Clear, traceable, actionable
RepositoryError::LockAcquisitionFailed {
    path: "./data/my-env/state.json",
    holder_pid: Some(12345),
}
// Message: "Failed to acquire lock for state file: ./data/my-env/state.json, held by process 12345"
// Action: User can check process 12345, wait, or kill it if stale

// ✅ Good: Includes source for traceability
RepositoryError::WriteStateFailed {
    path: "./data/my-env/state.json",
    source: io::Error(PermissionDenied),
}
// Message: "Failed to write state file: ./data/my-env/state.json"
// Source: "Permission denied"
// Action: User can check directory permissions
```

## 🧪 Testing Strategy

### Unit Tests

- Test each component in isolation
- Mock file system operations where appropriate
- Test error paths and edge cases
- Verify error messages and source chains

### Integration Tests

- Test full save/load cycle with real files
- Test concurrent access scenarios with threads
- Test stale lock detection with simulated crashes
- Test atomic write behavior
- Use temporary directories for isolation

### Property Tests (Optional)

- Round-trip property: `load(save(x)) == Some(x)`
- Idempotence property: `delete(delete(x)) == delete(x)`
- Atomicity property: Concurrent operations never corrupt data

### Concurrency Tests

- Multiple threads attempting simultaneous saves
- Multiple threads reading while one writes
- Lock timeout behavior under contention
- Stale lock cleanup during concurrent access

## 📝 Implementation Notes

### File System Operations

All file system operations must be wrapped in proper error handling:

```rust
// ✅ Good
fs::create_dir_all(&dir_path).map_err(|source| {
    RepositoryError::DirectoryCreationFailed {
        path: dir_path,
        source,
    }
})?;

// ❌ Bad - loses error context
fs::create_dir_all(&dir_path)?;
```

### Lock Ordering

To prevent deadlocks, always acquire locks in consistent order:

1. Environment directory lock (if needed)
2. State file lock
3. Lock file operations

### Resource Cleanup

Use RAII pattern for automatic cleanup:

- `FileLock` drops automatically release locks
- Use `tempfile` crate if need managed temp files
- Ensure locks released even on errors (Drop impl)

## 🔒 Security Considerations

### File Permissions

- State files should be readable only by owner
- Lock files should be readable/writable only by owner
- Consider setting umask appropriately

### Process ID Spoofing

- Process ID in lock file is for debugging, not security
- Don't rely on PIDs for authentication
- Lock files provide coordination, not protection

### State File Contents

- State files may contain sensitive information (SSH keys)
- Consider encryption for production use (future enhancement)
- Document security expectations in user guide

## ⚡ Performance Considerations

### I/O Optimization

- Fsync only on writes, not reads
- Consider buffering for batch operations (future)
- Lock acquisition timeout should be configurable

### Lock Contention

- Short critical sections minimize contention
- Read operations take locks (consistent reads)
- Consider read-write locks if read-heavy (future)

### File System Load

- Atomic writes create temporary files
- Lock files create additional I/O
- Acceptable for deployment tool usage patterns
