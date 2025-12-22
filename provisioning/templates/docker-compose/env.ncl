# Docker Compose Environment File Template
#
# This Nickel template generates the .env file for docker-compose services.
# Environment variables are used by Docker Compose to configure services at runtime.
#
# Output format: KEY=VALUE (one per line, sorted alphabetically)

# Import the user configuration
let config = import "../../values/config.ncl" in

# Extract configuration values
let tracker = config.tracker in
let core = tracker.core in
let database = core.database in
let http_api = tracker.http_api in

# Build the environment object with key-value pairs
{
  # =============================================================================
  # Tracker Service Configuration
  # =============================================================================

  TORRUST_TRACKER_CONFIG_TOML_PATH = "/etc/torrust/tracker/tracker.toml",
  TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER = database.driver,
  TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN = http_api.admin_token,

  # Tracker communication
  HTTP_API_BIND_ADDRESS = http_api.bind_address,

  # Storage paths (mounted to host)
  TRACKER_STORAGE_LIB = "./storage/tracker/lib:/var/lib/torrust/tracker:Z",
  TRACKER_STORAGE_LOG = "./storage/tracker/log:/var/log/torrust/tracker:Z",
  TRACKER_STORAGE_ETC = "./storage/tracker/etc:/etc/torrust/tracker:Z",
}
