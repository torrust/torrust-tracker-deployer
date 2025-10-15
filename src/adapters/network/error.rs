//! Network diagnostic tool error types

use thiserror::Error;

use crate::shared::command::CommandError;

/// Errors that can occur during network diagnostic operations
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Failed to execute netstat command
    #[error(
        "Failed to execute netstat command
Tip: Verify netstat is installed: 'which netstat' or 'apt-get install net-tools'"
    )]
    NetstatFailed(#[source] CommandError),

    /// Failed to execute ss command
    #[error(
        "Failed to execute ss command
Tip: Verify ss is installed: 'which ss' or 'apt-get install iproute2'"
    )]
    SsFailed(#[source] CommandError),
}

impl NetworkError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use torrust_tracker_deployer_lib::adapters::network::NetstatClient;
    ///
    /// # fn example() {
    /// let netstat = NetstatClient::new();
    ///
    /// if let Err(e) = netstat.list_tcp_listening_ports() {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::NetstatFailed(_) => {
                "Netstat Command Failed - Detailed Troubleshooting:

1. Check if netstat is installed:
   which netstat

2. Install netstat if missing:
   Debian/Ubuntu: sudo apt-get install net-tools
   RHEL/CentOS: sudo yum install net-tools
   Alpine: sudo apk add net-tools
   macOS: netstat is pre-installed

3. Verify permissions:
   - Some netstat options require root/sudo for full process information
   - Try running the command manually: netstat -tlnp

4. Alternative: Use ss command instead (modern Linux):
   - ss is the modern replacement for netstat
   - Usually available via iproute2 package

For more information, see: man netstat"
            }

            Self::SsFailed(_) => {
                "SS Command Failed - Detailed Troubleshooting:

1. Check if ss is installed:
   which ss

2. Install ss if missing:
   Debian/Ubuntu: sudo apt-get install iproute2
   RHEL/CentOS: sudo yum install iproute2
   Alpine: sudo apk add iproute2
   macOS: ss is not available, use netstat instead

3. Verify permissions:
   - Some ss options require root/sudo for full process information
   - Try running the command manually: ss -tlnp

4. Alternative: Use netstat command instead:
   - netstat is the traditional tool for network statistics
   - Usually available via net-tools package

For more information, see: man ss"
            }
        }
    }
}
