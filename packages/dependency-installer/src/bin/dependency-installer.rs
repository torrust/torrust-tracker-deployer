//! CLI binary for managing development dependencies
//!
//! This binary provides commands to check and list development dependencies
//! required for E2E tests in the Torrust Tracker Deployer project.
//!
//! # Exit Codes
//!
//! - 0: Success (all checks passed)
//! - 1: Missing dependencies
//! - 2: Invalid arguments
//! - 3: Internal error

use std::process;

use torrust_dependency_installer::app;

fn main() {
    let exit_code = app::run();
    process::exit(exit_code.into());
}
