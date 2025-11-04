//! CLI argument parsing structures
//!
//! This module defines the command-line interface structure and commands
//! for the dependency installer application.

use clap::{Parser, Subcommand};

use crate::Dependency;

/// Manage development dependencies for E2E tests
#[derive(Parser)]
#[command(name = "dependency-installer")]
#[command(version)]
#[command(about = "Manage development dependencies for E2E tests", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check if dependencies are installed
    Check {
        /// Specific dependency to check (if omitted, checks all)
        #[arg(short = 'd', long)]
        dependency: Option<Dependency>,
    },

    /// List all available tools and their status
    List,
}
