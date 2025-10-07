//! Trace file generation infrastructure
//!
//! This module provides a scalable architecture for generating trace files
//! with command-specific writers and shared infrastructure.

mod common;
mod configure;
mod provision;

pub use common::TraceWriterError;
pub use configure::ConfigureTraceWriter;
pub use provision::ProvisionTraceWriter;
