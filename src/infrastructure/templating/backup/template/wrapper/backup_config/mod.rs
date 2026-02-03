//! Backup configuration wrapper module
//!
//! Contains the context and template types for backup.conf.tera rendering.

pub mod context;
pub mod template;

pub use context::{BackupContext, BackupDatabaseConfig};
pub use template::{BackupTemplate, BackupTemplateError};
