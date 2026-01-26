//! Common utilities shared across release steps

use std::sync::Arc;

use crate::adapters::ansible::AnsibleClient;
use crate::domain::environment::{Environment, Releasing};

/// Create an Ansible client configured for the environment's build directory
///
/// This is a helper function to reduce duplication across step implementations.
#[must_use]
pub fn ansible_client(environment: &Environment<Releasing>) -> Arc<AnsibleClient> {
    Arc::new(AnsibleClient::new(environment.build_dir().join("ansible")))
}
