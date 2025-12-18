//! `MySQL` database configuration

use serde::{Deserialize, Serialize};

use crate::shared::Password;

/// `MySQL` database configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MysqlConfig {
    /// `MySQL` server host (e.g., "localhost", "mysql")
    pub host: String,
    /// `MySQL` server port (typically 3306)
    pub port: u16,
    /// Database name
    pub database_name: String,
    /// Database username
    pub username: String,
    /// Database password (redacted in debug output)
    pub password: Password,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_should_create_mysql_config() {
        let config = MysqlConfig {
            host: "localhost".to_string(),
            port: 3306,
            database_name: "tracker".to_string(),
            username: "tracker_user".to_string(),
            password: Password::from("secure_password"),
        };

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.database_name, "tracker");
        assert_eq!(config.username, "tracker_user");
        assert_eq!(config.password.expose_secret(), "secure_password");
    }

    #[test]
    fn it_should_serialize_mysql_config() {
        let config = MysqlConfig {
            host: "mysql".to_string(),
            port: 3306,
            database_name: "tracker".to_string(),
            username: "tracker_user".to_string(),
            password: Password::from("pass123"),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["host"], "mysql");
        assert_eq!(json["port"], 3306);
        assert_eq!(json["database_name"], "tracker");
        assert_eq!(json["username"], "tracker_user");
        assert_eq!(json["password"], "pass123");
    }

    #[test]
    fn it_should_deserialize_mysql_config() {
        let json = r#"{
            "host": "localhost",
            "port": 3306,
            "database_name": "tracker",
            "username": "tracker_user",
            "password": "secure_password"
        }"#;
        let config: MysqlConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.database_name, "tracker");
        assert_eq!(config.username, "tracker_user");
        assert_eq!(config.password.expose_secret(), "secure_password");
    }
}
