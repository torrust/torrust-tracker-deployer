//! External Process Execution
//!
//! Provides utilities for running the production application as an external
//! process for black-box testing.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Runs the production application as an external process
///
/// This struct provides methods for executing the application binary
/// with different command-line arguments for black-box testing.
pub struct ProcessRunner {
    working_dir: Option<PathBuf>,
}

impl ProcessRunner {
    /// Create a new process runner
    #[must_use]
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    /// Set the working directory for the test process (not the app working dir)
    ///
    /// This is the directory where the test command will be executed from,
    /// typically a temporary directory for test isolation.
    #[must_use]
    pub fn working_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.working_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Run the create command with the production binary
    ///
    /// This method runs `cargo run -- create environment --env-file <config_file>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory or config file path contains invalid UTF-8.
    pub fn run_create_command(&self, config_file: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");
        // If working directory is specified, we need to:
        // 1. Make the config file path absolute (cargo runs from project root)
        // 2. Pass --working-dir to tell the app where to store data
        if let Some(working_dir) = &self.working_dir {
            // Convert config file to absolute path
            let absolute_config = if config_file.starts_with("./") {
                working_dir.join(config_file.trim_start_matches("./"))
            } else {
                working_dir.join(config_file)
            };

            // Build command with absolute paths
            cmd.args([
                "run",
                "--",
                "create",
                "environment",
                "--env-file",
                absolute_config.to_str().unwrap(),
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args([
                "run",
                "--",
                "create",
                "environment",
                "--env-file",
                config_file,
            ]);
        }

        let output = cmd.output().context("Failed to execute create command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the provision command with the production binary
    ///
    /// This method runs `cargo run -- provision <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_provision_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "provision",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "provision", environment_name]);
        }

        let output = cmd
            .output()
            .context("Failed to execute provision command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the destroy command with the production binary
    ///
    /// This method runs `cargo run -- destroy <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_destroy_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "destroy",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "destroy", environment_name]);
        }

        let output = cmd.output().context("Failed to execute destroy command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the register command with the production binary
    ///
    /// This method runs `cargo run -- register <environment_name> --instance-ip <ip>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_register_command(
        &self,
        environment_name: &str,
        instance_ip: &str,
        ssh_port: Option<u16>,
    ) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            let mut args = vec![
                "run",
                "--",
                "register",
                environment_name,
                "--instance-ip",
                instance_ip,
            ];

            // Add optional SSH port
            let ssh_port_str = ssh_port.map(|p| p.to_string());
            if let Some(ref port_str) = ssh_port_str {
                args.extend(["--ssh-port", port_str]);
            }

            args.extend(["--working-dir", working_dir.to_str().unwrap()]);
            cmd.args(args);
        } else {
            // No working directory, use relative paths
            let mut args = vec![
                "run",
                "--",
                "register",
                environment_name,
                "--instance-ip",
                instance_ip,
            ];

            // Add optional SSH port
            let ssh_port_str = ssh_port.map(|p| p.to_string());
            if let Some(ref port_str) = ssh_port_str {
                args.extend(["--ssh-port", port_str]);
            }

            cmd.args(args);
        }

        let output = cmd.output().context("Failed to execute register command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the configure command with the production binary
    ///
    /// This method runs `cargo run -- configure <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_configure_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "configure",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "configure", environment_name]);
        }

        let output = cmd
            .output()
            .context("Failed to execute configure command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the test command with the production binary
    ///
    /// This method runs `cargo run -- test <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_test_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "test",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "test", environment_name]);
        }

        let output = cmd.output().context("Failed to execute test command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the release command with the production binary
    ///
    /// This method runs `cargo run -- release <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_release_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "release",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "release", environment_name]);
        }

        let output = cmd.output().context("Failed to execute release command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the run command with the production binary
    ///
    /// This method runs `cargo run -- run <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_run_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "run",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "run", environment_name]);
        }

        let output = cmd.output().context("Failed to execute run command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the list command with the production binary
    ///
    /// This method runs `cargo run -- list` with optional working directory
    /// for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_list_command(&self) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "list",
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "list"]);
        }

        let output = cmd.output().context("Failed to execute list command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the show command with the production binary
    ///
    /// This method runs `cargo run -- show <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_show_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "show",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "show", environment_name]);
        }

        let output = cmd.output().context("Failed to execute show command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the validate command with the production binary
    ///
    /// This method runs `cargo run -- validate -f <config_file>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path or config file path contains invalid UTF-8.
    pub fn run_validate_command(&self, config_file: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "validate",
                "-f",
                config_file,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "validate", "-f", config_file]);
        }

        let output = cmd.output().context("Failed to execute validate command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the purge command with the production binary
    ///
    /// This method runs `cargo run -- purge <environment_name> --force` with
    /// optional working directory for the application itself via `--working-dir`.
    /// Always uses `--force` flag to skip interactive confirmation prompts in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    /// Run the render command with environment name input mode
    ///
    /// This method runs `cargo run -- render --env-name <name> --instance-ip <ip> --output-dir <dir>`
    /// with optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// May panic if the working directory path is not valid UTF-8.
    pub fn run_render_command_with_env_name(
        &self,
        environment_name: &str,
        instance_ip: &str,
        output_dir: &str,
    ) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "render",
                "--env-name",
                environment_name,
                "--instance-ip",
                instance_ip,
                "--output-dir",
                output_dir,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args([
                "run",
                "--",
                "render",
                "--env-name",
                environment_name,
                "--instance-ip",
                instance_ip,
                "--output-dir",
                output_dir,
            ]);
        }

        let output = cmd
            .output()
            .context("Failed to execute render command with env-name")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the render command with config file input mode
    ///
    /// This method runs `cargo run -- render --env-file <path> --instance-ip <ip> --output-dir <dir>`
    /// with optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// May panic if the working directory path is not valid UTF-8.
    pub fn run_render_command_with_config_file(
        &self,
        config_file: &str,
        instance_ip: &str,
        output_dir: &str,
    ) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "render",
                "--env-file",
                config_file,
                "--instance-ip",
                instance_ip,
                "--output-dir",
                output_dir,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args([
                "run",
                "--",
                "render",
                "--env-file",
                config_file,
                "--instance-ip",
                instance_ip,
                "--output-dir",
                output_dir,
            ]);
        }

        let output = cmd
            .output()
            .context("Failed to execute render command with env-file")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the purge command with the production binary
    ///
    /// This method runs `cargo run -- purge <environment_name> --force`
    /// with optional working directory for the application itself via `--working-dir`.
    /// The `--force` flag is always used to skip interactive prompts.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// May panic if the working directory path is not valid UTF-8.
    pub fn run_purge_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = Command::new("cargo");

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "run",
                "--",
                "purge",
                environment_name,
                "--force",
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["run", "--", "purge", environment_name, "--force"]);
        }

        let output = cmd.output().context("Failed to execute purge command")?;

        Ok(ProcessResult::new(output))
    }
}

impl Default for ProcessRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper around process execution results
///
/// Provides convenient access to process output, exit status, and other
/// execution results.
pub struct ProcessResult {
    output: Output,
}

impl ProcessResult {
    fn new(output: Output) -> Self {
        Self { output }
    }

    /// Check if the process completed successfully
    #[must_use]
    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    /// Get the process stdout as a string
    #[must_use]
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).to_string()
    }

    /// Get the process stderr as a string
    #[must_use]
    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).to_string()
    }

    /// Get the process exit code
    #[must_use]
    pub fn exit_code(&self) -> Option<i32> {
        self.output.status.code()
    }
}
