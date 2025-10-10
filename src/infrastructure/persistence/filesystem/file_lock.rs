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
//! use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_lock::FileLock;
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
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing;

use super::process_id::ProcessId;

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
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_lock::FileLock;
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
    #[tracing::instrument(
        name = "file_lock_acquire",
        skip(file_path),
        fields(
            file = %file_path.display(),
            timeout_ms = timeout.as_millis(),
            pid = %ProcessId::current(),
        )
    )]
    pub fn acquire(file_path: &Path, timeout: Duration) -> Result<Self, FileLockError> {
        tracing::debug!("Attempting to acquire lock");

        let lock_file_path = Self::lock_file_path(file_path);
        let current_pid = ProcessId::current();
        let retry_strategy = LockRetryStrategy::new(timeout);

        tracing::trace!(
            lock_file = %lock_file_path.display(),
            "Lock file path determined"
        );

        let mut attempt = 0;
        loop {
            attempt += 1;
            tracing::trace!(attempt, "Lock acquisition attempt");

            match Self::try_acquire_once(&lock_file_path, current_pid) {
                AcquireAttemptResult::Success => {
                    tracing::debug!(attempts = attempt, "Lock acquired successfully");
                    return Ok(Self {
                        lock_file_path,
                        acquired: true,
                    });
                }
                AcquireAttemptResult::StaleProcess(pid) => {
                    tracing::warn!(
                        stale_pid = %pid,
                        attempt,
                        "Detected stale lock, cleaning up"
                    );
                    // Stale lock detected, clean it up and retry immediately
                    drop(fs::remove_file(&lock_file_path));
                    // Continue to next retry attempt
                }
                AcquireAttemptResult::TransientError => {
                    tracing::trace!(
                        attempt,
                        "Transient error during lock acquisition (likely race condition), retrying"
                    );
                    // Transient errors (like empty lock files) should be retried
                    // Wait a short time before retrying
                    LockRetryStrategy::wait();
                }
                AcquireAttemptResult::HeldByLiveProcess(pid) => {
                    tracing::trace!(
                        holder_pid = %pid,
                        attempt,
                        elapsed_ms = retry_strategy.start.elapsed().as_millis(),
                        "Lock held by live process"
                    );

                    // Process is alive, check if we've timed out
                    if retry_strategy.is_expired() {
                        tracing::warn!(
                            holder_pid = %pid,
                            attempts = attempt,
                            timeout_ms = timeout.as_millis(),
                            "Lock acquisition timeout"
                        );
                        return Err(FileLockError::AcquisitionTimeout {
                            path: lock_file_path,
                            holder_pid: Some(pid),
                            timeout,
                        });
                    }
                    // Wait before retrying
                    LockRetryStrategy::wait();
                }
                AcquireAttemptResult::Error(e) => {
                    tracing::warn!(
                        error = %e,
                        attempt,
                        "Error during lock acquisition"
                    );
                    return Err(e);
                }
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
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_lock::FileLock;
    ///
    /// let lock = FileLock::acquire(Path::new("test.json"), Duration::from_secs(5))?;
    /// // ... perform operations ...
    /// lock.release()?; // Explicit release with error handling
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[tracing::instrument(
        name = "file_lock_release",
        skip(self),
        fields(lock_file = %self.lock_file_path.display())
    )]
    pub fn release(mut self) -> Result<(), FileLockError> {
        tracing::debug!("Releasing lock");

        if self.acquired {
            fs::remove_file(&self.lock_file_path).map_err(|source| {
                tracing::warn!(error = %source, "Failed to remove lock file");
                FileLockError::ReleaseFailed {
                    path: self.lock_file_path.clone(),
                    source,
                }
            })?;
            self.acquired = false;
            tracing::debug!("Lock released successfully");
        } else {
            tracing::trace!("Lock was not acquired, nothing to release");
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

    /// Try to acquire the lock once
    ///
    /// Returns the result of a single acquisition attempt, classifying the outcome
    /// to help the retry logic make decisions
    fn try_acquire_once(lock_path: &Path, current_pid: ProcessId) -> AcquireAttemptResult {
        match Self::try_create_lock(lock_path, current_pid) {
            Ok(()) => AcquireAttemptResult::Success,
            Err(FileLockError::LockHeldByProcess { pid }) => {
                if pid.is_alive() {
                    AcquireAttemptResult::HeldByLiveProcess(pid)
                } else {
                    AcquireAttemptResult::StaleProcess(pid)
                }
            }
            Err(FileLockError::InvalidLockFile { ref content, .. }) if content.is_empty() => {
                // Empty lock file indicates a race condition during write
                // Treat as transient and retry
                AcquireAttemptResult::TransientError
            }
            Err(e) => AcquireAttemptResult::Error(e),
        }
    }

    /// Try to create lock file atomically with current process ID
    ///
    /// Uses `create_new` flag to ensure atomic creation - the operation fails
    /// if the file already exists, preventing race conditions.
    #[tracing::instrument(
        name = "file_lock_try_create",
        skip(lock_path),
        fields(lock_file = %lock_path.display(), pid = %pid)
    )]
    fn try_create_lock(lock_path: &Path, pid: ProcessId) -> Result<(), FileLockError> {
        use std::fs::OpenOptions;
        use std::io::Write;

        tracing::trace!("Attempting to create lock file");

        // Try to create the file exclusively (fails if exists)
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(mut file) => {
                tracing::trace!("Lock file created, writing PID");
                // Write our PID to the lock file
                write!(file, "{pid}").map_err(|source| {
                    tracing::warn!(error = %source, "Failed to write PID to lock file");
                    FileLockError::CreateFailed {
                        path: lock_path.to_path_buf(),
                        source,
                    }
                })?;
                // Flush to ensure PID is written to disk before other processes can read
                file.flush().map_err(|source| {
                    tracing::warn!(error = %source, "Failed to flush PID to lock file");
                    FileLockError::CreateFailed {
                        path: lock_path.to_path_buf(),
                        source,
                    }
                })?;
                tracing::debug!("Lock file created successfully");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                tracing::trace!("Lock file already exists, reading holder PID");
                // Lock file exists, read the holder PID
                let content = fs::read_to_string(lock_path).map_err(|source| {
                    tracing::warn!(error = %source, "Failed to read lock file");
                    FileLockError::ReadFailed {
                        path: lock_path.to_path_buf(),
                        source,
                    }
                })?;

                let holder_pid = content.trim().parse::<ProcessId>().map_err(|_| {
                    tracing::warn!(content = %content, "Invalid PID content in lock file");
                    FileLockError::InvalidLockFile {
                        path: lock_path.to_path_buf(),
                        content,
                    }
                })?;

                tracing::trace!(holder_pid = %holder_pid, "Lock held by process");
                Err(FileLockError::LockHeldByProcess { pid: holder_pid })
            }
            Err(source) => {
                tracing::warn!(error = %source, "Failed to create lock file");
                Err(FileLockError::CreateFailed {
                    path: lock_path.to_path_buf(),
                    source,
                })
            }
        }
    }

    /// Get the current state of lock acquisition (test helper)
    ///
    /// This method checks the lock state without actually acquiring the lock or waiting.
    /// It's primarily for testing to verify lock states in specific scenarios.
    ///
    /// # Note
    ///
    /// If the lock is available (Acquired state), this method briefly creates and
    /// then immediately removes the lock file to verify availability.
    #[cfg(test)]
    #[must_use]
    pub fn check_lock_state(file_path: &Path) -> LockAcquisitionState {
        let lock_path = Self::lock_file_path(file_path);
        let current_pid = ProcessId::current();

        match Self::try_create_lock(&lock_path, current_pid) {
            Ok(()) => {
                // Clean up the lock file we just created for testing
                drop(fs::remove_file(&lock_path));
                LockAcquisitionState::Acquired
            }
            Err(FileLockError::LockHeldByProcess { pid }) => {
                if pid.is_alive() {
                    LockAcquisitionState::Blocked(pid)
                } else {
                    LockAcquisitionState::FoundStaleLock(pid)
                }
            }
            Err(_) => LockAcquisitionState::Attempting,
        }
    }
}

impl Drop for FileLock {
    /// Automatically release the lock when the `FileLock` is dropped
    ///
    /// This ensures cleanup even if an error occurs during file operations.
    /// Errors during cleanup are logged but otherwise ignored as this is best-effort cleanup.
    fn drop(&mut self) {
        if self.acquired {
            // Best effort cleanup, log errors for observability
            if let Err(e) = fs::remove_file(&self.lock_file_path) {
                tracing::warn!(
                    lock_file = %self.lock_file_path.display(),
                    error = %e,
                    "Failed to remove lock file during drop"
                );
            } else {
                tracing::trace!(
                    lock_file = %self.lock_file_path.display(),
                    "Lock file removed successfully during drop"
                );
            }
            self.acquired = false;
        }
    }
}

// --- Lock Acquisition Helper Types ---

/// Represents the result of attempting to acquire a lock
///
/// This internal enum helps separate different failure modes during lock acquisition
enum AcquireAttemptResult {
    /// Lock was successfully acquired
    Success,
    /// Lock is held by a dead process (stale lock)
    StaleProcess(ProcessId),
    /// Lock is held by a live process
    HeldByLiveProcess(ProcessId),
    /// Transient error that should be retried (e.g., empty lock file during write race)
    TransientError,
    /// I/O or other error occurred
    Error(FileLockError),
}

/// Represents the state of lock acquisition process
///
/// This enum makes the lock acquisition state machine explicit and testable.
/// It's used primarily in tests to verify lock states without actually
/// acquiring locks or waiting for timeouts.
#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub enum LockAcquisitionState {
    /// Lock acquisition is being attempted
    Attempting,
    /// Found a lock held by a dead process (stale lock)
    FoundStaleLock(ProcessId),
    /// Lock is held by a live process
    Blocked(ProcessId),
    /// Lock was successfully acquired (or would be acquired)
    Acquired,
}

/// Manages retry logic for lock acquisition
///
/// Encapsulates timeout tracking and retry timing to keep the acquire logic clean
struct LockRetryStrategy {
    start: Instant,
    timeout: Duration,
}

impl LockRetryStrategy {
    /// Create a new retry strategy with the given timeout
    fn new(timeout: Duration) -> Self {
        Self {
            start: Instant::now(),
            timeout,
        }
    }

    /// Check if the timeout has expired
    fn is_expired(&self) -> bool {
        self.start.elapsed() >= self.timeout
    }

    /// Sleep before the next retry attempt
    fn wait() {
        std::thread::sleep(LOCK_RETRY_SLEEP);
    }
}

// --- Error Types ---

/// Errors related to file locking operations
#[derive(Debug, Error)]
pub enum FileLockError {
    /// Lock is held by another process
    ///
    /// This is an internal error used during lock acquisition retries.
    /// Users typically see `AcquisitionTimeout` instead.
    #[error("Lock held by process {pid}")]
    LockHeldByProcess { pid: ProcessId },

    /// Failed to acquire lock within timeout period
    ///
    /// This typically means another process is holding the lock.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to acquire lock for '{path}' within {timeout:?} (held by process {holder_pid:?})
Tip: Use 'ps -p {holder_pid:?}' to check if process is running"
    )]
    AcquisitionTimeout {
        path: PathBuf,
        holder_pid: Option<ProcessId>,
        timeout: Duration,
    },

    /// Failed to create lock file
    ///
    /// This usually indicates permission issues or file system problems.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to create lock file at '{path}': {source}
Tip: Check directory permissions and disk space"
    )]
    CreateFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to read lock file content
    ///
    /// This may indicate file system corruption or permission changes.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to read lock file at '{path}': {source}
Tip: Check file permissions and file system status"
    )]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Lock file contains invalid content
    ///
    /// Expected a process ID but found something else.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Invalid lock file content at '{path}': expected PID, found '{content}'
Tip: Remove the invalid lock file and let the system recreate it"
    )]
    InvalidLockFile { path: PathBuf, content: String },

    /// Failed to release lock file during cleanup
    ///
    /// This is usually not critical but the lock file may persist.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to release lock file at '{path}': {source}
Tip: The lock file may need manual cleanup"
    )]
    ReleaseFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl FileLockError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use std::time::Duration;
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_lock::FileLock;
    ///
    /// if let Err(e) = FileLock::acquire(Path::new("test.json"), Duration::from_secs(5)) {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # Ok::<(), ()>(())
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn help(&self) -> &'static str {
        match self {
            Self::AcquisitionTimeout { .. } => {
                "Lock Acquisition Timeout - Detailed Troubleshooting:

1. Check if the holder process is still running:
   Unix/Linux/macOS: ps -p <pid>
   Windows: tasklist /FI \"PID eq <pid>\"

2. If the process is running and should release the lock:
   - Wait for the process to complete its operation
   - Or increase the timeout duration in your configuration

3. If the process is stuck or hung:
   - Try graceful termination: kill <pid>  (Unix) or taskkill /PID <pid> (Windows)
   - Force terminate if needed: kill -9 <pid>  (Unix) or taskkill /F /PID <pid> (Windows)

4. If the process doesn't exist (stale lock):
   - This should be handled automatically by the lock system
   - If you see this error repeatedly, it indicates a bug
   - Please report at: https://github.com/torrust/torrust-tracker-deployer/issues

For more information, see the documentation on file locking."
            }

            Self::CreateFailed { .. } => {
                "Lock Creation Failed - Detailed Troubleshooting:

1. Check directory permissions:
   Unix: ls -la <directory>
   Windows: icacls <directory>
   - Ensure write access: chmod u+w <directory>  (Unix)

2. Verify parent directory exists:
   - Create if needed: mkdir -p <directory>  (Unix/Linux/macOS)
   - Create if needed: mkdir <directory>  (Windows)

3. Check available disk space:
   Unix: df -h
   Windows: wmic logicaldisk get size,freespace,caption
   - Free up space or use a different location if disk is full

4. Check for file system issues:
   - Run file system checks if problems persist
   - Try using a different directory
   - Check system logs for file system errors

If the problem persists, report it with system details."
            }

            Self::ReadFailed { .. } => {
                "Lock File Read Failed - Detailed Troubleshooting:

This error may indicate:
1. File system corruption
2. Permission changes after lock creation
3. Concurrent file deletion by another process

Troubleshooting steps:
1. Check if the lock file still exists:
   Unix: ls -la <path>.lock
   Windows: dir <path>.lock

2. Check file permissions:
   Unix: stat <path>.lock
   Windows: icacls <path>.lock

3. Check file system status:
   Unix: df -h && dmesg | tail
   Windows: chkdsk

4. If the error persists:
   - The lock file may be corrupted
   - You can manually remove it: rm <path>.lock  (Unix) or del <path>.lock  (Windows)
   - Let the system recreate it on next lock acquisition

Report persistent issues with full error context."
            }

            Self::InvalidLockFile { .. } => {
                "Invalid Lock File Content - Detailed Troubleshooting:

The lock file should contain only a process ID (numeric value).
This error indicates the file contains invalid content.

Common causes:
1. Manual modification of lock file (not recommended)
2. File system corruption
3. Lock file created by incompatible software
4. Encoding issues

Resolution steps:
1. Remove the invalid lock file:
   Unix: rm <path>.lock
   Windows: del <path>.lock

2. Let the system recreate it properly on next lock acquisition

3. Ensure no external tools or scripts are modifying .lock files

4. If using shared storage (NFS, CIFS, etc.):
   - Check for file system compatibility issues
   - Verify proper file locking support

Prevention:
- Never manually edit .lock files
- Ensure proper file system support for atomic operations
- Use appropriate locking mechanisms for shared storage

Report if this error occurs without manual intervention."
            }

            Self::ReleaseFailed { .. } => {
                "Lock Release Failed - Detailed Troubleshooting:

This is a cleanup error that occurs when removing the lock file.
It typically doesn't affect functionality, but the lock file may persist.

Common causes:
1. File was already deleted (race condition with another process)
2. Permissions changed after lock creation
3. File system issue during cleanup
4. File is open by another process

Steps to resolve:
1. Check if the lock file still exists:
   Unix: ls -la <path>.lock
   Windows: dir <path>.lock

2. If it exists and causes issues, manually remove it:
   Unix: rm <path>.lock
   Windows: del <path>.lock

3. Verify no other processes have the file open:
   Unix: lsof <path>.lock
   Windows: handle.exe <path>.lock  (requires Sysinternals)

Impact:
- This error usually doesn't affect the current operation
- The lock was already released from the application perspective
- Stale lock files will be cleaned up on next acquisition

Only report if this error occurs frequently or causes operational issues."
            }

            Self::LockHeldByProcess { .. } => {
                "This is an internal error used during lock acquisition.
If you see this error directly, it may indicate a logic error in the application.
Please report it with full context."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! # File Lock Test Suite
    //!
    //! Comprehensive tests for the file locking mechanism, organized into logical modules.
    //!
    //! ## 📋 Test Organization
    //!
    //! - **`basic_operations`**: Core lock acquisition and release functionality
    //! - **`concurrency`**: Multi-threaded scenarios and concurrent lock handling
    //! - **`stale_lock_handling`**: Detection and cleanup of stale locks from dead processes
    //! - **`timeout_behavior`**: Retry logic and timeout handling
    //! - **`error_handling`**: Error message validation and source chain preservation
    //! - **`lock_file_path_generation`**: Lock file path generation and validation
    //!
    //! ## 🛠️ Test Helpers
    //!
    //! - **`TestLockScenario`**: Builder pattern for configuring test scenarios
    //! - **`assert_lock_file_contains_current_pid`**: Verify lock file exists with correct PID
    //! - **`assert_lock_file_absent`**: Verify lock file doesn't exist

    use super::*;
    use rstest::rstest;
    use std::error::Error;
    use std::fs;
    use std::thread;
    use tempfile::TempDir;

    /// PID value that is highly unlikely to be a running process
    /// Used in tests to simulate stale locks from dead processes
    const FAKE_DEAD_PROCESS_PID: u32 = 999_999;

    /// Test helper to verify that a lock file exists and contains the current process ID
    fn assert_lock_file_contains_current_pid(file_path: &Path) {
        assert_lock_file_exists(file_path);
        assert_lock_file_contains_pid(file_path, ProcessId::current());
    }

    /// Test helper to verify that a lock file does not exist
    fn assert_lock_file_absent(file_path: &Path) {
        let lock_file_path = FileLock::lock_file_path(file_path);
        assert!(
            !lock_file_path.exists(),
            "Lock file should not exist at {lock_file_path:?}"
        );
    }

    /// Test helper to verify that a lock file exists (without checking content)
    fn assert_lock_file_exists(file_path: &Path) {
        let lock_file_path = FileLock::lock_file_path(file_path);
        assert!(
            lock_file_path.exists(),
            "Lock file should exist at {lock_file_path:?}"
        );
    }

    /// Test helper to verify that a lock file contains a specific PID
    fn assert_lock_file_contains_pid(file_path: &Path, expected_pid: ProcessId) {
        let lock_file_path = FileLock::lock_file_path(file_path);
        let pid_content =
            fs::read_to_string(&lock_file_path).expect("Should be able to read lock file");
        assert_eq!(
            pid_content.trim(),
            expected_pid.to_string(),
            "Lock file should contain PID {expected_pid}"
        );
    }

    /// Test helper to verify that lock acquisition failed with a timeout error
    fn assert_timeout_error(result: Result<FileLock, FileLockError>) {
        assert!(result.is_err(), "Expected timeout error");
        match result.unwrap_err() {
            FileLockError::AcquisitionTimeout { .. } => {}
            other => panic!("Expected AcquisitionTimeout, got: {other:?}"),
        }
    }

    /// Test helper to verify timeout error and check the holder PID
    fn assert_timeout_error_with_holder(
        result: Result<FileLock, FileLockError>,
        expected_holder: ProcessId,
    ) {
        assert!(result.is_err(), "Expected timeout error");
        match result.unwrap_err() {
            FileLockError::AcquisitionTimeout { holder_pid, .. } => {
                assert_eq!(
                    holder_pid,
                    Some(expected_holder),
                    "Expected holder PID {expected_holder}"
                );
            }
            other => panic!("Expected AcquisitionTimeout, got: {other:?}"),
        }
    }

    /// Test helper to verify invalid lock file error with expected content
    fn assert_invalid_lock_file_error(
        result: Result<FileLock, FileLockError>,
        expected_content: &str,
    ) {
        assert!(result.is_err(), "Expected invalid lock file error");
        match result.unwrap_err() {
            FileLockError::InvalidLockFile { content, .. } => {
                assert_eq!(
                    content, expected_content,
                    "Expected invalid content '{expected_content}'"
                );
            }
            other => panic!("Expected InvalidLockFile, got: {other:?}"),
        }
    }

    // ========================================================================
    // Test Builder - Builder pattern for test configuration
    // ========================================================================

    /// Builder for creating test lock scenarios with configurable parameters
    struct TestLockScenario {
        temp_dir: TempDir,
        file_name: String,
        timeout: Duration,
    }

    impl TestLockScenario {
        /// Create a new test scenario with default values
        fn new() -> Self {
            Self {
                temp_dir: TempDir::new().expect("Failed to create temporary directory for test"),
                file_name: "test.json".to_string(),
                timeout: Duration::from_secs(1),
            }
        }

        /// Set a custom file name for the lock file
        fn with_file_name(mut self, name: &str) -> Self {
            self.file_name = name.to_string();
            self
        }

        /// Set a custom timeout duration
        fn with_timeout(mut self, timeout: Duration) -> Self {
            self.timeout = timeout;
            self
        }

        /// Get the path to the file that will be locked
        fn file_path(&self) -> PathBuf {
            self.temp_dir.path().join(&self.file_name)
        }

        /// Get the path to the lock file
        fn lock_file_path(&self) -> PathBuf {
            FileLock::lock_file_path(&self.file_path())
        }

        /// Attempt to acquire a lock with the configured parameters
        fn acquire_lock(&self) -> Result<FileLock, FileLockError> {
            FileLock::acquire(&self.file_path(), self.timeout)
        }

        /// Create scenario with short timeout for failure tests (200ms)
        fn for_timeout_test() -> Self {
            Self::new().with_timeout(Duration::from_millis(200))
        }

        /// Create scenario with long timeout for success tests (5 seconds)
        fn for_success_test() -> Self {
            Self::new().with_timeout(Duration::from_secs(5))
        }

        /// Create a stale lock file with a dead process PID
        fn with_stale_lock(&self, fake_pid: u32) -> Result<(), std::io::Error> {
            fs::write(self.lock_file_path(), fake_pid.to_string())
        }

        /// Create a lock file with invalid content for error testing
        fn with_invalid_lock(&self, content: &str) -> Result<(), std::io::Error> {
            fs::write(self.lock_file_path(), content)
        }
    }

    // ========================================================================
    // Basic Operations - Core lock acquisition and release functionality
    // ========================================================================

    mod basic_operations {
        use super::*;

        #[test]
        fn it_should_successfully_acquire_lock() {
            // Arrange
            let scenario = TestLockScenario::new();

            // Act
            let lock = scenario.acquire_lock();

            // Assert
            assert!(lock.is_ok());
            let lock = lock.expect("Failed to acquire lock for basic operations test");
            assert!(lock.acquired);

            // Verify lock file exists and contains our PID
            assert_lock_file_contains_current_pid(&scenario.file_path());
        }

        #[test]
        fn it_should_release_lock_explicitly() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("explicit_release.json");

            // Act: Acquire and explicitly release
            let lock = scenario
                .acquire_lock()
                .expect("Failed to acquire lock for explicit release test");
            assert!(scenario.lock_file_path().exists());

            let release_result = lock.release();

            // Assert
            assert!(release_result.is_ok());
            assert!(!scenario.lock_file_path().exists());

            // Verify we can acquire again
            let lock2 = scenario.acquire_lock();
            assert!(lock2.is_ok());
        }

        #[test]
        fn it_should_release_lock_on_drop() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("drop_release.json");

            // Act: Acquire lock in inner scope
            {
                let _lock = scenario
                    .acquire_lock()
                    .expect("Failed to acquire lock for drop release test");
                assert!(scenario.lock_file_path().exists());
            } // Lock dropped here

            // Assert: Lock file should be removed
            assert_lock_file_absent(&scenario.file_path());

            // Verify we can acquire again
            let lock2 = scenario.acquire_lock();
            assert!(lock2.is_ok());
        }

        #[test]
        fn it_should_allow_sequential_locks_by_same_process() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("sequential.json");

            // Act & Assert: Acquire, release, acquire again
            let lock1 = scenario
                .acquire_lock()
                .expect("Failed to acquire first lock for sequential test");
            drop(lock1); // Release

            let lock2 = scenario.acquire_lock();
            assert!(lock2.is_ok());
        }
    }

    // ========================================================================
    // Concurrency - Tests for concurrent lock acquisition scenarios
    // ========================================================================

    mod concurrency {
        use super::*;

        #[test]
        fn it_should_prevent_concurrent_lock_acquisition() {
            // Arrange
            let scenario = TestLockScenario::new()
                .with_file_name("concurrent.json")
                .with_timeout(Duration::from_millis(500));

            // Act: First lock succeeds
            let _lock1 = scenario
                .acquire_lock()
                .expect("Failed to acquire first lock for concurrency test");

            // Act: Second lock fails immediately (timeout < retry interval)
            let lock2_result = FileLock::acquire(&scenario.file_path(), Duration::from_millis(50));

            // Assert
            assert_timeout_error_with_holder(lock2_result, ProcessId::current());
        }

        #[test]
        fn it_should_handle_concurrent_acquisitions_with_threads() {
            // Arrange
            let scenario = TestLockScenario::for_success_test().with_file_name("thread_test.json");
            let file_path = scenario.file_path();
            let file_path_clone = file_path.clone();

            // Act: Try to acquire lock from two threads
            let handle1 =
                thread::spawn(move || FileLock::acquire(&file_path, Duration::from_secs(2)));

            // Give first thread a head start
            thread::sleep(Duration::from_millis(50));

            let handle2 = thread::spawn(move || {
                FileLock::acquire(&file_path_clone, Duration::from_millis(100))
            });

            let result1 = handle1
                .join()
                .expect("Failed to join first thread in concurrency test");
            let result2 = handle2
                .join()
                .expect("Failed to join second thread in concurrency test");

            // Assert: One should succeed, one should timeout
            assert!(result1.is_ok() ^ result2.is_ok());
        }
    }

    // ========================================================================
    // Stale Lock Handling - Tests for cleaning up stale locks
    // ========================================================================

    mod stale_lock_handling {
        use super::*;

        #[test]
        fn it_should_clean_up_stale_lock_with_invalid_pid() {
            // Arrange
            let scenario = TestLockScenario::for_success_test().with_file_name("stale.json");
            scenario
                .with_stale_lock(FAKE_DEAD_PROCESS_PID)
                .expect("Failed to create stale lock file");

            // Act
            let lock_result = scenario.acquire_lock();

            // Assert: Should succeed by cleaning up stale lock
            assert!(lock_result.is_ok());

            // Verify new lock file has our PID
            assert_lock_file_contains_current_pid(&scenario.file_path());
        }

        #[test]
        fn it_should_handle_invalid_lock_file_content() {
            // Arrange
            let scenario = TestLockScenario::for_timeout_test().with_file_name("invalid.json");
            scenario
                .with_invalid_lock("not-a-number")
                .expect("Failed to create invalid lock file");

            // Act
            let lock_result = scenario.acquire_lock();

            // Assert
            assert_invalid_lock_file_error(lock_result, "not-a-number");
        }
    }

    // ========================================================================
    // Timeout Behavior - Tests for timeout and retry mechanisms
    // ========================================================================

    mod timeout_behavior {
        use super::*;

        #[test]
        fn it_should_timeout_when_lock_held_by_another_process() {
            // Arrange
            let scenario = TestLockScenario::for_timeout_test().with_file_name("timeout.json");
            let short_timeout = Duration::from_millis(200);

            // Act: Hold lock in first acquisition
            let _lock1 = FileLock::acquire(&scenario.file_path(), Duration::from_secs(5))
                .expect("Failed to acquire first lock for timeout test");

            // Try to acquire in same process (simulates another process)
            let lock2_result = FileLock::acquire(&scenario.file_path(), short_timeout);

            // Assert: Should timeout
            assert_timeout_error(lock2_result);
        }

        #[test]
        fn it_should_handle_lock_acquisition_with_retries() {
            // Arrange
            let scenario = TestLockScenario::for_success_test().with_file_name("retry.json");
            let file_path = scenario.file_path();
            let file_path_clone = file_path.clone();

            // Act: Hold lock briefly then release
            let handle = thread::spawn(move || {
                let lock = FileLock::acquire(&file_path, Duration::from_secs(1))
                    .expect("Failed to acquire lock in retry test thread");
                thread::sleep(Duration::from_millis(300));
                drop(lock); // Release after 300ms
            });

            // Give first thread time to acquire
            thread::sleep(Duration::from_millis(50));

            // Try to acquire with longer timeout - should succeed after retry
            let lock2_result = FileLock::acquire(&file_path_clone, Duration::from_secs(2));

            handle.join().expect("Failed to join thread in retry test");

            // Assert: Second lock should eventually succeed
            assert!(lock2_result.is_ok());
        }
    }

    // ========================================================================
    // Error Handling - Tests for error messages and error source preservation
    // ========================================================================

    mod error_handling {
        use super::*;

        #[test]
        fn it_should_include_brief_tips_in_error_messages() {
            let path = PathBuf::from("/test/path.json");

            // Test AcquisitionTimeout includes tip
            let timeout_err = FileLockError::AcquisitionTimeout {
                path: path.clone(),
                holder_pid: Some(ProcessId::from_raw(12345)),
                timeout: Duration::from_secs(5),
            };
            let msg = timeout_err.to_string();
            assert!(msg.contains("Tip:"), "Error message should contain a tip");
            assert!(
                msg.contains("ps -p"),
                "Tip should mention process check command"
            );

            // Test CreateFailed includes tip
            let io_error =
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
            let create_err = FileLockError::CreateFailed {
                path: path.clone(),
                source: io_error,
            };
            let msg = create_err.to_string();
            assert!(msg.contains("Tip:"), "Error message should contain a tip");
            assert!(
                msg.contains("permissions"),
                "Tip should mention permissions"
            );

            // Test ReadFailed includes tip
            let io_error =
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
            let read_err = FileLockError::ReadFailed {
                path: path.clone(),
                source: io_error,
            };
            let msg = read_err.to_string();
            assert!(msg.contains("Tip:"), "Error message should contain a tip");

            // Test InvalidLockFile includes tip
            let invalid_err = FileLockError::InvalidLockFile {
                path: path.clone(),
                content: "bad-content".to_string(),
            };
            let msg = invalid_err.to_string();
            assert!(msg.contains("Tip:"), "Error message should contain a tip");
            assert!(
                msg.contains("Remove"),
                "Tip should mention removing the file"
            );

            // Test ReleaseFailed includes tip
            let io_error =
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
            let release_err = FileLockError::ReleaseFailed {
                path: path.clone(),
                source: io_error,
            };
            let msg = release_err.to_string();
            assert!(msg.contains("Tip:"), "Error message should contain a tip");
        }

        #[test]
        fn it_should_provide_detailed_help_for_all_error_variants() {
            let path = PathBuf::from("/test/path.json");
            let io_error =
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");

            let test_cases = vec![
                (
                    "AcquisitionTimeout",
                    FileLockError::AcquisitionTimeout {
                        path: path.clone(),
                        holder_pid: Some(ProcessId::from_raw(12345)),
                        timeout: Duration::from_secs(5),
                    },
                ),
                (
                    "CreateFailed",
                    FileLockError::CreateFailed {
                        path: path.clone(),
                        source: io_error.kind().into(),
                    },
                ),
                (
                    "ReadFailed",
                    FileLockError::ReadFailed {
                        path: path.clone(),
                        source: io_error.kind().into(),
                    },
                ),
                (
                    "InvalidLockFile",
                    FileLockError::InvalidLockFile {
                        path: path.clone(),
                        content: "bad-content".to_string(),
                    },
                ),
                (
                    "ReleaseFailed",
                    FileLockError::ReleaseFailed {
                        path: path.clone(),
                        source: io_error.kind().into(),
                    },
                ),
                (
                    "LockHeldByProcess",
                    FileLockError::LockHeldByProcess {
                        pid: ProcessId::from_raw(12345),
                    },
                ),
            ];

            for (variant_name, error) in test_cases {
                let help = error.help();
                assert!(!help.is_empty(), "{variant_name}: Help should not be empty");
                assert!(
                    help.len() > 50,
                    "{variant_name}: Help should be detailed (at least 50 chars)"
                );
            }
        }

        #[test]
        fn it_should_include_platform_specific_commands_in_help() {
            let timeout_err = FileLockError::AcquisitionTimeout {
                path: PathBuf::from("/test/path.json"),
                holder_pid: Some(ProcessId::from_raw(12345)),
                timeout: Duration::from_secs(5),
            };

            let help = timeout_err.help();

            // Check for Unix commands
            assert!(
                help.contains("ps -p"),
                "Help should include Unix process check command"
            );
            assert!(
                help.contains("kill"),
                "Help should include Unix kill command"
            );

            // Check for Windows commands
            assert!(
                help.contains("tasklist"),
                "Help should include Windows process check command"
            );
            assert!(
                help.contains("taskkill"),
                "Help should include Windows kill command"
            );
        }

        #[test]
        fn it_should_display_error_messages_correctly() {
            let path = PathBuf::from("/test/path.json");

            // Test AcquisitionTimeout display
            let timeout_err = FileLockError::AcquisitionTimeout {
                path: path.clone(),
                holder_pid: Some(ProcessId::from_raw(12345)),
                timeout: Duration::from_secs(5),
            };
            let msg = timeout_err.to_string();
            assert!(msg.contains("Failed to acquire lock"));
            assert!(msg.contains("12345"));

            // Test LockHeldByProcess display
            let held_err = FileLockError::LockHeldByProcess {
                pid: ProcessId::from_raw(67890),
            };
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

    // ========================================================================
    // Lock File Path Generation - Tests for lock file path generation
    // ========================================================================

    mod lock_file_path_generation {
        use super::*;

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
    }

    // ========================================================================
    // Lock State Detection - Tests for lock acquisition state machine
    // ========================================================================

    mod lock_state_detection {
        use super::*;

        #[test]
        fn it_should_detect_acquired_state_when_no_lock_exists() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("state_acquired.json");

            // Act
            let state = FileLock::check_lock_state(&scenario.file_path());

            // Assert
            assert_eq!(state, LockAcquisitionState::Acquired);
        }

        #[test]
        fn it_should_detect_stale_lock_state() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("state_stale.json");
            scenario
                .with_stale_lock(FAKE_DEAD_PROCESS_PID)
                .expect("Failed to create stale lock file for state test");

            // Act
            let state = FileLock::check_lock_state(&scenario.file_path());

            // Assert
            assert_eq!(
                state,
                LockAcquisitionState::FoundStaleLock(ProcessId::from_raw(FAKE_DEAD_PROCESS_PID))
            );
        }

        #[test]
        fn it_should_detect_blocked_state_when_lock_held() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("state_blocked.json");
            let _lock = scenario
                .acquire_lock()
                .expect("Failed to acquire lock for state test");

            // Act
            let state = FileLock::check_lock_state(&scenario.file_path());

            // Assert
            assert_eq!(state, LockAcquisitionState::Blocked(ProcessId::current()));
        }

        #[test]
        fn it_should_detect_attempting_state_on_error() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("state_error.json");
            scenario
                .with_invalid_lock("invalid-pid-content")
                .expect("Failed to create invalid lock file for state test");

            // Act
            let state = FileLock::check_lock_state(&scenario.file_path());

            // Assert
            assert_eq!(state, LockAcquisitionState::Attempting);
        }
    }

    // ========================================================================
    // Tracing - Tests for observability and tracing instrumentation
    // ========================================================================

    mod tracing {
        use super::*;

        #[test]
        fn it_should_complete_lock_operations_with_tracing_enabled() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("traced.json");

            // Act: Acquire lock (tracing happens in background)
            let lock = scenario
                .acquire_lock()
                .expect("Failed to acquire lock with tracing");

            // Assert: Lock was acquired successfully
            assert_lock_file_exists(&scenario.file_path());
            assert_lock_file_contains_current_pid(&scenario.file_path());

            // Act: Release lock explicitly (tracing happens in background)
            lock.release().expect("Failed to release lock with tracing");

            // Assert: Lock was released successfully
            assert_lock_file_absent(&scenario.file_path());
        }

        #[test]
        fn it_should_trace_stale_lock_cleanup() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("stale_traced.json");
            scenario
                .with_stale_lock(FAKE_DEAD_PROCESS_PID)
                .expect("Failed to create stale lock for tracing test");

            // Act: Acquire should clean up stale lock (tracing shows cleanup)
            let lock = scenario
                .acquire_lock()
                .expect("Failed to acquire after stale lock cleanup");

            // Assert: Lock acquired successfully after cleanup
            assert_lock_file_contains_current_pid(&scenario.file_path());

            drop(lock);
        }

        #[test]
        fn it_should_trace_timeout_scenario() {
            // Arrange
            let scenario =
                TestLockScenario::for_timeout_test().with_file_name("timeout_traced.json");

            let _blocking_lock = scenario
                .acquire_lock()
                .expect("Failed to acquire blocking lock");

            // Act: Try to acquire with short timeout (tracing shows retry attempts)
            let result = FileLock::acquire(&scenario.file_path(), Duration::from_millis(200));

            // Assert: Should timeout (tracing shows all retry attempts)
            assert_timeout_error(result);
        }

        #[test]
        fn it_should_trace_invalid_lock_file_scenario() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("invalid_traced.json");
            let invalid_content = "not-a-valid-pid";
            scenario
                .with_invalid_lock(invalid_content)
                .expect("Failed to create invalid lock for tracing test");

            // Act: Try to acquire (tracing shows invalid content detection)
            let result = scenario.acquire_lock();

            // Assert: Should fail with invalid lock file error
            assert_invalid_lock_file_error(result, invalid_content);
        }

        #[test]
        fn it_should_trace_concurrent_acquisition_attempts() {
            // Arrange
            let scenario = TestLockScenario::new().with_file_name("concurrent_traced.json");

            // Act: Spawn threads that try to acquire concurrently
            let handles: Vec<_> = (0..3)
                .map(|_| {
                    let path = scenario.file_path();
                    std::thread::spawn(move || FileLock::acquire(&path, Duration::from_millis(200)))
                })
                .collect();

            // Collect results
            let results: Vec<_> = handles
                .into_iter()
                .map(|h| h.join().expect("Thread panicked"))
                .collect();

            // Assert: Exactly one should succeed, others should timeout
            let success_count = results.iter().filter(|r| r.is_ok()).count();
            assert_eq!(
                success_count, 1,
                "Exactly one thread should acquire the lock"
            );
        }
    }
}
