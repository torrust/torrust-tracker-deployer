//! Netstat CLI client implementation

use super::error::NetworkError;
use crate::shared::command::CommandExecutor;

/// Client for executing netstat commands
///
/// This client wraps netstat CLI operations using our `CommandExecutor` collaborator,
/// enabling testability and consistency with other external tool clients.
///
/// Netstat is a command-line tool that displays network connections, routing tables,
/// interface statistics, masquerade connections, and multicast memberships.
///
/// # Architecture
///
/// The client uses `CommandExecutor` as a collaborator for actual command execution,
/// following the same pattern as other adapters in this crate.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::adapters::network::NetstatClient;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let netstat = NetstatClient::new();
///
/// // List all TCP listening ports with process information
/// let output = netstat.list_tcp_listening_ports()?;
/// println!("Listening ports:\n{}", output);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct NetstatClient {
    command_executor: CommandExecutor,
}

impl Default for NetstatClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NetstatClient {
    /// Create a new netstat client
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::adapters::network::NetstatClient;
    ///
    /// let netstat = NetstatClient::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_executor: CommandExecutor::new(),
        }
    }

    /// List TCP listening ports with process information
    ///
    /// Executes `netstat -tlnp` to list all TCP listening sockets with:
    /// - `-t`: TCP connections only
    /// - `-l`: Listening sockets only
    /// - `-n`: Numeric addresses (no DNS resolution)
    /// - `-p`: Show process ID and name (may require root)
    ///
    /// # Returns
    ///
    /// The netstat output as a string
    ///
    /// # Errors
    ///
    /// Returns `NetworkError::NetstatFailed` if the command fails or netstat is not installed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::adapters::network::NetstatClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let netstat = NetstatClient::new();
    /// let output = netstat.list_tcp_listening_ports()?;
    ///
    /// // Parse output to find specific ports
    /// for line in output.lines() {
    ///     if line.contains(":8080") {
    ///         println!("Port 8080 is in use: {}", line);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_tcp_listening_ports(&self) -> Result<String, NetworkError> {
        let args = vec!["-tlnp"];

        self.command_executor
            .run_command("netstat", &args, None)
            .map(|result| result.stdout)
            .map_err(NetworkError::NetstatFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_netstat_client() {
        let _client = NetstatClient::new();
        // Successfully creating the client is sufficient for this test
    }

    #[test]
    fn it_should_have_default_implementation() {
        let _client = NetstatClient::default();
        // Successfully creating the client is sufficient for this test
    }

    // Note: We don't test actual command execution here as it requires netstat to be installed
    // and may require root permissions. Integration tests should cover actual execution.
}
