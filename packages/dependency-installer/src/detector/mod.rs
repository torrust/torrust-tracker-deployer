pub mod ansible;
pub mod cargo_machete;
pub mod lxd;
pub mod opentofu;

use crate::errors::DetectionError;

pub use ansible::AnsibleDetector;
pub use cargo_machete::CargoMacheteDetector;
pub use lxd::LxdDetector;
pub use opentofu::OpenTofuDetector;

/// Trait for detecting if a tool is installed
pub trait ToolDetector {
    /// Get the tool name for display purposes
    fn name(&self) -> &'static str;

    /// Check if the tool is already installed
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
