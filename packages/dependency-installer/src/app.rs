//! Application logic for the dependency installer CLI
//!
//! This module contains the core application logic for running the CLI.

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::DependencyManager;

/// Run the CLI application
///
/// # Errors
///
/// Returns an error if:
/// - Dependencies are missing
/// - Invalid tool name is provided
/// - Internal error occurs during dependency checking
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing based on verbose flag
    // Must set environment variable before calling init_tracing()
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    crate::init_tracing();

    let manager = DependencyManager::new();

    match cli.command {
        Commands::Check { tool } => crate::handlers::check::handle_check(&manager, tool),
        Commands::List => crate::handlers::list::handle_list(&manager),
    }
}
