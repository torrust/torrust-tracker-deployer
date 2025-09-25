//! Pre-flight cleanup module for E2E tests
//!
//! This module provides functionality to clean up any lingering resources
//! from previous test runs that may have been interrupted before cleanup.
//!
//! ## Migration Notice
//!
//! This module now serves as a compatibility layer that re-exports functions
//! from the new modularized structure:
//! - Container-specific functions are in `container::preflight_cleanup`
//! - VM-specific functions are in `virtual_machine::preflight_cleanup`  
//! - Common directory cleanup functions are in `preflight_cleanup_common`

use std::fmt;

use crate::infrastructure::adapters::opentofu::EmergencyDestroyError;

// Re-export functions from the new modular structure for backward compatibility
pub use crate::e2e::tasks::container::preflight_cleanup::cleanup_lingering_resources_docker;
pub use crate::e2e::tasks::virtual_machine::preflight_cleanup::cleanup_lingering_resources;

/// Errors that can occur during pre-flight cleanup operations
#[derive(Debug)]
pub enum PreflightCleanupError {
    /// Emergency destroy operation failed
    EmergencyDestroyFailed { source: EmergencyDestroyError },

    /// Resource conflicts detected that would prevent new test runs
    ResourceConflicts { details: String },
}

impl fmt::Display for PreflightCleanupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmergencyDestroyFailed { source } => {
                write!(f, "Emergency destroy operation failed: {source}")
            }
            Self::ResourceConflicts { details } => {
                write!(
                    f,
                    "Resource conflicts detected that would prevent new test runs: {details}"
                )
            }
        }
    }
}

impl std::error::Error for PreflightCleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EmergencyDestroyFailed { source } => Some(source),
            Self::ResourceConflicts { .. } => None,
        }
    }
}
