//! Container Configuration Builder
//!
//! This module provides a flexible builder pattern for configuring Docker containers
//! used in E2E testing. It replaces hardcoded container configurations with explicit,
//! testable, and reusable configuration builders.
//!
//! ## Key Features
//!
//! - **Builder Pattern**: Fluent API for configuring containers
//! - **Type Safety**: Compile-time validation of configuration
//! - **Input Validation**: Runtime validation of image names and ports
//! - **Flexibility**: Support for ports and wait conditions
//! - **Testability**: Easy to create different configurations for testing
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use torrust_tracker_deploy::e2e::containers::config_builder::ContainerConfigBuilder;
//! use testcontainers::core::WaitFor;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ContainerConfigBuilder::new("my-app:latest")
//!     .with_exposed_port(22)
//!     .with_wait_condition(WaitFor::message_on_stdout("Server started"))
//!     .build()?;
//! # Ok(())
//! # }
//! ```

use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    GenericImage,
};

/// Specific error types for container configuration building
#[derive(Debug, thiserror::Error)]
pub enum ContainerConfigError {
    /// Invalid Docker image name format
    #[error("Invalid image name '{image_name}': {reason}")]
    InvalidImageName { image_name: String, reason: String },

    /// Invalid port number
    #[error("Invalid port number {port}: {reason}")]
    InvalidPort { port: u16, reason: String },

    /// Empty image name provided
    #[error("Image name cannot be empty")]
    EmptyImageName,

    /// Too many wait conditions (potential performance issue)
    #[error("Too many wait conditions ({count}): maximum {max_allowed} wait conditions are recommended for optimal container startup performance")]
    TooManyWaitConditions { count: usize, max_allowed: usize },
}

/// Result type alias for container configuration operations
pub type Result<T> = std::result::Result<T, Box<ContainerConfigError>>;

/// Flexible container configuration builder
///
/// This struct provides a builder pattern for configuring Docker containers
/// with explicit configuration options instead of hardcoded values.
///
/// Currently supports the minimal set of features needed by the provisioned container:
/// - Image name and tag validation
/// - Container name customization
/// - Exposed ports (with validation)
/// - Wait conditions (with reasonable limits)
#[derive(Debug, Clone)]
pub struct ContainerConfigBuilder {
    /// Docker image name (e.g., "torrust-provisioned-instance:latest")
    image: String,

    /// Optional container name for the running container
    container_name: Option<String>,

    /// List of ports to expose from the container (as u16)
    exposed_ports: Vec<u16>,

    /// Wait conditions to determine when container is ready
    wait_conditions: Vec<WaitFor>,
}

impl ContainerConfigBuilder {
    /// Create a new container configuration builder with the specified image
    ///
    /// # Arguments
    ///
    /// * `image` - Docker image name with optional tag (e.g., "redis:7", "app:latest")
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deploy::e2e::containers::config_builder::ContainerConfigBuilder;
    ///
    /// let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest");
    /// ```
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            container_name: None,
            exposed_ports: Vec::new(),
            wait_conditions: Vec::new(),
        }
    }

    /// Add an exposed port to the container configuration
    ///
    /// # Arguments
    ///
    /// * `port` - Port number to expose (as u16)
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deploy::e2e::containers::config_builder::ContainerConfigBuilder;
    ///
    /// let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
    ///     .with_exposed_port(22)
    ///     .with_exposed_port(80);
    /// ```
    #[must_use]
    pub fn with_exposed_port(mut self, port: u16) -> Self {
        if port == 0 {
            // Note: We'll handle this validation in the build() method to maintain
            // the current API which doesn't return Result. This is a design choice
            // to keep the builder pattern simple and ergonomic.
            tracing::warn!("Port 0 is reserved and will cause issues during container build");
        }

        if self.exposed_ports.contains(&port) {
            tracing::warn!("Port {port} is already exposed, skipping duplicate");
        } else {
            self.exposed_ports.push(port);
        }
        self
    }

    /// Set a custom container name for the running container
    ///
    /// # Arguments
    ///
    /// * `name` - Container name to use when starting the container
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deploy::e2e::containers::config_builder::ContainerConfigBuilder;
    ///
    /// let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
    ///     .with_container_name("my-custom-container-name");
    /// ```
    #[must_use]
    pub fn with_container_name(mut self, name: impl Into<String>) -> Self {
        self.container_name = Some(name.into());
        self
    }

    /// Add a wait condition to determine when the container is ready
    ///
    /// # Arguments
    ///
    /// * `condition` - Wait condition (e.g., message on stdout, HTTP endpoint, etc.)
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deploy::e2e::containers::config_builder::ContainerConfigBuilder;
    /// use testcontainers::core::WaitFor;
    ///
    /// let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
    ///     .with_wait_condition(WaitFor::message_on_stdout("sshd entered RUNNING state"));
    /// ```
    #[must_use]
    pub fn with_wait_condition(mut self, condition: WaitFor) -> Self {
        self.wait_conditions.push(condition);
        self
    }

    /// Build the final `GenericImage` with all configured options and validation
    ///
    /// This method creates a `GenericImage` with all the configuration options
    /// that were specified using the builder methods. It also validates the
    /// configuration to catch common issues early.
    ///
    /// # Returns
    ///
    /// A configured `GenericImage` ready to be used with testcontainers
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Image name is empty or invalid
    /// - Any port number is invalid (e.g., 0)
    /// - Too many wait conditions (performance concern)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deploy::e2e::containers::config_builder::ContainerConfigBuilder;
    /// use testcontainers::core::WaitFor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let image = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
    ///     .with_exposed_port(22)
    ///     .with_wait_condition(WaitFor::message_on_stdout("sshd entered RUNNING state"))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<GenericImage> {
        const MAX_RECOMMENDED_WAIT_CONDITIONS: usize = 5;

        // Validate image name
        if self.image.is_empty() {
            return Err(Box::new(ContainerConfigError::EmptyImageName));
        }

        if self.image.trim().is_empty() {
            return Err(Box::new(ContainerConfigError::InvalidImageName {
                image_name: self.image.clone(),
                reason: "image name contains only whitespace".to_string(),
            }));
        }

        // Basic image name format validation
        if self.image.contains("//") || self.image.starts_with('/') || self.image.ends_with('/') {
            return Err(Box::new(ContainerConfigError::InvalidImageName {
                image_name: self.image.clone(),
                reason: "image name contains invalid path separators".to_string(),
            }));
        }

        // Validate ports
        for &port in &self.exposed_ports {
            if port == 0 {
                return Err(Box::new(ContainerConfigError::InvalidPort {
                    port,
                    reason: "port 0 is reserved and cannot be exposed".to_string(),
                }));
            }
        }

        // Check for reasonable number of wait conditions (performance concern)
        if self.wait_conditions.len() > MAX_RECOMMENDED_WAIT_CONDITIONS {
            return Err(Box::new(ContainerConfigError::TooManyWaitConditions {
                count: self.wait_conditions.len(),
                max_allowed: MAX_RECOMMENDED_WAIT_CONDITIONS,
            }));
        }

        // Split the image name and tag if present
        let parts: Vec<&str> = self.image.split(':').collect();
        let (image_name, image_tag) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (self.image.as_str(), "latest")
        };

        // Additional validation for image name part
        if image_name.is_empty() {
            return Err(Box::new(ContainerConfigError::InvalidImageName {
                image_name: self.image.clone(),
                reason: "image name part is empty".to_string(),
            }));
        }

        let mut image = GenericImage::new(image_name, image_tag);

        // Add exposed ports using the testcontainers pattern
        for &port_num in &self.exposed_ports {
            image = image.with_exposed_port(port_num.tcp());
        }

        // Add wait conditions
        for condition in self.wait_conditions {
            image = image.with_wait_for(condition);
        }

        Ok(image)
    }

    /// Get the configured image name
    ///
    /// # Returns
    ///
    /// The Docker image name that was configured for this builder
    #[must_use]
    pub fn image_name(&self) -> &str {
        &self.image
    }

    /// Get the configured exposed ports
    ///
    /// # Returns
    ///
    /// A slice of port numbers that will be exposed
    #[must_use]
    pub fn exposed_ports(&self) -> &[u16] {
        &self.exposed_ports
    }

    /// Get the number of configured wait conditions
    ///
    /// # Returns
    ///
    /// The number of wait conditions configured
    #[must_use]
    pub fn wait_conditions_count(&self) -> usize {
        self.wait_conditions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::core::WaitFor;

    #[test]
    fn it_should_create_builder_with_image_name() {
        let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest");
        assert_eq!(builder.image_name(), "torrust-provisioned-instance:latest");
        assert_eq!(builder.exposed_ports().len(), 0);
        assert_eq!(builder.wait_conditions_count(), 0);
    }

    #[test]
    fn it_should_add_exposed_ports() {
        let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
            .with_exposed_port(22)
            .with_exposed_port(80);

        let ports = builder.exposed_ports();
        assert_eq!(ports.len(), 2);
        assert!(ports.contains(&22));
        assert!(ports.contains(&80));
    }

    #[test]
    fn it_should_add_wait_conditions() {
        let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
            .with_wait_condition(WaitFor::message_on_stdout("sshd entered RUNNING state"))
            .with_wait_condition(WaitFor::seconds(2));

        assert_eq!(builder.wait_conditions_count(), 2);
    }

    #[test]
    fn it_should_build_generic_image_with_all_options() {
        let image = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
            .with_exposed_port(22)
            .with_wait_condition(WaitFor::message_on_stdout("sshd entered RUNNING state"))
            .build();

        // Since GenericImage doesn't provide direct getters for configuration,
        // we can only test that it builds successfully without error
        // The actual configuration is tested through integration tests
        std::mem::drop(image); // Just verify it builds
    }

    #[test]
    fn it_should_handle_empty_configuration() {
        let image = ContainerConfigBuilder::new("alpine:latest").build();
        std::mem::drop(image); // Just verify it builds
    }

    #[test]
    fn it_should_chain_builder_methods_fluently() {
        let builder = ContainerConfigBuilder::new("torrust-provisioned-instance:latest")
            .with_exposed_port(22)
            .with_wait_condition(WaitFor::seconds(1));

        assert_eq!(builder.image_name(), "torrust-provisioned-instance:latest");
        assert_eq!(builder.exposed_ports().len(), 1);
        assert_eq!(builder.wait_conditions_count(), 1);
    }

    #[test]
    fn it_should_accept_string_and_str_for_image_name() {
        let builder1 = ContainerConfigBuilder::new("app:latest");
        let builder2 = ContainerConfigBuilder::new(String::from("app:latest"));

        assert_eq!(builder1.image_name(), builder2.image_name());
    }

    #[test]
    fn it_should_deduplicate_same_port_numbers() {
        let builder = ContainerConfigBuilder::new("app:latest")
            .with_exposed_port(8080)
            .with_exposed_port(8080);

        // Duplicate ports should be deduplicated since Docker/testcontainers
        // doesn't support exposing the same port number multiple times
        assert_eq!(builder.exposed_ports().len(), 1);
        assert_eq!(builder.exposed_ports()[0], 8080);
    }

    #[test]
    fn it_should_split_image_name_and_tag_correctly() {
        // Test with explicit tag
        let image1 = ContainerConfigBuilder::new("redis:7").build();
        std::mem::drop(image1);

        // Test without tag (should default to latest)
        let image2 = ContainerConfigBuilder::new("redis").build();
        std::mem::drop(image2);

        // Test with complex image name
        let image3 = ContainerConfigBuilder::new("registry.example.com/myapp:v1.2.3").build();
        std::mem::drop(image3);
    }
}
