# Tracker Schema
# Type contracts for Torrust Tracker configuration

let database_schema = import "./database.ncl" in

{
  # Tracker core configuration
  TrackerCore = {
    private | Bool | optional,
    database | database_schema.Database | optional,
  },

  # UDP tracker listener configuration
  TrackerUdp = {
    bind_address | String | optional,
  },

  # HTTP tracker listener configuration
  TrackerHttp = {
    bind_address | String | optional,
  },

  # HTTP API configuration (required when http_api is provided)
  TrackerApi = {
    bind_address | String,
    admin_token | String,
  },

  # Complete tracker configuration
  Tracker = {
    core | TrackerCore,
    udp_trackers | Array TrackerUdp | optional,
    http_trackers | Array TrackerHttp | optional,
    http_api | TrackerApi,  # REQUIRED: user must provide (no default)
  },
}
