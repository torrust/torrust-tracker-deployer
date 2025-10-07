//! `OpenTofu` client for infrastructure management
//!
//! This module provides the `OpenTofuClient` which wraps `OpenTofu` command-line tools
//! to provide a Rust-native interface for infrastructure provisioning and management.
//!
//! ## Key Features
//!
//! - Full `OpenTofu` workflow support (init, plan, apply, destroy)
//! - Instance information extraction from Terraform state
//! - JSON output parsing for structured data access
//! - Working directory management for Terraform projects
//! - Comprehensive error handling for all operations
//!
//! ## Supported Operations
//!
//! - `init` - Initialize Terraform working directory
//! - `plan` - Create execution plan showing changes
//! - `apply` - Apply infrastructure changes
//! - `destroy` - Destroy managed infrastructure
//! - `output` - Extract output values from state

use std::net::IpAddr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::shared::command::{CommandError, CommandExecutor};

use super::json_parser::{OpenTofuJsonParser, ParseError};

/// Container information extracted from `OpenTofu` outputs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub image: String,
    pub ip_address: IpAddr,
    pub name: String,
    pub status: String,
}

/// Errors that can occur during `OpenTofu` operations
#[derive(Error, Debug)]
pub enum OpenTofuError {
    /// Command execution failed
    #[error("Command execution failed: {0}")]
    CommandError(#[from] CommandError),

    /// JSON parsing failed
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

impl crate::shared::Traceable for OpenTofuError {
    fn trace_format(&self) -> String {
        match self {
            Self::CommandError(e) => format!("OpenTofuError: Command execution failed - {e}"),
            Self::ParseError(e) => format!("OpenTofuError: JSON parsing failed - {e}"),
        }
    }

    fn trace_source(&self) -> Option<&dyn crate::shared::Traceable> {
        match self {
            Self::CommandError(e) => Some(e),
            Self::ParseError(_) => None, // ParseError doesn't implement Traceable
        }
    }

    fn error_kind(&self) -> crate::shared::ErrorKind {
        crate::shared::ErrorKind::InfrastructureOperation
    }
}

/// A specialized `OpenTofu` client for infrastructure management.
/// This client provides a consistent interface for `OpenTofu` operations:
/// - Initialize `OpenTofu` configurations  
/// - Plan infrastructure changes
/// - Apply infrastructure changes
/// - Destroy infrastructure
///
/// Uses `CommandExecutor` as a collaborator for actual command execution.
pub struct OpenTofuClient {
    working_dir: PathBuf,
    command_executor: CommandExecutor,
}

impl OpenTofuClient {
    /// Creates a new `OpenTofuClient`
    ///
    /// # Arguments
    /// * `working_dir` - Path to the directory containing `OpenTofu` configuration files
    #[must_use]
    pub fn new<P: Into<PathBuf>>(working_dir: P) -> Self {
        Self {
            working_dir: working_dir.into(),
            command_executor: CommandExecutor::new(),
        }
    }

    /// Initialize `OpenTofu` configuration
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The `OpenTofu` initialization fails
    /// * The working directory does not exist or is not accessible
    pub fn init(&self) -> Result<String, CommandError> {
        info!(
            "Initializing OpenTofu in directory: {}",
            self.working_dir.display()
        );

        self.command_executor
            .run_command("tofu", &["init"], Some(&self.working_dir))
            .map(|result| result.stdout)
    }

    /// Validate configuration syntax and consistency
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The `OpenTofu` validate fails due to syntax or consistency errors
    /// * The configuration is not initialized
    /// * The working directory does not exist or is not accessible
    pub fn validate(&self) -> Result<String, CommandError> {
        info!(
            "Validating OpenTofu configuration in directory: {}",
            self.working_dir.display()
        );

        self.command_executor
            .run_command("tofu", &["validate"], Some(&self.working_dir))
            .map(|result| result.stdout)
    }

    /// Plan infrastructure changes
    ///
    /// # Arguments
    ///
    /// * `extra_args` - Additional arguments to pass to the tofu plan command (e.g., "-var-file=variables.tfvars")
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The `OpenTofu` plan fails
    /// * The configuration is not initialized
    pub fn plan(&self, extra_args: &[&str]) -> Result<String, CommandError> {
        info!(
            "Planning infrastructure changes in directory: {}",
            self.working_dir.display()
        );

        let mut args = vec!["plan"];
        args.extend_from_slice(extra_args);

        self.command_executor
            .run_command("tofu", &args, Some(&self.working_dir))
            .map(|result| result.stdout)
    }

    /// Apply infrastructure changes
    ///
    /// # Arguments
    ///
    /// * `auto_approve` - Whether to automatically approve the changes without interactive confirmation
    /// * `extra_args` - Additional arguments to pass to the tofu apply command (e.g., "-var-file=variables.tfvars")
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The `OpenTofu` apply fails
    /// * The configuration is not initialized
    pub fn apply(&self, auto_approve: bool, extra_args: &[&str]) -> Result<String, CommandError> {
        info!(
            "Applying infrastructure changes in directory: {}",
            self.working_dir.display()
        );

        let mut args = vec!["apply"];
        args.extend_from_slice(extra_args);
        if auto_approve {
            args.push("-auto-approve");
        }

        self.command_executor
            .run_command("tofu", &args, Some(&self.working_dir))
            .map(|result| result.stdout)
    }

    /// Destroy infrastructure
    ///
    /// # Arguments
    ///
    /// * `auto_approve` - Whether to automatically approve the destruction without interactive confirmation
    /// * `extra_args` - Additional arguments to pass to the tofu destroy command (e.g., "-var-file=variables.tfvars")
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The stdout output if the command succeeds
    /// * `Err(CommandError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The `OpenTofu` destroy fails
    /// * The configuration is not initialized
    pub fn destroy(&self, auto_approve: bool, extra_args: &[&str]) -> Result<String, CommandError> {
        info!(
            "Destroying infrastructure in directory: {}",
            self.working_dir.display()
        );

        let mut args = vec!["destroy"];
        args.extend_from_slice(extra_args);
        if auto_approve {
            args.push("-auto-approve");
        }

        self.command_executor
            .run_command("tofu", &args, Some(&self.working_dir))
            .map(|result| result.stdout)
    }

    /// Get `OpenTofu` outputs and parse container information
    ///
    /// # Returns
    ///
    /// * `Ok(ContainerInfo)` - Parsed container information from `OpenTofu` outputs
    /// * `Err(OpenTofuError)` - Error describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The `OpenTofu` output command fails
    /// * The output cannot be parsed as JSON
    /// * The `instance_info` section is missing or malformed
    pub fn get_instance_info(&self) -> Result<InstanceInfo, OpenTofuError> {
        info!(
            "Getting OpenTofu outputs from directory: {}",
            self.working_dir.display()
        );

        let output = self.command_executor.run_command(
            "tofu",
            &["output", "-json"],
            Some(&self.working_dir),
        )?;

        let instance_info = OpenTofuJsonParser::parse_instance_info(&output.stdout)?;
        Ok(instance_info)
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
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let main_tf_content = r#"
terraform {
  required_version = ">= 1.0"
}

resource "null_resource" "test" {
  provisioner "local-exec" {
    command = "echo 'test'"
  }
}
"#;
        fs::write(temp_dir.path().join("main.tf"), main_tf_content).unwrap();
        temp_dir
    }

    #[test]
    fn it_should_create_opentofu_client_with_valid_parameters() {
        let client = OpenTofuClient::new("/path/to/config");

        assert_eq!(client.working_dir.to_string_lossy(), "/path/to/config");
    }

    #[test]
    fn it_should_create_opentofu_client_with_working_directory() {
        let client = OpenTofuClient::new("/path/to/config");

        assert_eq!(client.working_dir.to_string_lossy(), "/path/to/config");
        // Note: logging is now handled by the tracing crate via CommandExecutor
    }

    #[test]
    fn it_should_return_working_directory_path() {
        let client = OpenTofuClient::new("/test/path");

        assert_eq!(client.working_dir(), Path::new("/test/path"));
    }

    #[test]
    fn it_should_construct_pathbuf_from_string() {
        let path_str = "/some/test/path";
        let client = OpenTofuClient::new(path_str);

        assert_eq!(client.working_dir(), Path::new(path_str));
    }

    #[test]
    fn it_should_construct_pathbuf_from_path() {
        let path = Path::new("/another/test/path");
        let client = OpenTofuClient::new(path);

        assert_eq!(client.working_dir(), path);
    }

    // Integration test that would require OpenTofu to be installed
    // These tests are more suitable for integration testing in a CI environment
    #[ignore = "requires OpenTofu installation"]
    #[test]
    fn it_should_initialize_opentofu_configuration() {
        let temp_dir = create_test_config_dir();
        let client = OpenTofuClient::new(temp_dir.path());

        // This would fail if OpenTofu is not installed, so we ignore it by default
        match client.init() {
            Ok(output) => {
                assert!(
                    output.contains("Terraform has been successfully initialized")
                        || output.contains("OpenTofu has been successfully initialized")
                );
            }
            Err(_) => {
                // Expected if OpenTofu is not installed
                tracing::warn!("OpenTofu not available for testing");
            }
        }
    }

    #[ignore = "requires OpenTofu installation"]
    #[test]
    fn it_should_plan_opentofu_configuration() {
        let temp_dir = create_test_config_dir();
        let client = OpenTofuClient::new(temp_dir.path());

        // Initialize first (this would also be ignored if OpenTofu is not available)
        drop(client.init());

        // This would fail if OpenTofu is not installed, so we ignore it by default
        match client.plan(&[]) {
            Ok(_output) => {
                // Plan succeeded
            }
            Err(_) => {
                // Expected if OpenTofu is not installed
                tracing::warn!("OpenTofu not available for testing");
            }
        }
    }

    #[test]
    fn it_should_wrap_parse_error_in_opentofu_error() {
        use crate::infrastructure::external_tools::tofu::adapter::json_parser::OpenTofuJsonParser;

        let invalid_json = "not valid json";

        let parse_error = OpenTofuJsonParser::parse_instance_info(invalid_json).unwrap_err();
        let opentofu_error = OpenTofuError::ParseError(parse_error);

        assert!(matches!(opentofu_error, OpenTofuError::ParseError(_)));
        assert!(opentofu_error.to_string().contains("Parse error"));
    }

    #[test]
    fn it_should_wrap_command_error_in_opentofu_error() {
        let command_error = CommandError::StartupFailed {
            command: "tofu".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Command not found"),
        };
        let opentofu_error = OpenTofuError::CommandError(command_error);

        assert!(matches!(opentofu_error, OpenTofuError::CommandError(_)));
        assert!(opentofu_error
            .to_string()
            .contains("Command execution failed"));
    }
}
