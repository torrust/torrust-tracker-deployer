//! Trace file generation infrastructure
//!
//! This module provides a scalable architecture for generating trace files
//! with command-specific writers and shared infrastructure.
//!
//! ## Module Structure
//!
//! - `writer` - Trace writing infrastructure
//!   - `sections` - Formatting utilities for trace sections
//!   - `error` - Error types for trace writing operations
//!   - `common` - Shared file I/O operations
//!   - `commands` - Command-specific trace writers (provision, configure, release)

pub mod writer;

pub use writer::{
    ConfigureTraceWriter, ProvisionTraceWriter, ReleaseTraceWriter, TraceWriterError,
};
