//! SSH client implementation for secure remote command execution
//!
//! This module provides the `SshClient` which handles SSH connections and remote
//! command execution with predefined security settings optimized for automation.
//!
//! ## Key Features
//!
//! - Private key authentication with configurable credentials
//! - Automated host key management (disabled strict checking for automation)
//! - Connection timeout and retry mechanisms
//! - Comprehensive error handling for network and authentication issues
//! - Integration with the command execution framework
//!
//! The client is designed for automated deployment scenarios where security
//! is important but strict host key checking would interfere with automation.

use std::time::Duration;

use tracing::{info, warn};

use crate::shared::command::{CommandError, CommandExecutor};

use super::{SshConfig, SshError};

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
    ssh_config: SshConfig,
    command_executor: CommandExecutor,
}

impl SshClient {
    /// Creates a new `SshClient`
    ///
    /// # Arguments
    ///
    /// * `ssh_config` - SSH connection configuration containing credentials and host IP
    #[must_use]
    pub fn new(ssh_config: SshConfig) -> Self {
        Self {
            ssh_config,
            command_executor: CommandExecutor::new(),
        }
    }

    /// Get the SSH configuration
    ///
    /// Returns a reference to the SSH configuration used by this client.
    #[must_use]
    pub fn ssh_config(&self) -> &SshConfig {
        &self.ssh_config
    }

    /// Build SSH arguments for a connection
    fn build_ssh_args(&self, remote_command: &str, additional_options: &[&str]) -> Vec<String> {
        let mut args = vec![
            // Specify the private key file for authentication
            "-i".to_string(),
            self.ssh_config
                .ssh_priv_key_path()
                .to_string_lossy()
                .to_string(),
            // Disable strict host key checking for automation
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            // Disable known hosts file to avoid host key conflicts in automation
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
            // Set connection timeout for automation (prevents hanging)
            "-o".to_string(),
            format!(
                "ConnectTimeout={}",
                self.ssh_config.connection_config.connect_timeout_secs
            ),
            // Specify the SSH port to connect to
            "-p".to_string(),
            self.ssh_config.ssh_port().to_string(),
        ];

        // Add additional SSH options with explicit option flag
        for option in additional_options {
            args.push("-o".to_string());
            args.push((*option).to_string());
        }

        // SSH target: username@hostname
        args.push(format!(
            "{}@{}",
            self.ssh_config.ssh_username(),
            self.ssh_config.host_ip()
        ));
        // Remote command to execute
        args.push(remote_command.to_string());

        args
    }

    /// Execute a command on a remote host via SSH
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to execute on the remote host
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    /// * The remote command execution fails with a non-zero exit code
    pub fn execute(&self, remote_command: &str) -> Result<String, CommandError> {
        self.execute_with_options(remote_command, &[])
    }

    /// Execute a command on a remote host via SSH with additional SSH options
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to execute on the remote host
    /// * `additional_options` - Additional SSH options (e.g., `["ConnectTimeout=5"]`)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    /// * The remote command execution fails with a non-zero exit code
    fn execute_with_options(
        &self,
        remote_command: &str,
        additional_options: &[&str],
    ) -> Result<String, CommandError> {
        let args = self.build_ssh_args(remote_command, additional_options);
        let args_str: Vec<&str> = args.iter().map(std::string::String::as_str).collect();

        let result = self.command_executor.run_command("ssh", &args_str, None)?;

        // Process stderr for SSH warnings and log them
        self.process_ssh_warnings(&result.stderr);

        Ok(result.stdout)
    }

    /// Process SSH stderr output to detect and log warnings
    ///
    /// SSH writes various informational messages to stderr, including host key
    /// warnings. This method detects these warnings and logs them appropriately
    /// using the tracing framework so they are visible to users at warn level.
    ///
    /// # Arguments
    ///
    /// * `stderr` - The stderr output from the SSH command
    fn process_ssh_warnings(&self, stderr: &str) {
        if stderr.trim().is_empty() {
            return;
        }

        // Split stderr into lines and check each line for warnings
        for line in stderr.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("Warning:") {
                warn!(
                    operation = "ssh_warning",
                    host_ip = %self.ssh_config.host_ip(),
                    message = %trimmed_line,
                    "SSH warning detected"
                );
            }
        }
    }

    /// Check if a command succeeds on a remote host (returns only status)
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to execute on the remote host
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if command succeeded (exit code 0), false otherwise
    /// * `Err(CommandError)` - Error if SSH connection could not be established
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    pub fn check_command(&self, remote_command: &str) -> Result<bool, CommandError> {
        self.check_command_with_options(remote_command, &[])
    }

    /// Check if a command succeeds on a remote host with additional SSH options
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to execute on the remote host
    /// * `additional_options` - Additional SSH options (e.g., `["ConnectTimeout=5"]`)
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if command succeeded (exit code 0), false otherwise  
    /// * `Err(CommandError)` - Error if SSH connection could not be established
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The SSH connection cannot be established
    fn check_command_with_options(
        &self,
        remote_command: &str,
        additional_options: &[&str],
    ) -> Result<bool, CommandError> {
        match self.execute_with_options(remote_command, additional_options) {
            Ok(_) => Ok(true),
            Err(CommandError::ExecutionFailed { .. }) => Ok(false),
            Err(other) => Err(other),
        }
    }

    /// Test SSH connectivity to a host
    ///
    /// Uses the connection timeout configured in `SshConfig`.
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - true if SSH connection succeeds, false otherwise
    /// * `Err(CommandError)` - Error if SSH command could not be started
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The SSH command could not be started
    pub fn test_connectivity(&self) -> Result<bool, CommandError> {
        self.check_command("echo 'SSH connected'")
    }

    /// Wait for SSH connectivity to be established with retry logic
    ///
    /// This method will repeatedly attempt to connect via SSH until successful
    /// or the maximum number of attempts is reached. Progress is reported via
    /// structured logging using the `tracing` crate.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - SSH connectivity was successfully established
    /// * `Err(SshError)` - SSH connectivity could not be established after all attempts
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * SSH connectivity cannot be established after 30 attempts (60 seconds total)
    pub async fn wait_for_connectivity(&self) -> Result<(), SshError> {
        info!(
            operation = "ssh_connectivity",
            host_ip = %self.ssh_config.host_ip(),
            "Waiting for SSH connectivity"
        );

        let max_attempts = 30;
        let timeout_seconds = 60;
        let mut attempt = 0;

        while attempt < max_attempts {
            let result = self.test_connectivity();

            match result {
                Ok(true) => {
                    info!(
                        operation = "ssh_connectivity",
                        host_ip = %self.ssh_config.host_ip(),
                        status = "success",
                        "SSH connectivity established"
                    );
                    return Ok(());
                }
                Ok(false) => {
                    // Connection failed, continue trying
                    if (attempt + 1) % 5 == 0 {
                        info!(
                            operation = "ssh_connectivity",
                            host_ip = %self.ssh_config.host_ip(),
                            attempt = attempt + 1,
                            max_attempts = max_attempts,
                            "Still waiting for SSH connectivity"
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
            host_ip: self.ssh_config.host_ip().to_string(),
            attempts: max_attempts,
            timeout_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::SshCredentials;
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use crate::shared::Username;

    #[test]
    fn it_should_create_ssh_client_with_valid_parameters() {
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let credentials = SshCredentials::new(
            PathBuf::from("/path/to/key"),
            PathBuf::from("/path/to/key.pub"),
            Username::new("testuser").unwrap(),
        );
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);
        let ssh_client = SshClient::new(ssh_config);

        assert_eq!(
            ssh_client.ssh_config.ssh_priv_key_path().to_string_lossy(),
            "/path/to/key"
        );
        assert_eq!(ssh_client.ssh_config.ssh_username(), "testuser");
        assert_eq!(ssh_client.ssh_config.host_ip(), host_ip);
        // Note: verbose is now encapsulated in the CommandExecutor collaborator
    }

    #[test]
    fn it_should_create_ssh_client_with_connection_details() {
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let credentials = SshCredentials::new(
            PathBuf::from("/path/to/key"),
            PathBuf::from("/path/to/key.pub"),
            Username::new("testuser").unwrap(),
        );
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);
        let ssh_client = SshClient::new(ssh_config);

        assert_eq!(
            ssh_client.ssh_config.ssh_priv_key_path().to_string_lossy(),
            "/path/to/key"
        );
        assert_eq!(ssh_client.ssh_config.ssh_username(), "testuser");
        assert_eq!(ssh_client.ssh_config.host_ip(), host_ip);
        // Note: logging is now handled by the tracing crate via CommandExecutor
    }

    #[test]
    fn it_should_detect_ssh_warnings_in_stderr() {
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let credentials = SshCredentials::new(
            PathBuf::from("/path/to/key"),
            PathBuf::from("/path/to/key.pub"),
            Username::new("testuser").unwrap(),
        );
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);
        let ssh_client = SshClient::new(ssh_config);

        // Test stderr with SSH warning
        let stderr_with_warning =
            "Warning: Permanently added '10.140.190.144' (ED25519) to the list of known hosts.";

        // This test verifies the method exists and processes warnings correctly
        // In a real scenario, this would trigger tracing::warn! which would be captured
        // by a tracing subscriber in integration tests
        ssh_client.process_ssh_warnings(stderr_with_warning);

        // Test stderr without warning
        let stderr_without_warning = "Some other output";
        ssh_client.process_ssh_warnings(stderr_without_warning);

        // Test empty stderr
        ssh_client.process_ssh_warnings("");
    }
}
