use anyhow::Result;
use std::process::Command;
use std::time::Instant;
use tracing::{error, info};

/// Run the Rust clippy linter
///
/// # Errors
///
/// Returns an error if cargo clippy is not available or if the linting fails.
pub fn run_clippy_linter() -> Result<()> {
    let t = Instant::now();
    info!(target: "clippy", "Running Rust Clippy linter...");

    let mut cmd = Command::new("cargo");
    cmd.env("CARGO_INCREMENTAL", "0").args([
        "clippy",
        "--quiet",
        "--no-deps",
        "--tests",
        "--benches",
        "--examples",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "clippy::correctness",
        "-D",
        "clippy::suspicious",
        "-D",
        "clippy::complexity",
        "-D",
        "clippy::perf",
        "-D",
        "clippy::style",
        "-D",
        "clippy::pedantic",
    ]);

    let output = cmd.output()?;

    if output.status.success() {
        info!(target: "clippy", "Clippy linting completed successfully! ({:.3}s)", t.elapsed().as_secs_f64());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Print the output from clippy
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        println!();
        error!(target: "clippy", "Clippy linting failed. Please fix the issues above. ({:.3}s)", t.elapsed().as_secs_f64());
        Err(anyhow::anyhow!("Clippy linting failed"))
    }
}
