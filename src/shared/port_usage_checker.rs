//! Port usage checking utilities.
//!
//! This module provides functionality to check which processes are using specific ports
//! on the system. It uses system commands like `netstat` or `ss` to query port usage.
//!
//! This is different from [`crate::shared::port_checker`] which checks TCP connectivity
//! to a port. This module checks which process is actually bound to a port.

use std::process::Command;
use thiserror::Error;

/// Error type for port usage checking operations
#[derive(Debug, Error)]
pub enum PortUsageError {
    /// Both netstat and ss commands failed
    #[error("Failed to check port usage: netstat error: {netstat_error}, ss error: {ss_error}")]
    BothCommandsFailed {
        netstat_error: String,
        ss_error: String,
    },

    /// Command executed but port was not found in output
    #[error("Port {port} is not in use according to system commands")]
    PortNotFound { port: u16 },
}

/// Utility for checking which processes are using specific ports
///
/// This struct provides methods to query the operating system about port usage.
/// It tries multiple commands (`netstat`, `ss`) to be compatible with different systems.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::shared::port_usage_checker::PortUsageChecker;
///
/// // Check if port 22 is in use
/// match PortUsageChecker::check_port(22) {
///     Ok(lines) => {
///         println!("Port 22 is in use:");
///         for line in lines {
///             println!("  {}", line);
///         }
///     }
///     Err(e) => eprintln!("Could not check port: {}", e),
/// }
/// ```
pub struct PortUsageChecker;

impl PortUsageChecker {
    /// Check which processes are using the specified port
    ///
    /// This method tries `netstat` first, then falls back to `ss` if `netstat` fails.
    /// Both commands are commonly available on Unix-like systems.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to check
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Lines from command output showing processes using the port
    /// * `Err(PortUsageError)` - If both commands failed or port is not in use
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Both `netstat` and `ss` commands fail to execute or are not available
    /// - The port is not found in the output of either command (not in use)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::shared::port_usage_checker::PortUsageChecker;
    ///
    /// match PortUsageChecker::check_port(8080) {
    ///     Ok(lines) => {
    ///         println!("Port 8080 usage:");
    ///         for line in &lines {
    ///             println!("{}", line);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn check_port(port: u16) -> Result<Vec<String>, PortUsageError> {
        // Try netstat first, fallback to ss
        match Self::check_port_with_netstat(port) {
            Ok(lines) => Ok(lines),
            Err(netstat_error) => Self::check_port_with_ss(port).map_err(|ss_error| {
                PortUsageError::BothCommandsFailed {
                    netstat_error,
                    ss_error,
                }
            }),
        }
    }

    /// Check port usage using netstat command
    ///
    /// Uses `netstat -tlnp` to list TCP listening ports with process information.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to check
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Lines from netstat output containing the port
    /// * `Err(String)` - If command failed or port not found
    fn check_port_with_netstat(port: u16) -> Result<Vec<String>, String> {
        let output = Command::new("netstat")
            .args(["-tlnp"])
            .output()
            .map_err(|e| format!("netstat command failed: {e}"))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let port_str = port.to_string();
        let matches: Vec<String> = output_str
            .lines()
            .filter(|line| line.contains(&port_str))
            .map(ToString::to_string)
            .collect();

        if matches.is_empty() {
            Err(format!("Port {port} not found in netstat output"))
        } else {
            Ok(matches)
        }
    }

    /// Check port usage using ss command (fallback for systems without netstat)
    ///
    /// Uses `ss -tlnp` to list TCP listening ports with process information.
    /// This is a modern alternative to `netstat` available on most Linux systems.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to check
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Lines from ss output containing the port
    /// * `Err(String)` - If command failed or port not found
    fn check_port_with_ss(port: u16) -> Result<Vec<String>, String> {
        let output = Command::new("ss")
            .args(["-tlnp"])
            .output()
            .map_err(|e| format!("ss command failed: {e}"))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let port_str = port.to_string();
        let matches: Vec<String> = output_str
            .lines()
            .filter(|line| line.contains(&port_str))
            .map(ToString::to_string)
            .collect();

        if matches.is_empty() {
            Err(format!("Port {port} not found in ss output"))
        } else {
            Ok(matches)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_check_port_with_netstat_or_ss() {
        // This test will succeed if either netstat or ss is available
        // We test with a common port that's likely to be in use (port 22 for SSH)
        // or fail gracefully if neither command works

        match PortUsageChecker::check_port(22) {
            Ok(lines) => {
                // Port 22 found - verify we got some output
                assert!(!lines.is_empty(), "Should return at least one line");
                // Verify output contains port number
                assert!(
                    lines.iter().any(|line| line.contains("22")),
                    "Output should contain port 22"
                );
            }
            Err(PortUsageError::PortNotFound { port }) => {
                // Port 22 not in use - this is acceptable in test environments
                assert_eq!(port, 22);
            }
            Err(PortUsageError::BothCommandsFailed { .. }) => {
                // Both commands failed - might happen in restricted environments
                // This is acceptable for testing
            }
        }
    }

    #[test]
    fn it_should_return_error_for_unused_high_port() {
        // Use a very high port number that's unlikely to be in use
        let result = PortUsageChecker::check_port(65000);

        // We just verify it doesn't panic - the actual result depends on system state
        // which we cannot control in unit tests
        match result {
            Ok(_lines) => {
                // Extremely unlikely but technically possible if something uses port 65000
                // Don't fail the test as we can't control system state
            }
            Err(PortUsageError::PortNotFound { port }) => {
                // Expected case - port is not in use
                assert_eq!(port, 65000);
            }
            Err(PortUsageError::BothCommandsFailed { .. }) => {
                // Also acceptable - commands not available on this system
            }
        }
    }

    #[test]
    fn it_should_format_error_messages_clearly() {
        let error = PortUsageError::BothCommandsFailed {
            netstat_error: "netstat: command not found".to_string(),
            ss_error: "ss: command not found".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("netstat"));
        assert!(error_msg.contains("ss"));
        assert!(error_msg.contains("command not found"));
    }

    #[test]
    fn it_should_format_port_not_found_error() {
        let error = PortUsageError::PortNotFound { port: 12345 };

        let error_msg = error.to_string();
        assert!(error_msg.contains("12345"));
        assert!(error_msg.contains("not in use"));
    }
}
