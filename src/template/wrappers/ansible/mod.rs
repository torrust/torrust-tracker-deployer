//! Ansible template wrappers
//!
//! One module per template file in templates/ansible/

pub mod ansible_cfg;
pub mod install_docker;
pub mod install_docker_compose;
pub mod inventory;
pub mod wait_cloud_init;

// Re-export the main template structs for easier access
pub use ansible_cfg::AnsibleCfgTemplate;
pub use install_docker::InstallDockerTemplate;
pub use install_docker_compose::InstallDockerComposeTemplate;
pub use inventory::InventoryTemplate;
pub use wait_cloud_init::WaitCloudInitTemplate;
