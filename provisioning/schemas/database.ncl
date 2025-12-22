# Database Schema
# Type contracts for database configuration
# Supports SQLite (file-based) or MySQL (server-based) databases

{
  # Database base contract with optional fields
  # Validation logic in config layer ensures required fields per driver type
  Database = {
    driver | String,
    database_name | String | optional,
    host | String | optional,
    port | Number | optional,
    username | String | optional,
    password | String | optional,
  },
}
