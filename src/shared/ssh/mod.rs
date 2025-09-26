//! SSH wrapper module for secure remote operations
//!
//! This module provides comprehensive SSH functionality for secure remote command
//! execution and connectivity management. It includes credential management,
//! connection configuration, and client implementation with proper error handling.
//!
//! ## Module Components
//!
//! - `client` - SSH client implementation for remote command execution
//! - `connection` - SSH connection configuration and management
//! - `credentials` - SSH authentication credentials and key management
//! - `public_key` - SSH public key representation and validation
//! - `service_checker` - SSH service availability testing without authentication
//!
//! ## Key Features
//!
//! - Private key authentication with configurable credentials
//! - Connection timeout and retry mechanisms
//! - Secure remote command execution with error handling
//! - SSH service availability checking for connectivity testing
//! - Integration with deployment automation workflows
//!
//! The SSH wrapper is designed for automated deployment scenarios where
//! secure remote access is essential for configuration and management tasks.

pub mod client;
pub mod connection;
pub mod credentials;
pub mod public_key;
pub mod service_checker;

pub use client::SshClient;
pub use connection::SshConnection;
pub use credentials::SshCredentials;
pub use public_key::SshPublicKey;
pub use service_checker::SshServiceChecker;

use thiserror::Error;

use crate::shared::executor::CommandError;

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
