//! Bootstrap Module
//!
//! This module contains application initialization and bootstrap concerns.
//! It handles application lifecycle, logging setup, help display, and service container.
//!
//! ## Modules
//!
//! - `app` - Main application bootstrap and entry point logic
//! - `container` - Application service container for dependency injection
//! - `help` - Help and usage information display
//! - `logging` - Logging configuration and initialization

pub mod app;
pub mod container;
pub mod help;
pub mod logging;
pub mod sdk;

// Re-export commonly used types for convenience
pub use container::Container;
pub use logging::{LogFormat, LogOutput, LoggingBuilder, LoggingConfig};
