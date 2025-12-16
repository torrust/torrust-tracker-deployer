//! `MySQL` database configuration

use serde::{Deserialize, Serialize};

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
    /// Database password
    pub password: String,
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
            password: "secure_password".to_string(),
        };

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.database_name, "tracker");
        assert_eq!(config.username, "tracker_user");
        assert_eq!(config.password, "secure_password");
    }

    #[test]
    fn it_should_serialize_mysql_config() {
        let config = MysqlConfig {
            host: "mysql".to_string(),
            port: 3306,
            database_name: "tracker".to_string(),
            username: "tracker_user".to_string(),
            password: "pass123".to_string(),
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
        assert_eq!(config.password, "secure_password");
    }
}
