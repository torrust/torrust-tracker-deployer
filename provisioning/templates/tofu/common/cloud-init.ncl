# OpenTofu/Terraform Cloud-Init Configuration
#
# This Nickel template generates a cloud-init user data script for bootstrapping
# new cloud instances with SSH access, system updates, and runtime dependencies.
#
# Output format: YAML (cloud-init user_data format)

# Import the user configuration
let config = import "../../../values/config.ncl" in

# Extract configuration
let env = config.environment in
let ssh_creds = config.ssh_credentials in

# Read SSH public key content
let public_key_file = ssh_creds.public_key_path in

# Build the cloud-init YAML structure
{
  # Cloud-init version specification
  version = 2,

  # System updates on first boot
  system_update = {
    enabled = true,
  },

  # Operating system package updates
  apt = {
    preserve_sources_list = true,
    update = true,
    upgrade = "auto",
  },

  # Package installation
  packages = [
    "curl",
    "wget",
    "git",
    "build-essential",
    "libssl-dev",
    "pkg-config",
    "python3",
    "python3-pip",
  ],

  # User account creation
  users = [
    {
      name = ssh_creds.username,
      groups = ["sudo", "docker"],
      shell = "/bin/bash",
      sudo = ["ALL=(ALL) NOPASSWD:ALL"],
      lock_passwd = true,
      ssh_authorized_keys = [public_key_file],
      home = "/home/%{ssh_creds.username}",
      create_home = true,
    },
  ],

  # Final message shown when cloud-init completes
  final_message = "Cloud-init has finished: deployment $RESULT took $ELAPSED seconds",

  # Root commands to run (executed as root)
  runcmd = [
    "echo 'Cloud-init boot completed for environment: %{env.name}'",
    "hostnamectl set-hostname torrust-tracker",
  ],
}
