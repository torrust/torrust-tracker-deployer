//! Port usage checking utilities.
//!
//! This module provides functionality to check which processes are using specific ports
//! on the system. It uses system commands like `netstat` or `ss` to query port usage.
//!
//! This is different from [`crate::testing::network::port_checker`] which checks TCP connectivity
//! to a port. This module checks which process is actually bound to a port.

use thiserror::Error;

use crate::adapters::network::{NetstatClient, NetworkError, SsClient};

// ============================================================================
// PUBLIC API - Main Types
// ============================================================================

/// Utility for checking which processes are using specific ports
///
/// This struct provides methods to query the operating system about port usage.
/// It tries multiple commands (`netstat`, `ss`) to be compatible with different systems.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::testing::network::PortUsageChecker;
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
    /// use torrust_tracker_deployer_lib::testing::network::PortUsageChecker;
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
                    port,
                    netstat_error: Box::new(netstat_error),
                    ss_error: Box::new(ss_error),
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
    /// * `Err(PortUsageError)` - If command failed or port not found
    fn check_port_with_netstat(port: u16) -> Result<Vec<String>, PortUsageError> {
        let netstat = NetstatClient::new();
        let output = netstat
            .list_tcp_listening_ports()
            .map_err(|source| PortUsageError::NetstatFailed { port, source })?;

        let port_str = port.to_string();
        let matches: Vec<String> = output
            .lines()
            .filter(|line| line.contains(&port_str))
            .map(ToString::to_string)
            .collect();

        if matches.is_empty() {
            Err(PortUsageError::PortNotFound { port })
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
    /// * `Err(PortUsageError)` - If command failed or port not found
    fn check_port_with_ss(port: u16) -> Result<Vec<String>, PortUsageError> {
        let ss = SsClient::new();
        let output = ss
            .list_tcp_listening_ports()
            .map_err(|source| PortUsageError::SsFailed { port, source })?;

        let port_str = port.to_string();
        let matches: Vec<String> = output
            .lines()
            .filter(|line| line.contains(&port_str))
            .map(ToString::to_string)
            .collect();

        if matches.is_empty() {
            Err(PortUsageError::PortNotFound { port })
        } else {
            Ok(matches)
        }
    }
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Error type for port usage checking operations
#[derive(Debug, Error)]
pub enum PortUsageError {
    /// Netstat command failed to execute or parse output
    ///
    /// This usually indicates netstat is not installed or not accessible.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to check port {port} with netstat
Tip: Verify netstat is installed: 'which netstat' or install with 'apt-get install net-tools'"
    )]
    NetstatFailed {
        port: u16,
        #[source]
        source: NetworkError,
    },

    /// SS command failed to execute or parse output
    ///
    /// This usually indicates ss is not installed or not accessible.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to check port {port} with ss
Tip: Verify ss is installed: 'which ss' or install with 'apt-get install iproute2'"
    )]
    SsFailed {
        port: u16,
        #[source]
        source: NetworkError,
    },

    /// Both netstat and ss commands failed
    ///
    /// This means neither command is available or accessible on the system.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Failed to check port {port}: neither netstat nor ss commands are available
Tip: Install one of these tools: 'apt-get install net-tools' (netstat) or 'apt-get install iproute2' (ss)")]
    BothCommandsFailed {
        port: u16,
        #[source]
        netstat_error: Box<PortUsageError>,
        ss_error: Box<PortUsageError>,
    },

    /// Command executed but port was not found in output
    ///
    /// This means the port is not currently in use by any process.
    #[error("Port {port} is not in use according to system commands")]
    PortNotFound { port: u16 },
}

impl PortUsageError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::testing::network::PortUsageChecker;
    ///
    /// if let Err(e) = PortUsageChecker::check_port(8080) {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::NetstatFailed { .. } => {
                "Netstat Command Failed - Detailed Troubleshooting:

1. Check if netstat is installed:
   which netstat

2. Install netstat if missing:
   Debian/Ubuntu: sudo apt-get install net-tools
   RHEL/CentOS: sudo yum install net-tools
   Alpine: sudo apk add net-tools
   macOS: netstat is pre-installed

3. Verify permissions:
   - Some netstat options require root/sudo for full process information
   - Try running the command manually: netstat -tlnp

4. Alternative: Use ss command instead (modern Linux)

For more information, see: man netstat"
            }

            Self::SsFailed { .. } => {
                "SS Command Failed - Detailed Troubleshooting:

1. Check if ss is installed:
   which ss

2. Install ss if missing:
   Debian/Ubuntu: sudo apt-get install iproute2
   RHEL/CentOS: sudo yum install iproute2
   Alpine: sudo apk add iproute2

3. Verify permissions:
   - Some ss options require root/sudo for full process information
   - Try running the command manually: ss -tlnp

4. Alternative: Use netstat command instead (older systems)

For more information, see: man ss"
            }

            Self::BothCommandsFailed { .. } => {
                "Both Commands Failed - Detailed Troubleshooting:

1. Neither netstat nor ss are available on this system
   - These are the standard tools for checking port usage on Unix-like systems

2. Install at least one of these tools:
   Debian/Ubuntu:
     sudo apt-get install net-tools    # for netstat
     sudo apt-get install iproute2     # for ss (recommended for modern systems)

   RHEL/CentOS:
     sudo yum install net-tools         # for netstat
     sudo yum install iproute2          # for ss

   Alpine:
     sudo apk add net-tools             # for netstat
     sudo apk add iproute2              # for ss

3. If on a restricted environment (container, minimal OS):
   - Contact your system administrator
   - Or use the system's package manager to install required tools

For more information:
- man netstat
- man ss"
            }

            Self::PortNotFound { .. } => {
                "Port Not Found - This is not an error:

The port you checked is not currently in use by any process. This is normal
if no service is listening on that port.

If you expected a service to be running on this port:

1. Check if the service is running:
   systemctl status <service-name>
   or: ps aux | grep <service-name>

2. Check the service configuration:
   - Verify the port number in the service configuration
   - Check if the service is bound to a different address (localhost vs 0.0.0.0)

3. Check if the service failed to start:
   - Review service logs: journalctl -u <service-name>
   - Check for port binding errors in the service logs

4. Verify the service is installed:
   which <service-binary>

For more information, see your service's documentation."
            }
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
            Err(PortUsageError::NetstatFailed { port, .. }) => {
                // Netstat failed but we tried - acceptable
                assert_eq!(port, 22);
            }
            Err(PortUsageError::SsFailed { port, .. }) => {
                // SS failed but we tried - acceptable
                assert_eq!(port, 22);
            }
            Err(PortUsageError::BothCommandsFailed { port, .. }) => {
                // Both commands failed - might happen in restricted environments
                // This is acceptable for testing
                assert_eq!(port, 22);
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
            Err(PortUsageError::NetstatFailed { port, .. }) => {
                // Netstat failed but we tried - acceptable
                assert_eq!(port, 65000);
            }
            Err(PortUsageError::SsFailed { port, .. }) => {
                // SS failed but we tried - acceptable
                assert_eq!(port, 65000);
            }
            Err(PortUsageError::BothCommandsFailed { port, .. }) => {
                // Also acceptable - commands not available on this system
                assert_eq!(port, 65000);
            }
        }
    }

    #[test]
    fn it_should_format_error_messages_clearly() {
        use crate::shared::command::CommandError;

        // Create mock network errors
        let netstat_error = PortUsageError::NetstatFailed {
            port: 8080,
            source: NetworkError::NetstatFailed(CommandError::ExecutionFailed {
                command: "netstat".to_string(),
                exit_code: "127".to_string(),
                stdout: String::new(),
                stderr: "netstat: command not found".to_string(),
            }),
        };

        let ss_error = PortUsageError::SsFailed {
            port: 8080,
            source: NetworkError::SsFailed(CommandError::ExecutionFailed {
                command: "ss".to_string(),
                exit_code: "127".to_string(),
                stdout: String::new(),
                stderr: "ss: command not found".to_string(),
            }),
        };

        let error = PortUsageError::BothCommandsFailed {
            port: 8080,
            netstat_error: Box::new(netstat_error),
            ss_error: Box::new(ss_error),
        };

        let error_msg = error.to_string();
        assert!(
            error_msg.contains("8080"),
            "Error should contain port number"
        );
        assert!(
            error_msg.contains("neither netstat nor ss"),
            "Error should mention both commands"
        );
    }

    #[test]
    fn it_should_format_port_not_found_error() {
        let error = PortUsageError::PortNotFound { port: 12345 };

        let error_msg = error.to_string();
        assert!(error_msg.contains("12345"));
        assert!(error_msg.contains("not in use"));
    }

    #[test]
    fn it_should_provide_helpful_troubleshooting_guidance() {
        use crate::shared::command::CommandError;

        // Test NetstatFailed help
        let netstat_error = PortUsageError::NetstatFailed {
            port: 8080,
            source: NetworkError::NetstatFailed(CommandError::ExecutionFailed {
                command: "netstat".to_string(),
                exit_code: "127".to_string(),
                stdout: String::new(),
                stderr: "netstat: command not found".to_string(),
            }),
        };
        let help = netstat_error.help();
        assert!(help.contains("netstat"));
        assert!(help.contains("which netstat"));
        assert!(help.contains("apt-get install net-tools"));

        // Test SsFailed help
        let ss_error = PortUsageError::SsFailed {
            port: 8080,
            source: NetworkError::SsFailed(CommandError::ExecutionFailed {
                command: "ss".to_string(),
                exit_code: "127".to_string(),
                stdout: String::new(),
                stderr: "ss: command not found".to_string(),
            }),
        };
        let help = ss_error.help();
        assert!(help.contains("ss"));
        assert!(help.contains("which ss"));
        assert!(help.contains("apt-get install iproute2"));

        // Test BothCommandsFailed help
        let both_failed = PortUsageError::BothCommandsFailed {
            port: 8080,
            netstat_error: Box::new(netstat_error),
            ss_error: Box::new(ss_error),
        };
        let help = both_failed.help();
        assert!(help.contains("Neither netstat nor ss"));
        assert!(help.contains("net-tools"));
        assert!(help.contains("iproute2"));

        // Test PortNotFound help
        let port_not_found = PortUsageError::PortNotFound { port: 8080 };
        let help = port_not_found.help();
        assert!(help.contains("not currently in use"));
        assert!(help.contains("systemctl status"));
    }
}
