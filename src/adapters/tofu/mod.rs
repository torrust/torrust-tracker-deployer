//! `OpenTofu` infrastructure management wrapper
//!
//! This module provides a comprehensive interface for managing infrastructure using
//! `OpenTofu` (the open-source Terraform fork), including plan, apply, destroy operations
//! and JSON output parsing for instance information.
//!
//! ## Module Structure
//!
//! - `client` - Main `OpenTofuClient` for executing `OpenTofu` commands
//! - `json_parser` - JSON output parsing for `OpenTofu` state and plan information
//!
//! ## Key Features
//!
//! - Full infrastructure lifecycle management (init, plan, apply, destroy)
//! - State management and instance information extraction
//! - Emergency cleanup operations for testing scenarios
//! - Comprehensive error handling with detailed context

use std::path::Path;

pub mod client;
pub mod json_parser;

// Re-export the main types for easier access
pub use client::{InstanceInfo, OpenTofuClient, OpenTofuError};
pub use json_parser::ParseError;

/// Errors that can occur during emergency destroy operations
#[derive(Debug)]
pub enum EmergencyDestroyError {
    /// Command execution failed (e.g., tofu binary not found)
    CommandExecution { source: std::io::Error },

    /// `OpenTofu` destroy operation failed with error output
    DestroyFailed { stderr: String },
}

impl std::fmt::Display for EmergencyDestroyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandExecution { source } => {
                write!(f, "Failed to execute OpenTofu destroy command: {source}")
            }
            Self::DestroyFailed { stderr } => {
                write!(f, "OpenTofu destroy failed: {stderr}")
            }
        }
    }
}

impl std::error::Error for EmergencyDestroyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommandExecution { source } => Some(source),
            Self::DestroyFailed { .. } => None,
        }
    }
}

/// Emergency destroy operation for cleanup scenarios
///
/// This function performs a destructive `OpenTofu` destroy operation without prompting.
/// It's designed for use in Drop implementations and other cleanup scenarios where
/// interactive confirmation is not possible.
///
/// # Arguments
///
/// * `working_dir` - Directory containing the `OpenTofu` configuration files
///
/// # Returns
///
/// * `Result<(), EmergencyDestroyError>` - Success or concrete error from the destroy operation
///
/// # Errors
///
/// Returns an error if the `OpenTofu` destroy command fails or if there are issues
/// with command execution.
pub fn emergency_destroy<P: AsRef<Path>>(working_dir: P) -> Result<(), EmergencyDestroyError> {
    use std::process::Command;

    tracing::debug!(
        "Emergency destroy: Executing `OpenTofu` destroy in directory: {}",
        working_dir.as_ref().display()
    );

    let output = Command::new("tofu")
        .args(["destroy", "-auto-approve"])
        .current_dir(&working_dir)
        .output()
        .map_err(|source| EmergencyDestroyError::CommandExecution { source })?;

    if output.status.success() {
        tracing::debug!("Emergency destroy: `OpenTofu` destroy completed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        tracing::error!("Emergency destroy: `OpenTofu` destroy failed: {stderr}");
        Err(EmergencyDestroyError::DestroyFailed { stderr })
    }
}
