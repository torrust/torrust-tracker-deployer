use tracing::info;

use crate::command::command_exists;
use crate::Dependency;

use super::{DependencyDetector, DetectionError};

/// Detector for `Ansible` dependency
pub struct AnsibleDetector;

impl DependencyDetector for AnsibleDetector {
    fn name(&self) -> &'static str {
        "Ansible"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(dependency = "ansible", "Checking if Ansible is installed");

        let installed = command_exists("ansible").map_err(|e| DetectionError::DetectionFailed {
            dependency: Dependency::Ansible,
            source: std::io::Error::other(e.to_string()),
        })?;

        if installed {
            info!(dependency = "ansible", "Ansible is installed");
        } else {
            info!(dependency = "ansible", "Ansible is not installed");
        }

        Ok(installed)
    }
}
