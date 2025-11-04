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

use clap::{Parser, Subcommand};
use torrust_dependency_installer::{Dependency, DependencyManager};
use tracing::{error, info};

/// Manage development dependencies for E2E tests
#[derive(Parser)]
#[command(name = "dependency-installer")]
#[command(version)]
#[command(about = "Manage development dependencies for E2E tests", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if dependencies are installed
    Check {
        /// Specific tool to check (if omitted, checks all)
        #[arg(short, long)]
        tool: Option<String>,
    },

    /// List all available tools and their status
    List,
}

fn main() {
    let exit_code = match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e}");

            // Determine exit code based on error type
            let error_msg = e.to_string();
            if error_msg.contains("not installed") || error_msg.contains("Missing") {
                1 // Missing dependency
            } else if error_msg.contains("Unknown tool") || error_msg.contains("invalid") {
                2 // Invalid argument
            } else {
                3 // Internal error
            }
        }
    };

    process::exit(exit_code);
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize tracing based on verbose flag
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    torrust_dependency_installer::init_tracing();

    let manager = DependencyManager::new();

    match cli.command {
        Commands::Check { tool } => handle_check(&manager, tool),
        Commands::List => handle_list(&manager),
    }
}

fn handle_check(
    manager: &DependencyManager,
    tool: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match tool {
        Some(tool_name) => check_specific_tool(manager, &tool_name),
        None => check_all_tools(manager),
    }
}

fn check_all_tools(manager: &DependencyManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking all dependencies");
    println!("Checking dependencies...\n");

    let results = manager.check_all()?;
    let mut missing_count = 0;

    for result in &results {
        if result.installed {
            println!("✓ {}: installed", result.tool);
        } else {
            println!("✗ {}: not installed", result.tool);
            missing_count += 1;
        }
    }

    println!();
    if missing_count > 0 {
        let msg = format!(
            "Missing {missing_count} out of {} required dependencies",
            results.len()
        );
        error!("{}", msg);
        println!("{msg}");
        Err(msg.into())
    } else {
        info!("All dependencies are installed");
        println!("All dependencies are installed");
        Ok(())
    }
}

fn check_specific_tool(
    manager: &DependencyManager,
    tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(tool = tool_name, "Checking specific tool");

    // Parse tool name to Dependency enum
    let dep = parse_tool_name(tool_name)?;
    let detector = manager.get_detector(dep);

    let installed = detector.is_installed()?;

    if installed {
        info!(tool = detector.name(), "Tool is installed");
        println!("✓ {}: installed", detector.name());
        Ok(())
    } else {
        let msg = format!("{}: not installed", detector.name());
        error!(tool = detector.name(), "Tool is not installed");
        println!("✗ {msg}");
        Err(msg.into())
    }
}

fn handle_list(manager: &DependencyManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Listing all available tools");
    println!("Available tools:\n");

    let results = manager.check_all()?;
    for result in results {
        let status = if result.installed {
            "installed"
        } else {
            "not installed"
        };
        println!("- {} ({status})", result.tool);
    }

    Ok(())
}

fn parse_tool_name(name: &str) -> Result<Dependency, String> {
    match name.to_lowercase().as_str() {
        "cargo-machete" | "machete" => Ok(Dependency::CargoMachete),
        "opentofu" | "tofu" => Ok(Dependency::OpenTofu),
        "ansible" => Ok(Dependency::Ansible),
        "lxd" => Ok(Dependency::Lxd),
        _ => Err(format!(
            "Unknown tool: {name}. Available: cargo-machete, opentofu, ansible, lxd"
        )),
    }
}
