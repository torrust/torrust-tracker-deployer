//! Bootstrap Module
//!
//! This module contains application initialization and bootstrap concerns.
//! It handles application lifecycle, logging setup, and help display.
//!
//! ## Modules
//!
//! - `app` - Main application bootstrap and entry point logic
//! - `help` - Help and usage information display
//! - `logging` - Logging configuration and initialization

pub mod app;
pub mod help;
pub mod logging;

// Re-export commonly used types for convenience
pub use logging::{LogFormat, LogOutput, LoggingBuilder, LoggingConfig};
