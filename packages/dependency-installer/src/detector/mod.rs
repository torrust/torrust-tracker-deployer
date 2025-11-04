pub mod ansible;
pub mod cargo_machete;
pub mod lxd;
pub mod opentofu;

use thiserror::Error;

pub use ansible::AnsibleDetector;
pub use cargo_machete::CargoMacheteDetector;
pub use lxd::LxdDetector;
pub use opentofu::OpenTofuDetector;

/// Error types for detection operations
#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("Failed to detect tool '{tool}': {source}")]
    DetectionFailed {
        tool: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command execution failed for tool '{tool}': {message}")]
    CommandFailed { tool: String, message: String },
}

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
