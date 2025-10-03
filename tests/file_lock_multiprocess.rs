//! Multi-Process Integration Tests for File Locking
//!
//! These tests spawn actual child processes to verify true inter-process locking
//! behavior. They are more comprehensive than unit tests but slower to run.
//!
//! # Running These Tests
//!
//! ```bash
//! # Run all multi-process tests
//! cargo test --test file_lock_multiprocess
//!
//! # Run with verbose output
//! cargo test --test file_lock_multiprocess -- --nocapture
//! ```
//!
//! # Test Strategy
//!
//! 1. Spawn child processes that hold locks for specific durations
//! 2. Verify parent process cannot acquire while child holds lock
//! 3. Verify lock is released when child exits
//! 4. Test crash scenarios by killing child processes (Unix only)
//!
//! # Dependencies
//!
//! - Requires `lock_holder_helper` binary to be built
//! - Tests create temporary directories and lock files
//! - Platform-specific tests are marked with `#[cfg(unix)]`

use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;
use torrust_tracker_deploy::infrastructure::persistence::filesystem::file_lock::{
    FileLock, FileLockError,
};

/// Helper to spawn a child process that holds a lock for a specified duration
///
/// The child process will:
/// 1. Acquire the lock on the specified file
/// 2. Hold the lock for `duration_secs` seconds
/// 3. Release the lock and exit
///
/// # Arguments
///
/// * `lock_file` - Path to the file to lock
/// * `duration_secs` - How long to hold the lock (in seconds)
///
/// # Returns
///
/// The spawned child process handle
fn spawn_lock_holder(lock_file: &std::path::Path, duration_secs: u64) -> Child {
    Command::new(env!("CARGO_BIN_EXE_lock_holder_helper"))
        .arg(lock_file.to_str().expect("Invalid path"))
        .arg(duration_secs.to_string())
        .spawn()
        .expect("Failed to spawn child process")
}

#[test]
fn it_should_prevent_lock_acquisition_across_processes() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("cross_process.json");

    // Spawn child process that holds lock for 2 seconds
    let mut child = spawn_lock_holder(&lock_file, 2);

    // Give child time to acquire lock
    thread::sleep(Duration::from_millis(200));

    // Act: Try to acquire in parent - should fail due to timeout
    let result = FileLock::acquire(&lock_file, Duration::from_millis(500));

    // Assert: Should timeout because child holds the lock
    assert!(
        matches!(result, Err(FileLockError::AcquisitionTimeout { .. })),
        "Should timeout when child process holds lock"
    );

    // Cleanup: Wait for child to finish
    let exit_status = child.wait().expect("Failed to wait for child");
    assert!(
        exit_status.success(),
        "Child process should exit successfully"
    );
}

#[test]
fn it_should_acquire_lock_after_child_releases() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("handoff.json");

    // Spawn child that holds lock for 1 second
    let mut child = spawn_lock_holder(&lock_file, 1);

    // Give child time to acquire
    thread::sleep(Duration::from_millis(200));

    // Act: Try to acquire with 3 second timeout (child releases after 1 second)
    let result = FileLock::acquire(&lock_file, Duration::from_secs(3));

    // Assert: Should succeed after child releases lock
    assert!(
        result.is_ok(),
        "Should eventually acquire after child releases lock: {:?}",
        result.err()
    );

    // Cleanup
    let exit_status = child.wait().expect("Failed to wait for child");
    assert!(
        exit_status.success(),
        "Child process should exit successfully"
    );
}

#[test]
#[cfg(unix)] // Process killing is platform-specific
fn it_should_clean_up_stale_lock_after_process_crash() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("crash.json");

    // Spawn child that holds lock for a long time
    let mut child = spawn_lock_holder(&lock_file, 10);

    // Give child time to acquire
    thread::sleep(Duration::from_millis(200));

    // Act: Kill child process (simulating crash)
    child.kill().expect("Failed to kill child process");
    child.wait().expect("Failed to wait for child");

    // Small delay to ensure OS registers process death
    thread::sleep(Duration::from_millis(100));

    // Try to acquire lock - should succeed by cleaning stale lock
    let result = FileLock::acquire(&lock_file, Duration::from_secs(2));

    // Assert: Should clean up stale lock from crashed process
    assert!(
        result.is_ok(),
        "Should clean up stale lock from crashed process: {:?}",
        result.err()
    );
}

#[test]
fn it_should_handle_rapid_lock_handoff_between_processes() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("rapid_handoff.json");

    // Act: Spawn multiple children that quickly acquire and release
    let mut children = vec![];
    for i in 0..5 {
        let child = spawn_lock_holder(&lock_file, 1);
        // Small delay to allow sequential acquisition
        thread::sleep(Duration::from_millis(100));
        children.push((i, child));
    }

    // Wait for all children to complete
    for (i, mut child) in children {
        let exit_status = child.wait().expect("Failed to wait for child");
        assert!(exit_status.success(), "Child {i} should exit successfully");
    }

    // Assert: Should be able to acquire after all children finish
    let result = FileLock::acquire(&lock_file, Duration::from_secs(1));
    assert!(
        result.is_ok(),
        "Should acquire lock after rapid handoffs: {:?}",
        result.err()
    );
}

#[test]
fn it_should_handle_multiple_processes_competing_for_lock() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("competition.json");

    // Act: Spawn multiple children simultaneously trying to acquire
    let mut children = vec![];
    for i in 0..3 {
        let child = spawn_lock_holder(&lock_file, 2);
        children.push((i, child));
        // Start all children at roughly the same time
        thread::sleep(Duration::from_millis(10));
    }

    // Wait for all children to complete
    let mut successful = 0;
    let mut failed = 0;

    for (i, mut child) in children {
        let exit_status = child.wait().expect("Failed to wait for child");
        if exit_status.success() {
            successful += 1;
        } else {
            failed += 1;
        }
        println!("Child {i} exit status: {exit_status:?}");
    }

    // Assert: At least one should succeed, others may timeout
    assert!(
        successful >= 1,
        "At least one process should successfully acquire the lock"
    );

    println!("Competition test: {successful} successful, {failed} failed");
}

#[test]
fn it_should_allow_sequential_acquisition_by_different_processes() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("sequential.json");

    // Act & Assert: Run three sequential acquisitions
    for i in 0..3 {
        let mut child = spawn_lock_holder(&lock_file, 1);
        let exit_status = child.wait().expect("Failed to wait for child");
        assert!(
            exit_status.success(),
            "Sequential acquisition {i} should succeed"
        );
    }
}

#[test]
#[cfg(unix)] // Process states are platform-specific
fn it_should_detect_stale_locks_with_dead_process_ids() {
    use std::fs;
    use torrust_tracker_deploy::infrastructure::persistence::filesystem::process_id::ProcessId;

    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("stale_detection.json");
    let lock_file_path = FileLock::lock_file_path(&lock_file);

    // Create a stale lock file with a PID that's unlikely to exist
    let fake_pid = 999_999;
    fs::write(&lock_file_path, fake_pid.to_string()).expect("Failed to create stale lock file");

    // Verify the PID is indeed not alive
    let pid = ProcessId::from_raw(fake_pid);
    assert!(!pid.is_alive(), "Fake PID should not be alive");

    // Act: Try to acquire lock
    let result = FileLock::acquire(&lock_file, Duration::from_secs(2));

    // Assert: Should clean up stale lock and acquire successfully
    assert!(
        result.is_ok(),
        "Should clean up stale lock and acquire: {:?}",
        result.err()
    );
}

#[test]
fn it_should_handle_parent_acquiring_while_child_holds_lock() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lock_file = temp_dir.path().join("parent_child.json");

    // Spawn child that holds lock
    let mut child = spawn_lock_holder(&lock_file, 3);

    // Give child time to acquire
    thread::sleep(Duration::from_millis(200));

    // Act: Parent tries to acquire with short timeout
    let parent_result = FileLock::acquire(&lock_file, Duration::from_millis(500));

    // Assert: Parent should timeout
    assert!(
        matches!(parent_result, Err(FileLockError::AcquisitionTimeout { .. })),
        "Parent should timeout while child holds lock"
    );

    // Cleanup
    let exit_status = child.wait().expect("Failed to wait for child");
    assert!(exit_status.success(), "Child should exit successfully");

    // Verify parent can now acquire
    let parent_retry = FileLock::acquire(&lock_file, Duration::from_secs(1));
    assert!(
        parent_retry.is_ok(),
        "Parent should acquire after child releases"
    );
}

// Helper trait to get lock file path (expose for testing)
trait FileLockPathExt {
    fn lock_file_path(file_path: &std::path::Path) -> PathBuf;
}

impl FileLockPathExt for FileLock {
    fn lock_file_path(file_path: &std::path::Path) -> PathBuf {
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
}
