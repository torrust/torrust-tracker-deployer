# Tracker Defaults
# Default values with type contracts applied

let schemas = import "../schemas/tracker.ncl" in
let schemas_db = import "../schemas/database.ncl" in

{
  tracker | schemas.Tracker = {
    core | schemas.TrackerCore = {
      # Default: public tracker (not private)
      private = false,

      # Default database: SQLite with filename "tracker.db"
      database | schemas_db.Database = {
        driver = "sqlite3",
        database_name = "tracker.db",
      },
    },

    # Default UDP tracker: standard bittorrent port 6969
    udp_trackers = [
      ({ bind_address = "0.0.0.0:6969", } | schemas.TrackerUdp),
    ],

    # Default HTTP tracker: port 7070
    http_trackers = [
      ({ bind_address = "0.0.0.0:7070", } | schemas.TrackerHttp),
    ],

    # HTTP API: REQUIRED - must be provided by user in user_config
    # No default exists; user must explicitly define bind_address and admin_token
  },
}
