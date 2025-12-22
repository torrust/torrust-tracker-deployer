# Network Validators
# Validators for network addresses, ports, and connectivity configuration

let helpers = import "common.ncl" in

{
  # Re-export common network validators
  ValidPort = helpers.ValidPort,
  ValidBindAddress = helpers.ValidBindAddress,

  # Additional network validators can be added here
}
