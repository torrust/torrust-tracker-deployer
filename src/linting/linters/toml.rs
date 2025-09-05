use anyhow::Result;
use std::process::Command;
use tracing::{error, info};

use crate::linting::utils::{install_taplo, is_command_available};

/// Run the TOML linter using Taplo
///
/// # Errors
///
/// Returns an error if Taplo is not available, cannot be installed,
/// or if the linting fails.
pub fn run_toml_linter() -> Result<()> {
    // Check if taplo is installed
    if !is_command_available("taplo") {
        install_taplo()?;
    }

    info!(target: "toml", "Scanning TOML files...");

    // Run taplo check with recursive glob pattern
    let check_output = Command::new("taplo")
        .args(["check", "**/*.toml"])
        .output()?;

    if !check_output.status.success() {
        let stderr = String::from_utf8_lossy(&check_output.stderr);
        let stdout = String::from_utf8_lossy(&check_output.stdout);

        // Print the output from taplo
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        error!(target: "toml", "TOML linting failed. Please fix the issues above.");
        return Err(anyhow::anyhow!("TOML linting failed"));
    }

    // Run taplo format check with recursive glob pattern
    let format_output = Command::new("taplo")
        .args(["fmt", "--check", "**/*.toml"])
        .output()?;

    if format_output.status.success() {
        info!(target: "toml", "All TOML files passed linting and formatting checks!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&format_output.stderr);
        let stdout = String::from_utf8_lossy(&format_output.stdout);

        // Print the output from taplo
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        error!(target: "toml", "TOML formatting failed. Please fix the issues above.");
        error!(target: "toml", "Run 'taplo fmt **/*.toml' to auto-fix formatting issues.");
        Err(anyhow::anyhow!("TOML formatting failed"))
    }
}
