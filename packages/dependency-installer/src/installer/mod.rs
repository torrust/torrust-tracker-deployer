//! Dependency installation system
//!
//! This module provides a trait-based system for installing dependencies,
//! along with implementations for specific tools.

pub mod ansible;
pub mod cargo_machete;
pub mod lxd;
pub mod opentofu;

// External crates
use async_trait::async_trait;
use thiserror::Error;

// Internal crate
use crate::Dependency;

pub use ansible::AnsibleInstaller;
pub use cargo_machete::CargoMacheteInstaller;
pub use lxd::LxdInstaller;
pub use opentofu::OpenTofuInstaller;

// ============================================================================
// PUBLIC API - Traits
// ============================================================================

/// Trait for installing a dependency
#[async_trait]
pub trait DependencyInstaller: Send + Sync {
    /// Get the dependency name for display purposes
    fn name(&self) -> &'static str;

    /// Get the dependency enum value
    fn dependency(&self) -> Dependency;

    /// Install the dependency
    ///
    /// # Errors
    ///
    /// Returns an error if the installation process fails
    async fn install(&self) -> Result<(), InstallationError>;

    /// Check if the installer requires sudo privileges
    fn requires_sudo(&self) -> bool {
        false // Default implementation
    }
}

// ============================================================================
// ERROR TYPES - Secondary Concerns
// ============================================================================

/// Error types for installation operations
#[derive(Debug, Error)]
pub enum InstallationError {
    #[error("Failed to install dependency '{dependency}': {message}")]
    InstallationFailed {
        dependency: Dependency,
        message: String,
    },

    #[error("Command execution failed for dependency '{dependency}': {source}")]
    CommandFailed {
        dependency: Dependency,
        #[source]
        source: std::io::Error,
    },

    #[error("Installation requires sudo privileges but sudo is not available")]
    SudoNotAvailable,
}
