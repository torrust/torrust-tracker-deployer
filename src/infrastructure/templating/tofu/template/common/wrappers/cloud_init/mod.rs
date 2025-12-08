//! Template wrapper for templates/tofu/common/cloud-init.yml.tera
//!
//! This template has mandatory variables that must be provided at construction time.
//! This wrapper is shared by all providers since the cloud-init template is the same.

mod cloud_init_template;
pub mod context;

pub use cloud_init_template::CloudInitTemplate;
pub use context::{CloudInitContext, CloudInitContextBuilder, CloudInitContextError};
