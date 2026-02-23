//! Configuration validation errors with actionable help messages
//!
//! This module defines error types for configuration validation failures.
//! All errors follow the project's error handling principles by providing
//! clear, contextual, and actionable error messages with `.help()` methods.

use std::path::PathBuf;
use thiserror::Error;

use crate::domain::tracker::{
    HealthCheckApiConfigError, HttpApiConfigError, HttpTrackerConfigError, MysqlConfigError,
    SqliteConfigError, TrackerConfigError, UdpTrackerConfigError,
};
use crate::domain::EnvironmentNameError;
use crate::domain::ProfileNameError;
use crate::shared::UsernameError;

/// Errors that can occur during configuration validation
///
/// These errors follow the project's error handling principles by providing
/// clear, contextual, and actionable error messages through the `.help()` method.
#[derive(Debug, Error)]
pub enum CreateConfigError {
    /// Invalid environment name format
    #[error("Invalid environment name: {0}")]
    InvalidEnvironmentName(#[from] EnvironmentNameError),

    /// Invalid SSH username format
    #[error("Invalid SSH username: {0}")]
    InvalidUsername(#[from] UsernameError),

    /// Invalid profile name format
    #[error("Invalid profile name: {0}")]
    InvalidProfileName(#[from] ProfileNameError),

    /// Invalid instance name format
    #[error("Invalid instance name '{name}': {reason}")]
    InvalidInstanceName {
        /// The invalid instance name that was provided
        name: String,
        /// The reason why the name is invalid
        reason: String,
    },

    /// SSH private key file not found
    #[error("SSH private key file not found: {path}")]
    PrivateKeyNotFound { path: PathBuf },

    /// SSH public key file not found
    #[error("SSH public key file not found: {path}")]
    PublicKeyNotFound { path: PathBuf },

    /// SSH private key path must be absolute
    #[error("SSH private key path must be absolute: {path:?}")]
    RelativePrivateKeyPath { path: PathBuf },

    /// SSH public key path must be absolute
    #[error("SSH public key path must be absolute: {path:?}")]
    RelativePublicKeyPath { path: PathBuf },

    /// Invalid SSH port (must be 1-65535)
    #[error("Invalid SSH port: {port} (must be between 1 and 65535)")]
    InvalidPort { port: u16 },

    /// Invalid bind address format
    #[error("Invalid bind address '{address}': failed to parse as IP:PORT")]
    InvalidBindAddress {
        /// The invalid bind address that was provided
        address: String,
        /// The underlying parse error
        #[source]
        source: std::net::AddrParseError,
    },

    /// Dynamic port assignment (port 0) is not supported
    #[error("Dynamic port assignment (port 0) is not supported in bind address '{bind_address}'")]
    DynamicPortNotSupported {
        /// The bind address containing port 0
        bind_address: String,
    },

    /// Failed to serialize configuration template to JSON
    #[error("Failed to serialize configuration template to JSON")]
    TemplateSerializationFailed {
        #[source]
        source: serde_json::Error,
    },

    /// Failed to create parent directory for template file
    #[error("Failed to create directory: {path}")]
    TemplateDirectoryCreationFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to write template file
    #[error("Failed to write template file: {path}")]
    TemplateFileWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Grafana requires Prometheus to be enabled
    #[error("Grafana requires Prometheus to be enabled")]
    GrafanaRequiresPrometheus,

    /// Invalid Prometheus configuration
    #[error("Invalid Prometheus configuration: {0}")]
    InvalidPrometheusConfig(String),

    /// Invalid Backup configuration
    #[error("Invalid Backup configuration: {0}")]
    InvalidBackupConfig(String),

    /// Tracker configuration validation failed
    #[error("Tracker configuration validation failed: {0}")]
    TrackerConfigValidation(#[from] TrackerConfigError),

    /// HTTP API configuration validation failed (domain invariant violation)
    ///
    /// This error wraps domain-level validation errors from `HttpApiConfig::new()`,
    /// providing a bridge between domain errors and application-level error handling.
    #[error("HTTP API configuration invalid: {0}")]
    HttpApiConfigInvalid(#[from] HttpApiConfigError),

    /// UDP tracker configuration validation failed (domain invariant violation)
    ///
    /// This error wraps domain-level validation errors from `UdpTrackerConfig::new()`,
    /// providing a bridge between domain errors and application-level error handling.
    #[error("UDP tracker configuration invalid: {0}")]
    UdpTrackerConfigInvalid(#[from] UdpTrackerConfigError),

    /// HTTP tracker configuration validation failed (domain invariant violation)
    ///
    /// This error wraps domain-level validation errors from `HttpTrackerConfig::new()`,
    /// providing a bridge between domain errors and application-level error handling.
    #[error("HTTP tracker configuration invalid: {0}")]
    HttpTrackerConfigInvalid(#[from] HttpTrackerConfigError),

    /// Health Check API configuration validation failed (domain invariant violation)
    ///
    /// This error wraps domain-level validation errors from `HealthCheckApiConfig::new()`,
    /// providing a bridge between domain errors and application-level error handling.
    #[error("Health Check API configuration invalid: {0}")]
    HealthCheckApiConfigInvalid(#[from] HealthCheckApiConfigError),

    /// `SQLite` database configuration validation failed (domain invariant violation)
    ///
    /// This error wraps domain-level validation errors from `SqliteConfig::new()`,
    /// providing a bridge between domain errors and application-level error handling.
    #[error("SQLite database configuration invalid: {0}")]
    SqliteConfigInvalid(#[from] SqliteConfigError),

    /// `MySQL` database configuration validation failed (domain invariant violation)
    ///
    /// This error wraps domain-level validation errors from `MysqlConfig::new()`,
    /// providing a bridge between domain errors and application-level error handling.
    #[error("MySQL database configuration invalid: {0}")]
    MysqlConfigInvalid(#[from] MysqlConfigError),

    /// HTTPS configuration validation failed (domain invariant violation)
    ///
    /// This error wraps domain-level validation errors from `HttpsConfig::new()`,
    /// such as invalid admin email format.
    #[error("HTTPS configuration invalid: {0}")]
    HttpsConfigInvalid(#[from] crate::domain::https::HttpsConfigError),

    /// Invalid domain name format for TLS configuration
    #[error("Invalid domain '{domain}': {reason}")]
    InvalidDomain {
        /// The invalid domain that was provided
        domain: String,
        /// The reason why the domain is invalid
        reason: String,
    },

    // Note: TLS/HTTPS cross-service validation errors (TlsWithoutHttpsSection, HttpsSectionWithoutTls)
    // have been moved to domain layer. See UserInputsError variants:
    // - TlsServicesWithoutHttpsSection
    // - HttpsSectionWithoutTlsServices
    /// TLS proxy enabled but domain not specified
    #[error("TLS proxy enabled for {service_type} '{bind_address}' but domain is missing")]
    TlsProxyWithoutDomain {
        /// The type of service (e.g., "HTTP tracker", "API")
        service_type: String,
        /// The bind address of the service
        bind_address: String,
    },

    /// Cross-service invariant validation failed
    ///
    /// This error wraps domain-level cross-service validation errors from `UserInputs`,
    /// such as Grafana requiring Prometheus or HTTPS/TLS configuration mismatches.
    #[error("Cross-service configuration validation failed: {0}")]
    CrossServiceValidation(#[from] crate::domain::environment::UserInputsError),
}

impl CreateConfigError {
    /// Provides detailed troubleshooting guidance for configuration errors
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the configuration issue. This implements the project's tiered help
    /// system pattern for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::application::command_handlers::create::config::CreateConfigError;
    /// use std::path::PathBuf;
    ///
    /// let error = CreateConfigError::PrivateKeyNotFound {
    ///     path: PathBuf::from("/home/user/.ssh/missing_key"),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("private key file"));
    /// assert!(help.contains("Check that the file path is correct"));
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName(_) => {
                "Environment name validation failed.\n\
                 \n\
                 Valid environment names must:\n\
                 - Contain only lowercase letters (a-z) and numbers (0-9)\n\
                 - Use dashes (-) as word separators\n\
                 - Not start or end with separators\n\
                 - Not start with numbers\n\
                 \n\
                 Examples: 'dev', 'staging', 'e2e-config', 'production'\n\
                 \n\
                 Fix: Update the environment name in your configuration to follow these rules."
            }
            Self::InvalidUsername(_) => {
                "SSH username validation failed.\n\
                 \n\
                 Valid usernames must:\n\
                 - Be 1-32 characters long\n\
                 - Start with a letter (a-z, A-Z) or underscore (_)\n\
                 - Contain only letters, digits, underscores, and hyphens\n\
                 \n\
                 Common usernames: 'ubuntu', 'torrust', 'deploy', 'admin'\n\
                 \n\
                 Fix: Update the SSH username in your configuration to follow Linux username requirements."
            }
            Self::InvalidProfileName(_) => {
                "LXD profile name validation failed.\n\
                 \n\
                 Valid profile names must:\n\
                 - Be 1-63 characters long\n\
                 - Contain only ASCII letters, numbers, and dashes\n\
                 - Not start with a digit or dash\n\
                 - Not end with a dash\n\
                 \n\
                 Examples: 'torrust-profile', 'default', 'dev-profile'\n\
                 \n\
                 Fix: Update the profile_name in your provider configuration to follow these rules."
            }
            Self::InvalidInstanceName { .. } => {
                "Instance name validation failed.\n\
                 \n\
                 Valid instance names must:\n\
                 - Be 1-63 characters long\n\
                 - Contain only ASCII letters, numbers, and dashes\n\
                 - Not start with a digit or dash\n\
                 - Not end with a dash\n\
                 \n\
                 Examples: 'torrust-tracker-vm-dev', 'my-instance', 'prod-server-01'\n\
                 \n\
                 Note: If you omit instance_name, it will be auto-generated as 'torrust-tracker-vm-{env_name}'.\n\
                 \n\
                 Fix: Update the instance_name in your environment configuration to follow these rules, or remove it to use auto-generation."
            }
            Self::PrivateKeyNotFound { .. } => {
                "SSH private key file not found.\n\
                 \n\
                 The specified private key file does not exist or is not accessible.\n\
                 \n\
                 Common causes:\n\
                 - Incorrect file path in configuration\n\
                 - File was moved or deleted\n\
                 - Insufficient permissions to access the file\n\
                 \n\
                 Fix:\n\
                 1. Check that the file path is correct in your configuration\n\
                 2. Verify the file exists: ls -la <path>\n\
                 3. Ensure you have read permissions on the file\n\
                 4. Generate a new SSH key pair if needed: ssh-keygen -t rsa -b 4096"
            }
            Self::PublicKeyNotFound { .. } => {
                "SSH public key file not found.\n\
                 \n\
                 The specified public key file does not exist or is not accessible.\n\
                 \n\
                 Common causes:\n\
                 - Incorrect file path in configuration\n\
                 - File was moved or deleted\n\
                 - Insufficient permissions to access the file\n\
                 \n\
                 Fix:\n\
                 1. Check that the file path is correct in your configuration\n\
                 2. Verify the file exists: ls -la <path>\n\
                 3. Ensure you have read permissions on the file\n\
                 4. Generate public key from private key if needed: ssh-keygen -y -f <private_key> > <public_key>"
            }
            Self::RelativePrivateKeyPath { .. } => {
                // Note: Can't use format! in const context, so we use a static message
                // The actual path will be shown in the error message itself
                "SSH private key path must be absolute.\n\
                 \n\
                 SSH key paths must be absolute to ensure they work correctly across\n\
                 different working directories and command invocations.\n\
                 \n\
                 Fix:\n\
                 1. Convert relative path to absolute path:\n\
                 \n\
                 Use the `realpath` command to get the absolute path:\n\
                 \n\
                 realpath <your-relative-path>\n\
                 \n\
                 Example:\n\
                 - Current (relative): fixtures/testing_rsa\n\
                 - Command: realpath fixtures/testing_rsa\n\
                 - Result: /home/user/project/fixtures/testing_rsa\n\
                 \n\
                 2. Update your configuration file with the absolute path\n\
                 \n\
                 3. Alternative approaches:\n\
                 - Use ~ for home directory (e.g., ~/.ssh/id_rsa)\n\
                 - Use environment variables (e.g., $HOME/.ssh/id_rsa)\n\
                 \n\
                 Why absolute paths?\n\
                 - Commands may run from different working directories\n\
                 - Environment state persists paths that must remain valid\n\
                 - Multi-command workflows (create → provision → configure)"
            }
            Self::RelativePublicKeyPath { .. } => {
                "SSH public key path must be absolute.\n\
                 \n\
                 SSH key paths must be absolute to ensure they work correctly across\n\
                 different working directories and command invocations.\n\
                 \n\
                 Fix:\n\
                 1. Convert relative path to absolute path:\n\
                 \n\
                 Use the `realpath` command to get the absolute path:\n\
                 \n\
                 realpath <your-relative-path>\n\
                 \n\
                 Example:\n\
                 - Current (relative): fixtures/testing_rsa.pub\n\
                 - Command: realpath fixtures/testing_rsa.pub\n\
                 - Result: /home/user/project/fixtures/testing_rsa.pub\n\
                 \n\
                 2. Update your configuration file with the absolute path\n\
                 \n\
                 3. Alternative approaches:\n\
                 - Use ~ for home directory (e.g., ~/.ssh/id_rsa.pub)\n\
                 - Use environment variables (e.g., $HOME/.ssh/id_rsa.pub)\n\
                 \n\
                 Why absolute paths?\n\
                 - Commands may run from different working directories\n\
                 - Environment state persists paths that must remain valid\n\
                 - Multi-command workflows (create → provision → configure)"
            }
            Self::InvalidPort { .. } => {
                "Invalid SSH port number.\n\
                 \n\
                 SSH port must be between 1 and 65535.\n\
                 \n\
                 Common SSH ports:\n\
                 - 22 (standard SSH port)\n\
                 - 2222 (common alternative)\n\
                 \n\
                 Fix: Update the SSH port in your configuration to a valid port number (1-65535)."
            }
            Self::InvalidBindAddress { .. } => {
                "Invalid bind address format.\n\
                 \n\
                 Bind addresses must be in the format IP:PORT (e.g., '0.0.0.0:8080').\n\
                 \n\
                 Valid examples:\n\
                 - '0.0.0.0:6969' (bind to all interfaces on port 6969)\n\
                 - '127.0.0.1:7070' (bind to localhost on port 7070)\n\
                 - '[::]:1212' (bind to all IPv6 interfaces on port 1212)\n\
                 \n\
                 Common mistakes:\n\
                 - Missing port number (e.g., '0.0.0.0')\n\
                 - Invalid IP address format\n\
                 - Port number out of range (must be 1-65535)\n\
                 \n\
                 Fix: Update the bind_address in your configuration to use valid IP:PORT format."
            }
            Self::DynamicPortNotSupported { .. } => {
                "Dynamic port assignment (port 0) is not supported.\n\
                 \n\
                 Port 0 tells the operating system to assign any available port dynamically.\n\
                 This conflicts with our deployment workflow which requires:\n\
                 - Firewall rules configured before service starts\n\
                 - Predictable ports for health checks and monitoring\n\
                 - Consistent port numbers across deployment phases\n\
                 \n\
                 Why:\n\
                 The 'configure' command must open firewall ports before the tracker starts.\n\
                 With port 0, we won't know which port to open until after the service runs.\n\
                 \n\
                 Solution: Specify an explicit port number in your configuration:\n\
                 - UDP Tracker: Use a port like 6969 (default)\n\
                 - HTTP Tracker: Use a port like 7070 (default)\n\
                 - HTTP API: Use a port like 1212 (default)\n\
                 \n\
                 Example:\n\
                 Instead of: \"bind_address\": \"0.0.0.0:0\"\n\
                 Use:        \"bind_address\": \"0.0.0.0:6969\"\n\
                 \n\
                 See docs/decisions/port-zero-not-supported.md for details."
            }
            Self::TemplateSerializationFailed { .. } => {
                "Template serialization failed.\n\
                 \n\
                 This indicates an internal error in template generation.\n\
                 \n\
                 Common causes:\n\
                 - Software bug in template generation logic\n\
                 - Invalid data structure for JSON serialization\n\
                 \n\
                 Fix:\n\
                 1. Report this issue with full error details\n\
                 2. Check for application updates\n\
                 \n\
                 This is likely a software bug that needs to be reported."
            }
            Self::TemplateDirectoryCreationFailed { .. } => {
                "Failed to create directory for template file.\n\
                 \n\
                 Common causes:\n\
                 - Insufficient permissions to create directory\n\
                 - No disk space available\n\
                 - A file exists with the same name as the directory\n\
                 - Path length exceeds system limits\n\
                 \n\
                 Fix:\n\
                 1. Check write permissions for the parent directory\n\
                 2. Verify disk space is available: df -h\n\
                 3. Ensure no file exists with the same name as the directory\n\
                 4. Try using a shorter path"
            }
            Self::TemplateFileWriteFailed { .. } => {
                "Failed to write template file.\n\
                 \n\
                 Common causes:\n\
                 - Insufficient permissions to write file\n\
                 - No disk space available\n\
                 - File is open in another application\n\
                 - Antivirus software blocking file creation\n\
                 \n\
                 Fix:\n\
                 1. Check write permissions for the target file and directory\n\
                 2. Verify disk space is available: df -h\n\
                 3. Ensure the file is not open in another application\n\
                 4. Check if antivirus software is blocking file creation"
            }
            Self::GrafanaRequiresPrometheus => {
                "Grafana requires Prometheus to be enabled.\n\
                 \n\
                 Grafana is a visualization tool that displays metrics collected by Prometheus.\n\
                 It cannot function without Prometheus as its data source.\n\
                 \n\
                 Current configuration issue:\n\
                 - Grafana section is present in your configuration\n\
                 - Prometheus section is absent or disabled\n\
                 \n\
                 Fix (choose one):\n\
                 \n\
                 Option 1 - Enable Prometheus:\n\
                 Add a prometheus section to your environment configuration:\n\
                 \n\
                 \"prometheus\": {\n\
                   \"scrape_interval_in_secs\": 15\n\
                 }\n\
                 \n\
                 Option 2 - Disable Grafana:\n\
                 Remove the grafana section from your environment configuration\n\
                 \n\
                 Note: Prometheus can run independently without Grafana, but Grafana\n\
                 requires Prometheus to be enabled."
            }
            Self::InvalidPrometheusConfig(_) => {
                "Invalid Prometheus configuration.\n\
                 \n\
                 Prometheus scrape_interval must be a positive integer representing seconds.\n\
                 \n\
                 Requirements:\n\
                 - Must be greater than 0\n\
                 - Represents the interval in seconds between metric collections\n\
                 \n\
                 Common values:\n\
                 - 15 (default, recommended for most use cases)\n\
                 - 10 (high-frequency monitoring)\n\
                 - 30 (lower resource usage)\n\
                 - 60 (minimal monitoring overhead)\n\
                 \n\
                 Fix:\n\
                 Update the scrape_interval_in_secs in your configuration:\n\
                 \n\
                 \"prometheus\": {\n\
                   \"scrape_interval_in_secs\": 15\n\
                 }\n\
                 \n\
                 Note: The template automatically adds the 's' suffix (e.g., 15 becomes '15s'),\n\
                 so you only need to specify the numeric value."
            }
            Self::InvalidBackupConfig(_) => {
                "Invalid Backup configuration.\n\
                 \n\
                 Backup configuration errors can occur due to:\n\
                 1. Invalid cron schedule format\n\
                 2. Invalid retention days (must be greater than 0)\n\
                 \n\
                 Cron Schedule Requirements:\n\
                 - Must use 5-field format: minute hour day month weekday\n\
                 - Only supports: digits, *, -, /, , (comma), and spaces\n\
                 - Each field must have valid values for its position\n\
                 \n\
                 Common cron schedules:\n\
                 - \"0 3 * * *\" - 3:00 AM daily (default)\n\
                 - \"0 */6 * * *\" - Every 6 hours\n\
                 - \"0 0 * * 0\" - Midnight every Sunday\n\
                 - \"30 2 1 * *\" - 2:30 AM on the 1st of every month\n\
                 \n\
                 Retention Days Requirements:\n\
                 - Must be greater than 0\n\
                 - Represents how many days to keep backup files\n\
                 \n\
                 Fix:\n\
                 Update your backup configuration:\n\
                 \n\
                 \"backup\": {\n\
                   \"schedule\": \"0 3 * * *\",\n\
                   \"retention_days\": 7\n\
                 }\n\
                 \n\
                 Or use defaults by providing an empty object:\n\
                 \n\
                 \"backup\": {}\n\
                 \n\
                 Note: All fields have sensible defaults (3:00 AM daily, 7 days retention)."
            }
            Self::TrackerConfigValidation(_) => {
                "Tracker configuration validation failed.\n\
                 \n\
                 This error indicates a problem with the tracker service configuration,\n\
                 typically related to socket address (IP:Port:Protocol) conflicts.\n\
                 \n\
                 The error message above provides specific details about:\n\
                 - Which services are in conflict\n\
                 - The conflicting socket addresses\n\
                 - Why the configuration is invalid\n\
                 \n\
                 Common issues:\n\
                 1. Multiple services on same TCP port (HTTP tracker + API)\n\
                 2. Duplicate UDP tracker ports\n\
                 3. Duplicate HTTP tracker ports\n\
                 \n\
                 Note: UDP and TCP can share the same port (different protocols),\n\
                 but this is not recommended for clarity.\n\
                 \n\
                 Related: docs/external-issues/tracker/udp-tcp-port-sharing-allowed.md"
            }
            Self::HttpApiConfigInvalid(inner) => {
                // Delegate to domain error's help method for detailed guidance
                inner.help()
            }
            Self::UdpTrackerConfigInvalid(inner) => {
                // Delegate to domain error's help method for detailed guidance
                inner.help()
            }
            Self::HttpTrackerConfigInvalid(inner) => {
                // Delegate to domain error's help method for detailed guidance
                inner.help()
            }
            Self::HealthCheckApiConfigInvalid(inner) => {
                // Delegate to domain error's help method for detailed guidance
                inner.help()
            }
            Self::SqliteConfigInvalid(inner) => {
                // Delegate to domain error's help method for detailed guidance
                inner.help()
            }
            Self::MysqlConfigInvalid(inner) => {
                // Delegate to domain error's help method for detailed guidance
                inner.help()
            }
            Self::HttpsConfigInvalid(inner) => {
                // Delegate to domain error's help method for detailed guidance
                inner.help()
            }
            Self::InvalidDomain { .. } => {
                "Invalid domain name format for TLS configuration.\n\
                 \n\
                 Domain names are used for:\n\
                 - HTTPS certificate acquisition (Let's Encrypt HTTP-01 challenge)\n\
                 - Caddy reverse proxy routing\n\
                 - SNI-based TLS termination\n\
                 \n\
                 Requirements:\n\
                 - Contains only letters, numbers, dots, and hyphens\n\
                 - Has at least one dot (TLD separator)\n\
                 - Doesn't start or end with dots or hyphens\n\
                 \n\
                 Valid examples:\n\
                 - api.example.com\n\
                 - tracker.torrust.org\n\
                 - grafana.my-project.io\n\
                 \n\
                 Invalid examples:\n\
                 - localhost (no TLD)\n\
                 - -example.com (starts with hyphen)\n\
                 - example_domain.com (underscore not allowed)\n\
                 \n\
                 Fix:\n\
                 Update the domain in your service's tls configuration:\n\
                 \n\
                 \"tls\": {\n\
                   \"domain\": \"api.yourdomain.com\"\n\
                 }\n\
                 \n\
                 Note: The domain must point to your server's IP before certificate acquisition."
            }
            // Note: TLS/HTTPS cross-service validation errors now use UserInputsError
            // variants (TlsServicesWithoutHttpsSection, HttpsSectionWithoutTlsServices)
            // which are wrapped via CrossServiceValidation variant below.
            Self::TlsProxyWithoutDomain { .. } => {
                "TLS proxy enabled but domain is missing.\n\
                 \n\
                 When use_tls_proxy is set to true, you must also specify a domain name\n\
                 for the HTTPS certificate acquisition.\n\
                 \n\
                 The domain is required because:\n\
                 - Caddy needs it to request a Let's Encrypt certificate\n\
                 - SNI-based TLS termination routes requests to the correct service\n\
                 \n\
                 Fix:\n\
                 Add a domain when enabling the TLS proxy:\n\
                 \n\
                 For HTTP Tracker:\n\
                 \"http_trackers\": [{\n\
                   \"bind_address\": \"0.0.0.0:7070\",\n\
                   \"domain\": \"tracker.example.com\",\n\
                   \"use_tls_proxy\": true\n\
                 }]\n\
                 \n\
                 Alternatively, if you don't want HTTPS for this service,\n\
                 remove or set use_tls_proxy to false."
            }
            Self::CrossServiceValidation(e) => e.help(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EnvironmentName;
    use crate::shared::Username;

    #[test]
    fn it_should_return_error_when_environment_name_is_invalid() {
        let result = EnvironmentName::new("Invalid_Name");
        assert!(result.is_err());

        let error = CreateConfigError::from(result.unwrap_err());
        assert!(error.to_string().contains("Invalid environment name"));
        assert!(error.help().contains("lowercase letters"));
        assert!(error.help().contains("dashes"));
    }

    #[test]
    fn it_should_return_error_when_username_is_invalid() {
        let result = Username::new("123invalid");
        assert!(result.is_err());

        let error = CreateConfigError::from(result.unwrap_err());
        assert!(error.to_string().contains("Invalid SSH username"));
        assert!(error.help().contains("Start with a letter"));
        assert!(error.help().contains("1-32 characters"));
    }

    #[test]
    fn it_should_return_error_when_private_key_file_not_found() {
        let error = CreateConfigError::PrivateKeyNotFound {
            path: PathBuf::from("/nonexistent/key"),
        };
        assert!(error.to_string().contains("private key file not found"));
        assert!(error.to_string().contains("/nonexistent/key"));
        assert!(error.help().contains("Check that the file path is correct"));
        assert!(error.help().contains("ssh-keygen"));
    }

    #[test]
    fn it_should_return_error_when_public_key_file_not_found() {
        let error = CreateConfigError::PublicKeyNotFound {
            path: PathBuf::from("/nonexistent/key.pub"),
        };
        assert!(error.to_string().contains("public key file not found"));
        assert!(error.to_string().contains("/nonexistent/key.pub"));
        assert!(error.help().contains("Check that the file path is correct"));
        assert!(error.help().contains("ssh-keygen -y"));
    }

    #[test]
    fn it_should_return_error_when_port_is_invalid() {
        let error = CreateConfigError::InvalidPort { port: 0 };
        assert!(error.to_string().contains("Invalid SSH port"));
        assert!(error.to_string().contains("must be between 1 and 65535"));
        assert!(error.help().contains("22"));
        assert!(error.help().contains("2222"));
    }

    #[test]
    fn it_should_provide_help_messages_for_all_errors() {
        // Verify all error variants have help text
        let errors = vec![
            CreateConfigError::PrivateKeyNotFound {
                path: PathBuf::from("/test"),
            },
            CreateConfigError::PublicKeyNotFound {
                path: PathBuf::from("/test"),
            },
            CreateConfigError::InvalidPort { port: 0 },
            CreateConfigError::InvalidInstanceName {
                name: "invalid-".to_string(),
                reason: "ends with dash".to_string(),
            },
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Fix") || help.contains("Common"),
                "Help should contain actionable guidance"
            );
        }
    }

    #[test]
    fn it_should_return_error_when_instance_name_is_invalid() {
        let error = CreateConfigError::InvalidInstanceName {
            name: "invalid-".to_string(),
            reason: "Instance name must not end with a dash".to_string(),
        };

        assert!(error.to_string().contains("Invalid instance name"));
        assert!(error.to_string().contains("invalid-"));
        assert!(error.help().contains("1-63 characters"));
        assert!(error.help().contains("ASCII letters"));
        assert!(error.help().contains("auto-generation"));
    }

    #[test]
    fn it_should_return_error_when_template_serialization_fails() {
        // Simulate serialization error (hard to create naturally)
        let json_error = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error = CreateConfigError::TemplateSerializationFailed { source: json_error };

        assert!(error
            .to_string()
            .contains("serialize configuration template"));
        assert!(error.help().contains("internal error"));
        assert!(error.help().contains("Report this issue"));
    }

    #[test]
    fn it_should_return_error_when_template_directory_creation_fails() {
        let error = CreateConfigError::TemplateDirectoryCreationFailed {
            path: PathBuf::from("/test/path"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };

        assert!(error.to_string().contains("Failed to create directory"));
        assert!(error.to_string().contains("/test/path"));
        assert!(error.help().contains("permissions"));
        assert!(error.help().contains("df -h"));
    }

    #[test]
    fn it_should_return_error_when_template_file_write_fails() {
        let error = CreateConfigError::TemplateFileWriteFailed {
            path: PathBuf::from("/test/file.json"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test"),
        };

        assert!(error.to_string().contains("Failed to write template file"));
        assert!(error.to_string().contains("/test/file.json"));
        assert!(error.help().contains("permissions"));
        assert!(error.help().contains("disk space"));
    }

    #[test]
    fn it_should_return_error_when_grafana_requires_prometheus() {
        let error = CreateConfigError::GrafanaRequiresPrometheus;

        assert!(error.to_string().contains("Grafana requires Prometheus"));
        assert!(error.help().contains("Grafana section is present"));
        assert!(error.help().contains("Prometheus section is absent"));
        assert!(error.help().contains("Add a prometheus section"));
        assert!(error.help().contains("Remove the grafana section"));
        assert!(error.help().contains("scrape_interval"));
    }
}
