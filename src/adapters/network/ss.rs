//! SS (socket statistics) CLI client implementation

use super::error::NetworkError;
use crate::shared::command::CommandExecutor;

/// Client for executing ss (socket statistics) commands
///
/// This client wraps ss CLI operations using our `CommandExecutor` collaborator,
/// enabling testability and consistency with other external tool clients.
///
/// SS is a modern utility to investigate sockets and is the replacement for netstat.
/// It's faster and provides more detailed information about network connections.
///
/// # Architecture
///
/// The client uses `CommandExecutor` as a collaborator for actual command execution,
/// following the same pattern as other adapters in this crate.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::adapters::network::SsClient;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ss = SsClient::new();
///
/// // List all TCP listening ports with process information
/// let output = ss.list_tcp_listening_ports()?;
/// println!("Listening ports:\n{}", output);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SsClient {
    command_executor: CommandExecutor,
}

impl Default for SsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SsClient {
    /// Create a new ss client
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::adapters::network::SsClient;
    ///
    /// let ss = SsClient::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_executor: CommandExecutor::new(),
        }
    }

    /// List TCP listening ports with process information
    ///
    /// Executes `ss -tlnp` to list all TCP listening sockets with:
    /// - `-t`: TCP sockets only
    /// - `-l`: Listening sockets only
    /// - `-n`: Numeric addresses (no DNS resolution)
    /// - `-p`: Show process information (may require root)
    ///
    /// # Returns
    ///
    /// The ss output as a string
    ///
    /// # Errors
    ///
    /// Returns `NetworkError::SsFailed` if the command fails or ss is not installed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use torrust_tracker_deployer_lib::adapters::network::SsClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let ss = SsClient::new();
    /// let output = ss.list_tcp_listening_ports()?;
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
            .run_command("ss", &args, None)
            .map(|result| result.stdout)
            .map_err(NetworkError::SsFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_ss_client() {
        let _client = SsClient::new();
        // Successfully creating the client is sufficient for this test
    }

    #[test]
    fn it_should_have_default_implementation() {
        let _client = SsClient::default();
        // Successfully creating the client is sufficient for this test
    }

    // Note: We don't test actual command execution here as it requires ss to be installed
    // and may require root permissions. Integration tests should cover actual execution.
}
