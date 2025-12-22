# Ansible Variables Template
#
# This Nickel template generates the variables.yml file for Ansible playbooks.
# It defines all dynamic variables used across playbooks.
#
# Output format: YAML

# Import the user configuration
let config = import "../../values/config.ncl" in

# Extract configuration
let ssh = config.ssh_credentials in

# Build the variables object
{
  ssh_port = ssh.port,
  tracker_api_port = "1212",
  # Port lists for firewall configuration
  tracker_udp_ports = ["6969", "6970"],
  tracker_http_ports = ["7070", "7071"],
}
