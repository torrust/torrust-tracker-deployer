//! Container Image Builder for E2E Testing Containers
//!
//! This module provides a builder pattern for constructing and building container images
//! used in E2E testing scenarios. It separates the container image building logic from
//! the container lifecycle management, following the Single Responsibility Principle.
//!
//! ## Design
//!
//! The builder follows the standard Rust builder pattern with method chaining:
//!
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
//! use std::path::PathBuf;
//! use std::time::Duration;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let builder = ContainerImageBuilder::new()
//!     .with_name("torrust-provisioned-instance")
//!     .with_tag("latest")
//!     .with_dockerfile(PathBuf::from("docker/provisioned-instance/Dockerfile"))
//!     .with_context(PathBuf::from("."))
//!     .with_build_timeout(Duration::from_secs(300));
//!
//! builder.build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! The module provides specific error types for container build operations:
//! - `ContainerBuildExecution` - Command execution failures
//! - `ContainerBuildFailed` - Build process failures with stderr output
//! - `ImageNameRequired` - Image name not provided (use `with_name()`)
//! - `DockerfilePathRequired` - Dockerfile path not provided (use `with_dockerfile()`)
//!
//! ## Required Configuration
//!
//! The following must be provided before calling `build()`:
//! - **Image name**: Must be set with `with_name()`
//! - **Dockerfile path**: Must be set with `with_dockerfile()`
//!
//! ## Default Configuration
//!
//! The builder provides sensible defaults for optional parameters:
//! - **Tag**: "latest"
//! - **Build context**: "." (current directory)
//! - **Build timeout**: 300 seconds

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tracing::info;

/// Specific error types for container image building operations
#[derive(Debug, thiserror::Error)]
pub enum ContainerBuildError {
    /// Container build command execution failed
    #[error("Failed to execute docker build command for image '{image_name}:{tag}' using dockerfile '{dockerfile_path}' in context '{context_path}': {source}")]
    ContainerBuildExecution {
        image_name: String,
        tag: String,
        dockerfile_path: String,
        context_path: String,
        #[source]
        source: std::io::Error,
    },

    /// Container build process failed with non-zero exit code
    #[error("Docker build failed for image '{image_name}:{tag}' using dockerfile '{dockerfile_path}' in context '{context_path}' after {build_duration_secs}s with stderr: {stderr}")]
    ContainerBuildFailed {
        image_name: String,
        tag: String,
        dockerfile_path: String,
        context_path: String,
        build_duration_secs: u64,
        stderr: String,
    },

    /// Container build process timed out
    #[error("Docker build timed out after {timeout_secs}s for image '{image_name}:{tag}' using dockerfile '{dockerfile_path}' in context '{context_path}'")]
    ContainerBuildTimeout {
        image_name: String,
        tag: String,
        dockerfile_path: String,
        context_path: String,
        timeout_secs: u64,
    },

    /// Required image name was not provided
    #[error("Image name is required but was not provided")]
    ImageNameRequired,

    /// Required dockerfile path was not provided
    #[error("Dockerfile path is required but was not provided")]
    DockerfilePathRequired,

    /// Dockerfile does not exist at the specified path
    #[error("Dockerfile not found at path '{dockerfile_path}' (resolved to '{absolute_path}')")]
    DockerfileNotFound {
        dockerfile_path: String,
        absolute_path: String,
    },

    /// Context path does not exist
    #[error("Context path not found at '{context_path}' (resolved to '{absolute_path}')")]
    ContextPathNotFound {
        context_path: String,
        absolute_path: String,
    },
}

/// Result type alias for container build operations
pub type Result<T> = std::result::Result<T, Box<ContainerBuildError>>;

/// Builder for constructing and building container images
///
/// This builder follows the standard Rust builder pattern, allowing
/// method chaining to configure container image build parameters.
///
/// # Required Values
///
/// The following values must be provided before calling `build()`:
/// - **Image name**: Must be set with `with_name()`
/// - **Dockerfile path**: Must be set with `with_dockerfile()`
///
/// # Default Values
///
/// - **Tag**: "latest"
/// - **Build context**: "." (current directory)
/// - **Build timeout**: 300 seconds
#[derive(Debug, Clone)]
pub struct ContainerImageBuilder {
    image_name: Option<String>,
    tag: String,
    dockerfile_path: Option<PathBuf>,
    context_path: PathBuf,
    build_timeout: Duration,
}

impl ContainerImageBuilder {
    /// Create a new container image builder with default configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    ///
    /// let builder = ContainerImageBuilder::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            image_name: None,
            tag: "latest".to_string(),
            dockerfile_path: None,
            context_path: PathBuf::from("."),
            build_timeout: Duration::from_mins(5),
        }
    }

    /// Set the container image name
    ///
    /// # Arguments
    ///
    /// * `name` - The container image name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    ///
    /// let builder = ContainerImageBuilder::new()
    ///     .with_name("my-custom-image");
    /// ```
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.image_name = Some(name.into());
        self
    }

    /// Set the container image tag
    ///
    /// # Arguments
    ///
    /// * `tag` - The container image tag
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    ///
    /// let builder = ContainerImageBuilder::new()
    ///     .with_tag("v1.0.0");
    /// ```
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Set the path to the Dockerfile
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Dockerfile
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    /// use std::path::PathBuf;
    ///
    /// let builder = ContainerImageBuilder::new()
    ///     .with_dockerfile(PathBuf::from("custom/Dockerfile"));
    /// ```
    #[must_use]
    pub fn with_dockerfile(mut self, path: PathBuf) -> Self {
        self.dockerfile_path = Some(path);
        self
    }

    /// Set the Docker build context path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the build context directory
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    /// use std::path::PathBuf;
    ///
    /// let builder = ContainerImageBuilder::new()
    ///     .with_context(PathBuf::from("./app"));
    /// ```
    #[must_use]
    pub fn with_context(mut self, path: PathBuf) -> Self {
        self.context_path = path;
        self
    }

    /// Set the build timeout duration
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for build completion
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    /// use std::time::Duration;
    ///
    /// let builder = ContainerImageBuilder::new()
    ///     .with_build_timeout(Duration::from_secs(600));
    /// ```
    #[must_use]
    pub fn with_build_timeout(mut self, timeout: Duration) -> Self {
        self.build_timeout = timeout;
        self
    }

    /// Build the container image using the configured parameters
    ///
    /// This method executes the `docker build` command with the configured
    /// parameters. It provides detailed error information if the build fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Image name was not provided (use `with_name()`)
    /// - Dockerfile path was not provided (use `with_dockerfile()`)
    /// - Docker command cannot be executed (e.g., Docker not installed)
    /// - Docker build process fails (non-zero exit code)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let builder = ContainerImageBuilder::new()
    ///     .with_name("my-image")
    ///     .with_dockerfile(PathBuf::from("Dockerfile"));
    /// builder.build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(&self) -> Result<()> {
        let image_name = self
            .image_name
            .as_ref()
            .ok_or_else(|| Box::new(ContainerBuildError::ImageNameRequired))?;
        let dockerfile_path = self
            .dockerfile_path
            .as_ref()
            .ok_or_else(|| Box::new(ContainerBuildError::DockerfilePathRequired))?;

        let image_tag = format!("{}:{}", image_name, self.tag);
        let dockerfile_path_str = dockerfile_path.display().to_string();
        let context_path_str = self.context_path.display().to_string();

        // In CI environments, Docker `BuildKit` may have stale tags or cache conflicts.
        // Force remove any existing image before building to ensure a clean build state.
        drop(
            Command::new("docker")
                .args(["rmi", "-f", &image_tag])
                .output(),
        );

        info!(
            image_name = %image_name,
            tag = %self.tag,
            dockerfile = %dockerfile_path.display(),
            context = %self.context_path.display(),
            timeout_secs = self.build_timeout.as_secs(),
            "Building Docker image"
        );

        let start_time = std::time::Instant::now();
        let output = Command::new("docker")
            .args([
                "build",
                "-t",
                &image_tag,
                "-f",
                &dockerfile_path_str,
                "--force-rm", // Cleanup intermediate containers even on build failure
                &context_path_str,
            ])
            .output()
            .map_err(|source| {
                Box::new(ContainerBuildError::ContainerBuildExecution {
                    image_name: image_name.clone(),
                    tag: self.tag.clone(),
                    dockerfile_path: dockerfile_path_str.clone(),
                    context_path: context_path_str.clone(),
                    source,
                })
            })?;

        let build_duration = start_time.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Box::new(ContainerBuildError::ContainerBuildFailed {
                image_name: image_name.clone(),
                tag: self.tag.clone(),
                dockerfile_path: dockerfile_path_str,
                context_path: context_path_str,
                build_duration_secs: build_duration.as_secs(),
                stderr: stderr.to_string(),
            }));
        }

        info!(
            image_name = %image_name,
            tag = %self.tag,
            build_duration_ms = build_duration.as_millis(),
            "Docker image built successfully"
        );

        Ok(())
    }

    /// Get the full image tag (name:tag) that will be used for the build
    ///
    /// # Panics
    ///
    /// Panics if image name has not been set. Use `with_name()` first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::e2e::containers::ContainerImageBuilder;
    ///
    /// let builder = ContainerImageBuilder::new()
    ///     .with_name("my-app")
    ///     .with_tag("v1.0");
    ///     
    /// assert_eq!(builder.image_tag(), "my-app:v1.0");
    /// ```
    #[must_use]
    pub fn image_tag(&self) -> String {
        let image_name = self
            .image_name
            .as_ref()
            .expect("Image name must be set before calling image_tag()");
        format!("{}:{}", image_name, self.tag)
    }

    /// Get the image name if it has been set
    #[must_use]
    pub fn image_name(&self) -> Option<&str> {
        self.image_name.as_deref()
    }

    /// Get the image tag
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Get the dockerfile path if it has been set
    #[must_use]
    pub fn dockerfile_path(&self) -> Option<&PathBuf> {
        self.dockerfile_path.as_ref()
    }

    /// Get the build context path
    #[must_use]
    pub fn context_path(&self) -> &PathBuf {
        &self.context_path
    }

    /// Get the build timeout
    #[must_use]
    pub fn build_timeout(&self) -> Duration {
        self.build_timeout
    }
}

impl Default for ContainerImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn it_should_create_builder_with_default_values() {
        let builder = ContainerImageBuilder::new();

        assert_eq!(builder.image_name(), None);
        assert_eq!(builder.tag(), "latest");
        assert_eq!(builder.dockerfile_path(), None);
        assert_eq!(builder.context_path(), &PathBuf::from("."));
        assert_eq!(builder.build_timeout(), Duration::from_mins(5));
    }

    #[test]
    fn it_should_create_default_builder() {
        let builder = ContainerImageBuilder::default();

        assert_eq!(builder.image_name(), None);
        assert_eq!(builder.tag(), "latest");
    }

    #[test]
    fn it_should_configure_image_name() {
        let builder = ContainerImageBuilder::new().with_name("custom-image");

        assert_eq!(builder.image_name(), Some("custom-image"));
        assert_eq!(builder.image_tag(), "custom-image:latest");
    }

    #[test]
    fn it_should_configure_image_tag() {
        let builder = ContainerImageBuilder::new()
            .with_name("test-image")
            .with_tag("v1.2.3");

        assert_eq!(builder.tag(), "v1.2.3");
        assert_eq!(builder.image_tag(), "test-image:v1.2.3");
    }

    #[test]
    fn it_should_configure_dockerfile_path() {
        let dockerfile_path = PathBuf::from("custom/Dockerfile");
        let builder = ContainerImageBuilder::new().with_dockerfile(dockerfile_path.clone());

        assert_eq!(builder.dockerfile_path(), Some(&dockerfile_path));
    }

    #[test]
    fn it_should_configure_context_path() {
        let context_path = PathBuf::from("./app");
        let builder = ContainerImageBuilder::new().with_context(context_path.clone());

        assert_eq!(builder.context_path(), &context_path);
    }

    #[test]
    fn it_should_configure_build_timeout() {
        let timeout = Duration::from_mins(10);
        let builder = ContainerImageBuilder::new().with_build_timeout(timeout);

        assert_eq!(builder.build_timeout(), timeout);
    }

    #[test]
    fn it_should_chain_configuration_methods() {
        let builder = ContainerImageBuilder::new()
            .with_name("my-app")
            .with_tag("v2.0")
            .with_dockerfile(PathBuf::from("custom/Dockerfile"))
            .with_context(PathBuf::from("./src"))
            .with_build_timeout(Duration::from_mins(15));

        assert_eq!(builder.image_name(), Some("my-app"));
        assert_eq!(builder.tag(), "v2.0");
        assert_eq!(builder.image_tag(), "my-app:v2.0");
        assert_eq!(
            builder.dockerfile_path(),
            Some(&PathBuf::from("custom/Dockerfile"))
        );
        assert_eq!(builder.context_path(), &PathBuf::from("./src"));
        assert_eq!(builder.build_timeout(), Duration::from_mins(15));
    }

    #[test]
    fn it_should_have_proper_error_display_messages() {
        let error = ContainerBuildError::ContainerBuildFailed {
            image_name: "test-image".to_string(),
            tag: "v1.0".to_string(),
            dockerfile_path: "/path/to/Dockerfile".to_string(),
            context_path: "/build/context".to_string(),
            build_duration_secs: 120,
            stderr: "build error message".to_string(),
        };

        let message = error.to_string();
        assert!(message.contains("Docker build failed"));
        assert!(message.contains("test-image:v1.0"));
        assert!(message.contains("build error message"));
        assert!(message.contains("/path/to/Dockerfile"));
        assert!(message.contains("/build/context"));
        assert!(message.contains("120s"));
    }

    #[test]
    fn it_should_preserve_error_chain_for_docker_build_execution() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "docker not found");
        let error = ContainerBuildError::ContainerBuildExecution {
            image_name: "test-image".to_string(),
            tag: "v1.0".to_string(),
            dockerfile_path: "/path/to/Dockerfile".to_string(),
            context_path: "/build/context".to_string(),
            source: io_error,
        };

        assert!(error
            .to_string()
            .contains("Failed to execute docker build command"));
        assert!(error.to_string().contains("test-image:v1.0"));
        assert!(error.to_string().contains("/path/to/Dockerfile"));
        assert!(error.to_string().contains("/build/context"));
        assert!(error.source().is_some());
    }

    #[test]
    fn it_should_fail_build_when_image_name_not_provided() {
        let builder = ContainerImageBuilder::new().with_dockerfile(PathBuf::from("Dockerfile"));

        let result = builder.build();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(*error, ContainerBuildError::ImageNameRequired));
        assert!(error.to_string().contains("Image name is required"));
    }

    #[test]
    fn it_should_fail_build_when_dockerfile_path_not_provided() {
        let builder = ContainerImageBuilder::new().with_name("test-image");

        let result = builder.build();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            *error,
            ContainerBuildError::DockerfilePathRequired
        ));
        assert!(error.to_string().contains("Dockerfile path is required"));
    }

    #[test]
    #[should_panic(expected = "Image name must be set before calling image_tag()")]
    fn it_should_panic_when_calling_image_tag_without_image_name() {
        let builder = ContainerImageBuilder::new();
        drop(builder.image_tag());
    }

    // Note: Actual docker build integration tests would require Docker
    // and are better suited for the e2e test binaries
}
