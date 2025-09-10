use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

use tracing::info;

use crate::command::{CommandError, CommandExecutor};

/// Errors that can occur during SSH operations
#[derive(Error, Debug)]
pub enum SshError {
    /// SSH connectivity could not be established within the timeout period
    #[error("SSH connectivity to '{host_ip}' could not be established after {attempts} attempts ({timeout_seconds} seconds)")]
    ConnectivityTimeout {
        host_ip: String,
        attempts: u32,
        timeout_seconds: u32,
    },

    /// Underlying command execution failed
    #[error("SSH command execution failed: {source}")]
    CommandFailed {
        #[source]
        source: CommandError,
    },
}

/// A specialized SSH client with predefined security settings
///
/// This client provides a secure SSH interface for connecting to remote hosts with:
/// - Private key authentication
/// - Disabled strict host key checking (for automation)
/// - No known hosts file usage
/// - Consistent connection settings
///
/// Uses `CommandExecutor` as a collaborator for actual command execution.
pub struct SshClient {
    ssh_key_path: PathBuf,
    username: String,
    command_executor: CommandExecutor,
}

impl SshClient {
    /// Creates a new `SshClient`
    ///
    /// # Arguments
    /// * `ssh_key_path` - Path to the SSH private key file
    /// * `username` - Username for SSH connections (typically "torrust")
    /// * `verbose` - Whether to log commands being executed
    #[must_use]
    pub fn new<P: Into<PathBuf>>(
        ssh_key_path: P,
        username: impl Into<String>,
        verbose: bool,
    ) -> Self {
        Self {
            ssh_key_path: ssh_key_path.into(),
            username: username.into(),
            command_executor: CommandExecutor::new(verbose),
        }
    }

    /// Build SSH arguments for a connection
    fn build_ssh_args(
        &self,
        host_ip: &str,
        remote_command: &str,
        additional_options: &[&str],
    ) -> Vec<String> {
        let mut args = vec![
            "-i".to_string(),
            self.ssh_key_path.to_string_lossy().to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
        ];

        // Add additional SSH options
        for option in additional_options {
            args.push("-o".to_string());
            args.push((*option).to_string());
        }

        args.push(format!("{}@{}", self.username, host_ip));
        args.push(remote_command.to_string());

        args
    }

    /// Execute a command on a remote host via SSH
    ///
    /// # Arguments
    /// * `host_ip` - IP address of the target host
    /// * `remote_command` - Command to execute on the remote host
    ///
    /// # Returns
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    /// * The remote command execution fails with a non-zero exit code
    pub fn execute(&self, host_ip: &str, remote_command: &str) -> Result<String, CommandError> {
        self.execute_with_options(host_ip, remote_command, &[])
    }

    /// Execute a command on a remote host via SSH with additional SSH options
    ///
    /// # Arguments
    /// * `host_ip` - IP address of the target host
    /// * `remote_command` - Command to execute on the remote host
    /// * `additional_options` - Additional SSH options (e.g., `["ConnectTimeout=5"]`)
    ///
    /// # Returns
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    /// * The remote command execution fails with a non-zero exit code
    fn execute_with_options(
        &self,
        host_ip: &str,
        remote_command: &str,
        additional_options: &[&str],
    ) -> Result<String, CommandError> {
        let args = self.build_ssh_args(host_ip, remote_command, additional_options);
        let args_str: Vec<&str> = args.iter().map(std::string::String::as_str).collect();

        self.command_executor.run_command("ssh", &args_str, None)
    }

    /// Check if a command succeeds on a remote host (returns only status)
    ///
    /// # Arguments
    /// * `host_ip` - IP address of the target host
    /// * `remote_command` - Command to execute on the remote host
    ///
    /// # Returns
    /// * `Ok(bool)` - true if command succeeded (exit code 0), false otherwise
    /// * `Err(CommandError)` - Error if SSH connection could not be established
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    pub fn check_command(&self, host_ip: &str, remote_command: &str) -> Result<bool, CommandError> {
        self.check_command_with_options(host_ip, remote_command, &[])
    }

    /// Check if a command succeeds on a remote host with additional SSH options
    ///
    /// # Arguments
    /// * `host_ip` - IP address of the target host  
    /// * `remote_command` - Command to execute on the remote host
    /// * `additional_options` - Additional SSH options (e.g., `["ConnectTimeout=5"]`)
    ///
    /// # Returns
    /// * `Ok(bool)` - true if command succeeded (exit code 0), false otherwise  
    /// * `Err(CommandError)` - Error if SSH connection could not be established
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    fn check_command_with_options(
        &self,
        host_ip: &str,
        remote_command: &str,
        additional_options: &[&str],
    ) -> Result<bool, CommandError> {
        match self.execute_with_options(host_ip, remote_command, additional_options) {
            Ok(_) => Ok(true),
            Err(CommandError::ExecutionFailed { .. }) => Ok(false),
            Err(other) => Err(other),
        }
    }

    /// Test SSH connectivity to a host
    ///
    /// # Arguments
    /// * `host_ip` - IP address of the target host
    ///
    /// # Returns
    /// * `Ok(bool)` - true if SSH connection succeeds, false otherwise
    /// * `Err(CommandError)` - Error if SSH command could not be started
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The SSH command could not be started
    pub fn test_connectivity(&self, host_ip: &str) -> Result<bool, CommandError> {
        self.check_command_with_options(host_ip, "echo 'SSH connected'", &["ConnectTimeout=5"])
    }

    /// Wait for SSH connectivity to be established with retry logic
    ///
    /// This method will repeatedly attempt to connect via SSH until successful
    /// or the maximum number of attempts is reached. Progress is reported via
    /// structured logging using the `tracing` crate.
    ///
    /// # Arguments
    /// * `host_ip` - IP address of the target host
    ///
    /// # Returns
    /// * `Ok(())` - SSH connectivity was successfully established
    /// * `Err(SshError)` - SSH connectivity could not be established after all attempts
    ///
    /// # Errors
    /// This function will return an error if:
    /// * SSH connectivity cannot be established after 30 attempts (60 seconds total)
    pub async fn wait_for_connectivity(&self, host_ip: &str) -> Result<(), SshError> {
        info!("🔌 Waiting for SSH connectivity to {}", host_ip);

        let max_attempts = 30;
        let timeout_seconds = 60;
        let mut attempt = 0;

        while attempt < max_attempts {
            let result = self.test_connectivity(host_ip);

            match result {
                Ok(true) => {
                    info!("✅ SSH connectivity established to {}", host_ip);
                    return Ok(());
                }
                Ok(false) => {
                    // Connection failed, continue trying
                    #[allow(clippy::manual_is_multiple_of)]
                    if (attempt + 1) % 5 == 0 {
                        info!(
                            "   Still waiting for SSH to {}... (attempt {}/{})",
                            host_ip,
                            attempt + 1,
                            max_attempts
                        );
                    }

                    tokio::time::sleep(Duration::from_secs(2)).await;
                    attempt += 1;
                }
                Err(e) => {
                    return Err(SshError::CommandFailed { source: e });
                }
            }
        }

        Err(SshError::ConnectivityTimeout {
            host_ip: host_ip.to_string(),
            attempts: max_attempts,
            timeout_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_ssh_client_with_valid_parameters() {
        let ssh_client = SshClient::new("/path/to/key", "testuser", false);

        assert_eq!(ssh_client.ssh_key_path.to_string_lossy(), "/path/to/key");
        assert_eq!(ssh_client.username, "testuser");
        // Note: verbose is now encapsulated in the CommandExecutor collaborator
    }

    #[test]
    fn it_should_create_ssh_client_with_verbose_enabled() {
        let ssh_client = SshClient::new("/path/to/key", "testuser", true);

        assert_eq!(ssh_client.ssh_key_path.to_string_lossy(), "/path/to/key");
        assert_eq!(ssh_client.username, "testuser");
        // Note: verbose is now encapsulated in the CommandExecutor collaborator
    }
}
