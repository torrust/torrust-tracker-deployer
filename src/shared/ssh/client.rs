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
    // ============================================================================
    // PUBLIC API - Constructors
    // ============================================================================

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

    // ============================================================================
    // PUBLIC API - Accessors
    // ============================================================================

    /// Get the SSH configuration
    ///
    /// Returns a reference to the SSH configuration used by this client.
    #[must_use]
    pub fn ssh_config(&self) -> &SshConfig {
        &self.ssh_config
    }

    // ============================================================================
    // PUBLIC API - Command Execution
    // ============================================================================

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

    // ============================================================================
    // PUBLIC API - Connectivity Testing
    // ============================================================================

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
    /// * SSH connectivity cannot be established after the configured maximum attempts
    pub async fn wait_for_connectivity(&self) -> Result<(), SshError> {
        info!(
            operation = "ssh_connectivity",
            host_ip = %self.ssh_config.host_ip(),
            "Waiting for SSH connectivity"
        );

        let conn_config = &self.ssh_config.connection_config;
        let max_attempts = conn_config.max_retry_attempts;
        let timeout_seconds = conn_config.total_timeout_secs();
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
                    if (attempt + 1) % conn_config.retry_log_frequency == 0 {
                        info!(
                            operation = "ssh_connectivity",
                            host_ip = %self.ssh_config.host_ip(),
                            attempt = attempt + 1,
                            max_attempts = max_attempts,
                            "Still waiting for SSH connectivity"
                        );
                    }

                    tokio::time::sleep(Duration::from_secs(u64::from(
                        conn_config.retry_interval_secs,
                    )))
                    .await;
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

    // ============================================================================
    // PRIVATE - Helper Methods
    // ============================================================================

    /// Build default SSH options for automation
    ///
    /// Returns a map of default SSH option keys to their values:
    /// - `StrictHostKeyChecking`: `no` (disable host key verification)
    /// - `UserKnownHostsFile`: `/dev/null` (ignore known hosts file)
    /// - `ConnectTimeout`: configured timeout (prevents hanging)
    ///
    /// These defaults ensure reliable automation but can be overridden by
    /// user-provided options in `additional_options`.
    fn build_default_ssh_options(&self) -> std::collections::HashMap<String, String> {
        let mut defaults = std::collections::HashMap::new();
        defaults.insert("StrictHostKeyChecking".to_string(), "no".to_string());
        defaults.insert("UserKnownHostsFile".to_string(), "/dev/null".to_string());
        defaults.insert(
            "ConnectTimeout".to_string(),
            self.ssh_config
                .connection_config
                .connect_timeout_secs
                .to_string(),
        );
        defaults
    }

    /// Extract SSH option key from an option string
    ///
    /// Parses option strings in formats like:
    /// - `"Key=value"` → `"Key"`
    /// - `"Key"` → `"Key"`
    ///
    /// Returns `None` if the option string is empty or malformed.
    fn extract_option_key(option: &str) -> Option<String> {
        option.split('=').next().map(|s| s.trim().to_string())
    }

    /// Build SSH arguments for a connection
    ///
    /// Constructs the complete SSH command arguments including:
    /// 1. Authentication credentials (private key)
    /// 2. User-provided additional options (take precedence)
    /// 3. Default options (only if not overridden by user)
    /// 4. Connection details (port, host)
    /// 5. Remote command to execute
    ///
    /// ## Option Override Behavior
    ///
    /// User-provided options in `additional_options` take precedence over defaults:
    /// - If a user provides `StrictHostKeyChecking=yes`, it will override the default `no`
    /// - If a user provides `ConnectTimeout=30`, it will override the configured default
    /// - Default options are only added if the user hasn't provided them
    ///
    /// This allows users full control while providing sensible defaults for automation.
    fn build_ssh_args(&self, remote_command: &str, additional_options: &[&str]) -> Vec<String> {
        let mut args = vec![
            // Specify the private key file for authentication
            "-i".to_string(),
            self.ssh_config
                .ssh_priv_key_path()
                .to_string_lossy()
                .to_string(),
        ];

        // Build default options map
        let mut defaults = self.build_default_ssh_options();

        // Specify the SSH port to connect to
        args.push("-p".to_string());
        args.push(self.ssh_config.ssh_port().to_string());

        // Add user-provided options FIRST (they take precedence)
        // and remove them from defaults so we don't add them twice
        for option in additional_options {
            args.push("-o".to_string());
            args.push((*option).to_string());

            // Remove this option key from defaults to prevent duplication
            if let Some(key) = Self::extract_option_key(option) {
                defaults.remove(&key);
            }
        }

        // Add remaining default options (those not overridden by user)
        for (key, value) in defaults {
            args.push("-o".to_string());
            args.push(format!("{key}={value}"));
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

    /// Execute a command with additional SSH options
    ///
    /// This method allows passing custom SSH options for specific commands,
    /// useful for advanced scenarios like connection keep-alive or custom timeouts.
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to execute on the remote host
    /// * `additional_options` - SSH options (e.g., `["ServerAliveInterval=60"]`)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use torrust_tracker_deployer_lib::shared::ssh::{SshClient, SshConfig, SshCredentials};
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use std::path::PathBuf;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/path/to/key"),
    ///     PathBuf::from("/path/to/key.pub"),
    ///     Username::new("user")?,
    /// );
    /// let config = SshConfig::with_default_port(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
    /// );
    /// let client = SshClient::new(config);
    ///
    /// // Keep connection alive during long-running command
    /// let output = client.execute_with_options(
    ///     "long_running_task",
    ///     &["ServerAliveInterval=60", "ServerAliveCountMax=3"]
    /// )?;
    ///
    /// // Use custom connection timeout for specific command
    /// let output = client.execute_with_options(
    ///     "quick_check",
    ///     &["ConnectTimeout=2"]
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CommandError::ExecutionFailed` if the command exits with non-zero status,
    /// or `CommandError::IoError` if SSH execution fails.
    pub fn execute_with_options(
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

    /// Check if a command succeeds with additional SSH options
    ///
    /// Wrapper around [`execute_with_options`] that returns `true` if the command
    /// exits with code 0, `false` otherwise. Ideal for service checks and validation.
    ///
    /// # Arguments
    ///
    /// * `remote_command` - Command to execute on the remote host
    /// * `additional_options` - SSH options (e.g., `["ConnectTimeout=2"]`)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use torrust_tracker_deployer_lib::shared::ssh::{SshClient, SshConfig, SshCredentials};
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use std::path::PathBuf;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/path/to/key"),
    ///     PathBuf::from("/path/to/key.pub"),
    ///     Username::new("user")?,
    /// );
    /// let config = SshConfig::with_default_port(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
    /// );
    /// let client = SshClient::new(config);
    ///
    /// // Quick service check with short timeout
    /// let is_running = client.check_command_with_options(
    ///     "systemctl is-active myservice",
    ///     &["ConnectTimeout=2"]
    /// )?;
    ///
    /// if is_running {
    ///     println!("Service is running");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `CommandError::IoError` if SSH connection fails.
    /// Command failures (non-zero exit) return `Ok(false)`, not an error.
    ///
    /// [`execute_with_options`]: Self::execute_with_options
    pub fn check_command_with_options(
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
}

#[cfg(test)]
mod tests {
    use super::super::SshCredentials;
    use super::*;
    use std::fs;
    use std::net::{IpAddr, Ipv4Addr};
    use tempfile::TempDir;

    use crate::shared::Username;

    /// Helper to create test SSH credentials with temporary key files
    ///
    /// Creates a temporary directory with actual (fake) SSH key files for testing.
    /// The temporary directory is automatically cleaned up when the returned `TempDir` is dropped.
    ///
    /// # Returns
    ///
    /// A tuple of (`TempDir`, `SshCredentials`) where:
    /// - `TempDir` must be kept alive to prevent cleanup during the test
    /// - `SshCredentials` contains paths to the temporary key files
    fn create_test_ssh_credentials() -> (TempDir, SshCredentials) {
        let temp_dir =
            TempDir::new().expect("Failed to create temp directory for SSH key test files");

        let priv_key_path = temp_dir.path().join("test_key");
        let pub_key_path = temp_dir.path().join("test_key.pub");

        // Create actual (empty) key files for realism
        fs::write(&priv_key_path, "fake private key content")
            .expect("Failed to write test private key");
        fs::write(&pub_key_path, "fake public key content")
            .expect("Failed to write test public key");

        let credentials = SshCredentials::new(
            priv_key_path,
            pub_key_path,
            Username::new("testuser").unwrap(),
        );

        (temp_dir, credentials)
    }

    #[test]
    fn it_should_create_ssh_client_with_valid_parameters() {
        // Arrange
        let (_temp_dir, credentials) = create_test_ssh_credentials();
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);

        // Act
        let ssh_client = SshClient::new(ssh_config);

        // Assert
        assert!(ssh_client.ssh_config.ssh_priv_key_path().exists());
        assert!(ssh_client.ssh_config.ssh_pub_key_path().exists());
        assert_eq!(ssh_client.ssh_config.ssh_username(), "testuser");
        assert_eq!(ssh_client.ssh_config.host_ip(), host_ip);
        // Note: verbose is now encapsulated in the CommandExecutor collaborator

        // TempDir automatically cleans up when dropped
    }

    #[test]
    fn it_should_create_ssh_client_with_connection_details() {
        // Arrange
        let (_temp_dir, credentials) = create_test_ssh_credentials();
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);

        // Act
        let ssh_client = SshClient::new(ssh_config);

        // Assert
        assert!(ssh_client.ssh_config.ssh_priv_key_path().exists());
        assert!(ssh_client.ssh_config.ssh_pub_key_path().exists());
        assert_eq!(ssh_client.ssh_config.ssh_username(), "testuser");
        assert_eq!(ssh_client.ssh_config.host_ip(), host_ip);
        // Note: logging is now handled by the tracing crate via CommandExecutor

        // TempDir automatically cleans up when dropped
    }

    #[test]
    fn it_should_detect_ssh_warnings_in_stderr() {
        // Arrange
        let (_temp_dir, credentials) = create_test_ssh_credentials();
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
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

        // TempDir automatically cleans up when dropped
    }

    #[test]
    fn it_should_build_default_ssh_options_as_hashmap() {
        // Arrange
        let (_temp_dir, credentials) = create_test_ssh_credentials();
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);
        let expected_timeout = ssh_config.connection_config.connect_timeout_secs;
        let ssh_client = SshClient::new(ssh_config);

        // Act
        let default_options = ssh_client.build_default_ssh_options();

        // Assert: Should contain 3 key-value pairs
        assert_eq!(default_options.len(), 3);

        // Verify expected keys and values
        assert_eq!(
            default_options.get("StrictHostKeyChecking"),
            Some(&"no".to_string())
        );
        assert_eq!(
            default_options.get("UserKnownHostsFile"),
            Some(&"/dev/null".to_string())
        );
        assert_eq!(
            default_options.get("ConnectTimeout"),
            Some(&expected_timeout.to_string())
        );
    }

    #[test]
    fn it_should_build_ssh_args_with_user_options_before_defaults() {
        // Arrange
        let (_temp_dir, credentials) = create_test_ssh_credentials();
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);
        let ssh_client = SshClient::new(ssh_config);

        // Act
        let args = ssh_client.build_ssh_args("echo test", &["ServerAliveInterval=60"]);

        // Assert: User options should appear before default options
        let args_string = args.join(" ");

        // Find positions of key options
        let server_alive_pos = args
            .iter()
            .position(|s| s == "ServerAliveInterval=60")
            .expect("ServerAliveInterval should be present");

        let strict_pos = args
            .iter()
            .position(|s| s == "StrictHostKeyChecking=no")
            .expect("StrictHostKeyChecking should be present");

        // User option should come before default option (SSH uses first-occurrence-wins)
        assert!(
            server_alive_pos < strict_pos,
            "User-provided options should appear before defaults for first-occurrence-wins precedence"
        );

        // Verify command structure
        assert!(args_string.contains("-i")); // Private key
        assert!(args_string.contains("StrictHostKeyChecking=no")); // Default option
        assert!(args_string.contains("ServerAliveInterval=60")); // User option
        assert!(args_string.contains("-p")); // Port
        assert!(args_string.contains("testuser@")); // Username
        assert!(args_string.contains("echo test")); // Command
    }

    #[test]
    fn it_should_allow_users_to_override_default_options() {
        // Arrange
        let (_temp_dir, credentials) = create_test_ssh_credentials();
        let host_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ssh_config = SshConfig::with_default_port(credentials, host_ip);
        let ssh_client = SshClient::new(ssh_config);

        // Act: Override default StrictHostKeyChecking
        let args = ssh_client.build_ssh_args("echo test", &["StrictHostKeyChecking=yes"]);

        // Assert: Only user-provided option should be present (not the default)
        let strict_yes_count = args
            .iter()
            .filter(|s| *s == "StrictHostKeyChecking=yes")
            .count();
        let strict_no_count = args
            .iter()
            .filter(|s| *s == "StrictHostKeyChecking=no")
            .count();

        assert_eq!(
            strict_yes_count, 1,
            "User-provided StrictHostKeyChecking=yes should be present"
        );
        assert_eq!(
            strict_no_count, 0,
            "Default StrictHostKeyChecking=no should be excluded when user provides override"
        );
    }
}
