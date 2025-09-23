use anyhow::Result;
use std::process::Command;
use tracing::{error, info};

/// Check if a command is available in the system PATH
///
/// # Errors
///
/// Returns false if the command is not found or if the check fails.
#[must_use]
pub fn is_command_available(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Install a tool using npm globally
///
/// # Errors
///
/// Returns an error if npm is not available or if the installation fails.
pub fn install_npm_tool(tool: &str) -> Result<()> {
    info!("Installing {tool}...");

    let output = Command::new("npm").args(["install", "-g", tool]).output()?;

    if output.status.success() {
        info!("{tool} installed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Failed to install {tool}: {stderr}");
        Err(anyhow::anyhow!("Failed to install {tool}"))
    }
}
