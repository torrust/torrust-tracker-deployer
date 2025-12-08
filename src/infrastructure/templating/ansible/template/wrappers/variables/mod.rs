//! Wrapper for templates/ansible/variables.yml.tera

pub mod context;
pub mod template;

pub use context::{AnsibleVariablesContext, AnsibleVariablesContextError};
pub use template::AnsibleVariablesTemplate;
