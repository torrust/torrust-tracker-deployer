# Features Schema
# Type contracts for optional features (Prometheus, Grafana)

{
  # Prometheus monitoring configuration (all optional - user decides what to set)
  PrometheusConfig = {
    enabled | Bool | optional,
    bind_address | String | optional,
    scrape_interval | Number | optional,
  },

  # Grafana visualization configuration (all optional - user decides what to set)
  GrafanaConfig = {
    enabled | Bool | optional,
    bind_address | String | optional,
    admin_password | String | optional,
  },

  # All optional features (user provides what they want)
  Features = {
    prometheus | PrometheusConfig | optional,
    grafana | GrafanaConfig | optional,
  },
}
