//! Template rendering for Tracker configuration

pub mod project_generator;
pub mod tracker_config;

pub use project_generator::{TrackerProjectGenerator, TrackerProjectGeneratorError};
pub use tracker_config::{TrackerConfigRenderer, TrackerConfigRendererError};
