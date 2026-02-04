//! Backup template renderer module
//!
//! Contains the renderers for backup configuration templates.

mod backup_config;
mod maintenance_cron;
mod project_generator;

pub use backup_config::{BackupConfigRenderer, BackupConfigRendererError};
pub use maintenance_cron::{MaintenanceCronRenderer, MaintenanceCronRendererError};
pub use project_generator::{BackupProjectGenerator, BackupProjectGeneratorError};
