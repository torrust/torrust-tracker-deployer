//! SSH error types and implementations
//!
//! This module defines the error types that can occur during SSH operations,
//! including connectivity timeouts and command execution failures.

use thiserror::Error;

use crate::shared::command::CommandError;

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
