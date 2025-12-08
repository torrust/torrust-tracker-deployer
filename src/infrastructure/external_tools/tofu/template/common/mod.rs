//! Common `OpenTofu` template functionality shared across providers.
//!
//! This module contains template renderers and utilities that are not
//! specific to any particular infrastructure provider.

pub mod renderer;
pub mod wrappers;

pub use renderer::cloud_init::{CloudInitRenderer, CloudInitRendererError};
pub use renderer::{TofuProjectGenerator, TofuProjectGeneratorError};
pub use wrappers::{
    CloudInitContext, CloudInitContextBuilder, CloudInitContextError, CloudInitTemplate,
};
