# Ansible Inventory Configuration Template
#
# This Nickel template generates the Ansible inventory.yml file.
# It defines hosts and connection parameters for both LXD VMs and Docker containers.
#
# Output format: YAML

# Import the user configuration
let config = import "../../values/config.ncl" in

# Extract SSH configuration
let ssh = config.ssh_credentials in

# Build the inventory structure
{
  all = {
    hosts = {
      "torrust-tracker-vm" = {
        ansible_host = "10.0.0.100",  # Placeholder - will be replaced with actual IP
        ansible_port = ssh.port,
        ansible_user = ssh.username,
        ansible_connection = "ssh",
        ansible_ssh_private_key_file = ssh.private_key_path,
        ansible_ssh_common_args = "-o StrictHostKeyChecking=no",
      },
    },
    vars = {
      ansible_python_interpreter = "/usr/bin/python3",
    },
  },
}
