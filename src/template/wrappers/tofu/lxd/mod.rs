//! `OpenTofu` LXD template wrappers
//!
//! One module per template file in templates/tofu/lxd/

pub mod cloud_init;
pub mod main_tf;

// Re-export the main template structs for easier access
pub use cloud_init::CloudInitTemplate;
pub use main_tf::MainTfTemplate;
