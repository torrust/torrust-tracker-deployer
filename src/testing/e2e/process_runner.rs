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
///
/// By default the runner falls back to `cargo run --` so it can be used
/// from `src/bin/` programs that do not have a pre-built binary available.
/// In integration tests (`tests/`), you should always call
/// [`with_binary`](Self::with_binary) with
/// `env!("CARGO_BIN_EXE_torrust-tracker-deployer")` so that Cargo's
/// pre-built binary is used directly, eliminating ~13 s of `cargo run`
/// startup overhead per invocation.
pub struct ProcessRunner {
    working_dir: Option<PathBuf>,
    log_dir: Option<PathBuf>,
    /// Path to the pre-built binary. When `None`, falls back to `cargo run`.
    binary: Option<PathBuf>,
}

impl ProcessRunner {
    /// Create a new process runner
    ///
    /// Falls back to `cargo run --` for executing the application. In
    /// integration tests prefer [`with_binary`](Self::with_binary) to avoid
    /// the ~13 s `cargo run` startup overhead.
    #[must_use]
    pub fn new() -> Self {
        Self {
            working_dir: None,
            log_dir: None,
            binary: None,
        }
    }

    /// Set the pre-built binary to use instead of `cargo run`.
    ///
    /// In integration tests pass `env!("CARGO_BIN_EXE_torrust-tracker-deployer")`
    /// here. Cargo automatically builds the binary before running the
    /// integration test, so the binary is always up-to-date.
    #[must_use]
    pub fn with_binary<P: AsRef<Path>>(mut self, binary: P) -> Self {
        self.binary = Some(binary.as_ref().to_path_buf());
        self
    }

    /// Build the base [`Command`] for the application.
    ///
    /// When a binary path is set, returns `Command::new(binary)`.
    /// Otherwise returns `Command::new("cargo")` pre-loaded with
    /// `["run", "--"]` so callers only need to append sub-command args.
    fn make_command(&self) -> Command {
        if let Some(binary) = &self.binary {
            Command::new(binary)
        } else {
            let mut cmd = Command::new("cargo");
            cmd.args(["run", "--"]);
            cmd
        }
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

    /// Set the log directory for the application
    ///
    /// This is passed as `--log-dir` to the application to control where
    /// logs are written, enabling test isolation.
    #[must_use]
    pub fn log_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.log_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Run the create command with the production binary
    ///
    /// This method runs `create environment --env-file <config_file>` with
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
        let mut cmd = self.make_command();
        // If working directory is specified, we need to:
        // 1. Make the config file path absolute (the binary runs from project root)
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
                "create",
                "environment",
                "--env-file",
                absolute_config.to_str().unwrap(),
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            // No working directory, use relative paths
            cmd.args(["create", "environment", "--env-file", config_file]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute create command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the provision command with the production binary
    ///
    /// This method runs `provision <environment_name>` with
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            // Build command with working directory
            cmd.args([
                "provision",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["provision", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd
            .output()
            .context("Failed to execute provision command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the destroy command with the production binary
    ///
    /// This method runs `destroy <environment_name>` with
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "destroy",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["destroy", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute destroy command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the register command with the production binary
    ///
    /// This method runs `register <environment_name> --instance-ip <ip>` with
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
        let mut cmd = self.make_command();
        cmd.args(["register", environment_name, "--instance-ip", instance_ip]);

        // Add optional SSH port
        if let Some(port) = ssh_port {
            cmd.args(["--ssh-port", &port.to_string()]);
        }

        // Add working-dir if specified
        if let Some(working_dir) = &self.working_dir {
            cmd.args(["--working-dir", working_dir.to_str().unwrap()]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute register command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the configure command with the production binary
    ///
    /// This method runs `configure <environment_name>` with
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "configure",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["configure", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd
            .output()
            .context("Failed to execute configure command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the test command with the production binary
    ///
    /// This method runs `test <environment_name>` with
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "test",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["test", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute test command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the release command with the production binary
    ///
    /// This method runs `release <environment_name>` with
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "release",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["release", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute release command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the run command with the production binary
    ///
    /// This method runs `run <environment_name>` with
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "run",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["run", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute run command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the list command with the production binary
    ///
    /// This method runs `list` with optional working directory
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args(["list", "--working-dir", working_dir.to_str().unwrap()]);
        } else {
            cmd.arg("list");
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute list command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the exists command with the production binary
    ///
    /// This method runs `exists <environment_name>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory path contains invalid UTF-8.
    pub fn run_exists_command(&self, environment_name: &str) -> Result<ProcessResult> {
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "exists",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["exists", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute exists command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the show command with the production binary
    ///
    /// This method runs `show <environment_name>` with
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "show",
                environment_name,
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["show", environment_name]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd.output().context("Failed to execute show command")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the validate command with the production binary
    ///
    /// This method runs `validate -f <config_file>` with
    /// optional working directory for the application itself via `--working-dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// Panics if the working directory or log directory path contains invalid UTF-8.
    pub fn run_validate_command(&self, config_file: &str) -> Result<ProcessResult> {
        let mut cmd = self.make_command();
        cmd.args(["validate", "-f", config_file]);

        // Add working-dir if specified
        if let Some(working_dir) = &self.working_dir {
            cmd.arg("--working-dir");
            cmd.arg(working_dir);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
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
    /// This method runs `render --env-name <name> --instance-ip <ip> --output-dir <dir>`
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
        let mut cmd = self.make_command();
        cmd.args([
            "render",
            "--env-name",
            environment_name,
            "--instance-ip",
            instance_ip,
            "--output-dir",
            output_dir,
        ]);

        if let Some(working_dir) = &self.working_dir {
            cmd.args(["--working-dir", working_dir.to_str().unwrap()]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd
            .output()
            .context("Failed to execute render command with env-name")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the render command with config file input mode
    ///
    /// This method runs `render --env-file <path> --instance-ip <ip> --output-dir <dir>`
    /// with optional working directory and log directory for test isolation.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    ///
    /// # Panics
    ///
    /// May panic if the working directory or log directory path is not valid UTF-8.
    pub fn run_render_command_with_config_file(
        &self,
        config_file: &str,
        instance_ip: &str,
        output_dir: &str,
    ) -> Result<ProcessResult> {
        let mut cmd = self.make_command();
        cmd.args([
            "render",
            "--env-file",
            config_file,
            "--instance-ip",
            instance_ip,
            "--output-dir",
            output_dir,
        ]);

        // Add working-dir if specified
        if let Some(working_dir) = &self.working_dir {
            cmd.arg("--working-dir");
            cmd.arg(working_dir);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
        }

        let output = cmd
            .output()
            .context("Failed to execute render command with env-file")?;

        Ok(ProcessResult::new(output))
    }

    /// Run the purge command with the production binary
    ///
    /// This method runs `purge <environment_name> --force`
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
        let mut cmd = self.make_command();

        if let Some(working_dir) = &self.working_dir {
            cmd.args([
                "purge",
                environment_name,
                "--force",
                "--working-dir",
                working_dir.to_str().unwrap(),
            ]);
        } else {
            cmd.args(["purge", environment_name, "--force"]);
        }

        // Add log-dir if specified
        if let Some(log_dir) = &self.log_dir {
            cmd.arg("--log-dir");
            cmd.arg(log_dir);
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
