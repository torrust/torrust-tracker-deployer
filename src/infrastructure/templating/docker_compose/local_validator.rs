//! Local `docker compose config` validator
//!
//! This module validates a rendered `docker-compose.yml` by running
//! `docker compose config --quiet` locally **before** the file is uploaded to
//! the remote VM.  Catching structural errors at render time provides a faster
//! failure loop and a more actionable error message than waiting for Docker
//! Compose to reject the file at `run` time.
//!
//! ## Why validate locally?
//!
//! `docker compose config --quiet` validates the file structure (YAML syntax,
//! required field types, etc.) without starting any services.  It exits with
//! code 0 on success and a non-zero exit code with a descriptive message on
//! failure.  This is a cheap, dependency-free check — `docker` is already a
//! project requirement.
//!
//! ## Example failure caught by this validator
//!
//! An empty `networks:` key (no list items) produces:
//!
//! ```console
//! $ docker compose config --quiet
//! services.tracker.networks must be a list
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::local_validator::validate_docker_compose_file;
//!
//! validate_docker_compose_file(Path::new("build/my-env/docker-compose")).unwrap();
//! ```

use std::path::Path;
use std::process::Command;

use thiserror::Error;

/// Errors that can occur when validating a rendered `docker-compose.yml`
#[derive(Error, Debug)]
pub enum DockerComposeLocalValidationError {
    /// The `docker` binary could not be executed (not installed or not in PATH)
    #[error(
        "Failed to run 'docker compose config --quiet' — is Docker installed and in PATH?\n\
         Source: {source}"
    )]
    CommandExecutionFailed {
        /// The underlying OS error
        #[source]
        source: std::io::Error,
    },

    /// `docker compose config` ran but reported validation errors
    #[error(
        "Rendered docker-compose.yml failed validation.\n\
         Fix the template or environment configuration and re-render.\n\
         Docker Compose error:\n{output}"
    )]
    InvalidDockerComposeFile {
        /// Combined stdout+stderr from `docker compose config`
        output: String,
    },
}

impl DockerComposeLocalValidationError {
    /// Returns a help message with steps the user can take to resolve the issue
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::CommandExecutionFailed { .. } => {
                "Install Docker and ensure it is added to your PATH, then re-run the command."
                    .to_string()
            }
            Self::InvalidDockerComposeFile { .. } => {
                "Review the docker-compose.yml in your build directory.\n\
                 Common causes:\n\
                 - A service has an empty `networks:` key (no networks assigned)\n\
                 - A required field is missing or has the wrong type\n\
                 Check the error output above for the exact location of the problem."
                    .to_string()
            }
        }
    }
}

/// Validates a rendered `docker-compose.yml` by running `docker compose config --quiet`
///
/// The command is executed in `compose_dir` so that it picks up the
/// `docker-compose.yml` (and optionally `.env`) that live there.
///
/// # Arguments
///
/// * `compose_dir` — directory containing the rendered `docker-compose.yml`
///
/// # Errors
///
/// - [`DockerComposeLocalValidationError::CommandExecutionFailed`] if `docker`
///   cannot be executed (not installed, not in PATH, OS error, …)
/// - [`DockerComposeLocalValidationError::InvalidDockerComposeFile`] if the
///   file fails structural validation
///
/// # Example
///
/// ```rust,ignore
/// validate_docker_compose_file(Path::new("build/my-env/docker-compose"))?;
/// ```
pub fn validate_docker_compose_file(
    compose_dir: &Path,
) -> Result<(), DockerComposeLocalValidationError> {
    tracing::debug!(
        compose_dir = %compose_dir.display(),
        "Validating rendered docker-compose.yml with 'docker compose config --quiet'"
    );

    let output = Command::new("docker")
        .args(["compose", "config", "--quiet"])
        .current_dir(compose_dir)
        .output()
        .map_err(|source| DockerComposeLocalValidationError::CommandExecutionFailed { source })?;

    if output.status.success() {
        tracing::debug!(
            compose_dir = %compose_dir.display(),
            "docker-compose.yml passed local validation"
        );
        return Ok(());
    }

    // Collect any output produced by docker compose (errors may appear on
    // stdout or stderr depending on the Docker version).
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let combined = match (stderr.is_empty(), stdout.is_empty()) {
        (false, _) => stderr,
        (true, false) => stdout,
        (true, true) => format!(
            "docker compose exited with code {}",
            output.status.code().unwrap_or(-1)
        ),
    };

    tracing::warn!(
        compose_dir = %compose_dir.display(),
        error = %combined,
        "docker-compose.yml failed local validation"
    );

    Err(DockerComposeLocalValidationError::InvalidDockerComposeFile { output: combined })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn write_compose_file(dir: &TempDir, content: &str) {
        fs::write(dir.path().join("docker-compose.yml"), content)
            .expect("Failed to write docker-compose.yml");
    }

    #[test]
    fn it_should_pass_validation_for_a_valid_docker_compose_file() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        write_compose_file(
            &dir,
            r"
services:
  tracker:
    image: torrust/tracker:develop
    container_name: tracker
    networks:
      - metrics_network

networks:
  metrics_network:
    driver: bridge
",
        );

        let result = validate_docker_compose_file(dir.path());
        assert!(
            result.is_ok(),
            "Valid docker-compose.yml should pass validation; error: {result:?}"
        );
    }

    #[test]
    fn it_should_fail_validation_for_a_docker_compose_file_with_empty_networks_key() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        // This is the exact invalid output the template used to produce:
        // an empty `networks:` key followed by another key.
        write_compose_file(
            &dir,
            r"
services:
  tracker:
    image: torrust/tracker:develop
    container_name: tracker
    networks:
    ports:
      - '6969:6969/udp'
",
        );

        let result = validate_docker_compose_file(dir.path());
        assert!(
            matches!(
                result,
                Err(DockerComposeLocalValidationError::InvalidDockerComposeFile { .. })
            ),
            "Empty networks: key should fail validation; got: {result:?}"
        );
    }

    #[test]
    fn it_should_return_help_message_for_invalid_file_error() {
        let error = DockerComposeLocalValidationError::InvalidDockerComposeFile {
            output: "services.tracker.networks must be a list".to_string(),
        };
        let help = error.help();
        assert!(!help.is_empty(), "Help message must not be empty");
        assert!(
            help.contains("networks"),
            "Help should mention the networks key"
        );
    }

    #[test]
    fn it_should_return_help_message_for_command_execution_failed() {
        let error = DockerComposeLocalValidationError::CommandExecutionFailed {
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let help = error.help();
        assert!(!help.is_empty(), "Help message must not be empty");
        assert!(
            help.contains("Docker"),
            "Help should mention Docker installation"
        );
    }
}
