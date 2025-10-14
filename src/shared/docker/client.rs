//! Docker CLI client implementation

use std::path::Path;

use super::error::DockerError;
use crate::shared::command::CommandExecutor;

/// Client for executing Docker CLI commands
///
/// This client wraps Docker CLI operations using our `CommandExecutor` collaborator,
/// enabling testability and consistency with other external tool clients (Ansible,
/// `OpenTofu`, LXD). Each Docker subcommand is exposed as a separate method.
///
/// # Architecture
///
/// The client uses `CommandExecutor` as a collaborator for actual command execution,
/// following the same pattern as `AnsibleClient`, `TofuClient`, and `LxdClient`.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::shared::docker::DockerClient;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let docker = DockerClient::new();
///
/// // Build an image
/// docker.build_image("docker/app", "my-app", "latest")?;
///
/// // Check if it exists
/// let exists = docker.image_exists("my-app", "latest")?;
/// assert!(exists);
/// # Ok(())
/// # }
/// ```
pub struct DockerClient {
    command_executor: CommandExecutor,
}

impl Default for DockerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerClient {
    /// Create a new Docker client
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    ///
    /// let docker = DockerClient::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_executor: CommandExecutor::new(),
        }
    }

    /// Build a Docker image from a Dockerfile directory
    ///
    /// Executes `docker build -t <name>:<tag> <path>` to build an image.
    ///
    /// # Arguments
    ///
    /// * `dockerfile_dir` - Path to directory containing the Dockerfile
    /// * `image_name` - Name for the Docker image (e.g., "my-ssh-server")
    /// * `image_tag` - Tag for the image (e.g., "latest")
    ///
    /// # Returns
    ///
    /// The build output on success
    ///
    /// # Errors
    ///
    /// Returns `DockerError::BuildFailed` if the build command fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker = DockerClient::new();
    /// docker.build_image("docker/ssh-server", "my-ssh", "latest")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_image<P: AsRef<Path>>(
        &self,
        dockerfile_dir: P,
        image_name: &str,
        image_tag: &str,
    ) -> Result<String, DockerError> {
        let image = format!("{image_name}:{image_tag}");
        let path = dockerfile_dir.as_ref().display().to_string();
        let args = vec!["build", "-t", &image, &path];

        self.command_executor
            .run_command("docker", &args, None)
            .map(|result| result.stdout)
            .map_err(|source| DockerError::BuildFailed { image, source })
    }

    /// List Docker images with optional repository filter
    ///
    /// Executes `docker images` with formatting to get structured output.
    ///
    /// # Arguments
    ///
    /// * `repository` - Optional repository name to filter by
    ///
    /// # Returns
    ///
    /// A vector of image information strings in format:
    /// "repository:tag|id|size"
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ListImagesFailed` if the command fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker = DockerClient::new();
    /// // List all images
    /// let all_images = docker.list_images(None)?;
    ///
    /// // List specific repository
    /// let ubuntu_images = docker.list_images(Some("ubuntu"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_images(&self, repository: Option<&str>) -> Result<Vec<String>, DockerError> {
        let format_str = "{{.Repository}}:{{.Tag}}|{{.ID}}|{{.Size}}";
        let mut args = vec!["images", "--format", format_str];

        if let Some(repo) = repository {
            args.push(repo);
        }

        let result = self
            .command_executor
            .run_command("docker", &args, None)
            .map_err(DockerError::ListImagesFailed)?;

        Ok(result.stdout.lines().map(ToString::to_string).collect())
    }

    /// List Docker containers
    ///
    /// Executes `docker ps` with formatting to get structured output.
    ///
    /// # Arguments
    ///
    /// * `all` - If true, shows all containers (including stopped ones)
    ///
    /// # Returns
    ///
    /// A vector of container information strings in format:
    /// "id|name|status"
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ListContainersFailed` if the command fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker = DockerClient::new();
    /// // List only running containers
    /// let running = docker.list_containers(false)?;
    ///
    /// // List all containers
    /// let all_containers = docker.list_containers(true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_containers(&self, all: bool) -> Result<Vec<String>, DockerError> {
        let format_str = "{{.ID}}|{{.Names}}|{{.Status}}";
        let mut args = vec!["ps", "--format", format_str];

        if all {
            args.push("-a");
        }

        let result = self
            .command_executor
            .run_command("docker", &args, None)
            .map_err(DockerError::ListContainersFailed)?;

        Ok(result.stdout.lines().map(ToString::to_string).collect())
    }

    /// Get logs from a Docker container
    ///
    /// Executes `docker logs <container-id>` to retrieve container logs.
    ///
    /// # Arguments
    ///
    /// * `container_id` - ID or name of the container
    ///
    /// # Returns
    ///
    /// The container's logs as a string
    ///
    /// # Errors
    ///
    /// Returns `DockerError::GetLogsFailed` if the command fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker = DockerClient::new();
    /// let logs = docker.get_container_logs("my-container")?;
    /// println!("Container logs:\n{}", logs);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_container_logs(&self, container_id: &str) -> Result<String, DockerError> {
        let args = vec!["logs", container_id];

        self.command_executor
            .run_command("docker", &args, None)
            .map(|result| result.stdout)
            .map_err(|source| DockerError::GetLogsFailed {
                container_id: container_id.to_string(),
                source,
            })
    }

    /// Check if a Docker image exists locally
    ///
    /// Uses `list_images` to check for the presence of a specific image.
    ///
    /// # Arguments
    ///
    /// * `image_name` - Name of the image
    /// * `image_tag` - Tag of the image
    ///
    /// # Returns
    ///
    /// `true` if the image exists, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ListImagesFailed` if the command fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::shared::docker::DockerClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker = DockerClient::new();
    /// if docker.image_exists("ubuntu", "latest")? {
    ///     println!("Ubuntu image is available");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn image_exists(&self, image_name: &str, image_tag: &str) -> Result<bool, DockerError> {
        let filter = format!("{image_name}:{image_tag}");
        let images = self.list_images(Some(&filter))?;
        Ok(!images.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_docker_client() {
        let _docker = DockerClient::new();
    }
}
