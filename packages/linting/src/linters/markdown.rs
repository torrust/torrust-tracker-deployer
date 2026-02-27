use anyhow::Result;
use std::env;
use std::process::Command;
use std::time::Instant;
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

    // Get the current working directory (should be repo root)
    let repo_root = env::current_dir()?;
    let config_path = repo_root.join(".markdownlint.json");

    // Run the linter
    let t = Instant::now();
    info!(target: "markdown", "Scanning markdown files...");

    // Find all markdown files, excluding terraform and target directories
    let find_output = Command::new("find")
        .current_dir(&repo_root)
        .args([
            ".",
            "-name",
            "*.md",
            "-type",
            "f",
            "-not",
            "-path",
            "*/.terraform/*",
            "-not",
            "-path",
            "./target/*",
        ])
        .output()?;

    if !find_output.status.success() {
        error!(target: "markdown", "Failed to find markdown files");
        return Err(anyhow::anyhow!("Failed to find markdown files"));
    }

    let files = String::from_utf8_lossy(&find_output.stdout);
    let file_list: Vec<&str> = files.lines().filter(|s| !s.is_empty()).collect();

    // Run markdownlint on all found files
    let mut cmd = Command::new("markdownlint");
    cmd.current_dir(&repo_root);
    cmd.arg("--config").arg(&config_path);
    cmd.arg("--dot"); // Include files in dot directories like .github/
    cmd.args(&file_list);

    let output = cmd.output()?;

    if output.status.success() {
        info!(target: "markdown", "All markdown files passed linting! ({:.3}s)", t.elapsed().as_secs_f64());
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
        error!(target: "markdown", "Markdown linting failed. Please fix the issues above. ({:.3}s)", t.elapsed().as_secs_f64());
        Err(anyhow::anyhow!("Markdown linting failed"))
    }
}
