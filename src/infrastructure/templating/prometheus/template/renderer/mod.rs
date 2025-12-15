//! Template rendering for Prometheus configuration

pub mod project_generator;
pub mod prometheus_config;

pub use project_generator::{PrometheusProjectGenerator, PrometheusProjectGeneratorError};
pub use prometheus_config::{PrometheusConfigRenderer, PrometheusConfigRendererError};
