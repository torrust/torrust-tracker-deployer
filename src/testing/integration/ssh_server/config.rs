//! Configuration for SSH server containers

use std::path::PathBuf;

use super::constants::{
    CONTAINER_STARTUP_WAIT_SECS, DEFAULT_TEST_PASSWORD, DEFAULT_TEST_USERNAME, DOCKERFILE_DIR,
    MOCK_SSH_PORT, SSH_SERVER_IMAGE_NAME, SSH_SERVER_IMAGE_TAG,
};

/// Configuration for SSH server containers
///
/// This struct defines all configurable parameters for SSH server containers.
/// Use the builder pattern to create custom configurations, or use `default()`
/// for standard test scenarios.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::testing::integration::ssh_server::SshServerConfig;
///
/// // Use default configuration
/// let config = SshServerConfig::default();
///
/// // Customize configuration
/// let config = SshServerConfig::builder()
///     .username("customuser")
///     .password("custompass")
///     .startup_wait_secs(15)
///     .build();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshServerConfig {
    /// Docker image name for the SSH server
    pub image_name: String,

    /// Docker image tag for the SSH server
    pub image_tag: String,

    /// Test username configured in the SSH server
    pub username: String,

    /// Test password configured in the SSH server
    pub password: String,

    /// Container startup wait time in seconds
    pub startup_wait_secs: u64,

    /// Path to the Dockerfile directory
    pub dockerfile_dir: PathBuf,

    /// Port to use for mock container (default: 2222)
    pub mock_port: u16,
}

impl SshServerConfig {
    /// Create a builder for custom configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::integration::ssh_server::SshServerConfig;
    ///
    /// let config = SshServerConfig::builder()
    ///     .username("testuser")
    ///     .password("testpass")
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> SshServerConfigBuilder {
        SshServerConfigBuilder::default()
    }
}

impl Default for SshServerConfig {
    /// Create configuration with default values from constants
    ///
    /// Default values:
    /// - Image: `torrust-ssh-server:latest`
    /// - Username: `testuser`
    /// - Password: `testpass`
    /// - Startup wait: 10 seconds
    /// - Dockerfile: `docker/ssh-server`
    /// - Mock port: 2222
    ///
    /// Note: The SSH container always uses port 22 internally (this is not configurable)
    fn default() -> Self {
        Self {
            image_name: SSH_SERVER_IMAGE_NAME.to_string(),
            image_tag: SSH_SERVER_IMAGE_TAG.to_string(),
            username: DEFAULT_TEST_USERNAME.to_string(),
            password: DEFAULT_TEST_PASSWORD.to_string(),
            startup_wait_secs: CONTAINER_STARTUP_WAIT_SECS,
            dockerfile_dir: PathBuf::from(DOCKERFILE_DIR),
            mock_port: MOCK_SSH_PORT,
        }
    }
}

/// Builder for SSH server configuration
///
/// Provides a fluent API for constructing custom SSH server configurations.
/// Any field not explicitly set will use the default value.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::testing::integration::ssh_server::SshServerConfig;
///
/// let config = SshServerConfig::builder()
///     .image_name("custom-ssh-server")
///     .image_tag("v2.0")
///     .username("admin")
///     .password("secret123")
///     .startup_wait_secs(20)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct SshServerConfigBuilder {
    image_name: Option<String>,
    image_tag: Option<String>,
    username: Option<String>,
    password: Option<String>,
    startup_wait_secs: Option<u64>,
    dockerfile_dir: Option<PathBuf>,
    mock_port: Option<u16>,
}

impl SshServerConfigBuilder {
    /// Set the Docker image name
    #[must_use]
    pub fn image_name(mut self, name: impl Into<String>) -> Self {
        self.image_name = Some(name.into());
        self
    }

    /// Set the Docker image tag
    #[must_use]
    pub fn image_tag(mut self, tag: impl Into<String>) -> Self {
        self.image_tag = Some(tag.into());
        self
    }

    /// Set the test username
    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set the test password
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the container startup wait time in seconds
    #[must_use]
    pub fn startup_wait_secs(mut self, secs: u64) -> Self {
        self.startup_wait_secs = Some(secs);
        self
    }

    /// Set the Dockerfile directory path
    #[must_use]
    pub fn dockerfile_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dockerfile_dir = Some(dir.into());
        self
    }

    /// Set the mock SSH server port
    #[must_use]
    pub fn mock_port(mut self, port: u16) -> Self {
        self.mock_port = Some(port);
        self
    }

    /// Build the configuration
    ///
    /// Any fields not explicitly set will use default values from constants.
    #[must_use]
    pub fn build(self) -> SshServerConfig {
        let defaults = SshServerConfig::default();
        SshServerConfig {
            image_name: self.image_name.unwrap_or(defaults.image_name),
            image_tag: self.image_tag.unwrap_or(defaults.image_tag),
            username: self.username.unwrap_or(defaults.username),
            password: self.password.unwrap_or(defaults.password),
            startup_wait_secs: self.startup_wait_secs.unwrap_or(defaults.startup_wait_secs),
            dockerfile_dir: self.dockerfile_dir.unwrap_or(defaults.dockerfile_dir),
            mock_port: self.mock_port.unwrap_or(defaults.mock_port),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_config_with_default_values() {
        let config = SshServerConfig::default();

        assert_eq!(config.image_name, "torrust-ssh-server");
        assert_eq!(config.image_tag, "latest");
        assert_eq!(config.username, "testuser");
        assert_eq!(config.password, "testpass");
        assert_eq!(config.startup_wait_secs, 10);
        assert_eq!(config.dockerfile_dir, PathBuf::from("docker/ssh-server"));
        assert_eq!(config.mock_port, 2222);
    }

    #[test]
    fn it_should_build_config_with_custom_values() {
        let config = SshServerConfig::builder()
            .image_name("custom-ssh")
            .image_tag("v2.0")
            .username("admin")
            .password("secret")
            .startup_wait_secs(15)
            .dockerfile_dir("custom/path")
            .mock_port(3333)
            .build();

        assert_eq!(config.image_name, "custom-ssh");
        assert_eq!(config.image_tag, "v2.0");
        assert_eq!(config.username, "admin");
        assert_eq!(config.password, "secret");
        assert_eq!(config.startup_wait_secs, 15);
        assert_eq!(config.dockerfile_dir, PathBuf::from("custom/path"));
        assert_eq!(config.mock_port, 3333);
    }

    #[test]
    fn it_should_use_defaults_for_unset_builder_fields() {
        let config = SshServerConfig::builder().username("customuser").build();

        // Custom value
        assert_eq!(config.username, "customuser");

        // Default values for unset fields
        assert_eq!(config.image_name, "torrust-ssh-server");
        assert_eq!(config.image_tag, "latest");
        assert_eq!(config.password, "testpass");
        assert_eq!(config.startup_wait_secs, 10);
        assert_eq!(config.mock_port, 2222);
    }

    #[test]
    fn it_should_allow_chaining_builder_methods() {
        let config = SshServerConfig::builder()
            .image_name("test")
            .image_tag("v1")
            .username("user1")
            .password("pass1")
            .startup_wait_secs(5)
            .build();

        assert_eq!(config.image_name, "test");
        assert_eq!(config.username, "user1");
        assert_eq!(config.startup_wait_secs, 5);
    }

    #[test]
    fn it_should_be_cloneable() {
        let config1 = SshServerConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1, config2);
    }

    #[test]
    fn it_should_allow_customizing_mock_port() {
        let config = SshServerConfig::builder().mock_port(5555).build();

        assert_eq!(config.mock_port, 5555);

        // Other fields should use defaults
        assert_eq!(config.username, "testuser");
        assert_eq!(config.password, "testpass");
    }
}
