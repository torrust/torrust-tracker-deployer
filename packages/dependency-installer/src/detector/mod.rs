pub mod ansible;
pub mod cargo_machete;
pub mod lxd;
pub mod opentofu;

use thiserror::Error;

use crate::Dependency;

pub use ansible::AnsibleDetector;
pub use cargo_machete::CargoMacheteDetector;
pub use lxd::LxdDetector;
pub use opentofu::OpenTofuDetector;

/// Error types for detection operations
#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("Failed to detect dependency '{dependency}': {source}")]
    DetectionFailed {
        dependency: Dependency,
        #[source]
        source: std::io::Error,
    },

    #[error("Command execution failed for dependency '{dependency}': {message}")]
    CommandFailed {
        dependency: Dependency,
        message: String,
    },
}

/// Trait for detecting if a dependency is installed
pub trait DependencyDetector {
    /// Get the dependency name for display purposes
    fn name(&self) -> &'static str;

    /// Check if the dependency is already installed
    ///
    /// # Errors
    ///
    /// Returns an error if the detection process fails
    fn is_installed(&self) -> Result<bool, DetectionError>;

    /// Get the required version (if applicable)
    fn required_version(&self) -> Option<&str> {
        None // Default implementation
    }
}
