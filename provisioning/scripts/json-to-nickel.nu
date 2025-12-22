#!/usr/bin/env nu
# Convert TypeDialog JSON output to Nickel configuration file (Nushell variant)
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
# - Function signatures with explicit types
# - No try-catch: Uses `do { } | complete`
# - External commands prefixed with `^`
# - String interpolation: [$var] for variables, ($expr) for expressions
#
# Usage:
#   nu ./json-to-nickel.nu <input.json> <output.ncl>

# Extract value from JSON with optional default
def extract-json [json: record, key: string, default: string = ""]: nothing -> string {
    let maybe_value = ($json | get --optional $key)
    if ($maybe_value == null) { $default } else { $maybe_value }
}

def main [input_json: string, output_nickel: string]: nothing -> nothing {
    if not ($input_json | path exists) {
        print -e $"Error: Input JSON file not found: ($input_json)"
        exit 1
    }

    # Load JSON
    let config = (open $input_json)

    # Extract environment section
    let env_name = (extract-json $config "environment_name")
    let instance_name = (extract-json $config "instance_name")

    # Extract provider section
    let provider = (extract-json $config "provider")

    # Extract provider-specific values
    let lxd_profile = if ($provider == "lxd") {
        extract-json $config "lxd_profile_name"
    } else {
        ""
    }

    let hetzner_token = if ($provider == "hetzner") {
        extract-json $config "hetzner_api_token"
    } else {
        ""
    }

    let hetzner_server = if ($provider == "hetzner") {
        extract-json $config "hetzner_server_type"
    } else {
        ""
    }

    let hetzner_location = if ($provider == "hetzner") {
        extract-json $config "hetzner_location"
    } else {
        ""
    }

    let hetzner_image = if ($provider == "hetzner") {
        extract-json $config "hetzner_image"
    } else {
        ""
    }

    # Extract SSH section
    let ssh_private_key = (extract-json $config "ssh_private_key_path")
    let ssh_public_key = (extract-json $config "ssh_public_key_path")
    let ssh_username = (extract-json $config "ssh_username" "torrust")
    let ssh_port = (extract-json $config "ssh_port" "22")

    # Extract database section
    let database_driver = (extract-json $config "database_driver")

    let sqlite_db = if ($database_driver == "sqlite3") {
        extract-json $config "sqlite_database_name"
    } else {
        ""
    }

    let mysql_host = if ($database_driver == "mysql") {
        extract-json $config "mysql_host"
    } else {
        ""
    }

    let mysql_port = if ($database_driver == "mysql") {
        extract-json $config "mysql_port"
    } else {
        ""
    }

    let mysql_db = if ($database_driver == "mysql") {
        extract-json $config "mysql_database_name"
    } else {
        ""
    }

    let mysql_user = if ($database_driver == "mysql") {
        extract-json $config "mysql_username"
    } else {
        ""
    }

    let mysql_pass = if ($database_driver == "mysql") {
        extract-json $config "mysql_password"
    } else {
        ""
    }

    # Extract tracker section
    let tracker_private = (extract-json $config "tracker_private_mode" "false")
    let udp_bind = (extract-json $config "udp_tracker_bind_address")
    let http_bind = (extract-json $config "http_tracker_bind_address")
    let api_bind = (extract-json $config "http_api_bind_address")
    let api_token = (extract-json $config "http_api_admin_token")

    # Extract features section
    let enable_prometheus = (extract-json $config "enable_prometheus" "false")
    let enable_grafana = (extract-json $config "enable_grafana" "false")

    let prometheus_bind = if ($enable_prometheus == "true") {
        extract-json $config "prometheus_bind_address"
    } else {
        ""
    }

    let prometheus_interval = if ($enable_prometheus == "true") {
        extract-json $config "prometheus_scrape_interval" "15"
    } else {
        ""
    }

    let grafana_bind = if ($enable_grafana == "true") {
        extract-json $config "grafana_bind_address"
    } else {
        ""
    }

    let grafana_pass = if ($enable_grafana == "true") {
        extract-json $config "grafana_admin_password"
    } else {
        ""
    }

    # Build Nickel configuration
    let timestamp = (date now | format date '%Y-%m-%dT%H:%M:%SZ')

    # Build instance_name section
    let instance_section = if ($instance_name != "") {
        $"    instance_name = validators_instance.ValidInstanceName \"($instance_name)\",\n"
    } else {
        ""
    }

    # Build provider section (does NOT include environment closing brace)
    let provider_section = if ($provider == "lxd") {
        $"  provider = {\n    provider = \"lxd\",\n    profile_name = validators_instance.ValidInstanceName \"($lxd_profile)\",\n  },"
    } else if ($provider == "hetzner") {
        $"  provider = {\n    provider = \"hetzner\",\n    api_token = \"($hetzner_token)\",\n    server_type = \"($hetzner_server)\",\n    location = \"($hetzner_location)\",\n    image = \"($hetzner_image)\",\n  },"
    } else {
        ""
    }

    # Build database section
    let database_section = if ($database_driver == "sqlite3") {
        $"        {\n          driver = \"sqlite3\",\n          database_name = \"($sqlite_db)\",\n        },"
    } else if ($database_driver == "mysql") {
        $"        {\n          driver = \"mysql\",\n          host = \"($mysql_host)\",\n          port = validators_common.ValidPort ($mysql_port),\n          database_name = \"($mysql_db)\",\n          username = \"($mysql_user)\",\n          password = \"($mysql_pass)\",\n        },"
    } else {
        ""
    }

    # Build prometheus section
    let prometheus_section = if ($enable_prometheus == "true") {
        $"      bind_address = \"($prometheus_bind)\",\n      scrape_interval = ($prometheus_interval),"
    } else {
        ""
    }

    # Build grafana section
    let grafana_section = if ($enable_grafana == "true") {
        $"      bind_address = \"($grafana_bind)\",\n      admin_password = \"($grafana_pass)\","
    } else {
        ""
    }

    # Construct the complete Nickel file - using pure string interpolation
    let nickel_content = $"# Environment configuration \(generated from TypeDialog\)
# Generated: ($timestamp)

let schemas = import \"../schemas/environment.ncl\" in
let defaults = import \"../defaults/environment.ncl\" in
let validators = import \"../validators/environment.ncl\" in
let validators_instance = import \"../validators/instance.ncl\" in
let validators_username = import \"../validators/username.ncl\" in
let validators_common = import \"../validators/common.ncl\" in
let validators_network = import \"../validators/network.ncl\" in

let user_config = {
  environment = {
    name = validators.ValidEnvironmentName \"($env_name)\",
($instance_section)  },

($provider_section)
  ssh_credentials = {
    private_key_path = \"($ssh_private_key)\",
    public_key_path = \"($ssh_public_key)\",
    username = validators_username.ValidUsername \"($ssh_username)\",
    port = validators_common.ValidPort ($ssh_port),
  },

  tracker = {
    core = {
      private = ($tracker_private),
      database =
($database_section)
    },
    udp_trackers = [
      { bind_address = validators_network.ValidBindAddress \"($udp_bind)\" },
    ],
    http_trackers = [
      { bind_address = validators_network.ValidBindAddress \"($http_bind)\" },
    ],
    http_api = {
      bind_address = validators_network.ValidBindAddress \"($api_bind)\",
      admin_token = \"($api_token)\",
    },
  },

  features = {
    prometheus = {
      enabled = ($enable_prometheus),
($prometheus_section)
    },
    grafana = {
      enabled = ($enable_grafana),
($grafana_section)
    },
  },
} in

# Merge defaults with user config
defaults & user_config
"

    # Write to file
    $nickel_content | save --force $output_nickel

    print $"✅ Nickel file generated: ($output_nickel)"
}

# Script is a library - call main directly:
# nu -c 'source ./json-to-nickel.nu; main "input.json" "output.ncl"'
