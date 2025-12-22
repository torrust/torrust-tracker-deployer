# Prometheus Configuration Template
#
# Generates prometheus.yml configuration for monitoring Torrust Tracker metrics.
# This template imports the user configuration and extracts relevant values
# for Prometheus scrape configuration.

# Import user configuration
let config = import "../../values/config.ncl" in

# Extract port from bind_address
# Currently uses default port 1212 (HTTP API default)
# TODO: implement dynamic port extraction from "0.0.0.0:1212" format
let api_port = "1212" in

# Define Prometheus configuration structure
{
  # Global Prometheus settings
  global = {
    # How often to scrape metrics from targets (in seconds)
    scrape_interval = "30s",
  },

  # Scrape job configurations
  scrape_configs = [
    {
      # Job for tracker statistics
      job_name = "tracker_stats",
      metrics_path = "/api/v1/stats",
      params = {
        # API token for authentication
        token = [config.tracker.http_api.admin_token],
        # Request Prometheus format output
        format = ["prometheus"],
      },
      static_configs = [
        {
          # Target: tracker service on configured HTTP API port
          targets = [
            "tracker:%{api_port}"
          ],
        },
      ],
    },

    {
      # Job for detailed tracker metrics
      job_name = "tracker_metrics",
      metrics_path = "/api/v1/metrics",
      params = {
        token = [config.tracker.http_api.admin_token],
        format = ["prometheus"],
      },
      static_configs = [
        {
          targets = [
            "tracker:%{api_port}"
          ],
        },
      ],
    },
  ],
}
