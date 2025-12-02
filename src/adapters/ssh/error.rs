//! SSH error types and implementations
//!
//! This module defines the error types that can occur during SSH operations,
//! including connectivity timeouts and command execution failures.

use thiserror::Error;

use crate::shared::command::CommandError;

/// Errors that can occur during SSH operations
#[derive(Error, Debug)]
pub enum SshError {
    /// Failed to establish SSH connectivity within timeout period
    ///
    /// This typically means the SSH service is not yet available or the
    /// instance is still booting. Use `.help()` for detailed troubleshooting.
    #[error("Failed to establish SSH connectivity to {host_ip} after {attempts} attempts ({timeout_seconds}s total)
Tip: Check if instance is fully booted and SSH service is running")]
    ConnectivityTimeout {
        host_ip: String,
        attempts: u32,
        timeout_seconds: u32,
    },

    /// SSH command execution failed
    ///
    /// The underlying SSH command execution encountered an error.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "SSH command execution failed: {source}
Tip: Check command syntax and remote host permissions"
    )]
    CommandFailed {
        #[source]
        source: CommandError,
    },
}

impl SshError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::adapters::ssh::SshError;
    ///
    /// let error = SshError::ConnectivityTimeout {
    ///     host_ip: "192.168.1.100".to_string(),
    ///     attempts: 30,
    ///     timeout_seconds: 60,
    /// };
    ///
    /// // Display brief error
    /// eprintln!("Error: {error}");
    ///
    /// // Display detailed help when needed
    /// eprintln!("\nTroubleshooting:\n{}", error.help());
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::ConnectivityTimeout { .. } => {
                "SSH Connectivity Timeout - Detailed Troubleshooting:

1. Verify the instance is running:
   - Check VM/server status using your provider tools
   - Ensure instance has finished booting (may take 30-60s)

2. Check SSH service status:
   - SSH into the server and run: systemctl status ssh
   - Or check console logs for cloud instances

3. Verify network connectivity:
   - Ping the IP address: ping <host_ip>
   - Check firewall rules allow port 22
   - Verify no network issues between hosts

4. Check SSH configuration:
   - Ensure SSH service is enabled on boot
   - Verify sshd_config allows key authentication
   - Check SSH key permissions (should be 600 or 400)

5. Try manual connection to see specific error:
   ssh -i <key_path> -o ConnectTimeout=5 -o StrictHostKeyChecking=no <user>@<host_ip>

6. Increase timeout if needed:
   - Slow networks may need more time
   - Use custom SshConnectionConfig with higher max_retry_attempts or retry_interval_secs

For more information, see the SSH troubleshooting documentation."
            }

            Self::CommandFailed { .. } => {
                "SSH Command Failed - Detailed Troubleshooting:

1. Check the underlying command error for specific details
   - Review the error message for hints about what went wrong
   - Common issues: command not found, permission denied, syntax errors

2. Verify SSH authentication is working:
   - Test connection: ssh <user>@<host> 'echo test'
   - Check SSH key permissions (should be 600 or 400)
   - Verify user has proper access on remote host

3. Ensure remote command is valid:
   - Test command directly on remote host first
   - Check for typos in command syntax
   - Verify required tools/packages are installed

4. Check for permission issues:
   - Does the SSH user have sufficient privileges?
   - Try with sudo if appropriate: ssh <user>@<host> 'sudo command'
   - Review remote host logs for access denied messages

5. Debug with verbose SSH output:
   ssh -vvv <user>@<host> '<command>'

For more information, see the command execution documentation."
            }
        }
    }
}

impl crate::shared::Traceable for SshError {
    fn trace_format(&self) -> String {
        match self {
            Self::ConnectivityTimeout {
                host_ip,
                attempts,
                timeout_seconds,
            } => {
                format!("SshError: Connectivity timeout to '{host_ip}' after {attempts} attempts ({timeout_seconds} seconds)")
            }
            Self::CommandFailed { source } => {
                format!("SshError: SSH command failed - {source}")
            }
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::ConnectivityTimeout { .. } => None,
            Self::CommandFailed { source } => Some(source),
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        crate::shared::ErrorKind::NetworkConnectivity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod error_messages {
        use super::*;

        #[test]
        fn it_should_include_context_in_connectivity_timeout_error() {
            let error = SshError::ConnectivityTimeout {
                host_ip: "192.168.1.100".to_string(),
                attempts: 30,
                timeout_seconds: 60,
            };

            let message = error.to_string();
            assert!(message.contains("192.168.1.100"));
            assert!(message.contains("30 attempts"));
            assert!(message.contains("60s total"));
        }

        #[test]
        fn it_should_include_brief_tip_in_connectivity_timeout_error() {
            let error = SshError::ConnectivityTimeout {
                host_ip: "10.0.0.1".to_string(),
                attempts: 5,
                timeout_seconds: 10,
            };

            let message = error.to_string();
            assert!(message.contains("Tip:"));
            assert!(message.contains("instance is fully booted"));
        }

        #[test]
        fn it_should_include_brief_tip_in_command_failed_error() {
            let cmd_error = CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            };

            let error = SshError::CommandFailed { source: cmd_error };

            let message = error.to_string();
            assert!(message.contains("Tip:"));
            assert!(message.contains("command syntax"));
        }
    }

    mod help_methods {
        use super::*;

        #[test]
        fn it_should_provide_detailed_help_for_connectivity_timeout() {
            let error = SshError::ConnectivityTimeout {
                host_ip: "192.168.1.100".to_string(),
                attempts: 30,
                timeout_seconds: 60,
            };

            let help = error.help();

            // Verify key troubleshooting steps are present
            assert!(help.contains("Verify the instance is running"));
            assert!(help.contains("Check SSH service status"));
            assert!(help.contains("Verify network connectivity"));
            assert!(help.contains("Check SSH configuration"));
            assert!(help.contains("Try manual connection"));
            assert!(help.contains("Increase timeout if needed"));
        }

        #[test]
        fn it_should_include_actionable_commands_in_connectivity_help() {
            let error = SshError::ConnectivityTimeout {
                host_ip: "10.0.0.1".to_string(),
                attempts: 10,
                timeout_seconds: 20,
            };

            let help = error.help();

            // Verify actionable commands are present
            assert!(help.contains("systemctl status ssh"));
            assert!(help.contains("ping <host_ip>"));
            assert!(help.contains("ssh -i"));
            assert!(help.contains("SshConnectionConfig"));
        }

        #[test]
        fn it_should_provide_detailed_help_for_command_failed() {
            let cmd_error = CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            };

            let error = SshError::CommandFailed { source: cmd_error };
            let help = error.help();

            // Verify key troubleshooting steps are present
            assert!(help.contains("Check the underlying command error"));
            assert!(help.contains("Verify SSH authentication"));
            assert!(help.contains("Ensure remote command is valid"));
            assert!(help.contains("Check for permission issues"));
            assert!(help.contains("Debug with verbose SSH output"));
        }

        #[test]
        fn it_should_include_actionable_commands_in_command_failed_help() {
            let cmd_error = CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            };

            let error = SshError::CommandFailed { source: cmd_error };
            let help = error.help();

            // Verify actionable commands are present
            assert!(help.contains("ssh <user>@<host> 'echo test'"));
            assert!(help.contains("sudo"));
            assert!(help.contains("ssh -vvv"));
        }

        #[test]
        fn it_should_provide_help_for_all_error_variants() {
            // ConnectivityTimeout
            let error1 = SshError::ConnectivityTimeout {
                host_ip: "192.168.1.1".to_string(),
                attempts: 5,
                timeout_seconds: 10,
            };
            assert!(!error1.help().is_empty());

            // CommandFailed
            let cmd_error = CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            };
            let error2 = SshError::CommandFailed { source: cmd_error };
            assert!(!error2.help().is_empty());
        }
    }

    mod error_display {
        use super::*;

        #[test]
        fn it_should_implement_display_trait() {
            let error = SshError::ConnectivityTimeout {
                host_ip: "192.168.1.100".to_string(),
                attempts: 30,
                timeout_seconds: 60,
            };

            let display = format!("{error}");
            assert!(!display.is_empty());
        }

        #[test]
        fn it_should_implement_debug_trait() {
            let error = SshError::ConnectivityTimeout {
                host_ip: "192.168.1.100".to_string(),
                attempts: 30,
                timeout_seconds: 60,
            };

            let debug = format!("{error:?}");
            assert!(!debug.is_empty());
        }
    }

    mod error_source_chaining {
        use super::*;
        use std::error::Error;

        #[test]
        fn it_should_preserve_source_error_for_command_failed() {
            let cmd_error = CommandError::ExecutionFailed {
                command: "test".to_string(),
                exit_code: "1".to_string(),
                stdout: String::new(),
                stderr: "error".to_string(),
            };

            let error = SshError::CommandFailed { source: cmd_error };

            // Verify source is preserved
            assert!(error.source().is_some());
        }

        #[test]
        fn it_should_have_no_source_for_connectivity_timeout() {
            let error = SshError::ConnectivityTimeout {
                host_ip: "192.168.1.100".to_string(),
                attempts: 30,
                timeout_seconds: 60,
            };

            // Verify no source for this error type
            assert!(error.source().is_none());
        }
    }
}
