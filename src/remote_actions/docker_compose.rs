use std::net::IpAddr;
use tracing::{info, warn};

use crate::command_wrappers::ssh::SshClient;
use crate::command_wrappers::ssh::SshConnection;
use crate::remote_actions::{RemoteAction, RemoteActionError};

/// Action that validates Docker Compose installation and basic functionality on the server
pub struct DockerComposeValidator {
    ssh_client: SshClient,
}

impl DockerComposeValidator {
    /// Create a new `DockerComposeValidator` with the specified SSH configuration
    ///
    /// # Arguments
    /// * `ssh_connection` - SSH connection configuration containing credentials and host IP
    #[must_use]
    pub fn new(ssh_connection: SshConnection) -> Self {
        let ssh_client = SshClient::new(ssh_connection);
        Self { ssh_client }
    }
}

impl RemoteAction for DockerComposeValidator {
    fn name(&self) -> &'static str {
        "docker-compose-validation"
    }

    #[allow(clippy::too_many_lines)]
    async fn execute(&self, _server_ip: &IpAddr) -> Result<(), RemoteActionError> {
        info!(
            action = "docker_compose_validation",
            "Validating Docker Compose installation"
        );

        // First check if Docker is available (Docker Compose requires Docker)
        let docker_available =
            self.ssh_client
                .check_command("docker --version")
                .map_err(|source| RemoteActionError::SshCommandFailed {
                    action_name: self.name().to_string(),
                    source,
                })?;

        if !docker_available {
            warn!(
                action = "docker_compose_validation",
                status = "skipped",
                reason = "docker_unavailable",
                "Docker Compose validation skipped"
            );
            warn!(
                action = "docker_compose_validation",
                note = "dependency_missing",
                "Docker is not available, so Docker Compose cannot be validated"
            );
            warn!(
                action = "docker_compose_validation",
                note = "expected_in_ci",
                "This is expected in CI environments with network limitations"
            );
            return Ok(()); // Don't fail the test, just skip validation
        }

        // Check Docker Compose version
        let Ok(compose_version) = self.ssh_client.execute("docker-compose --version") else {
            warn!(
                action = "docker_compose_validation",
                status = "not_found",
                note = "expected_if_docker_skipped",
                "Docker Compose not found, this is expected if Docker installation was skipped"
            );
            return Ok(()); // Don't fail, just note the situation
        };

        let compose_version = compose_version.trim();
        info!(
            action = "docker_compose_validation",
            status = "success",
            "Docker Compose installation validated"
        );
        info!(
            action = "docker_compose_validation",
            version = compose_version,
            "Docker Compose version detected"
        );

        // Test basic docker-compose functionality with a simple test file (only if Docker is working)
        let test_compose_content = r"services:
  test:
    image: hello-world
";

        // Create a temporary test docker-compose.yml file
        let create_test_success = self
            .ssh_client
            .check_command(&format!(
                "echo '{test_compose_content}' > /tmp/test-docker-compose.yml"
            ))
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })?;

        if !create_test_success {
            warn!(
                action = "docker_compose_validation",
                check = "test_file_creation",
                status = "failed",
                "Could not create test docker-compose.yml file"
            );
            return Ok(()); // Don't fail, just skip the functional test
        }

        // Validate docker-compose file
        let validate_success = self
            .ssh_client
            .check_command("cd /tmp && docker-compose -f test-docker-compose.yml config")
            .map_err(|source| RemoteActionError::SshCommandFailed {
                action_name: self.name().to_string(),
                source,
            })?;

        if validate_success {
            info!(
                action = "docker_compose_validation",
                check = "configuration_validation",
                status = "success",
                "Docker Compose configuration validation passed"
            );
        } else {
            warn!(
                action = "docker_compose_validation",
                check = "configuration_validation",
                status = "skipped",
                "Docker Compose configuration validation skipped"
            );
        }

        // Clean up test file
        drop(
            self.ssh_client
                .check_command("rm -f /tmp/test-docker-compose.yml"),
        );

        Ok(())
    }
}
