# Database Defaults
# Default values with type contracts applied

let schemas = import "../schemas/database.ncl" in

{
  # Default database: SQLite (simple, file-based)
  database | schemas.Database = {
    driver = "sqlite3",
    database_name = "tracker.db",
  },
}
