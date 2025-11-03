//! Wrapper for templates/ansible/variables.yml.tera

pub mod context;

pub use context::{AnsibleVariablesContext, AnsibleVariablesContextError};
