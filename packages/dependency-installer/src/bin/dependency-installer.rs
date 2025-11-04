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
    let exit_code = match app::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e}");
            error_to_exit_code(e.as_ref())
        }
    };

    process::exit(exit_code);
}

/// Determine exit code based on error type
///
/// # Exit Codes
///
/// - 1: Missing dependencies (contains "not installed" or "Missing")
/// - 2: Invalid arguments (contains "Unknown tool" or "invalid")
/// - 3: Internal error (all other errors)
fn error_to_exit_code(error: &dyn std::error::Error) -> i32 {
    let error_msg = error.to_string();
    if error_msg.contains("not installed") || error_msg.contains("Missing") {
        1 // Missing dependency
    } else if error_msg.contains("Unknown tool") || error_msg.contains("invalid") {
        2 // Invalid argument
    } else {
        3 // Internal error
    }
}
