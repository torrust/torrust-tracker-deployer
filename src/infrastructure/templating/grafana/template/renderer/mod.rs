//! Grafana Template Renderers
//!
//! Contains the project generator that orchestrates rendering of all Grafana
//! provisioning configuration templates.

pub mod project_generator;

pub use project_generator::{GrafanaProjectGenerator, GrafanaProjectGeneratorError};
