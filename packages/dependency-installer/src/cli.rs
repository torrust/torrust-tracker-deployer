//! CLI argument parsing structures
//!
//! This module defines the command-line interface structure and commands
//! for the dependency installer application.

use clap::{Parser, Subcommand, ValueEnum};

use crate::Dependency;

/// Log level for controlling output verbosity
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    /// Disable all logging
    Off,
    /// Show only errors
    Error,
    /// Show warnings and errors
    Warn,
    /// Show info, warnings, and errors (default)
    Info,
    /// Show debug logs and above
    Debug,
    /// Show all logs including trace
    Trace,
}

impl LogLevel {
    /// Convert to tracing Level
    ///
    /// Returns None for Off level
    #[must_use]
    pub fn to_tracing_level(self) -> Option<tracing::Level> {
        match self {
            Self::Off => None,
            Self::Error => Some(tracing::Level::ERROR),
            Self::Warn => Some(tracing::Level::WARN),
            Self::Info => Some(tracing::Level::INFO),
            Self::Debug => Some(tracing::Level::DEBUG),
            Self::Trace => Some(tracing::Level::TRACE),
        }
    }
}

/// Manage development dependencies for E2E tests
#[derive(Parser)]
#[command(name = "dependency-installer")]
#[command(version)]
#[command(about = "Manage development dependencies for E2E tests", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Set logging level (default: info)
    #[arg(short = 'l', long, value_enum, default_value = "info", global = true)]
    pub log_level: LogLevel,

    /// Enable verbose output (equivalent to --log-level debug)
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

    /// Install dependencies
    Install {
        /// Specific dependency to install (if omitted, installs all)
        #[arg(short = 'd', long)]
        dependency: Option<Dependency>,
    },

    /// List all available tools and their status
    List,
}
