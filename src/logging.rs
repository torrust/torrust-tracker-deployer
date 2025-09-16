//! Simplified Structured Logging Configuration
//!
//! Provides basic logging configuration with tracing spans for the three-level architecture:
//! - Commands (Level 1): Top-level orchestration
//! - Steps (Level 2): Mid-level execution units  
//! - Remote Actions (Level 3): Leaf-level operations

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Logging format options for different environments
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum LogFormat {
    /// Pretty-printed console output for development (default)
    Pretty,
    /// JSON output for production environments
    Json,
    /// Compact console output for minimal verbosity
    Compact,
}

/// Initialize the tracing subscriber with default pretty formatting
///
/// Sets up structured logging with:
/// - Pretty-printed console output for development
/// - Environment-based filtering via `RUST_LOG`
/// - Support for hierarchical spans across three levels
///
/// # Example
/// ```rust
/// use torrust_tracker_deploy::logging;
///
/// // Initialize logging at the start of your application
/// logging::init();
/// ```
pub fn init() {
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
}

/// Initialize the tracing subscriber with JSON formatting
///
/// Sets up structured logging with:
/// - JSON output format for production environments
/// - Environment-based filtering via `RUST_LOG`
/// - Machine-readable log format for monitoring systems
///
/// # Example
/// ```rust
/// use torrust_tracker_deploy::logging;
///
/// // Initialize JSON logging for production
/// logging::init_json();
/// ```
pub fn init_json() {
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
}

/// Initialize the tracing subscriber with compact formatting
///
/// Sets up structured logging with:
/// - Compact console output for minimal verbosity
/// - Environment-based filtering via `RUST_LOG`
/// - Space-efficient format for development
///
/// # Example
/// ```rust
/// use torrust_tracker_deploy::logging;
///
/// // Initialize compact logging
/// logging::init_compact();
/// ```
pub fn init_compact() {
    tracing_subscriber::registry()
        .with(fmt::layer().compact())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
}

/// Initialize logging based on the chosen format
///
/// This is a convenience function that calls the appropriate initialization
/// function based on the provided `LogFormat`.
///
/// # Arguments
/// * `format` - The logging format to use
///
/// # Example
/// ```rust
/// use torrust_tracker_deploy::logging::{LogFormat, init_with_format};
///
/// // Initialize with JSON format
/// init_with_format(&LogFormat::Json);
/// ```
pub fn init_with_format(format: &LogFormat) {
    match format {
        LogFormat::Pretty => init(),
        LogFormat::Json => init_json(),
        LogFormat::Compact => init_compact(),
    }
}
