//! Builder for Ubuntu test containers

use std::path::{Path, PathBuf};

use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};

use super::container_id::ContainerId;
use super::image_builder::ImageBuilder;
use super::running_binary_container::RunningBinaryContainer;

/// Builder for creating Ubuntu test containers with a binary
///
/// This builder provides a fluent API for configuring and starting Ubuntu containers
/// with a binary installed. Call `new()` with the binary path, then `start().await`
/// to launch the container and install the binary.
pub struct UbuntuContainerBuilder {
    binary_path: PathBuf,
}

impl UbuntuContainerBuilder {
    /// Create a new Ubuntu container builder
    ///
    /// # Arguments
    ///
    /// * `binary_path` - Path to the binary on the host
    ///
    /// # Returns
    ///
    /// A builder that can start the container with the binary
    pub fn new(binary_path: &Path) -> Self {
        Self {
            binary_path: binary_path.to_path_buf(),
        }
    }

    /// Start the container with the binary
    ///
    /// This method uses a Docker image (dependency-installer-test:ubuntu-24.04) that includes
    /// sudo, curl, build-essential, and Rust nightly. The image is built automatically if it
    /// doesn't exist locally.
    ///
    /// # Returns
    ///
    /// A running container ready for test execution with all prerequisites installed
    #[allow(dead_code)] // Used by tests
    pub async fn start(self) -> RunningBinaryContainer {
        // Build the Docker image if it doesn't exist
        ImageBuilder::new()
            .build_if_missing()
            .expect("Failed to build test Docker image");

        // Use the custom image with all dependencies
        let image = GenericImage::new("dependency-installer-test", "ubuntu-24.04")
            .with_wait_for(WaitFor::seconds(2))
            .with_cmd(vec!["sleep", "infinity"]);

        // Start the container
        let container = image.start().await.expect("Failed to start container");

        // Get container ID for docker CLI operations
        let container_id = ContainerId::new(container.id().to_string())
            .expect("Docker container ID should always be valid hexadecimal");

        // Create the container wrapper
        let test_container = RunningBinaryContainer::new(container, container_id);

        // Copy the binary into the container
        test_container
            .copy_file_to_container(&self.binary_path, "/usr/local/bin/dependency-installer");

        // Make the binary executable
        test_container.exec(&["chmod", "+x", "/usr/local/bin/dependency-installer"]);

        test_container
    }
}
