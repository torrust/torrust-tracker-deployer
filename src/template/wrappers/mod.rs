//! Template wrapper implementations organized by directory structure
//!
//! This module mirrors the templates/ directory structure with one module per template file:
//!
//! ## Ansible Templates (templates/ansible/)
//! - `ansible::inventory` - templates/ansible/inventory.yml (with mandatory fields)
//! - `ansible::ansible_cfg` - templates/ansible/ansible.cfg (static)
//! - `ansible::install_docker` - templates/ansible/install-docker.yml (static)
//! - `ansible::install_docker_compose` - templates/ansible/install-docker-compose.yml (static)
//! - `ansible::wait_cloud_init` - templates/ansible/wait-cloud-init.yml (static)
//!
//! ## `OpenTofu` Templates (templates/tofu/)
//! - `tofu::lxd::main_tf` - templates/tofu/lxd/main.tf (static)
//! - `tofu::lxd::cloud_init` - templates/tofu/lxd/cloud-init.yml (static)
//! - `tofu::lxd::readme` - templates/tofu/lxd/README.md (static)

pub mod ansible;
pub mod tofu;

/// Test function to verify module accessibility
#[must_use]
pub fn test_module_access() -> String {
    "template_wrappers module is accessible".to_string()
}
