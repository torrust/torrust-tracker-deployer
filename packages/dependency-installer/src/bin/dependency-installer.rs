//! CLI binary for managing development dependencies
//!
//! This binary provides commands to check, install, and list development dependencies
//! required for E2E tests in the Torrust Tracker Deployer project.
//!
//! # Exit Codes
//!
//! **Exit codes are the official way to determine command success or failure.**
//! Scripts and automation tools should rely on exit codes, not parse the logging output.
//!
//! - **0**: Success (all checks or installations passed)
//! - **1**: Missing dependencies or installation failures
//! - **2**: Invalid arguments
//! - **3**: Internal error
//!
//! # Output Format
//!
//! The binary uses **structured logging** (via the `tracing` crate) for observability.
//! This output is designed for human reading and debugging, **not for parsing**.
//!
//! **For automation:**
//! - ✅ **DO** check the exit code to determine success/failure
//! - ❌ **DON'T** parse the log output - it may change and is not considered stable API
//!
//! # Examples
//!
//! ```bash
//! # Check exit code for success/failure
//! if dependency-installer check --log-level off; then
//!     echo "All dependencies installed"
//! else
//!     echo "Missing dependencies (exit code: $?)"
//! fi
//!
//! # Install and check result
//! dependency-installer install --dependency opentofu --log-level off
//! if [ $? -eq 0 ]; then
//!     echo "Installation succeeded"
//! fi
//! ```

use std::process;

use torrust_dependency_installer::app;

#[tokio::main]
async fn main() {
    let exit_code = app::run().await;
    process::exit(exit_code.into());
}
