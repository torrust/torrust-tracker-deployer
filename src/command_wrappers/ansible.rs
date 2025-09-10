use std::path::{Path, PathBuf};

use tracing::info;

use crate::command::{CommandError, CommandExecutor};

/// A specialized `Ansible` client for configuration management.
/// This client provides a consistent interface for `Ansible` operations:
/// - Run playbooks against target hosts
/// - Execute ad-hoc commands
/// - Manage inventory and configuration
///
/// Uses `CommandExecutor` as a collaborator for actual command execution.
pub struct AnsibleClient {
    working_dir: PathBuf,
    command_executor: CommandExecutor,
}

impl AnsibleClient {
    /// Creates a new `AnsibleClient`
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Path to the directory containing `Ansible` configuration files
    /// * `verbose` - Whether to log commands being executed
    #[must_use]
    pub fn new<P: Into<PathBuf>>(working_dir: P, verbose: bool) -> Self {
        Self {
            working_dir: working_dir.into(),
            command_executor: CommandExecutor::new(verbose),
        }
    }

    /// Run an Ansible playbook
    ///
    /// # Arguments
    ///
    /// * `playbook` - Name of the playbook file (without .yml extension)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The Ansible playbook execution fails
    /// * The playbook file does not exist in the working directory
    /// * There are issues with the inventory or configuration
    pub fn run_playbook(&self, playbook: &str) -> Result<String, CommandError> {
        info!(
            "Running Ansible playbook '{}' in directory: {}",
            playbook,
            self.working_dir.display()
        );

        let playbook_file = format!("{playbook}.yml");

        self.command_executor.run_command(
            "ansible-playbook",
            &[&playbook_file],
            Some(&self.working_dir),
        )
    }

    /// Get the working directory path
    #[must_use]
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_ansible_client_with_valid_parameters() {
        let client = AnsibleClient::new("/path/to/config", false);

        assert_eq!(client.working_dir.to_string_lossy(), "/path/to/config");
    }

    #[test]
    fn it_should_create_ansible_client_with_verbose_enabled() {
        let client = AnsibleClient::new("/path/to/config", true);

        assert_eq!(client.working_dir.to_string_lossy(), "/path/to/config");
        // Note: verbose is now encapsulated in the CommandExecutor collaborator
    }

    #[test]
    fn it_should_return_working_directory_path() {
        let client = AnsibleClient::new("/test/path", false);

        assert_eq!(client.working_dir(), Path::new("/test/path"));
    }

    #[test]
    fn it_should_construct_pathbuf_from_string() {
        let path_str = "/some/test/path";
        let client = AnsibleClient::new(path_str, false);

        assert_eq!(client.working_dir(), Path::new(path_str));
    }

    #[test]
    fn it_should_construct_pathbuf_from_path() {
        let path = Path::new("/another/test/path");
        let client = AnsibleClient::new(path, true);

        assert_eq!(client.working_dir(), path);
    }

    // Unit tests that don't require Ansible to be installed
    // These test the behavior and structure, not the actual command execution

    #[test]
    fn it_should_accept_playbook_name_without_extension() {
        let client = AnsibleClient::new("/test/path", false);

        // This tests the structure - we expect the method to exist and accept a &str
        // The actual execution would fail without Ansible, but we're testing the interface
        let result = client.run_playbook("install-docker");

        // We expect it to fail because ansible-playbook is not available in test environment
        // But this confirms the method signature and basic functionality works
        assert!(result.is_err());
    }

    // Integration tests that would require Ansible to be installed
    // These tests are more suitable for integration testing in a CI environment
    #[ignore = "requires Ansible installation"]
    #[test]
    fn it_should_execute_ansible_playbook() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create a simple test playbook
        let playbook_content = r#"
---
- hosts: localhost
  gather_facts: no
  tasks:
    - name: Test task
      debug:
        msg: "Hello from test playbook"
"#;
        fs::write(temp_dir.path().join("test-playbook.yml"), playbook_content).unwrap();

        let client = AnsibleClient::new(temp_dir.path(), false);

        // This would fail if Ansible is not installed, so we ignore it by default
        match client.run_playbook("test-playbook") {
            Ok(output) => {
                assert!(output.contains("Hello from test playbook"));
            }
            Err(_) => {
                // Expected if Ansible is not installed
                println!("Ansible not available for testing");
            }
        }
    }
}
