//! Common error types for `OpenTofu` template wrappers.
//!
//! This module provides shared error types used by template wrappers across providers.

use thiserror::Error;

use crate::domain::template::{FileOperationError, TemplateEngineError};

/// Errors that can occur during variables template operations
#[derive(Error, Debug)]
pub enum VariablesTemplateError {
    /// Template engine error
    #[error("Template engine error: {source}")]
    TemplateEngineError {
        #[from]
        source: TemplateEngineError,
    },

    /// File I/O operation failed
    #[error("File operation failed: {source}")]
    FileOperationError {
        #[from]
        source: FileOperationError,
    },
}
