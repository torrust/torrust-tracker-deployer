//! Output sink implementations for different destinations
//!
//! This module contains implementations of the `OutputSink` trait for various
//! output destinations.

pub use composite::CompositeSink;
pub use file::FileSink;
pub use standard::StandardSink;
pub use telemetry::TelemetrySink;

// Re-export writers for use within user_output module (including tests)
pub(in crate::presentation::user_output) use writers::{StderrWriter, StdoutWriter};

mod composite;
mod file;
mod standard;
mod telemetry;
pub(in crate::presentation::user_output) mod writers;
