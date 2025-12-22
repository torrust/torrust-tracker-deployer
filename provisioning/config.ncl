# Torrust Tracker Configuration
# Generated from config-form.toml

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

let validators = import "../validators/environment.ncl" in
let validators_instance = import "../validators/instance.ncl" in
let validators_username = import "../validators/username.ncl" in
let validators_common = import "../validators/common.ncl" in
let validators_network = import "../validators/network.ncl" in

# ============================================================================
# USER CONFIGURATION
# ============================================================================
let user_config = {
  # ========================================================================
  # ENVIRONMENT IDENTIFICATION
  # ========================================================================
  environment = {
    name = validators.ValidEnvironmentName "dev",
  },
  # ========================================================================
  # INFRASTRUCTURE PROVIDER
  # ========================================================================
  provider = {
    provider = "lxd",
    profile_name = "d",
  },
  # ========================================================================
  # SSH CREDENTIALS
  # ========================================================================
  ssh_credentials = {
    private_key_path = "./config.ncl",
    public_key_path = "./config.ncl",
    username = validators_username.ValidUsername "torrust",
    port = validators_common.ValidPort 22,
  },
  # ========================================================================
  # DATABASE CONFIGURATION
  # ========================================================================
  tracker = {
    core = {
      private = false,
      database = {
        driver = "sqlite3",
        database_name = "tracker.db",
      },
    },
    udp_trackers = [
      {
        bind_address = validators_network.ValidBindAddress "0.0.0.0:6969",
      },
    ],
    http_trackers = [
      {
        bind_address = validators_network.ValidBindAddress "0.0.0.0:7077",
      },
    ],
    http_api = {
      bind_address = validators_network.ValidBindAddress "0.0.0.0:1212",

      admin_token = "12",

    },
  },
  # ========================================================================
  # MONITORING FEATURES
  # ========================================================================
} in

# ============================================================================
# MERGE DEFAULTS + USER CONFIG
# ============================================================================
defaults_env & defaults_ssh & defaults_provider & defaults_tracker & defaults_features & user_config
#user_config
