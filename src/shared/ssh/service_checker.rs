//! SSH Service Checker
//!
//! This module provides functionality to check if SSH service is available on a remote host
//! without requiring authentication. It's designed for connectivity testing only - like a "ping"
//! for SSH services to verify that the SSH daemon is running and accepting connections.
//!
//! ## Key Features
//!
//! - Pure connectivity testing without authentication
//! - Minimal SSH command execution to test service availability
//! - Distinguishes between "service not available" and "service available but auth failed"
//! - Lightweight and focused on service discovery
//!
//! ## Usage
//!
//! ```rust,no_run
//! use torrust_tracker_deploy::shared::ssh::SshServiceChecker;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let checker = SshServiceChecker::new();
//! let is_available = checker.is_service_available("192.168.1.1", 22)?;
//! if is_available {
//!     println!("SSH service is available");
//! } else {
//!     println!("SSH service is not available");
//! }
//! # Ok(())
//! # }
//! ```

use std::process::Command;
use tracing::debug;

/// SSH Service availability checker errors
#[derive(Debug, thiserror::Error)]
pub enum SshServiceError {
    /// Command execution failed (e.g., ssh binary not found, process interrupted)
    #[error("Failed to execute SSH service check command: {source}")]
    CommandExecutionFailed {
        #[source]
        source: std::io::Error,
    },
}

/// Result type for SSH service operations
pub type Result<T> = std::result::Result<T, SshServiceError>;

/// SSH Service Checker for testing service availability
///
/// This checker performs lightweight connectivity tests to determine if an SSH daemon
/// is running and accepting connections on a given host and port. It does not attempt
/// to authenticate or establish a working SSH session.
///
/// The checker uses minimal SSH commands with short timeouts and batch mode to quickly
/// determine service availability without user interaction.
#[derive(Debug)]
pub struct SshServiceChecker {
    /// Connection timeout in seconds for SSH attempts
    connect_timeout: u16,
}

impl Default for SshServiceChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SshServiceChecker {
    /// Create a new SSH service checker with default settings
    ///
    /// Default connection timeout is 5 seconds.
    #[must_use]
    pub fn new() -> Self {
        Self { connect_timeout: 5 }
    }

    /// Create a new SSH service checker with custom connection timeout
    ///
    /// # Arguments
    /// * `connect_timeout` - Timeout in seconds for connection attempts
    #[must_use]
    pub fn with_timeout(connect_timeout: u16) -> Self {
        Self { connect_timeout }
    }

    /// Check if SSH service is available on the specified host and port
    ///
    /// This method attempts a minimal SSH connection to test service availability.
    /// It distinguishes between:
    /// - Service not available (connection refused, no route to host)
    /// - Service available (authentication failures are considered as service available)
    ///
    /// # Arguments
    /// * `host` - The hostname or IP address to test
    /// * `port` - The SSH port to test
    ///
    /// # Returns
    /// * `Ok(true)` - SSH service is available and accepting connections
    /// * `Ok(false)` - SSH service is not available or not reachable
    /// * `Err(SshServiceError)` - Command execution error (e.g., ssh binary not found)
    ///
    /// # Errors
    /// Returns an error if the SSH command cannot be executed (e.g., ssh binary not found
    /// or process was terminated by signal).
    pub fn is_service_available(&self, host: &str, port: u16) -> Result<bool> {
        debug!(
            host = host,
            port = port,
            timeout = self.connect_timeout,
            "Testing SSH service availability"
        );

        let output = Command::new("ssh")
            .args([
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                &format!("ConnectTimeout={}", self.connect_timeout),
                "-o",
                "BatchMode=yes", // Non-interactive mode
                "-p",
                &port.to_string(),
                &format!("test@{host}"),
                "echo",
                "connectivity_test",
            ])
            .output()
            .map_err(|source| SshServiceError::CommandExecutionFailed { source })?;

        // Analyze the command result to determine service availability
        match output.status.code() {
            Some(0) => {
                // SSH command succeeded - service is definitely available
                debug!(
                    host = host,
                    port = port,
                    "SSH service available (command succeeded)"
                );
                Ok(true)
            }
            Some(255) => {
                // Exit code 255 can indicate different scenarios
                let stderr = String::from_utf8_lossy(&output.stderr);

                if stderr.contains("Connection refused") || stderr.contains("No route to host") {
                    // Service is not available or host is not reachable
                    debug!(
                        host = host,
                        port = port,
                        error = %stderr.trim(),
                        "SSH service not available"
                    );
                    Ok(false)
                } else {
                    // Authentication failed, permission denied, etc. - service is available
                    debug!(
                        host = host,
                        port = port,
                        error = %stderr.trim(),
                        "SSH service available (authentication failed)"
                    );
                    Ok(true)
                }
            }
            Some(exit_code) => {
                // Other non-zero exit codes typically indicate service is available
                // but there are other issues (auth, command execution, etc.)
                debug!(
                    host = host,
                    port = port,
                    exit_code = exit_code,
                    "SSH service available (non-zero exit code)"
                );
                Ok(true)
            }
            None => {
                // Process was terminated by signal - treat as command execution error
                Err(SshServiceError::CommandExecutionFailed {
                    source: std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        "SSH process terminated by signal",
                    ),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_ssh_service_checker_with_defaults() {
        let checker = SshServiceChecker::new();
        assert_eq!(checker.connect_timeout, 5);
    }

    #[test]
    fn it_should_create_ssh_service_checker_with_custom_timeout() {
        let checker = SshServiceChecker::with_timeout(10);
        assert_eq!(checker.connect_timeout, 10);
    }

    #[test]
    fn it_should_implement_default_trait() {
        let checker = SshServiceChecker::default();
        assert_eq!(checker.connect_timeout, 5);
    }

    #[test]
    fn it_should_have_proper_error_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "ssh command not found");
        let error = SshServiceError::CommandExecutionFailed { source: io_error };

        assert!(error
            .to_string()
            .contains("Failed to execute SSH service check command"));
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn it_should_support_debug_formatting() {
        let checker = SshServiceChecker::new();
        let debug_str = format!("{checker:?}");
        assert!(debug_str.contains("SshServiceChecker"));
        assert!(debug_str.contains("connect_timeout"));
    }

    // Note: We don't include integration tests that actually connect to SSH services
    // as they would be flaky and depend on external services. The actual connectivity
    // testing logic is documented through these unit tests and the implementation.
}
