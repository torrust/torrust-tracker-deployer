//! Main Application Bootstrap
//!
//! This module provides a thin bootstrap layer for the Torrust Tracker Deployer CLI.
//! It handles application initialization, logging setup, and command dispatch while
//! delegating all CLI parsing and business logic to the presentation layer.
//!
//! ## Responsibilities
//!
//! - **Application Lifecycle**: Initialize and shutdown the application
//! - **Logging Setup**: Configure logging based on CLI arguments
//! - **Command Dispatch**: Route commands to the presentation layer for execution
//! - **Exit Handling**: Manage application exit codes and cleanup
//!
//! ## Design Principles
//!
//! - **Thin Layer**: Minimal logic, maximum delegation to appropriate layers
//! - **Single Responsibility**: Focus only on application bootstrap concerns
//! - **Clean Separation**: No CLI parsing or business logic in this module

use std::sync::Arc;

use clap::Parser;
use tracing::info;

use crate::bootstrap;
use crate::presentation::cli::dispatch::route_command;
use crate::presentation::cli::dispatch::ExecutionContext;
use crate::presentation::cli::error::handle_error;
use crate::presentation::cli::Cli;

/// Main application entry point
///
/// This function serves as the application bootstrap, handling:
/// 1. CLI argument parsing (delegated to presentation layer)
/// 2. Logging initialization using `LoggingConfig`
/// 3. Service container creation for dependency injection
/// 4. Command execution (delegated to presentation layer)
/// 5. Error handling and exit code management
///
/// # Panics
///
/// This function will panic if:
/// - Log directory cannot be created (filesystem permissions issue)
/// - Logging initialization fails (usually means it was already initialized)
///
/// Both panics are intentional as logging is critical for observability.
pub async fn run() {
    let cli = Cli::parse();

    let logging_config = cli.global.logging_config();

    bootstrap::logging::init_subscriber(logging_config);

    info!(
        app = "torrust-tracker-deployer",
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %cli.global.log_dir.display(),
        log_file_format = ?cli.global.log_file_format,
        log_stderr_format = ?cli.global.log_stderr_format,
        log_output = ?cli.global.log_output,
        "Application started"
    );

    // Initialize service container for dependency injection
    let container = Arc::new(bootstrap::Container::new(
        cli.global.verbosity_level(),
        &cli.global.working_dir,
    ));
    let context = ExecutionContext::new(container, cli.global.clone());

    match cli.command {
        Some(command) => {
            if let Err(e) = route_command(command, &cli.global.working_dir, &context).await {
                handle_error(&e, &context.user_output());
                std::process::exit(1);
            }
        }
        None => {
            bootstrap::help::display_getting_started();
        }
    }

    info!("Application finished");
}
