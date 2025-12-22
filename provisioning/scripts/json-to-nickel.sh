#!/bin/bash
# Convert TypeDialog JSON output to Nickel configuration file
#
# This script takes JSON output from TypeDialog and generates a Nickel
# configuration file that merges user values with schemas/defaults/validators.
#
# Usage:
#   ./json-to-nickel.sh <input.json> <output.ncl>
#
# Arguments:
#   input.json  - JSON file from TypeDialog (required)
#   output.ncl  - Nickel output file (required)

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <input.json> <output.ncl>" >&2
    exit 1
fi

readonly INPUT_JSON="$1"
readonly OUTPUT_NICKEL="$2"

if [[ ! -f "$INPUT_JSON" ]]; then
    echo "Error: Input JSON file not found: $INPUT_JSON" >&2
    exit 1
fi

# ============================================================================
# EXTRACT VALUES FROM JSON
# ============================================================================

extract_json() {
    local key="$1"
    local default="${2:-}"
    jq -r ".${key} // \"${default}\"" "$INPUT_JSON"
}

# Environment section
ENV_NAME=$(extract_json "environment_name")
INSTANCE_NAME=$(extract_json "instance_name" "")

# Provider section
PROVIDER=$(extract_json "provider")

# Provider-specific values
if [[ "$PROVIDER" == "lxd" ]]; then
    LXD_PROFILE=$(extract_json "lxd_profile_name")
elif [[ "$PROVIDER" == "hetzner" ]]; then
    HETZNER_TOKEN=$(extract_json "hetzner_api_token")
    HETZNER_SERVER=$(extract_json "hetzner_server_type")
    HETZNER_LOCATION=$(extract_json "hetzner_location")
    HETZNER_IMAGE=$(extract_json "hetzner_image")
fi

# SSH section
SSH_PRIVATE_KEY=$(extract_json "ssh_private_key_path")
SSH_PUBLIC_KEY=$(extract_json "ssh_public_key_path")
SSH_USERNAME=$(extract_json "ssh_username" "torrust")
SSH_PORT=$(extract_json "ssh_port" "22")

# Database section
DATABASE_DRIVER=$(extract_json "database_driver")

if [[ "$DATABASE_DRIVER" == "sqlite3" ]]; then
    SQLITE_DB=$(extract_json "sqlite_database_name")
elif [[ "$DATABASE_DRIVER" == "mysql" ]]; then
    MYSQL_HOST=$(extract_json "mysql_host")
    MYSQL_PORT=$(extract_json "mysql_port")
    MYSQL_DB=$(extract_json "mysql_database_name")
    MYSQL_USER=$(extract_json "mysql_username")
    MYSQL_PASS=$(extract_json "mysql_password")
fi

# Tracker section
TRACKER_PRIVATE=$(extract_json "tracker_private_mode" "false")
UDP_BIND=$(extract_json "udp_tracker_bind_address")
HTTP_BIND=$(extract_json "http_tracker_bind_address")
API_BIND=$(extract_json "http_api_bind_address")
API_TOKEN=$(extract_json "http_api_admin_token")

# Features section
ENABLE_PROMETHEUS=$(extract_json "enable_prometheus" "false")
ENABLE_GRAFANA=$(extract_json "enable_grafana" "false")

if [[ "$ENABLE_PROMETHEUS" == "true" ]]; then
    PROMETHEUS_BIND=$(extract_json "prometheus_bind_address")
    PROMETHEUS_INTERVAL=$(extract_json "prometheus_scrape_interval" "15")
fi

if [[ "$ENABLE_GRAFANA" == "true" ]]; then
    GRAFANA_BIND=$(extract_json "grafana_bind_address")
    GRAFANA_PASS=$(extract_json "grafana_admin_password")
fi

# ============================================================================
# GENERATE NICKEL FILE
# ============================================================================

cat > "$OUTPUT_NICKEL" <<'NICKEL_TEMPLATE'
# Environment configuration (generated from TypeDialog)
# Generated: $(date -Iseconds)

let schemas = import "../schemas/environment.ncl" in
let defaults = import "../defaults/environment.ncl" in
let validators = import "../validators/environment.ncl" in
let validators_instance = import "../validators/instance.ncl" in
let validators_username = import "../validators/username.ncl" in
let validators_common = import "../validators/common.ncl" in
let validators_network = import "../validators/network.ncl" in

let user_config = {
NICKEL_TEMPLATE

# Append environment section
cat >> "$OUTPUT_NICKEL" <<EOF
  environment = {
    name = validators.ValidEnvironmentName "$ENV_NAME",
EOF

if [[ -n "$INSTANCE_NAME" ]]; then
    cat >> "$OUTPUT_NICKEL" <<EOF
    instance_name = validators_instance.ValidInstanceName "$INSTANCE_NAME",
EOF
fi

cat >> "$OUTPUT_NICKEL" <<'EOF'
  },

EOF

# Append provider section
if [[ "$PROVIDER" == "lxd" ]]; then
    cat >> "$OUTPUT_NICKEL" <<EOF
  provider = {
    provider = "lxd",
    profile_name = validators_instance.ValidInstanceName "$LXD_PROFILE",
  },

EOF
elif [[ "$PROVIDER" == "hetzner" ]]; then
    cat >> "$OUTPUT_NICKEL" <<EOF
  provider = {
    provider = "hetzner",
    api_token = "$HETZNER_TOKEN",
    server_type = "$HETZNER_SERVER",
    location = "$HETZNER_LOCATION",
    image = "$HETZNER_IMAGE",
  },

EOF
fi

# Append SSH section
cat >> "$OUTPUT_NICKEL" <<EOF
  ssh_credentials = {
    private_key_path = "$SSH_PRIVATE_KEY",
    public_key_path = "$SSH_PUBLIC_KEY",
    username = validators_username.ValidUsername "$SSH_USERNAME",
    port = validators_common.ValidPort $SSH_PORT,
  },

EOF

# Append tracker core + database section
cat >> "$OUTPUT_NICKEL" <<EOF
  tracker = {
    core = {
      private = $TRACKER_PRIVATE,
      database =
EOF

if [[ "$DATABASE_DRIVER" == "sqlite3" ]]; then
    cat >> "$OUTPUT_NICKEL" <<EOF
        {
          driver = "sqlite3",
          database_name = "$SQLITE_DB",
        },
EOF
elif [[ "$DATABASE_DRIVER" == "mysql" ]]; then
    cat >> "$OUTPUT_NICKEL" <<EOF
        {
          driver = "mysql",
          host = "$MYSQL_HOST",
          port = validators_common.ValidPort $MYSQL_PORT,
          database_name = "$MYSQL_DB",
          username = "$MYSQL_USER",
          password = "$MYSQL_PASS",
        },
EOF
fi

# Append tracker services
cat >> "$OUTPUT_NICKEL" <<EOF
    },
    udp_trackers = [
      { bind_address = validators_network.ValidBindAddress "$UDP_BIND" },
    ],
    http_trackers = [
      { bind_address = validators_network.ValidBindAddress "$HTTP_BIND" },
    ],
    http_api = {
      bind_address = validators_network.ValidBindAddress "$API_BIND",
      admin_token = "$API_TOKEN",
    },
  },

EOF

# Append features section
cat >> "$OUTPUT_NICKEL" <<EOF
  features = {
    prometheus = {
      enabled = $ENABLE_PROMETHEUS,
EOF

if [[ "$ENABLE_PROMETHEUS" == "true" ]]; then
    cat >> "$OUTPUT_NICKEL" <<EOF
      bind_address = "$PROMETHEUS_BIND",
      scrape_interval = $PROMETHEUS_INTERVAL,
EOF
fi

cat >> "$OUTPUT_NICKEL" <<EOF
    },
    grafana = {
      enabled = $ENABLE_GRAFANA,
EOF

if [[ "$ENABLE_GRAFANA" == "true" ]]; then
    cat >> "$OUTPUT_NICKEL" <<EOF
      bind_address = "$GRAFANA_BIND",
      admin_password = "$GRAFANA_PASS",
EOF
fi

cat >> "$OUTPUT_NICKEL" <<'EOF'
    },
  },
} in

# Merge defaults with user config
defaults & user_config
EOF

echo "✅ Nickel file generated: $OUTPUT_NICKEL"
