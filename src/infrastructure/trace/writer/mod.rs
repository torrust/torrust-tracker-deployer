//! Trace file writer infrastructure
//!
//! This module provides the infrastructure for writing trace files:
//! - `sections` - Formatting utilities for trace sections
//! - `error` - Error types for trace writing operations
//! - `common` - Shared file I/O operations
//! - `commands` - Command-specific trace writers

pub mod commands;
mod common;
mod error;
mod sections;

pub use commands::{ConfigureTraceWriter, ProvisionTraceWriter, ReleaseTraceWriter};
pub use error::TraceWriterError;
