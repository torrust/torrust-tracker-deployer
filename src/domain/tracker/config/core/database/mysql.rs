//! `MySQL` database configuration

use serde::{Deserialize, Deserializer, Serialize};

use crate::shared::Password;

/// Error type for `MySQL` configuration validation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MysqlConfigError {
    /// Database name cannot be empty
    #[error("MySQL database name cannot be empty")]
    EmptyDatabaseName,
    /// Host cannot be empty
    #[error("MySQL host cannot be empty")]
    EmptyHost,
    /// Port 0 is not valid
    #[error("MySQL port 0 is not valid")]
    InvalidPort,
    /// Username cannot be empty
    #[error("MySQL username cannot be empty")]
    EmptyUsername,
}

impl MysqlConfigError {
    /// Returns detailed help text for resolving this error
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::EmptyDatabaseName => {
                "MySQL database name cannot be empty.\n\
                 \n\
                 The database_name field specifies the MySQL database to use.\n\
                 \n\
                 Fix:\n\
                 Set the database_name field in your database configuration:\n\
                 \n\
                 \"database\": {\n\
                   \"driver\": \"mysql\",\n\
                   \"database_name\": \"tracker\",\n\
                   ...\n\
                 }"
            }
            Self::EmptyHost => {
                "MySQL host cannot be empty.\n\
                 \n\
                 The host field specifies the MySQL server address.\n\
                 \n\
                 Common values:\n\
                 - \"localhost\" for local MySQL server\n\
                 - \"mysql\" when using Docker Compose service name\n\
                 - IP address or hostname for remote servers\n\
                 \n\
                 Fix:\n\
                 Set the host field in your database configuration:\n\
                 \n\
                 \"database\": {\n\
                   \"driver\": \"mysql\",\n\
                   \"host\": \"mysql\",\n\
                   ...\n\
                 }"
            }
            Self::InvalidPort => {
                "MySQL port 0 is not valid.\n\
                 \n\
                 Port 0 means dynamic port assignment, which is not supported\n\
                 for database connections.\n\
                 \n\
                 The default MySQL port is 3306.\n\
                 \n\
                 Fix:\n\
                 Set a valid port in your database configuration:\n\
                 \n\
                 \"database\": {\n\
                   \"driver\": \"mysql\",\n\
                   \"port\": 3306,\n\
                   ...\n\
                 }"
            }
            Self::EmptyUsername => {
                "MySQL username cannot be empty.\n\
                 \n\
                 The username field specifies the MySQL user for authentication.\n\
                 \n\
                 Fix:\n\
                 Set the username field in your database configuration:\n\
                 \n\
                 \"database\": {\n\
                   \"driver\": \"mysql\",\n\
                   \"username\": \"tracker_user\",\n\
                   ...\n\
                 }"
            }
        }
    }
}

/// `MySQL` database configuration
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MysqlConfig {
    /// `MySQL` server host (e.g., "localhost", "mysql")
    host: String,
    /// `MySQL` server port (typically 3306)
    port: u16,
    /// Database name
    database_name: String,
    /// Database username
    username: String,
    /// Database password (redacted in debug output)
    password: Password,
}

impl MysqlConfig {
    /// Creates a new `MySQL` configuration with validation
    ///
    /// # Errors
    ///
    /// - `EmptyDatabaseName` if database name is empty
    /// - `EmptyHost` if host is empty
    /// - `InvalidPort` if port is 0
    /// - `EmptyUsername` if username is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::MysqlConfig;
    /// use torrust_tracker_deployer_lib::shared::Password;
    ///
    /// let config = MysqlConfig::new(
    ///     "localhost",
    ///     3306,
    ///     "tracker",
    ///     "tracker_user",
    ///     Password::from("secure_password"),
    /// ).unwrap();
    ///
    /// assert_eq!(config.host(), "localhost");
    /// assert_eq!(config.port(), 3306);
    /// assert_eq!(config.database_name(), "tracker");
    /// ```
    pub fn new(
        host: impl Into<String>,
        port: u16,
        database_name: impl Into<String>,
        username: impl Into<String>,
        password: Password,
    ) -> Result<Self, MysqlConfigError> {
        let host = host.into();
        let database_name = database_name.into();
        let username = username.into();

        if host.is_empty() {
            return Err(MysqlConfigError::EmptyHost);
        }
        if port == 0 {
            return Err(MysqlConfigError::InvalidPort);
        }
        if database_name.is_empty() {
            return Err(MysqlConfigError::EmptyDatabaseName);
        }
        if username.is_empty() {
            return Err(MysqlConfigError::EmptyUsername);
        }

        Ok(Self {
            host,
            port,
            database_name,
            username,
            password,
        })
    }

    /// Returns the `MySQL` server host
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the `MySQL` server port
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the database name
    #[must_use]
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Returns the database username
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns a reference to the database password
    #[must_use]
    pub fn password(&self) -> &Password {
        &self.password
    }
}

/// Intermediate struct for deserialization
#[derive(Deserialize)]
struct MysqlConfigRaw {
    host: String,
    port: u16,
    database_name: String,
    username: String,
    password: Password,
}

impl<'de> Deserialize<'de> for MysqlConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = MysqlConfigRaw::deserialize(deserializer)?;
        Self::new(
            raw.host,
            raw.port,
            raw.database_name,
            raw.username,
            raw.password,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_mysql_config_when_all_fields_are_valid() {
        let config = MysqlConfig::new(
            "localhost",
            3306,
            "tracker",
            "tracker_user",
            Password::from("secure_password"),
        )
        .unwrap();

        assert_eq!(config.host(), "localhost");
        assert_eq!(config.port(), 3306);
        assert_eq!(config.database_name(), "tracker");
        assert_eq!(config.username(), "tracker_user");
        assert_eq!(config.password().expose_secret(), "secure_password");
    }

    #[test]
    fn it_should_reject_empty_host_when_creating_mysql_config() {
        let result = MysqlConfig::new("", 3306, "tracker", "user", Password::from("pass"));
        assert!(matches!(result, Err(MysqlConfigError::EmptyHost)));
    }

    #[test]
    fn it_should_reject_port_zero_when_creating_mysql_config() {
        let result = MysqlConfig::new("localhost", 0, "tracker", "user", Password::from("pass"));
        assert!(matches!(result, Err(MysqlConfigError::InvalidPort)));
    }

    #[test]
    fn it_should_reject_empty_database_name_when_creating_mysql_config() {
        let result = MysqlConfig::new("localhost", 3306, "", "user", Password::from("pass"));
        assert!(matches!(result, Err(MysqlConfigError::EmptyDatabaseName)));
    }

    #[test]
    fn it_should_reject_empty_username_when_creating_mysql_config() {
        let result = MysqlConfig::new("localhost", 3306, "tracker", "", Password::from("pass"));
        assert!(matches!(result, Err(MysqlConfigError::EmptyUsername)));
    }

    #[test]
    fn it_should_serialize_mysql_config() {
        let config = MysqlConfig::new(
            "mysql",
            3306,
            "tracker",
            "tracker_user",
            Password::from("pass123"),
        )
        .unwrap();

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["host"], "mysql");
        assert_eq!(json["port"], 3306);
        assert_eq!(json["database_name"], "tracker");
        assert_eq!(json["username"], "tracker_user");
        assert_eq!(json["password"], "pass123");
    }

    #[test]
    fn it_should_deserialize_mysql_config_when_valid() {
        let json = r#"{
            "host": "localhost",
            "port": 3306,
            "database_name": "tracker",
            "username": "tracker_user",
            "password": "secure_password"
        }"#;
        let config: MysqlConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.host(), "localhost");
        assert_eq!(config.port(), 3306);
        assert_eq!(config.database_name(), "tracker");
        assert_eq!(config.username(), "tracker_user");
        assert_eq!(config.password().expose_secret(), "secure_password");
    }

    #[test]
    fn it_should_reject_deserialization_when_port_is_zero() {
        let json = r#"{
            "host": "localhost",
            "port": 0,
            "database_name": "tracker",
            "username": "user",
            "password": "pass"
        }"#;
        let result: Result<MysqlConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
