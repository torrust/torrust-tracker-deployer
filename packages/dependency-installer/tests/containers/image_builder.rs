//! Container Image Builder for Package Tests
//!
//! Simplified image builder for building test container images on-demand.

use std::path::PathBuf;
use std::process::Command;

use tracing::{error, info};

/// Error types for container image building
#[derive(Debug, thiserror::Error)]
pub enum ImageBuildError {
    /// Docker build command execution failed
    #[error("Failed to execute docker build command: {0}")]
    CommandFailed(#[from] std::io::Error),

    /// Docker build process failed with non-zero exit code
    #[error("Docker build failed with stderr: {0}")]
    BuildFailed(String),
}

/// Builder for constructing test container images
///
/// This is a simplified version optimized for package testing.
/// It builds the dependency-installer-test:ubuntu-24.04 image if it doesn't exist.
pub struct ImageBuilder {
    image_name: String,
    tag: String,
    dockerfile_path: PathBuf,
    context_path: PathBuf,
}

impl ImageBuilder {
    /// Create a new image builder for the standard test image
    ///
    /// Uses the default configuration:
    /// - Image: dependency-installer-test:ubuntu-24.04
    /// - Dockerfile: packages/dependency-installer/docker/ubuntu-24.04.Dockerfile
    /// - Context: packages/dependency-installer
    #[must_use]
    pub fn new() -> Self {
        Self {
            image_name: "dependency-installer-test".to_string(),
            tag: "ubuntu-24.04".to_string(),
            dockerfile_path: PathBuf::from("docker/ubuntu-24.04.Dockerfile"),
            context_path: PathBuf::from("."),
        }
    }

    /// Build the Docker image if it doesn't already exist
    ///
    /// This method first checks if the image exists. If it does, it skips the build.
    /// If it doesn't exist, it builds the image using docker build.
    ///
    /// When multiple tests run in parallel, concurrent builds may race to tag the
    /// same image. If a build fails with "already exists", it means another test
    /// already built the image, so this is treated as success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Docker command cannot be executed
    /// - Docker build process fails
    pub fn build_if_missing(&self) -> Result<(), ImageBuildError> {
        let full_image_name = format!("{}:{}", self.image_name, self.tag);

        // Check if image already exists
        if Self::image_exists(&full_image_name) {
            info!(
                image = full_image_name,
                "Docker image already exists, skipping build"
            );
            return Ok(());
        }

        info!(image = full_image_name, "Building Docker image...");

        // Build the image with --force-rm to clean up intermediate containers.
        // When parallel tests build the same image concurrently, the second build
        // may fail with "already exists" at the tagging step — this is handled
        // gracefully below by treating it as success.
        let output = Command::new("docker")
            .arg("build")
            .arg("--force-rm") // Remove intermediate containers after build
            .arg("-f")
            .arg(&self.dockerfile_path)
            .arg("-t")
            .arg(&full_image_name)
            .arg(&self.context_path)
            .output()?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // When multiple tests run in parallel, they may all call build_if_missing()
            // simultaneously. Both check image_exists() → false, both start building.
            // The first build succeeds and tags the image. The second build completes
            // all steps successfully but fails at the final export/tagging step with
            // "already exists" because the tag was claimed by the first build.
            // This is not a real failure — the image is available for use.
            if stderr.contains("already exists") || stdout.contains("already exists") {
                info!(
                    image = full_image_name,
                    "Docker image was built by a concurrent process, treating as success"
                );
                return Ok(());
            }

            error!(
                image = full_image_name,
                exit_code = output.status.code(),
                stdout_len = stdout.len(),
                stderr_len = stderr.len(),
                "Docker build failed"
            );
            // Include both stdout and stderr for better debugging
            let combined_output = format!("{stdout}\n{stderr}");
            return Err(ImageBuildError::BuildFailed(combined_output));
        }

        info!(image = full_image_name, "Successfully built Docker image");

        Ok(())
    }

    /// Check if a Docker image exists locally
    fn image_exists(full_image_name: &str) -> bool {
        Command::new("docker")
            .arg("image")
            .arg("inspect")
            .arg(full_image_name)
            .output()
            .is_ok_and(|output| output.status.success())
    }
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_builder_with_defaults() {
        let builder = ImageBuilder::new();
        assert_eq!(builder.image_name, "dependency-installer-test");
        assert_eq!(builder.tag, "ubuntu-24.04");
    }
}
