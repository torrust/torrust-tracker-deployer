# Tracker Configuration Template - TOML format
#
# This Nickel template generates the tracker.toml configuration file.
# It uses user-provided configuration and builds the complete tracker setup.
#
# Generated from: user provisioning configuration
# Output format: TOML

# Import the user configuration
let config = import "../../values/config.ncl" in

# Extract all values into local variables to avoid circular references
let tracker = config.tracker in
let core = tracker.core in

# Database configuration
let db_driver = core.database.driver in
let db_name = core.database.database_name in
let db_path =
  if db_driver == "sqlite3" then
    "/var/lib/torrust/tracker/database/%{db_name}"
  else
    "mysql://user:pass@localhost:3306/tracker"
in

# Tracker configuration
let tracker_private = core.private in

# HTTP API configuration
let http_api_bind_addr = tracker.http_api.bind_address in

# UDP trackers
let udp_list = tracker.udp_trackers in

# HTTP trackers
let http_list = tracker.http_trackers in

# Export the final TOML structure as JSON (for conversion to TOML)
{
  metadata = {
    app = "torrust-tracker",
    purpose = "configuration",
    schema_version = "2.0.0",
  },
  logging = {
    threshold = "info",
  },
  core = {
    listed = false,
    private = tracker_private,
    tracker_policy = {
      persistent_torrent_completed_stat = true,
    },
    announce_policy = {
      interval = 300,
      interval_min = 300,
    },
    net = {
      on_reverse_proxy = true,
    },
    database = {
      driver = db_driver,
      path = db_path,
    },
  },
  udp_trackers = udp_list,
  http_trackers = http_list,
  http_api = {
    bind_address = http_api_bind_addr,
  },
}
