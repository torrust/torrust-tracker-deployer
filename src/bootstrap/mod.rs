//! Bootstrap Module
//!
//! This module contains application initialization and bootstrap concerns.
//! It handles application lifecycle, dependency injection, logging setup,
//! and help display.
//!
//! ## Modules
//!
//! - `app` - Main application bootstrap and entry point logic
//! - `container` - Service container for dependency injection
//! - `help` - Help and usage information display
//! - `logging` - Logging configuration and initialization

pub mod app;
pub mod container;
pub mod help;
pub mod logging;

// Re-export commonly used types for convenience
pub use container::Services;
pub use logging::{LogFormat, LogOutput, LoggingBuilder, LoggingConfig};
