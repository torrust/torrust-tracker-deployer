//! Grafana Template Renderers
//!
//! Contains the project generator that orchestrates rendering of all Grafana
//! provisioning configuration templates.

pub mod datasource;
pub mod project_generator;

pub use datasource::{DatasourceRenderer, DatasourceRendererError};
pub use project_generator::{GrafanaProjectGenerator, GrafanaProjectGeneratorError};
