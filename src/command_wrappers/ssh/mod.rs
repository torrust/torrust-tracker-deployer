pub mod client;
pub mod connection;
pub mod credentials;

pub use client::SshClient;
pub use connection::SshConnection;
pub use credentials::SshCredentials;

use thiserror::Error;

use crate::command::CommandError;

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
