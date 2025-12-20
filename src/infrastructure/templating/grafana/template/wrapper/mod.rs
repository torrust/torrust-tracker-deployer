//! Grafana template wrappers
//!
//! This module provides wrappers for Grafana Tera templates.

pub mod datasource;

pub use datasource::{DatasourceContext, DatasourceTemplate};
