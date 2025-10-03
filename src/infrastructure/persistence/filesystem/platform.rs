//! Platform-specific process checking functionality
//!
//! This module provides a unified interface for checking if a process is alive
//! across different operating systems (Unix and Windows).
//!
//! # Platform Support
//!
//! - **Unix/Linux/macOS**: Uses `kill -0` signal to check process existence
//! - **Windows**: Uses `tasklist` command to query running processes
//!
//! # Design
//!
//! The implementation uses command-line tools rather than OS APIs because:
//! - Simple and portable across different Unix flavors
//! - No additional dependencies required
//! - Sufficient for our use case (not performance-critical)

use std::process::Command;

use super::process_id::ProcessId;

/// Check if a process with the given PID is currently running
///
/// # Platform Behavior
///
/// ## Unix/Linux/macOS
/// Uses `kill -0` which sends signal 0 to the process.
/// - Returns `true` if process exists and is accessible
/// - Returns `false` if process doesn't exist or permission denied
/// - Signal 0 doesn't actually send a signal, just checks permissions
///
/// ## Windows
/// Uses `tasklist /FI "PID eq {pid}"` to query process status.
/// - Returns `true` if process appears in task list
/// - Returns `false` if process doesn't exist
///
/// # Arguments
///
/// * `pid` - Process ID to check
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::infrastructure::persistence::filesystem::platform;
/// use torrust_tracker_deploy::infrastructure::persistence::filesystem::process_id::ProcessId;
///
/// let current_pid = ProcessId::current();
/// assert!(platform::is_process_alive(current_pid));
/// ```
#[cfg(unix)]
pub fn is_process_alive(pid: ProcessId) -> bool {
    // On Unix, we can send signal 0 to check if process exists
    // This doesn't actually send a signal, just checks permissions
    match Command::new("kill")
        .arg("-0")
        .arg(pid.as_u32().to_string())
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Check if a process with the given PID is currently running (Windows)
///
/// See main documentation for `is_process_alive`.
#[cfg(windows)]
pub fn is_process_alive(pid: ProcessId) -> bool {
    // On Windows, try to query the process using tasklist
    Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {}", pid.as_u32()))
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains(&pid.as_u32().to_string()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_detect_current_process_as_alive() {
        let current_pid = ProcessId::current();
        assert!(
            is_process_alive(current_pid),
            "Current process should always be detected as alive"
        );
    }

    #[test]
    fn it_should_detect_fake_process_as_dead() {
        // Use a PID that is very unlikely to exist
        let fake_pid = ProcessId::from_raw(999_999);
        assert!(
            !is_process_alive(fake_pid),
            "Fake PID 999999 should not be detected as alive"
        );
    }

    #[test]
    fn it_should_handle_pid_1_correctly() {
        // PID 1 is init/systemd on Unix, System on Windows
        // In most Unix systems it should be alive, but in some environments
        // (like containers or when running with insufficient permissions),
        // we may not be able to check it
        #[cfg(unix)]
        {
            let init_pid = ProcessId::from_raw(1);
            // Just verify we can call the function without panicking
            // The result may vary depending on permissions and environment
            let _result = is_process_alive(init_pid);
            // On traditional Unix systems, PID 1 is the init process and should be alive
            // But in containers or restricted environments, we may not have permission to check
        }

        // On Windows, PID 1 may or may not exist, so we don't test it
    }
}
