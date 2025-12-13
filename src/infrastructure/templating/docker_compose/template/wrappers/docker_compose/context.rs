//! Context for the docker-compose.yml.tera template
//!
//! This module defines the structure and validation for Docker Compose services
//! that will be rendered into the docker-compose.yml file.

use serde::Serialize;

/// Database configuration for docker-compose template
#[derive(Serialize, Debug, Clone)]
pub struct DatabaseConfig {
    /// Database driver: "sqlite3" or "mysql"
    pub driver: String,
    /// MySQL-specific configuration (only present when driver == "mysql")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql: Option<MysqlConfig>,
}

/// `MySQL`-specific configuration
#[derive(Serialize, Debug, Clone)]
pub struct MysqlConfig {
    /// `MySQL` root password
    pub root_password: String,
    /// `MySQL` database name
    pub database: String,
    /// `MySQL` user
    pub user: String,
    /// `MySQL` password
    pub password: String,
    /// `MySQL` port
    pub port: u16,
}

/// Context for rendering the docker-compose.yml template
///
/// Contains all variables needed for the Docker Compose service configuration.
#[derive(Serialize, Debug, Clone)]
pub struct DockerComposeContext {
    /// Database configuration
    pub database: DatabaseConfig,
}

impl DockerComposeContext {
    /// Creates a new `DockerComposeContext` with `SQLite` configuration (default)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::DockerComposeContext;
    ///
    /// let context = DockerComposeContext::new_sqlite();
    /// assert_eq!(context.database().driver(), "sqlite3");
    /// ```
    #[must_use]
    pub fn new_sqlite() -> Self {
        Self {
            database: DatabaseConfig {
                driver: "sqlite3".to_string(),
                mysql: None,
            },
        }
    }

    /// Creates a new `DockerComposeContext` with `MySQL` configuration
    ///
    /// # Arguments
    ///
    /// * `root_password` - `MySQL` root password
    /// * `database` - `MySQL` database name
    /// * `user` - `MySQL` user
    /// * `password` - `MySQL` password
    /// * `port` - `MySQL` port
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::DockerComposeContext;
    ///
    /// let context = DockerComposeContext::new_mysql(
    ///     "root_pass".to_string(),
    ///     "tracker_db".to_string(),
    ///     "tracker_user".to_string(),
    ///     "user_pass".to_string(),
    ///     3306,
    /// );
    /// assert_eq!(context.database().driver(), "mysql");
    /// ```
    #[must_use]
    pub fn new_mysql(
        root_password: String,
        database: String,
        user: String,
        password: String,
        port: u16,
    ) -> Self {
        Self {
            database: DatabaseConfig {
                driver: "mysql".to_string(),
                mysql: Some(MysqlConfig {
                    root_password,
                    database,
                    user,
                    password,
                    port,
                }),
            },
        }
    }

    /// Get the database configuration
    #[must_use]
    pub fn database(&self) -> &DatabaseConfig {
        &self.database
    }
}

impl DatabaseConfig {
    /// Get the database driver name
    #[must_use]
    pub fn driver(&self) -> &str {
        &self.driver
    }

    /// Get the `MySQL` configuration if present
    #[must_use]
    pub fn mysql(&self) -> Option<&MysqlConfig> {
        self.mysql.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_context_with_sqlite_configuration() {
        let context = DockerComposeContext::new_sqlite();

        assert_eq!(context.database().driver(), "sqlite3");
        assert!(context.database().mysql().is_none());
    }

    #[test]
    fn it_should_create_context_with_mysql_configuration() {
        let context = DockerComposeContext::new_mysql(
            "root123".to_string(),
            "tracker".to_string(),
            "tracker_user".to_string(),
            "pass456".to_string(),
            3306,
        );

        assert_eq!(context.database().driver(), "mysql");
        assert!(context.database().mysql().is_some());

        let mysql = context.database().mysql().unwrap();
        assert_eq!(mysql.root_password, "root123");
        assert_eq!(mysql.database, "tracker");
        assert_eq!(mysql.user, "tracker_user");
        assert_eq!(mysql.password, "pass456");
        assert_eq!(mysql.port, 3306);
    }

    #[test]
    fn it_should_be_serializable_with_sqlite() {
        let context = DockerComposeContext::new_sqlite();

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("sqlite3"));
        assert!(!serialized.contains("mysql"));
    }

    #[test]
    fn it_should_be_serializable_with_mysql() {
        let context = DockerComposeContext::new_mysql(
            "root".to_string(),
            "db".to_string(),
            "user".to_string(),
            "pass".to_string(),
            3306,
        );

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("mysql"));
        assert!(serialized.contains("root"));
        assert!(serialized.contains("db"));
        assert!(serialized.contains("user"));
        assert!(serialized.contains("pass"));
        assert!(serialized.contains("3306"));
    }

    #[test]
    fn it_should_be_cloneable() {
        let context = DockerComposeContext::new_mysql(
            "root".to_string(),
            "db".to_string(),
            "user".to_string(),
            "pass".to_string(),
            3306,
        );

        let cloned = context.clone();
        assert_eq!(cloned.database().driver(), "mysql");
    }
}
