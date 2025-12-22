# Docker Compose Configuration Template
#
# This Nickel template generates the docker-compose.yml file.
# It defines all services (tracker, optional prometheus, optional MySQL) and their configuration.
#
# Output format: YAML

# Import the user configuration
let config = import "../../values/config.ncl" in

# Extract configuration
let tracker_config = config.tracker in
let core = tracker_config.core in
let database = core.database in
let http_api = tracker_config.http_api in
let features = config.features in

# Extract port from bind_address (format: "0.0.0.0:PORT")
let extract_port = fun bind_addr =>
  let parts = bind_addr | std.string.split ":" in
  if (std.array.length parts) >= 2 then
    parts | std.array.last
  else
    "1212"
in

# Prometheus configuration (optional)
let prometheus_enabled = features.prometheus.enabled in

# Base tracker service
let tracker_service = {
  image = "torrust/tracker:develop",
  container_name = "tracker",
  tty = true,
  restart = "unless-stopped",
  environment = {
    USER_ID = "1000",
    TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER = database.driver,
    TORRUST_TRACKER_CONFIG_TOML_PATH = "/etc/torrust/tracker/tracker.toml",
    TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN = http_api.admin_token,
  },
  networks = ["backend_network"],
  ports = [
    "%{http_api.bind_address}:9091/tcp",
  ],
  volumes = [
    "./storage/tracker/lib:/var/lib/torrust/tracker:Z",
    "./storage/tracker/log:/var/log/torrust/tracker:Z",
    "./storage/tracker/etc:/etc/torrust/tracker:Z",
  ],
  logging = {
    options = {
      "max-size" = "10m",
      "max-file" = "10",
    },
  },
} in

# Build services object with tracker
let services = {
  tracker = tracker_service,
} in

# Conditionally add Prometheus service if enabled
let services_final =
  if prometheus_enabled then
    services & {
      prometheus = {
        image = "prom/prometheus:v3.0.1",
        container_name = "prometheus",
        tty = true,
        restart = "unless-stopped",
        networks = ["backend_network"],
        ports = ["9090:9090"],
        volumes = ["./storage/prometheus/etc:/etc/prometheus:Z"],
        logging = {
          options = {
            "max-size" = "10m",
            "max-file" = "10",
          },
        },
        depends_on = ["tracker"],
      },
    }
  else
    services
in

# Build the complete docker-compose structure
{
  version = "3.9",
  services = services_final,
  networks = {
    backend_network = {},
  },
}
