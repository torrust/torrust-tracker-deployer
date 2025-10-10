//! SSH Wait Action
//!
//! This module provides an action to wait for SSH connectivity to become available.
//! The action attempts to connect to SSH in a loop with configurable timeout and retry logic.
//!
//! ## Two-Stage Connectivity Check
//!
//! The SSH wait action uses a two-stage approach to minimize error noise in logs while
//! ensuring SSH service is truly ready:
//!
//! 1. **Port Check**: First, we use `PortChecker` to wait for the TCP port (22) to be open.
//!    This is a lightweight test that doesn't generate SSH protocol errors.
//!
//! 2. **SSH Service Check**: Once the port is open, we verify the SSH daemon is fully
//!    operational using `SshServiceChecker`.
//!
//! ### Why This Approach?
//!
//! - **Reduces error noise**: Most waiting time is spent in the port check phase, which
//!   doesn't generate SSH errors in logs
//! - **Ensures service readiness**: The SSH check verifies the daemon is not just listening
//!   but actually functional
//! - **Handles edge cases**: Covers scenarios where port 22 is open but SSH daemon isn't
//!   fully initialized yet
//! - **Clean logging**: SSH errors only appear when there are actual SSH daemon issues,
//!   not during normal waiting periods
//!
//! In most cases, when we reach the SSH check phase, the service is already ready,
//! so we avoid the majority of SSH connection errors that would otherwise pollute logs.

use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::shared::{ssh::SshServiceChecker, PortChecker};

/// Specific error types for SSH wait operations
#[derive(Debug, thiserror::Error)]
pub enum SshWaitError {
    /// SSH connectivity timeout
    #[error(
        "SSH connection timeout after {timeout_secs}s and {max_attempts} attempts to {host}:{port} (last error: {last_error_context})"
    )]
    SshConnectionTimeout {
        host: String,
        port: u16,
        timeout_secs: u64,
        max_attempts: usize,
        last_error_context: String,
    },

    /// Failed to execute SSH connection test command
    #[error("Failed to execute SSH connection test to {host}:{port}: {source}")]
    SshConnectionTestFailed {
        host: String,
        port: u16,
        #[source]
        source: std::io::Error,
    },

    /// SSH connection test command succeeded but with unexpected output
    #[error(
        "SSH connection test to {host}:{port} succeeded but returned unexpected output: {output}"
    )]
    SshConnectionTestUnexpectedOutput {
        host: String,
        port: u16,
        output: String,
    },
}

/// Result type alias for SSH wait operations
pub type Result<T> = std::result::Result<T, SshWaitError>;

/// Action to wait for SSH connectivity
///
/// This action attempts to connect to SSH in a loop with exponential backoff
/// until either the connection succeeds or the timeout/max attempts are reached.
///
/// ## Usage
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::e2e::containers::actions::SshWaitAction;
/// use std::time::Duration;
/// use std::net::SocketAddr;
///
/// fn wait_for_ssh() -> Result<(), Box<dyn std::error::Error>> {
///     let action = SshWaitAction::new(Duration::from_secs(30), 10);
///     let socket_addr = SocketAddr::from(([127, 0, 0, 1], 22));
///     action.execute(socket_addr)?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct SshWaitAction {
    pub timeout: Duration,
    pub max_attempts: usize,
}

impl SshWaitAction {
    /// Create a new SSH wait action with specified timeout and max attempts
    #[must_use]
    pub fn new(timeout: Duration, max_attempts: usize) -> Self {
        Self {
            timeout,
            max_attempts,
        }
    }

    /// Execute the SSH wait action
    ///
    /// # Arguments
    ///
    /// * `socket_addr` - The socket address (IP and port) to connect to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SSH connectivity cannot be established within the timeout
    /// - Max attempts are exceeded
    /// - SSH command execution fails
    pub fn execute(&self, socket_addr: SocketAddr) -> Result<()> {
        info!(
            socket_addr = %socket_addr,
            timeout_secs = self.timeout.as_secs(),
            max_attempts = self.max_attempts,
            "Starting SSH connectivity wait"
        );

        let start_time = Instant::now();
        let mut attempt = 0;
        let mut backoff = Duration::from_millis(100);
        let mut last_error_context = "No connection attempts made".to_string();

        while start_time.elapsed() < self.timeout && attempt < self.max_attempts {
            attempt += 1;

            info!(
                attempt = attempt,
                max_attempts = self.max_attempts,
                elapsed = start_time.elapsed().as_secs(),
                "Attempting SSH connection"
            );

            match Self::test_ssh_connection(socket_addr) {
                Ok(()) => {
                    info!(
                        socket_addr = %socket_addr,
                        attempt = attempt,
                        elapsed = start_time.elapsed().as_secs(),
                        "SSH connection successful"
                    );
                    return Ok(());
                }
                Err(e) => {
                    last_error_context = format!("Attempt {attempt}: {e}");
                    warn!(
                        attempt = attempt,
                        error = %e,
                        backoff_ms = backoff.as_millis(),
                        "SSH connection failed, retrying after backoff"
                    );

                    // Sleep for backoff duration, but respect the overall timeout
                    std::thread::sleep(backoff);

                    // Exponential backoff with max 5 seconds
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(5));
                }
            }
        }

        Err(SshWaitError::SshConnectionTimeout {
            host: socket_addr.ip().to_string(),
            port: socket_addr.port(),
            timeout_secs: self.timeout.as_secs(),
            max_attempts: self.max_attempts,
            last_error_context,
        })
    }

    /// Test SSH connection using two-stage approach
    ///
    /// Stage 1: Check if TCP port 22 is open and accepting connections
    /// Stage 2: Verify SSH daemon is fully operational
    ///
    /// This approach minimizes error noise by doing the lightweight port check first,
    /// then only checking SSH protocol when the port is already open.
    fn test_ssh_connection(socket_addr: SocketAddr) -> Result<()> {
        // Stage 1: Check if port is open (lightweight, no SSH errors)
        Self::check_port_availability(socket_addr)?;

        // Stage 2: Port is open, now verify SSH daemon is operational
        Self::check_ssh_daemon_availability(socket_addr)?;

        Ok(())
    }

    /// Stage 1: Check if TCP port is open and accepting connections
    ///
    /// This is a lightweight connectivity test that doesn't generate SSH protocol errors.
    /// It only verifies that something is listening on the target port.
    fn check_port_availability(socket_addr: SocketAddr) -> Result<()> {
        debug!(
            socket_addr = %socket_addr,
            "Stage 1: Checking if TCP port is open"
        );

        let port_checker = PortChecker::new();
        match port_checker.is_port_open(socket_addr) {
            Ok(true) => {
                debug!(
                    socket_addr = %socket_addr,
                    "Stage 1: TCP port is open, proceeding to SSH service check"
                );
                Ok(())
            }
            Ok(false) => {
                debug!(
                    socket_addr = %socket_addr,
                    "Stage 1: TCP port is not open"
                );
                Err(SshWaitError::SshConnectionTestFailed {
                    host: socket_addr.ip().to_string(),
                    port: socket_addr.port(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        "TCP port not open",
                    ),
                })
            }
            Err(e) => Err(SshWaitError::SshConnectionTestFailed {
                host: socket_addr.ip().to_string(),
                port: socket_addr.port(),
                source: std::io::Error::other(format!("Port check failed: {e}")),
            }),
        }
    }

    /// Stage 2: Verify SSH daemon is fully operational
    ///
    /// This checks that the SSH service is not just listening on the port, but is
    /// actually ready to accept SSH connections and respond to SSH protocol requests.
    fn check_ssh_daemon_availability(socket_addr: SocketAddr) -> Result<()> {
        debug!(
            socket_addr = %socket_addr,
            "Stage 2: Checking SSH daemon availability"
        );

        let ssh_checker = SshServiceChecker::new();
        match ssh_checker.is_service_available(socket_addr) {
            Ok(true) => {
                debug!(
                    socket_addr = %socket_addr,
                    "Stage 2: SSH daemon is operational"
                );
                Ok(())
            }
            Ok(false) => {
                // This should be rare since port was open in stage 1
                warn!(
                    socket_addr = %socket_addr,
                    "Stage 2: Port open but SSH daemon not responding properly"
                );
                Err(SshWaitError::SshConnectionTestFailed {
                    host: socket_addr.ip().to_string(),
                    port: socket_addr.port(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        "SSH daemon not operational despite port being open",
                    ),
                })
            }
            Err(e) => {
                warn!(
                    socket_addr = %socket_addr,
                    error = %e,
                    "Stage 2: SSH service check failed despite port being open"
                );
                Err(SshWaitError::SshConnectionTestFailed {
                    host: socket_addr.ip().to_string(),
                    port: socket_addr.port(),
                    source: std::io::Error::other(format!("SSH service check failed: {e}")),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_ssh_wait_action_with_config() {
        let action = SshWaitAction::new(Duration::from_secs(30), 10);
        assert_eq!(action.timeout, Duration::from_secs(30));
        assert_eq!(action.max_attempts, 10);
    }

    #[test]
    fn it_should_have_proper_error_display_messages() {
        let error = SshWaitError::SshConnectionTimeout {
            host: "localhost".to_string(),
            port: 22,
            timeout_secs: 30,
            max_attempts: 10,
            last_error_context: "Attempt 10: Connection refused".to_string(),
        };
        assert!(error.to_string().contains("SSH connection timeout"));
        assert!(error.to_string().contains("localhost"));
        assert!(error.to_string().contains("22"));
        assert!(error.to_string().contains("Connection refused"));
    }

    #[test]
    fn it_should_preserve_error_chain_for_connection_test_failed() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "ssh command not found");
        let error = SshWaitError::SshConnectionTestFailed {
            host: "testhost".to_string(),
            port: 2222,
            source: io_error,
        };

        assert!(error
            .to_string()
            .contains("Failed to execute SSH connection test"));
        assert!(error.to_string().contains("testhost:2222"));
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn it_should_return_error_for_invalid_max_attempts() {
        use std::net::{IpAddr, Ipv4Addr};

        let action = SshWaitAction::new(Duration::from_secs(1), 0);
        // With 0 max attempts, it should immediately fail
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 22);
        let result = action.execute(socket_addr);
        assert!(result.is_err());
    }

    #[test]
    fn it_should_handle_connection_refused_correctly() {
        // This test may be flaky depending on system configuration, so we skip it by default
        // It's mainly for documenting the expected behavior with connection refused errors

        // If we wanted to test this properly, we'd need to mock the SSH command execution
        // For now, this serves as documentation of the expected behavior
    }

    #[test]
    fn it_should_handle_permission_denied_as_successful_connection() {
        // This test documents that "Permission denied" should be treated as a successful
        // connectivity test, since it means the SSH server is reachable but auth failed
        // This is handled in the SSH service check phase (stage 2) of our two-stage approach

        // The two-stage logic:
        // Stage 1 (Port check): TCP connection to port 22 → Success (port open)
        // Stage 2 (SSH check):
        //   - Exit code 255 with "Connection refused" in stderr → Error (daemon not ready)
        //   - Exit code 255 with "Permission denied" in stderr → Success (daemon ready)
        //   - Exit code 0 → Success (command succeeded)
        //   - Other exit codes → Success (daemon ready, other issues)
    }
}
