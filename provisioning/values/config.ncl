# Torrust Tracker Environment Configuration - Example
#
# This is a fully documented example showing all possible configuration options.
# Copy this file and customize it for your environment:
#
#   cp provisioning/values/config.ncl provisioning/values/my-env.ncl
#   vim provisioning/values/my-env.ncl
#   ./provisioning/scripts/validate-nickel.sh provisioning/values/my-env.ncl
#   ./provisioning/scripts/nickel-to-json.sh provisioning/values/my-env.ncl envs/my-env.json
#
# Then create the environment:
#
#   cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json

# ============================================================================
# IMPORTS: Schemas, Defaults, Validators
# ============================================================================

let schemas_env = import "../schemas/environment.ncl" in
let schemas_ssh = import "../schemas/ssh.ncl" in
let schemas_provider = import "../schemas/provider.ncl" in
let schemas_tracker = import "../schemas/tracker.ncl" in
let schemas_features = import "../schemas/features.ncl" in

let defaults_env = import "../defaults/environment.ncl" in
let defaults_ssh = import "../defaults/ssh.ncl" in
let defaults_provider = import "../defaults/provider.ncl" in
let defaults_tracker = import "../defaults/tracker.ncl" in
let defaults_features = import "../defaults/features.ncl" in

let constraints = import "../constraints.toml" in

let validators = import "../validators/environment.ncl" in
let validators_instance = import "../validators/instance.ncl" in
let validators_username = import "../validators/username.ncl" in
let validators_common = import "../validators/common.ncl" in
let validators_network = import "../validators/network.ncl" in
let validators_tracker = import "../validators/tracker.ncl" in

# ============================================================================
# USER CONFIGURATION
# ============================================================================

let user_config = {
  # ========================================================================
  # ENVIRONMENT IDENTIFICATION
  # ========================================================================
  environment = {
    name = validators.ValidEnvironmentName "dev",
    # instance_name = validators_instance.ValidInstanceName "dev-vm",
  },

  # ========================================================================
  # INFRASTRUCTURE PROVIDER - LXD
  # ========================================================================
  provider = {
    provider = "lxd",
    profile_name = "torrust-profile-dev",
  },

  # ========================================================================
  # SSH CREDENTIALS
  # ========================================================================
  ssh_credentials = {
    private_key_path = "fixtures/testing_rsa",
    public_key_path = "fixtures/testing_rsa.pub",
    username = validators_username.ValidUsername "torrust",
    port = validators_common.ValidPort 22,
  },

  # ========================================================================
  # TRACKER CONFIGURATION
  # ========================================================================
  tracker = {
    # Merge with defaults - can omit fields to use defaults
    core = defaults_tracker.tracker.core & {
      private = false,
      database = {
        driver = "sqlite3",
        database_name = "tracker.db",
      },
    },

    # ====================================================================
    # HTTP TRACKERS: Choose one option (uncomment what you need)
    # ====================================================================
    # Option A: Inherit defaults (0.0.0.0:7070)
    # http_trackers = validators_tracker.ValidTrackerArrayFull
    #   defaults_tracker.tracker.http_trackers
    #   constraints.tracker.http.min_items
    #   constraints.tracker.http.max_items,

    # Option B: Replace with custom ports (can have multiple entries)
    http_trackers = validators_tracker.ValidTrackerArrayFull
      [
        {
          bind_address = validators_network.ValidBindAddress "0.0.0.0:7070",
        },
        {
          bind_address = validators_network.ValidBindAddress "0.0.0.0:7071",
        },
      ]
      constraints.tracker.http.min_items
      constraints.tracker.http.max_items,

    # ====================================================================
    # UDP TRACKERS: Choose one option (uncomment what you need)
    # ====================================================================
    # Option A: Inherit defaults (0.0.0.0:6969)
    # udp_trackers = validators_tracker.ValidTrackerArrayFull
    #   defaults_tracker.tracker.udp_trackers
    #   constraints.tracker.udp.min_items
    #   constraints.tracker.udp.max_items,

    # Option B: Replace with custom port (can have multiple entries)
    udp_trackers = validators_tracker.ValidTrackerArrayFull
      [
        {
          bind_address = validators_network.ValidBindAddress "0.0.0.0:6969",
        },
        {
          bind_address = validators_network.ValidBindAddress "0.0.0.0:6970",
        },
      ]
      constraints.tracker.udp.min_items
      constraints.tracker.udp.max_items,

    # Define http_api (required)
    http_api = {
      bind_address = validators_network.ValidBindAddress "0.0.0.0:1212",
      admin_token = "MyAccessToken",
    },
  },

  # ========================================================================
  # OPTIONAL FEATURES
  # ========================================================================
  features = {
    prometheus = {
      enabled = false,
    },
    grafana = {
      enabled = false,
    },
  },
} in

# ============================================================================
# MERGE DEFAULTS + USER CONFIG
# ============================================================================
# Merge strategy:
# - tracker must always be defined in user_config (not optional)
# - core merges with defaults_tracker.tracker.core (can omit fields to inherit)
# - Arrays: reference defaults (udp_trackers) or replace completely (http_trackers)
# - http_api must be provided by user (no default available)
defaults_env & defaults_ssh & defaults_provider & defaults_features & user_config
