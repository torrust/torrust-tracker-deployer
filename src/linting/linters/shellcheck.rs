use anyhow::Result;
use std::process::Command;
use tracing::{error, info, warn};

use crate::linting::utils::{install_shellcheck, is_command_available};

/// Find shell scripts in the current directory
///
/// # Errors
///
/// Returns an error if the find command fails.
fn find_shell_scripts() -> Result<Vec<String>> {
    let mut files = Vec::new();

    // Find .sh files
    let sh_output = Command::new("find")
        .args([
            ".",
            "-name",
            "*.sh",
            "-type",
            "f",
            "-not",
            "-path",
            "*/.git/*",
            "-not",
            "-path",
            "*/.terraform/*",
        ])
        .output()?;

    if sh_output.status.success() {
        let stdout = String::from_utf8_lossy(&sh_output.stdout);
        files.extend(stdout.lines().filter(|s| !s.is_empty()).map(String::from));
    }

    // Find .bash files
    let bash_output = Command::new("find")
        .args([
            ".",
            "-name",
            "*.bash",
            "-type",
            "f",
            "-not",
            "-path",
            "*/.git/*",
            "-not",
            "-path",
            "*/.terraform/*",
        ])
        .output()?;

    if bash_output.status.success() {
        let stdout = String::from_utf8_lossy(&bash_output.stdout);
        files.extend(stdout.lines().filter(|s| !s.is_empty()).map(String::from));
    }

    Ok(files)
}

/// Run the `ShellCheck` linter
///
/// # Errors
///
/// Returns an error if shellcheck is not available, cannot be installed,
/// or if the linting fails.
pub fn run_shellcheck_linter() -> Result<()> {
    info!(target: "shellcheck", "Running ShellCheck on shell scripts...");

    // Check if shellcheck is installed
    if !is_command_available("shellcheck") {
        warn!(target: "shellcheck", "shellcheck not found. Attempting to install...");
        install_shellcheck()?;
    }

    // Find shell scripts
    let shell_files = find_shell_scripts()?;

    if shell_files.is_empty() {
        warn!(target: "shellcheck", "No shell scripts found");
        return Ok(());
    }

    info!(target: "shellcheck", "Found {} shell script(s) to check", shell_files.len());

    // Prepare the shellcheck command
    let mut cmd = Command::new("shellcheck");
    cmd.args(["--source-path=SCRIPTDIR", "--exclude=SC1091"]);
    cmd.args(&shell_files);

    let output = cmd.output()?;

    if output.status.success() {
        info!(target: "shellcheck", "shellcheck passed");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Print the output from shellcheck
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        error!(target: "shellcheck", "shellcheck failed");
        Err(anyhow::anyhow!("shellcheck failed"))
    }
}
