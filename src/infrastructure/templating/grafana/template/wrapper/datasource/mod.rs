//! Datasource template wrapper
//!
//! This module provides the context and template for rendering the prometheus.yml.tera datasource configuration.

pub mod context;
pub mod template;

pub use context::DatasourceContext;
pub use template::DatasourceTemplate;
