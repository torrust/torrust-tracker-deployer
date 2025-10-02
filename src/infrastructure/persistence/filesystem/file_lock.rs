//! File Locking Mechanism with Process ID Tracking
//!
//! This module provides a portable file locking mechanism that works across Unix and Windows
//! by creating lock files containing the process ID of the lock holder. This approach enables:
//! - Detection and automatic cleanup of stale locks from crashed processes
//! - Debugging by showing which process holds a lock
//! - Timeout-based lock acquisition
//!
//! # Design
//!
//! The locking mechanism uses separate `.lock` files rather than OS-level file locks because:
//! - Process ID tracking: Can identify and clean up stale locks
//! - Portability: Works consistently on Unix and Windows
//! - Debuggability: Lock holder can be identified by reading the lock file
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::path::Path;
//! use std::time::Duration;
//! use torrust_tracker_deploy::infrastructure::persistence::filesystem::file_lock::FileLock;
//!
//! let state_file = Path::new("./data/test-env/state.json");
//! let lock = FileLock::acquire(state_file, Duration::from_secs(5))?;
//!
//! // Perform file operations...
//! // Lock is automatically released when dropped
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Interval in milliseconds between lock acquisition retry attempts
const LOCK_RETRY_INTERVAL_MS: u64 = 100;

/// Duration to sleep between lock acquisition retry attempts
const LOCK_RETRY_SLEEP: Duration = Duration::from_millis(LOCK_RETRY_INTERVAL_MS);

/// File locking mechanism with process ID tracking
///
/// Provides exclusive access to files by creating lock files that contain
/// the process ID of the lock holder. This prevents race conditions when multiple
/// processes attempt to access the same file concurrently.
///
/// # Lock Files
///
/// Lock files are named `{file}.lock` and contain the process ID as text.
/// Example: `./data/my-env/state.json.lock` contains "12345"
///
/// # Stale Lock Detection
///
/// If a process crashes while holding a lock, the lock file remains but the
/// process is dead. This implementation detects stale locks by checking if
/// the process ID in the lock file is still running, then automatically cleans
/// up and retries.
///
/// # RAII Pattern
///
/// The lock is automatically released when the `FileLock` is dropped, ensuring
/// cleanup even if an error occurs during file operations.
#[derive(Debug)]
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
    /// * `file_path` - Path to the file to lock (the actual file, not the lock file)
    /// * `timeout` - Maximum time to wait for lock acquisition
    ///
    /// # Returns
    ///
    /// Returns `FileLock` on successful acquisition, which will automatically release
    /// the lock when dropped.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Another process holds the lock and timeout expires (`AcquisitionTimeout`)
    /// - Lock file cannot be created due to permissions (`CreateFailed`)
    /// - I/O error occurs during lock operations
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use std::time::Duration;
    /// use torrust_tracker_deploy::infrastructure::persistence::filesystem::file_lock::FileLock;
    ///
    /// let file_path = Path::new("./data/test/state.json");
    /// let timeout = Duration::from_secs(10);
    ///
    /// match FileLock::acquire(file_path, timeout) {
    ///     Ok(lock) => {
    ///         // Perform operations on the file
    ///         // Lock automatically released when lock goes out of scope
    ///     }
    ///     Err(e) => eprintln!("Failed to acquire lock: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
                        drop(fs::remove_file(&lock_file_path));
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

                    // Wait before retrying
                    std::thread::sleep(LOCK_RETRY_SLEEP);
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Release the lock by removing the lock file
    ///
    /// This is called automatically when the `FileLock` is dropped, but can
    /// also be called explicitly for better error handling.
    ///
    /// # Errors
    ///
    /// Returns error if the lock file cannot be removed due to I/O issues.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use std::time::Duration;
    /// use torrust_tracker_deploy::infrastructure::persistence::filesystem::file_lock::FileLock;
    ///
    /// let lock = FileLock::acquire(Path::new("test.json"), Duration::from_secs(5))?;
    /// // ... perform operations ...
    /// lock.release()?; // Explicit release with error handling
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Get the lock file path for a given file path
    ///
    /// Appends `.lock` to the file path. For example:
    /// - `state.json` → `state.json.lock`
    /// - `data/env/state.json` → `data/env/state.json.lock`
    fn lock_file_path(file_path: &Path) -> PathBuf {
        let mut lock_path = file_path.to_path_buf();
        let current_extension = lock_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let new_extension = if current_extension.is_empty() {
            "lock".to_string()
        } else {
            format!("{current_extension}.lock")
        };
        lock_path.set_extension(new_extension);
        lock_path
    }

    /// Try to create lock file atomically with current process ID
    ///
    /// Uses `create_new` flag to ensure atomic creation - the operation fails
    /// if the file already exists, preventing race conditions.
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
                write!(file, "{pid}").map_err(|source| FileLockError::CreateFailed {
                    path: lock_path.to_path_buf(),
                    source,
                })?;
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Lock file exists, read the holder PID
                let content =
                    fs::read_to_string(lock_path).map_err(|source| FileLockError::ReadFailed {
                        path: lock_path.to_path_buf(),
                        source,
                    })?;

                let holder_pid =
                    content
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| FileLockError::InvalidLockFile {
                            path: lock_path.to_path_buf(),
                            content,
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
    ///
    /// Uses platform-specific methods to check process existence:
    /// - Unix: `kill -0` command (doesn't actually send a signal)
    /// - Windows: `tasklist` command to query running processes
    #[cfg(unix)]
    fn is_process_alive(pid: u32) -> bool {
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
        // On Windows, try to query the process
        std::process::Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {pid}"))
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
}

impl Drop for FileLock {
    /// Automatically release the lock when the `FileLock` is dropped
    ///
    /// This ensures cleanup even if an error occurs during file operations.
    /// Errors during cleanup are ignored as this is best-effort cleanup.
    fn drop(&mut self) {
        if self.acquired {
            // Best effort cleanup, ignore errors on drop
            drop(fs::remove_file(&self.lock_file_path));
        }
    }
}

/// Errors related to file locking operations
#[derive(Debug, Error)]
pub enum FileLockError {
    /// Lock is held by another process
    ///
    /// This is an internal error used during lock acquisition retries.
    /// Users typically see `AcquisitionTimeout` instead.
    #[error("Lock held by process {pid}")]
    LockHeldByProcess { pid: u32 },

    /// Failed to acquire lock within timeout period
    ///
    /// Another process holds the lock and the timeout expired before it was released.
    /// The holder's PID is included if it could be determined.
    #[error(
        "Failed to acquire lock for {path:?} within {timeout:?} (held by process {holder_pid:?})"
    )]
    AcquisitionTimeout {
        path: PathBuf,
        holder_pid: Option<u32>,
        timeout: Duration,
    },

    /// Failed to create lock file
    ///
    /// Common causes:
    /// - Insufficient permissions to create file
    /// - Parent directory doesn't exist
    /// - Disk full or I/O error
    #[error("Failed to create lock file: {path:?}")]
    CreateFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to read lock file
    ///
    /// Occurs when a lock file exists but cannot be read to determine the holder PID.
    /// May indicate file system issues or permission problems.
    #[error("Failed to read lock file: {path:?}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Lock file contains invalid content
    ///
    /// The lock file exists but doesn't contain a valid process ID.
    /// This may indicate corruption or manual file creation.
    #[error("Invalid lock file content at {path:?}: {content}")]
    InvalidLockFile { path: PathBuf, content: String },

    /// Failed to release lock
    ///
    /// Occurs when the lock file cannot be removed during explicit release.
    /// Note: Errors during Drop are silently ignored.
    #[error("Failed to release lock file: {path:?}")]
    ReleaseFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::error::Error;
    use std::fs;
    use std::thread;
    use tempfile::TempDir;

    // Test helper to create a temp file path
    fn create_temp_file_path(temp_dir: &TempDir, name: &str) -> PathBuf {
        temp_dir.path().join(name)
    }

    #[test]
    fn it_should_successfully_acquire_lock() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "test.json");
        let timeout = Duration::from_secs(1);

        // Act
        let lock = FileLock::acquire(&file_path, timeout);

        // Assert
        assert!(lock.is_ok());
        let lock = lock.unwrap();
        assert!(lock.acquired);

        // Verify lock file exists
        let lock_file_path = FileLock::lock_file_path(&file_path);
        assert!(lock_file_path.exists());

        // Verify lock file contains our PID
        let pid_content = fs::read_to_string(&lock_file_path).unwrap();
        let expected_pid = process::id().to_string();
        assert_eq!(pid_content, expected_pid);
    }

    #[test]
    fn it_should_prevent_concurrent_lock_acquisition() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "concurrent.json");
        let timeout = Duration::from_millis(500);

        // Act: First lock succeeds
        let _lock1 = FileLock::acquire(&file_path, timeout).unwrap();

        // Act: Second lock fails immediately (timeout < retry interval)
        let lock2_result = FileLock::acquire(&file_path, Duration::from_millis(50));

        // Assert
        assert!(lock2_result.is_err());
        match lock2_result.unwrap_err() {
            FileLockError::AcquisitionTimeout { holder_pid, .. } => {
                assert_eq!(holder_pid, Some(process::id()));
            }
            other => panic!("Expected AcquisitionTimeout, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_release_lock_explicitly() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "explicit_release.json");
        let lock_file_path = FileLock::lock_file_path(&file_path);

        // Act: Acquire and explicitly release
        let lock = FileLock::acquire(&file_path, Duration::from_secs(1)).unwrap();
        assert!(lock_file_path.exists());

        let release_result = lock.release();

        // Assert
        assert!(release_result.is_ok());
        assert!(!lock_file_path.exists());

        // Verify we can acquire again
        let lock2 = FileLock::acquire(&file_path, Duration::from_secs(1));
        assert!(lock2.is_ok());
    }

    #[test]
    fn it_should_release_lock_on_drop() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "drop_release.json");
        let lock_file_path = FileLock::lock_file_path(&file_path);

        // Act: Acquire lock in inner scope
        {
            let _lock = FileLock::acquire(&file_path, Duration::from_secs(1)).unwrap();
            assert!(lock_file_path.exists());
        } // Lock dropped here

        // Assert: Lock file should be removed
        assert!(!lock_file_path.exists());

        // Verify we can acquire again
        let lock2 = FileLock::acquire(&file_path, Duration::from_secs(1));
        assert!(lock2.is_ok());
    }

    #[test]
    fn it_should_timeout_when_lock_held_by_another_process() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "timeout.json");
        let short_timeout = Duration::from_millis(200);

        // Act: Hold lock in first thread
        let _lock1 = FileLock::acquire(&file_path, Duration::from_secs(5)).unwrap();

        // Try to acquire in same process (simulates another process)
        let lock2_result = FileLock::acquire(&file_path, short_timeout);

        // Assert
        assert!(lock2_result.is_err());
        match lock2_result.unwrap_err() {
            FileLockError::AcquisitionTimeout { timeout, .. } => {
                // Verify timeout value is approximately what we set
                assert!(timeout <= short_timeout + Duration::from_millis(50));
            }
            other => panic!("Expected AcquisitionTimeout, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_clean_up_stale_lock_with_invalid_pid() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "stale.json");
        let lock_file_path = FileLock::lock_file_path(&file_path);

        // Create a lock file with a (hopefully) non-existent PID
        let fake_pid = 999_999_u32;
        fs::write(&lock_file_path, fake_pid.to_string()).unwrap();

        // Act: Try to acquire lock - should detect stale lock and succeed
        let lock_result = FileLock::acquire(&file_path, Duration::from_secs(1));

        // Assert: Should succeed by cleaning up stale lock
        assert!(lock_result.is_ok());

        // Verify new lock file has our PID
        let pid_content = fs::read_to_string(&lock_file_path).unwrap();
        assert_eq!(pid_content, process::id().to_string());
    }

    #[test]
    fn it_should_handle_invalid_lock_file_content() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "invalid.json");
        let lock_file_path = FileLock::lock_file_path(&file_path);

        // Create lock file with invalid content
        fs::write(&lock_file_path, "not-a-number").unwrap();

        // Act
        let lock_result = FileLock::acquire(&file_path, Duration::from_millis(100));

        // Assert
        assert!(lock_result.is_err());
        match lock_result.unwrap_err() {
            FileLockError::InvalidLockFile { content, .. } => {
                assert_eq!(content, "not-a-number");
            }
            other => panic!("Expected InvalidLockFile, got: {other:?}"),
        }
    }

    #[rstest]
    #[case("test.json", "test.json.lock")]
    #[case("data/state.json", "data/state.json.lock")]
    #[case("/abs/path/file.txt", "/abs/path/file.txt.lock")]
    #[case("no_extension", "no_extension.lock")]
    fn it_should_generate_correct_lock_file_path(#[case] input: &str, #[case] expected: &str) {
        let input_path = Path::new(input);
        let lock_path = FileLock::lock_file_path(input_path);
        assert_eq!(lock_path.to_string_lossy(), expected);
    }

    #[test]
    fn it_should_allow_sequential_locks_by_same_process() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "sequential.json");

        // Act & Assert: Acquire, release, acquire again
        let lock1 = FileLock::acquire(&file_path, Duration::from_secs(1)).unwrap();
        drop(lock1); // Release

        let lock2 = FileLock::acquire(&file_path, Duration::from_secs(1));
        assert!(lock2.is_ok());
    }

    #[test]
    fn it_should_handle_concurrent_acquisitions_with_threads() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "thread_test.json");
        let file_path_clone = file_path.clone();

        // Act: Try to acquire lock from two threads
        let handle1 = thread::spawn(move || FileLock::acquire(&file_path, Duration::from_secs(2)));

        // Give first thread a head start
        thread::sleep(Duration::from_millis(50));

        let handle2 =
            thread::spawn(move || FileLock::acquire(&file_path_clone, Duration::from_millis(100)));

        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        // Assert: One should succeed, one should timeout
        assert!(result1.is_ok() ^ result2.is_ok());
    }

    #[test]
    fn it_should_display_error_messages_correctly() {
        let path = PathBuf::from("/test/path.json");

        // Test AcquisitionTimeout display
        let timeout_err = FileLockError::AcquisitionTimeout {
            path: path.clone(),
            holder_pid: Some(12345),
            timeout: Duration::from_secs(5),
        };
        let msg = timeout_err.to_string();
        assert!(msg.contains("Failed to acquire lock"));
        assert!(msg.contains("12345"));

        // Test LockHeldByProcess display
        let held_err = FileLockError::LockHeldByProcess { pid: 67890 };
        let msg = held_err.to_string();
        assert!(msg.contains("Lock held"));
        assert!(msg.contains("67890"));

        // Test InvalidLockFile display
        let invalid_err = FileLockError::InvalidLockFile {
            path: path.clone(),
            content: "bad-content".to_string(),
        };
        let msg = invalid_err.to_string();
        assert!(msg.contains("Invalid lock file"));
        assert!(msg.contains("bad-content"));
    }

    #[test]
    fn it_should_handle_lock_acquisition_with_retries() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file_path(&temp_dir, "retry.json");
        let file_path_clone = file_path.clone();

        // Act: Hold lock briefly then release
        let handle = thread::spawn(move || {
            let lock = FileLock::acquire(&file_path, Duration::from_secs(1)).unwrap();
            thread::sleep(Duration::from_millis(300));
            drop(lock); // Release after 300ms
        });

        // Give first thread time to acquire
        thread::sleep(Duration::from_millis(50));

        // Try to acquire with longer timeout - should succeed after retry
        let lock2_result = FileLock::acquire(&file_path_clone, Duration::from_secs(2));

        handle.join().unwrap();

        // Assert: Second lock should eventually succeed
        assert!(lock2_result.is_ok());
    }

    #[test]
    fn it_should_preserve_error_source_chain() {
        // Test that errors preserve source information
        let path = PathBuf::from("/test/path.json");
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");

        let create_failed = FileLockError::CreateFailed {
            path,
            source: io_error,
        };

        // Verify source is preserved
        assert!(create_failed.source().is_some());
    }
}
