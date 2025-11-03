use tracing::info;

use crate::command::command_exists;
use crate::detector::ToolDetector;
use crate::errors::DetectionError;

/// Detector for `Ansible` tool
pub struct AnsibleDetector;

impl ToolDetector for AnsibleDetector {
    fn name(&self) -> &'static str {
        "Ansible"
    }

    fn is_installed(&self) -> Result<bool, DetectionError> {
        info!(tool = "ansible", "Checking if Ansible is installed");

        let installed = command_exists("ansible").map_err(|e| DetectionError::DetectionFailed {
            tool: self.name().to_string(),
            source: std::io::Error::other(e.to_string()),
        })?;

        if installed {
            info!(tool = "ansible", "Ansible is installed");
        } else {
            info!(tool = "ansible", "Ansible is not installed");
        }

        Ok(installed)
    }
}
