//! Constants for SSH server container configuration

/// Docker image name for the SSH server container
pub const SSH_SERVER_IMAGE_NAME: &str = "torrust-ssh-server";

/// Docker image tag for the SSH server container
pub const SSH_SERVER_IMAGE_TAG: &str = "latest";

/// SSH port inside the container
pub const SSH_CONTAINER_PORT: u16 = 22;

/// Mock SSH server port (for testing without Docker)
pub const MOCK_SSH_PORT: u16 = 2222;

/// Default test username configured in the SSH server
pub const DEFAULT_TEST_USERNAME: &str = "testuser";

/// Default test password configured in the SSH server
pub const DEFAULT_TEST_PASSWORD: &str = "testpass";

/// Container startup wait time in seconds
pub const CONTAINER_STARTUP_WAIT_SECS: u64 = 10;

/// Relative path to SSH server Dockerfile directory
pub const DOCKERFILE_DIR: &str = "docker/ssh-server";
