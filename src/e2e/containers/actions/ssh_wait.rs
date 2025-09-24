//! SSH Wait Action
//!
//! This module provides an action to wait for SSH connectivity to become available.
//! The action attempts to connect to SSH in a loop with configurable timeout and retry logic.

use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Specific error types for SSH wait operations
#[derive(Debug, thiserror::Error)]
pub enum SshWaitError {
    /// SSH connectivity timeout
    #[error(
        "SSH connection timeout after {timeout_secs}s and {max_attempts} attempts to {host}:{port}"
    )]
    SshConnectionTimeout {
        host: String,
        port: u16,
        timeout_secs: u64,
        max_attempts: usize,
    },

    /// Failed to execute SSH connection test command
    #[error("Failed to execute SSH connection test: {source}")]
    SshConnectionTestFailed {
        #[source]
        source: std::io::Error,
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
/// use torrust_tracker_deploy::e2e::containers::actions::SshWaitAction;
/// use std::time::Duration;
///
/// fn wait_for_ssh() -> Result<(), Box<dyn std::error::Error>> {
///     let action = SshWaitAction::new(Duration::from_secs(30), 10);
///     action.execute("127.0.0.1", 22)?;
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
    /// * `host` - The host to connect to
    /// * `port` - The SSH port to connect to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SSH connectivity cannot be established within the timeout
    /// - Max attempts are exceeded
    /// - SSH command execution fails
    pub fn execute(&self, host: &str, port: u16) -> Result<()> {
        info!(
            host = host,
            port = port,
            timeout_secs = self.timeout.as_secs(),
            max_attempts = self.max_attempts,
            "Starting SSH connectivity wait"
        );

        let start_time = Instant::now();
        let mut attempt = 0;
        let mut backoff = Duration::from_millis(100);

        while start_time.elapsed() < self.timeout && attempt < self.max_attempts {
            attempt += 1;

            info!(
                attempt = attempt,
                max_attempts = self.max_attempts,
                elapsed = start_time.elapsed().as_secs(),
                "Attempting SSH connection"
            );

            match Self::test_ssh_connection(host, port) {
                Ok(()) => {
                    info!(
                        host = host,
                        port = port,
                        attempt = attempt,
                        elapsed = start_time.elapsed().as_secs(),
                        "SSH connection successful"
                    );
                    return Ok(());
                }
                Err(e) => {
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
            host: host.to_string(),
            port,
            timeout_secs: self.timeout.as_secs(),
            max_attempts: self.max_attempts,
        })
    }

    /// Test SSH connection by attempting a simple SSH command
    fn test_ssh_connection(host: &str, port: u16) -> Result<()> {
        use std::process::Command;

        let output = Command::new("ssh")
            .args([
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                "ConnectTimeout=5",
                "-o",
                "BatchMode=yes", // Non-interactive mode
                "-p",
                &port.to_string(),
                &format!("test@{host}"),
                "echo",
                "test",
            ])
            .output()
            .map_err(|source| SshWaitError::SshConnectionTestFailed { source })?;

        // For SSH connectivity testing, we just need to know if the SSH server
        // is responding, even if authentication fails
        if output.status.code() == Some(255) {
            // SSH connection failed (server not reachable)
            Err(SshWaitError::SshConnectionTestFailed {
                source: std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "SSH server not reachable",
                ),
            })
        } else {
            // SSH server is responding (even if auth failed, that's OK for connectivity test)
            Ok(())
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
        };
        assert!(error.to_string().contains("SSH connection timeout"));
        assert!(error.to_string().contains("localhost"));
        assert!(error.to_string().contains("22"));
    }

    #[test]
    fn it_should_preserve_error_chain_for_connection_test_failed() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "ssh command not found");
        let error = SshWaitError::SshConnectionTestFailed { source: io_error };

        assert!(error
            .to_string()
            .contains("Failed to execute SSH connection test"));
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn it_should_return_error_for_invalid_max_attempts() {
        let action = SshWaitAction::new(Duration::from_secs(1), 0);
        // With 0 max attempts, it should immediately fail
        let result = action.execute("127.0.0.1", 22);
        assert!(result.is_err());
    }
}
