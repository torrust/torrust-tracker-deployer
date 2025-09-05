use anyhow::Result;
use std::process::Command;
use tracing::{error, info};

use crate::linting::utils::{install_yamllint, is_command_available};

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
    info!(target: "yaml", "Scanning YAML files...");

    let output = Command::new("yamllint")
        .args(["-c", ".yamllint-ci.yml", "."])
        .output()?;

    if output.status.success() {
        info!(target: "yaml", "All YAML files passed linting!");
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
        error!(target: "yaml", "YAML linting failed. Please fix the issues above.");
        Err(anyhow::anyhow!("YAML linting failed"))
    }
}
