//! Tracker template module
//!
//! This module provides template rendering functionality for Torrust Tracker configuration.
//!
//! ## Architecture
//!
//! Follows the Project Generator pattern with three layers:
//! - **Context** (`TrackerContext`) - Variables needed by templates
//! - **Template** (`TrackerTemplate`) - Wraps template with context
//! - **Renderer** (`TrackerConfigRenderer`) - Renders specific .tera files
//! - **`ProjectGenerator`** (`TrackerProjectGenerator`) - Orchestrates all renderers

pub mod template;

pub use template::renderer::{TrackerProjectGenerator, TrackerProjectGeneratorError};
pub use template::{
    TrackerConfigRenderer, TrackerConfigRendererError, TrackerContext, TrackerTemplate,
};
