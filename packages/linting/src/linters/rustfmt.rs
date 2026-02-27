use anyhow::Result;
use std::process::Command;
use std::time::Instant;
use tracing::{error, info};

/// Run the Rust formatter check
///
/// # Errors
///
/// Returns an error if cargo fmt is not available or if the formatting check fails.
pub fn run_rustfmt_linter() -> Result<()> {
    let t = Instant::now();
    info!(target: "rustfmt", "Running Rust formatter check...");

    let output = Command::new("cargo")
        .args(["fmt", "--check", "--quiet"])
        .output()?;

    if output.status.success() {
        info!(target: "rustfmt", "Rust formatting check passed! ({:.3}s)", t.elapsed().as_secs_f64());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Print the output from cargo fmt
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        error!(target: "rustfmt", "Rust formatting check failed. Run 'cargo fmt' to fix formatting. ({:.3}s)", t.elapsed().as_secs_f64());
        Err(anyhow::anyhow!("Rust formatting check failed"))
    }
}
