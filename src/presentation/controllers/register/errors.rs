//! Error types for the Register Subcommand
//!
//! This module defines error types that can occur during CLI register command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use std::net::IpAddr;

use thiserror::Error;

use crate::application::command_handlers::register::errors::RegisterCommandHandlerError;
use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::views::progress::ProgressReporterError;

/// Register command specific errors
///
/// This enum contains all error variants specific to the register command,
/// including environment validation, IP address parsing, and registration failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum RegisterSubcommandError {
    // ===== Environment Validation Errors =====
    /// Environment name validation failed
    #[error("Invalid environment name '{name}': {source}
Tip: Environment names must be 1-63 characters, start with letter/digit, contain only letters/digits/hyphens")]
    InvalidEnvironmentName {
        name: String,
        #[source]
        source: EnvironmentNameError,
    },

    // ===== IP Address Errors =====
    /// Invalid IP address format
    #[error(
        "Invalid IP address '{value}': {reason}
Tip: Use IPv4 (e.g., 192.168.1.100) or IPv6 (e.g., 2001:db8::1) format without port numbers"
    )]
    InvalidIpAddress { value: String, reason: String },

    // ===== Register Operation Errors =====
    /// Register operation failed
    #[error(
        "Failed to register instance with environment '{name}': {source}
Tip: Check logs and try running with --log-output file-and-stderr for more details"
    )]
    RegisterOperationFailed {
        name: String,
        #[source]
        source: Box<RegisterCommandHandlerError>,
    },

    // ===== Connectivity Errors =====
    /// SSH connectivity validation failed
    #[error(
        "Cannot connect to instance at {address}: {reason}
Tip: Verify the instance is running and SSH is accessible"
    )]
    ConnectivityFailed { address: IpAddr, reason: String },

    // ===== Internal Errors =====
    /// Progress reporting failed
    #[error(
        "Failed to report progress: {source}
Tip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    ProgressReportingFailed {
        #[source]
        source: ProgressReporterError,
    },
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for RegisterSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl RegisterSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::controllers::register::errors::RegisterSubcommandError;
    ///
    /// let error = RegisterSubcommandError::InvalidIpAddress {
    ///     value: "not-an-ip".to_string(),
    ///     reason: "invalid format".to_string(),
    /// };
    /// let help = error.help();
    /// assert!(help.contains("IP address"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName { .. } => {
                "Invalid Environment Name - Detailed Troubleshooting:

1. Check environment name format:
   - Length: Must be 1-63 characters
   - Start: Must begin with a letter or digit
   - Characters: Only letters, digits, and hyphens allowed
   - No special characters: Avoid spaces, underscores, dots

2. Valid examples:
   - 'production'
   - 'staging-01'
   - 'dev-environment'

3. Invalid examples:
   - 'prod_01' (underscore not allowed)
   - '-production' (cannot start with hyphen)
   - 'prod.env' (dots not allowed)

For more information, see environment naming documentation."
            }

            Self::InvalidIpAddress { .. } => {
                "Invalid IP Address - Detailed Troubleshooting:

1. Check IP address format:
   - IPv4: Use dotted decimal notation (e.g., 192.168.1.100)
   - IPv6: Use colon-separated hexadecimal (e.g., 2001:db8::1)

2. Common mistakes:
   - Including port number (use just IP, not IP:22)
   - Extra whitespace or quotes
   - Using hostname instead of IP address

3. Examples:
   - Valid IPv4: 192.168.1.100, 10.0.0.1
   - Valid IPv6: 2001:db8::1, ::1, fe80::1
   - Invalid: 192.168.1.100:22, localhost, my-server.local

Usage:
  torrust-tracker-deployer register my-env --instance-ip 192.168.1.100"
            }

            Self::RegisterOperationFailed { .. } => {
                "Register Operation Failed - Detailed Troubleshooting:

1. Check environment state:
   - The environment must be in 'Created' state
   - If already provisioned, use 'configure' command instead
   - To re-register, destroy and recreate the environment

2. Verify instance accessibility:
   - Instance is running and reachable
   - SSH service is running on the instance
   - Firewall allows SSH connections

3. Check SSH credentials:
   - SSH key matches the one configured in the environment
   - Public key is installed on the instance
   - Username has access to the instance

4. Enable verbose logging for more details:
   torrust-tracker-deployer --log-output file-and-stderr register <env> --instance-ip <ip>

For more troubleshooting, see docs/debugging.md"
            }

            Self::ConnectivityFailed { .. } => {
                "SSH Connectivity Failed - Detailed Troubleshooting:

1. Verify instance is reachable:
   ping <instance-ip>

2. Verify SSH port is open:
   nc -zv <instance-ip> 22

3. Test SSH manually:
   ssh -i <key-path> <user>@<instance-ip>

4. Common issues:
   - Instance not running: Start the VM/container
   - SSH not installed: Install openssh-server on the instance
   - Firewall blocking: Allow port 22 in firewall rules
   - Wrong key: Ensure public key is in ~/.ssh/authorized_keys
   - Key permissions: chmod 600 <private-key>

5. Check environment configuration:
   - Verify SSH username matches instance user
   - Verify SSH key path is correct
   - Verify SSH port matches instance configuration

For SSH setup, see docs/debugging.md"
            }

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Internal Error:

This error indicates a critical internal bug in the progress reporting system.

1. Try running the command again
2. If the issue persists, please report it as a bug

When reporting:
- Include the full error message
- Run with verbose logging: --log-output file-and-stderr
- Include the log file from data/logs/"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod error_construction {
        use super::*;

        #[test]
        fn it_should_create_invalid_ip_address_error() {
            let error = RegisterSubcommandError::InvalidIpAddress {
                value: "not-an-ip".to_string(),
                reason: "invalid format".to_string(),
            };

            assert!(error.to_string().contains("not-an-ip"));
            assert!(error.to_string().contains("invalid format"));
        }
    }

    mod help_methods {
        use super::*;

        #[test]
        fn it_should_provide_help_for_all_error_variants() {
            use std::net::{IpAddr, Ipv4Addr};

            use crate::domain::environment::name::EnvironmentNameError;

            let errors: Vec<RegisterSubcommandError> = vec![
                RegisterSubcommandError::InvalidEnvironmentName {
                    name: String::new(),
                    source: EnvironmentNameError::Empty,
                },
                RegisterSubcommandError::InvalidIpAddress {
                    value: "bad".to_string(),
                    reason: "test".to_string(),
                },
                RegisterSubcommandError::ConnectivityFailed {
                    address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                    reason: "connection refused".to_string(),
                },
            ];

            for error in errors {
                let help = error.help();
                assert!(!help.is_empty(), "Help should not be empty for: {error}");
                assert!(
                    help.contains("Troubleshooting") || help.contains("Error"),
                    "Help should contain guidance for: {error}"
                );
            }
        }
    }
}
