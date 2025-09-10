use std::path::Path;

pub mod client;
pub mod json_parser;

// Re-export the main types for easier access
pub use client::{InstanceInfo, OpenTofuClient, OpenTofuError};
pub use json_parser::ParseError;

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
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error from the destroy operation
///
/// # Errors
///
/// Returns an error if the `OpenTofu` destroy command fails or if there are issues
/// with command execution.
pub fn emergency_destroy<P: AsRef<Path>>(working_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    tracing::debug!(
        "Emergency destroy: Executing `OpenTofu` destroy in directory: {}",
        working_dir.as_ref().display()
    );

    let output = Command::new("tofu")
        .args(["destroy", "-auto-approve"])
        .current_dir(&working_dir)
        .output()?;

    if output.status.success() {
        tracing::debug!("Emergency destroy: `OpenTofu` destroy completed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Emergency destroy: `OpenTofu` destroy failed: {stderr}");
        Err(format!("`OpenTofu` destroy failed: {stderr}").into())
    }
}
