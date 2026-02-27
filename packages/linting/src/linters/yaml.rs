use anyhow::Result;
use std::process::Command;
use std::time::Instant;
use tracing::{error, info, warn};

use crate::utils::is_command_available;

/// Install yamllint using system package manager
///
/// # Errors
///
/// Returns an error if no supported package manager is found or if installation fails.
fn install_yamllint() -> Result<()> {
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

/// Run the YAML linter
///
/// # Errors
///
/// Returns an error if yamllint is not available, cannot be installed,
/// or if the linting fails.
pub fn run_yaml_linter() -> Result<()> {
    // Check if yamllint is installed
    if !is_command_available("yamllint") {
        install_yamllint()?;
    }

    // Run the linter with the configuration file
    let t = Instant::now();
    info!(target: "yaml", "Scanning YAML files...");

    let output = Command::new("yamllint")
        .args(["-c", ".yamllint-ci.yml", "."])
        .output()?;

    if output.status.success() {
        info!(target: "yaml", "All YAML files passed linting! ({:.3}s)", t.elapsed().as_secs_f64());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Print the output from yamllint
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        error!(target: "yaml", "YAML linting failed. Please fix the issues above. ({:.3}s)", t.elapsed().as_secs_f64());
        Err(anyhow::anyhow!("YAML linting failed"))
    }
}
