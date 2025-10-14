//! SSH configuration and management
//!
//! This module provides the `SshConfig` struct which encapsulates all the information
//! needed to establish an SSH connection to a remote host, including credentials and
//! target host socket address.
//!
//! ## Key Components
//!
//! - Connection configuration combining credentials and target host socket address
//! - Socket address management (IP address and port) for remote instances
//! - Integration with SSH credentials for authentication
//!
//! The connection configuration is used by SSH clients to establish secure
//! connections for remote command execution.

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use super::SshCredentials;

/// Default SSH port number.
pub const DEFAULT_SSH_PORT: u16 = 22;

/// Default SSH connection timeout in seconds
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u32 = 5;

/// Default maximum number of connection retry attempts
pub const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 30;

/// Default retry interval in seconds
pub const DEFAULT_RETRY_INTERVAL_SECS: u32 = 2;

/// Default retry log frequency (log every N attempts)
pub const DEFAULT_RETRY_LOG_FREQUENCY: u32 = 5;

/// SSH connection behavior configuration
///
/// Groups all connection-related parameters (timeouts, retries, logging)
/// separately from authentication credentials and target address.
///
/// This type encapsulates connection behavior settings that affect how
/// SSH connections are established and retried, distinct from authentication
/// credentials and the target host address.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::shared::ssh::SshConnectionConfig;
///
/// // Use default configuration
/// let config = SshConnectionConfig::default();
///
/// // Create custom configuration for fast testing
/// let fast_config = SshConnectionConfig::new(1, 5, 1, 2);
///
/// // Create custom configuration for slow networks
/// let slow_config = SshConnectionConfig::new(30, 20, 6, 3);
/// ```
#[derive(Clone, Debug)]
pub struct SshConnectionConfig {
    /// SSH connection timeout in seconds
    pub connect_timeout_secs: u32,
    /// Maximum number of connection retry attempts
    pub max_retry_attempts: u32,
    /// Seconds to wait between retry attempts
    pub retry_interval_secs: u32,
    /// Log progress every N retry attempts
    pub retry_log_frequency: u32,
}

impl SshConnectionConfig {
    /// Create a new connection configuration with custom values
    ///
    /// # Arguments
    ///
    /// * `connect_timeout_secs` - SSH connection timeout in seconds
    /// * `max_retry_attempts` - Maximum number of connection retry attempts
    /// * `retry_interval_secs` - Seconds to wait between retry attempts
    /// * `retry_log_frequency` - Log progress every N retry attempts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::shared::ssh::SshConnectionConfig;
    ///
    /// // Fast configuration for testing
    /// let fast = SshConnectionConfig::new(1, 5, 1, 2);
    ///
    /// // Slow configuration for unreliable networks
    /// let slow = SshConnectionConfig::new(30, 20, 6, 3);
    /// ```
    #[must_use]
    pub fn new(
        connect_timeout_secs: u32,
        max_retry_attempts: u32,
        retry_interval_secs: u32,
        retry_log_frequency: u32,
    ) -> Self {
        Self {
            connect_timeout_secs,
            max_retry_attempts,
            retry_interval_secs,
            retry_log_frequency,
        }
    }

    /// Calculate total wait time in seconds (`max_retry_attempts` × `retry_interval_secs`)
    ///
    /// Returns the maximum time that will be spent waiting for SSH connectivity
    /// if all retry attempts are exhausted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::shared::ssh::SshConnectionConfig;
    ///
    /// let config = SshConnectionConfig::default();
    /// assert_eq!(config.total_timeout_secs(), 60); // 30 attempts × 2 seconds
    /// ```
    #[must_use]
    pub fn total_timeout_secs(&self) -> u32 {
        self.max_retry_attempts * self.retry_interval_secs
    }
}

impl Default for SshConnectionConfig {
    /// Default connection configuration (production settings)
    ///
    /// Uses constants defined at module level:
    /// - Connection timeout: `DEFAULT_CONNECT_TIMEOUT_SECS` (5 seconds)
    /// - Max retry attempts: `DEFAULT_MAX_RETRY_ATTEMPTS` (30)
    /// - Retry interval: `DEFAULT_RETRY_INTERVAL_SECS` (2 seconds)
    /// - Retry log frequency: `DEFAULT_RETRY_LOG_FREQUENCY` (every 5 attempts)
    /// - Total wait time: 30 × 2 = 60 seconds
    fn default() -> Self {
        Self {
            connect_timeout_secs: DEFAULT_CONNECT_TIMEOUT_SECS,
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
            retry_interval_secs: DEFAULT_RETRY_INTERVAL_SECS,
            retry_log_frequency: DEFAULT_RETRY_LOG_FREQUENCY,
        }
    }
}

/// SSH connection configuration for a specific remote instance.
///
/// Contains both the SSH credentials and the target host socket address,
/// representing everything needed to establish an SSH connection.
#[derive(Clone)]
pub struct SshConfig {
    /// SSH authentication credentials.
    pub credentials: SshCredentials,

    /// Socket address (IP address and port) of the target host for SSH connections.
    ///
    /// This contains both the IP address and port number of the remote instance
    /// that the SSH client will connect to.
    pub socket_addr: SocketAddr,

    /// SSH connection behavior configuration (timeouts, retries, logging).
    pub connection_config: SshConnectionConfig,
}

impl SshConfig {
    /// Creates a new SSH connection configuration with default connection settings.
    ///
    /// Uses `SshConnectionConfig::default()` for connection behavior parameters.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deployer_lib::shared::{Username, ssh::{SshCredentials, SshConfig}};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     Username::new("ubuntu").unwrap(),
    /// );
    /// let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 22);
    /// let config = SshConfig::new(
    ///     credentials,
    ///     socket_addr,
    /// );
    /// ```
    #[must_use]
    pub fn new(credentials: SshCredentials, ssh_socket_addr: SocketAddr) -> Self {
        Self {
            credentials,
            socket_addr: ssh_socket_addr,
            connection_config: SshConnectionConfig::default(),
        }
    }

    /// Creates a new SSH connection configuration with custom connection settings.
    ///
    /// Use this constructor when you need to customize connection behavior
    /// (timeouts, retries, logging) for specific scenarios.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deployer_lib::shared::{Username, ssh::{SshCredentials, SshConfig, SshConnectionConfig}};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     Username::new("ubuntu").unwrap(),
    /// );
    /// let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 22);
    ///
    /// // Custom configuration for fast testing
    /// let connection_config = SshConnectionConfig::new(1, 5, 1, 2);
    /// let config = SshConfig::with_connection_config(
    ///     credentials,
    ///     socket_addr,
    ///     connection_config,
    /// );
    /// ```
    #[must_use]
    pub fn with_connection_config(
        credentials: SshCredentials,
        ssh_socket_addr: SocketAddr,
        connection_config: SshConnectionConfig,
    ) -> Self {
        Self {
            credentials,
            socket_addr: ssh_socket_addr,
            connection_config,
        }
    }

    /// Creates a new SSH connection configuration with the default port (22).
    ///
    /// This is a convenience method for when you want to use the standard SSH port.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deployer_lib::shared::{Username, ssh::{SshCredentials, SshConfig}};
    /// let credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     Username::new("ubuntu").unwrap(),
    /// );
    /// let config = SshConfig::with_default_port(
    ///     credentials,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
    /// );
    /// ```
    #[must_use]
    pub fn with_default_port(credentials: SshCredentials, host_ip: IpAddr) -> Self {
        let socket_addr = SocketAddr::new(host_ip, DEFAULT_SSH_PORT);
        Self::new(credentials, socket_addr)
    }

    /// Access the SSH private key path.
    #[must_use]
    pub fn ssh_priv_key_path(&self) -> &PathBuf {
        &self.credentials.ssh_priv_key_path
    }

    /// Access the SSH public key path.
    #[must_use]
    pub fn ssh_pub_key_path(&self) -> &PathBuf {
        &self.credentials.ssh_pub_key_path
    }

    /// Access the SSH username.
    #[must_use]
    pub fn ssh_username(&self) -> &str {
        self.credentials.ssh_username.as_str()
    }

    /// Access the SSH port.
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.socket_addr.port()
    }

    /// Access the host IP address.
    #[must_use]
    pub fn host_ip(&self) -> IpAddr {
        self.socket_addr.ip()
    }

    /// Access the socket address.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    /// Access the connection timeout in seconds.
    #[must_use]
    pub fn connection_timeout_secs(&self) -> u32 {
        self.connection_config.connect_timeout_secs
    }
}
