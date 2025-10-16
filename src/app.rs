//! Main application module for the Torrust Tracker Deployer CLI
//!
//! This module contains the CLI structure and main application logic.
//! It initializes logging and handles the application lifecycle.

use clap::Parser;
use std::path::PathBuf;
use tracing::info;

use torrust_tracker_deployer_lib::logging::{LogFormat, LogOutput, LoggingBuilder};

/// Command-line interface for Torrust Tracker Deployer
#[derive(Parser)]
#[command(name = "torrust-tracker-deployer")]
#[command(about = "Automated deployment infrastructure for Torrust Tracker")]
#[command(version)]
#[allow(clippy::struct_field_names)] // CLI arguments intentionally share 'log_' prefix for clarity
pub struct Cli {
    /// Logging format (default: compact)
    ///
    /// - pretty: Pretty-printed output for development
    /// - json: JSON output for production environments
    /// - compact: Compact output for minimal verbosity
    #[arg(long, value_enum, default_value = "compact", global = true)]
    pub log_format: LogFormat,

    /// Log output mode (default: file-only for production)
    ///
    /// - file-only: Write logs to file only (production mode)
    /// - file-and-stderr: Write logs to both file and stderr (development/testing mode)
    #[arg(long, value_enum, default_value = "file-only", global = true)]
    pub log_output: LogOutput,

    /// Log directory (default: ./data/logs)
    ///
    /// Directory where log files will be written. The log file will be
    /// named 'log.txt' inside this directory. Parent directories will be
    /// created automatically if they don't exist.
    ///
    /// Note: If the directory cannot be created due to filesystem permissions,
    /// the application will exit with an error. Logging is critical for
    /// observability and the application cannot function without it.
    #[arg(long, default_value = "./data/logs", global = true)]
    pub log_dir: PathBuf,
}

/// Main application entry point
///
/// This function initializes logging, displays information to the user,
/// and prepares the application for future command processing.
///
/// # Panics
///
/// This function will panic if:
/// - Log directory cannot be created (filesystem permissions issue)
/// - Logging initialization fails (usually means it was already initialized)
///
/// Both panics are intentional as logging is critical for observability.
pub fn run() {
    let cli = Cli::parse();

    // Clone values for logging before moving them
    let log_format = cli.log_format.clone();
    let log_output = cli.log_output;
    let log_dir = cli.log_dir.clone();

    // Initialize logging FIRST before any other logic
    LoggingBuilder::new(&cli.log_dir)
        .with_format(cli.log_format)
        .with_output(cli.log_output)
        .init();

    // Log startup event with configuration details
    info!(
        app = "torrust-tracker-deployer",
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %log_dir.display(),
        log_format = ?log_format,
        log_output = ?log_output,
        "Application started"
    );

    // Display info to user (keep existing behavior for now)
    println!("🏗️  Torrust Tracker Deployer");
    println!("=========================");
    println!();
    println!("This repository provides automated deployment infrastructure for Torrust tracker projects.");
    println!("The infrastructure includes VM provisioning with OpenTofu and configuration");
    println!("management with Ansible playbooks.");
    println!();
    println!("📋 Getting Started:");
    println!("   Please follow the instructions in the README.md file to:");
    println!("   1. Set up the required dependencies (OpenTofu, Ansible, LXD)");
    println!("   2. Provision the deployment infrastructure");
    println!("   3. Deploy and configure the services");
    println!();
    println!("🧪 Running E2E Tests:");
    println!("   Use the e2e tests binaries to run end-to-end tests:");
    println!("   cargo e2e-provision && cargo e2e-config");
    println!();
    println!("📖 For detailed instructions, see: README.md");

    info!("Application finished");
}
