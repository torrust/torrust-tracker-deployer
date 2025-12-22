# OpenTofu/Terraform Variables for LXD
#
# This Nickel template generates the terraform.tfvars file for OpenTofu LXD provisioning.
# It defines all variables needed to provision LXD virtual machines.
#
# Output format: HCL (Terraform/OpenTofu .tfvars format)

# Import the user configuration
let config = import "../../../values/config.ncl" in

# Extract configuration
let environment = config.environment in
let provider = config.provider in
let ssh_creds = config.ssh_credentials in

# Build the variables object
{
  # Environment and naming
  environment_name = environment.name,
  instance_name = "torrust-tracker-vm",

  # LXD provider configuration
  provider_type = provider.provider,
  lxd_profile_name = provider.profile_name,

  # SSH configuration
  ssh_public_key = ssh_creds.public_key_path,
  ssh_username = ssh_creds.username,
  ssh_port = ssh_creds.port,

  # Instance configuration
  instance_cpu = 2,
  instance_memory = "2GB",
  instance_disk = "20GB",

  # Network configuration
  network_bridge = "lxdbr0",

  # Image configuration
  image_remote = "ubuntu",
  image_name = "22.04",
}
