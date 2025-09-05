use anyhow::Result;
use std::process::Command;
use tracing::{error, info};

use crate::utils::{install_npm_tool, is_command_available};

/// Run the markdown linter
///
/// # Errors
///
/// Returns an error if markdownlint is not available, cannot be installed,
/// or if the linting fails.
pub fn run_markdown_linter() -> Result<()> {
    // Check if markdownlint is installed
    if !is_command_available("markdownlint") {
        install_npm_tool("markdownlint-cli")?;
    }

    // Run the linter
    info!(target: "markdown", "Scanning markdown files...");

    // Find all markdown files, excluding terraform directories (like the bash version)
    let find_output = Command::new("find")
        .args([
            ".",
            "-name",
            "*.md",
            "-type",
            "f",
            "-not",
            "-path",
            "*/.terraform/*",
        ])
        .output()?;

    if !find_output.status.success() {
        error!(target: "markdown", "Failed to find markdown files");
        return Err(anyhow::anyhow!("Failed to find markdown files"));
    }

    let files = String::from_utf8_lossy(&find_output.stdout);
    let file_list: Vec<&str> = files.lines().filter(|s| !s.is_empty()).collect();

    if file_list.is_empty() {
        info!(target: "markdown", "No markdown files found");
        return Ok(());
    }

    // Run markdownlint on all found files
    let mut cmd = Command::new("markdownlint");
    cmd.args(&file_list);

    let output = cmd.output()?;

    if output.status.success() {
        info!(target: "markdown", "All markdown files passed linting!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Print the output from markdownlint (it usually goes to stdout)
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        error!(target: "markdown", "Markdown linting failed. Please fix the issues above.");
        Err(anyhow::anyhow!("Markdown linting failed"))
    }
}
