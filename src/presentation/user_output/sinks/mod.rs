//! Output sink implementations for different destinations
//!
//! This module contains implementations of the `OutputSink` trait for various
//! output destinations.

pub use composite::CompositeSink;
pub use file::FileSink;
pub use standard::StandardSink;
pub use telemetry::TelemetrySink;

mod composite;
mod file;
mod standard;
mod telemetry;
pub(in crate::presentation::user_output) mod writers;
