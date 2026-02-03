//! Backup template module
//!
//! Handles backup configuration template rendering.

pub mod renderer;
pub mod wrapper;

pub use renderer::{
    BackupConfigRenderer, BackupConfigRendererError, BackupProjectGenerator,
    BackupProjectGeneratorError,
};
pub use wrapper::{BackupContext, BackupDatabaseConfig, BackupTemplate, BackupTemplateError};
