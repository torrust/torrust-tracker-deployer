//! Backup infrastructure module
//!
//! Provides template rendering infrastructure for backup configuration files.

pub mod template;

pub use template::{
    BackupConfigRenderer, BackupConfigRendererError, BackupContext, BackupDatabaseConfig,
    BackupProjectGenerator, BackupProjectGeneratorError, BackupTemplate, BackupTemplateError,
};
