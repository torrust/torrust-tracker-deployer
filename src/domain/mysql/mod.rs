//! `MySQL` database service domain types
//!
//! This module defines the `MySQL` service configuration for Docker Compose topology.
//! This is distinct from `domain::tracker::MysqlConfig` which configures the
//! tracker's database connection settings.

pub mod config;

pub use config::MysqlServiceConfig;
