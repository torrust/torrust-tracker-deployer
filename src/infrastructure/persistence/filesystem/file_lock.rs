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
use tracing;

/// Interval in milliseconds between lock acquisition retry attempts
const LOCK_RETRY_INTERVAL_MS: u64 = 100;

/// Duration to sleep between lock acquisition retry attempts
const LOCK_RETRY_SLEEP: Duration = Duration::from_millis(LOCK_RETRY_INTERVAL_MS);

// --- Platform-Specific Module ---

/// Platform-specific functionality for process management
///
/// This module encapsulates platform-dependent code for checking process status.
/// It provides a unified interface while implementing platform-specific details
/// for Unix and Windows systems.
mod platform {
    use super::ProcessId;

    /// Check if a process with the given PID is currently running
    ///
    /// Uses platform-specific methods:
    /// - Unix: `kill -0` command (doesn't send signal, just checks permissions)
    /// - Windows: `tasklist` command to query running processes
    #[cfg(unix)]
    pub fn is_process_alive(pid: ProcessId) -> bool {
        // On Unix, we can send signal 0 to check if process exists
        // This doesn't actually send a signal, just checks permissions
        match std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.as_u32().to_string())
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Check if a process with the given PID is currently running
    ///
    /// Uses platform-specific methods:
    /// - Unix: `kill -0` command (doesn't send signal, just checks permissions)
    /// - Windows: `tasklist` command to query running processes
    #[cfg(windows)]
    pub fn is_process_alive(pid: ProcessId) -> bool {
        // On Windows, try to query the process
        std::process::Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid.as_u32()))
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).contains(&pid.as_u32().to_string())
            })
            .unwrap_or(false)
    }
}

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
        let current_pid = ProcessId::current();
        let retry_strategy = LockRetryStrategy::new(timeout);

        loop {
            match Self::try_acquire_once(&lock_file_path, current_pid) {
                AcquireAttemptResult::Success => {
                    return Ok(Self {
                        lock_file_path,
                        acquired: true,
                    });
                }
                AcquireAttemptResult::StaleProcess(_pid) => {
                    // Stale lock detected, clean it up and retry immediately
                    drop(fs::remove_file(&lock_file_path));
                    // Continue to next retry attempt
                }
                AcquireAttemptResult::HeldByLiveProcess(pid) => {
                    // Process is alive, check if we've timed out
                    if retry_strategy.is_expired() {
                        return Err(FileLockError::AcquisitionTimeout {
                            path: lock_file_path,
                            holder_pid: Some(pid),
                            timeout,
                        });
                    }
                    // Wait before retrying
                    LockRetryStrategy::wait();
                }
                AcquireAttemptResult::Error(e) => return Err(e),
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
            Err(e) => AcquireAttemptResult::Error(e),
        }
    }

    /// Try to create lock file atomically with current process ID
    ///
    /// Uses `create_new` flag to ensure atomic creation - the operation fails
    /// if the file already exists, preventing race conditions.
    fn try_create_lock(lock_path: &Path, pid: ProcessId) -> Result<(), FileLockError> {
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

                let holder_pid = content.trim().parse::<ProcessId>().map_err(|_| {
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
                    path = ?self.lock_file_path,
                    error = %e,
                    "Failed to remove lock file during drop"
                );
            }
            self.acquired = false;
        }
    }
}

// --- Lock Acquisition Helper Types ---

/// Process ID newtype for type safety
///
/// Wraps a u32 process ID to provide type safety and prevent accidental misuse.
/// This ensures PIDs are only used in appropriate contexts and makes the code
/// more self-documenting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessId(u32);

impl ProcessId {
    /// Get the current process ID
    #[must_use]
    pub fn current() -> Self {
        Self(process::id())
    }

    /// Create a `ProcessId` from a raw u32
    #[must_use]
    #[allow(dead_code)]
    pub fn from_raw(pid: u32) -> Self {
        Self(pid)
    }

    /// Get the raw u32 value
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Check if this process is currently alive
    #[must_use]
    pub fn is_alive(&self) -> bool {
        platform::is_process_alive(*self)
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ProcessId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

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
    /// Another process holds the lock and the timeout expired before it was released.
    /// The holder's PID is included if it could be determined.
    #[error(
        "Failed to acquire lock for {path:?} within {timeout:?} (held by process {holder_pid:?})"
    )]
    AcquisitionTimeout {
        path: PathBuf,
        holder_pid: Option<ProcessId>,
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
}
