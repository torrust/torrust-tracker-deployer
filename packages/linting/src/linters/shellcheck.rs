use anyhow::Result;
use std::process::Command;
use std::time::Instant;
use tracing::{error, info, warn};

use crate::utils::is_command_available;

/// Install shellcheck using system package manager
///
/// # Errors
///
/// Returns an error if no supported package manager is found or if installation fails.
fn install_shellcheck() -> Result<()> {
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
    let t = Instant::now();
    info!(target: "shellcheck", "Running ShellCheck on shell scripts...");

    // Check if shellcheck is installed
    if !is_command_available("shellcheck") {
        warn!(target: "shellcheck", "shellcheck not found. Attempting to install...");
        install_shellcheck()?;
    }

    // Find shell scripts
    let shell_files = find_shell_scripts()?;

    if shell_files.is_empty() {
        warn!(target: "shellcheck", "No shell scripts found ({:.3}s)", t.elapsed().as_secs_f64());
        return Ok(());
    }

    info!(target: "shellcheck", "Found {} shell script(s) to check", shell_files.len());

    // Prepare the shellcheck command
    let mut cmd = Command::new("shellcheck");
    cmd.args(["--source-path=SCRIPTDIR", "--exclude=SC1091"]);
    cmd.args(&shell_files);

    let output = cmd.output()?;

    if output.status.success() {
        info!(target: "shellcheck", "shellcheck passed ({:.3}s)", t.elapsed().as_secs_f64());
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
        error!(target: "shellcheck", "shellcheck failed ({:.3}s)", t.elapsed().as_secs_f64());
        Err(anyhow::anyhow!("shellcheck failed"))
    }
}
