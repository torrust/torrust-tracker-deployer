use anyhow::Result;
use std::process::Command;
use tracing::{error, info, warn};

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
    info!("Installing {}...", tool);

    let output = Command::new("npm").args(["install", "-g", tool]).output()?;

    if output.status.success() {
        info!("{} installed successfully", tool);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Failed to install {}: {}", tool, stderr);
        Err(anyhow::anyhow!("Failed to install {}", tool))
    }
}

/// Install yamllint using system package manager
///
/// # Errors
///
/// Returns an error if no supported package manager is found or if installation fails.
pub fn install_yamllint() -> Result<()> {
    info!("Installing yamllint...");

    // Try different package managers
    if is_command_available("apt-get") {
        let output = Command::new("sudo").args(["apt-get", "update"]).output()?;

        if !output.status.success() {
            warn!("Failed to update package list");
        }

        let output = Command::new("sudo")
            .args(["apt-get", "install", "-y", "yamllint"])
            .output()?;

        if output.status.success() {
            info!("yamllint installed successfully");
            return Ok(());
        }
    } else if is_command_available("dnf") {
        let output = Command::new("sudo")
            .args(["dnf", "install", "-y", "yamllint"])
            .output()?;

        if output.status.success() {
            info!("yamllint installed successfully");
            return Ok(());
        }
    } else if is_command_available("pacman") {
        let output = Command::new("sudo")
            .args(["pacman", "-S", "--noconfirm", "yamllint"])
            .output()?;

        if output.status.success() {
            info!("yamllint installed successfully");
            return Ok(());
        }
    } else if is_command_available("pip3") {
        let output = Command::new("pip3")
            .args(["install", "--user", "yamllint"])
            .output()?;

        if output.status.success() {
            info!("yamllint installed successfully");
            return Ok(());
        }
    }

    error!("Could not install yamllint. Please install it manually.");
    Err(anyhow::anyhow!("Could not install yamllint"))
}

/// Install shellcheck using system package manager
///
/// # Errors
///
/// Returns an error if no supported package manager is found or if installation fails.
pub fn install_shellcheck() -> Result<()> {
    info!("Installing ShellCheck...");

    // Try different package managers
    if is_command_available("apt-get") {
        let output = Command::new("sudo").args(["apt-get", "update"]).output()?;

        if !output.status.success() {
            warn!("Failed to update package list");
        }

        let output = Command::new("sudo")
            .args(["apt-get", "install", "-y", "shellcheck"])
            .output()?;

        if output.status.success() {
            info!("shellcheck installed successfully");
            return Ok(());
        }
    } else if is_command_available("dnf") {
        let output = Command::new("sudo")
            .args(["dnf", "install", "-y", "ShellCheck"])
            .output()?;

        if output.status.success() {
            info!("shellcheck installed successfully");
            return Ok(());
        }
    } else if is_command_available("pacman") {
        let output = Command::new("sudo")
            .args(["pacman", "-S", "--noconfirm", "shellcheck"])
            .output()?;

        if output.status.success() {
            info!("shellcheck installed successfully");
            return Ok(());
        }
    } else if is_command_available("brew") {
        let output = Command::new("brew")
            .args(["install", "shellcheck"])
            .output()?;

        if output.status.success() {
            info!("shellcheck installed successfully");
            return Ok(());
        }
    }

    error!("Could not install shellcheck: unsupported package manager");
    info!("Please install shellcheck manually: https://github.com/koalaman/shellcheck#installing");
    Err(anyhow::anyhow!("Could not install shellcheck"))
}
