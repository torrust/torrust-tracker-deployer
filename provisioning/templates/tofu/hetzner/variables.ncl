# OpenTofu/Terraform Variables for Hetzner Cloud
#
# This Nickel template generates the terraform.tfvars file for OpenTofu Hetzner provisioning.
# It defines all variables needed to provision servers on Hetzner Cloud.
#
# Output format: HCL (Terraform/OpenTofu .tfvars format)

# Import the user configuration
let config = import "../../../values/config.ncl" in

# Extract configuration
let env = config.environment in
let provider = config.provider in
let ssh_creds = config.ssh_credentials in

# Build the variables object
{
  # Environment and naming
  environment_name = env.name,
  instance_name = "torrust-tracker",

  # Hetzner provider configuration
  provider_type = provider.provider,
  hetzner_api_token = "YOUR_HETZNER_API_TOKEN",

  # SSH configuration
  ssh_public_key = ssh_creds.public_key_path,
  ssh_username = ssh_creds.username,
  ssh_port = ssh_creds.port,

  # Server configuration
  server_type = "cx11",
  image = "ubuntu-22.04",
  location = "fsn1",

  # Network configuration
  enable_public_net = true,
  enable_protection = false,

  # DNS configuration
  hostname = "torrust-tracker",
}
