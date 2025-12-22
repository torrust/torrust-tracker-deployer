# Features Defaults
# Default values with type contracts applied
# Note: Features are completely optional - user decides what to enable/configure

let schemas = import "../schemas/features.ncl" in

{
  features | schemas.Features = {
    # Prometheus: optional structure (user provides if needed)
    # Default: not specified (optional in schema)
    # prometheus = {...}

    # Grafana: optional structure (user provides if needed)
    # Default: not specified (optional in schema)
    # grafana = {...}
  },
}
