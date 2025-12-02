# Hetzner Cloud Provider Configuration
#
# This is the main OpenTofu configuration for deploying Torrust Tracker
# environments to Hetzner Cloud.
#
# Resources created:
# - SSH key: Imported from local keypair for secure access
# - Server: Hetzner Cloud server running Ubuntu with cloud-init configuration
#
# Dependencies:
# - variables.tfvars: Runtime variables (API token, server settings, SSH config)
# - cloud-init.yml: Server initialization script (rendered from template)

terraform {
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.47"
    }
  }
  required_version = ">= 1.0"
}

# Configure the Hetzner Cloud provider with the API token from variables
provider "hcloud" {
  token = var.hcloud_api_token
}

# ============================================================================
# Variables
# ============================================================================

variable "hcloud_api_token" {
  description = "Hetzner Cloud API token for authentication"
  type        = string
  sensitive   = true
}

variable "ssh_public_key" {
  description = "Public SSH key content for server access"
  type        = string
}

variable "ssh_key_name" {
  description = "Name for the SSH key resource in Hetzner Cloud"
  type        = string
}

variable "server_name" {
  description = "Name for the server instance"
  type        = string
}

variable "server_type" {
  description = "Hetzner Cloud server type (e.g., cx22, cx32)"
  type        = string
}

variable "server_image" {
  description = "Operating system image for the server"
  type        = string
  default     = "ubuntu-24.04"
}

variable "server_location" {
  description = "Hetzner Cloud datacenter location (e.g., nbg1, fsn1, hel1)"
  type        = string
}

variable "server_labels" {
  description = "Labels to apply to the server for organization"
  type        = map(string)
  default     = {}
}

# ============================================================================
# Resources
# ============================================================================

# Create or import the SSH key for server access
resource "hcloud_ssh_key" "torrust" {
  name       = var.ssh_key_name
  public_key = var.ssh_public_key
}

# Create the Hetzner Cloud server
resource "hcloud_server" "torrust" {
  name        = var.server_name
  image       = var.server_image
  server_type = var.server_type
  location    = var.server_location
  labels      = var.server_labels

  ssh_keys = [
    hcloud_ssh_key.torrust.id
  ]

  # Cloud-init configuration for initial server setup
  user_data = file("${path.module}/cloud-init.yml")

  # Ensure SSH key is created before the server
  depends_on = [hcloud_ssh_key.torrust]
}

# ============================================================================
# Outputs
# ============================================================================

# IMPORTANT: This output is parsed by src/adapters/tofu/json_parser.rs
# The output name "instance_info" and all fields (name, image, status, ip_address)
# are required by the parser and must remain present with these exact names.
output "instance_info" {
  description = "Information about the created server"
  value = {
    name       = hcloud_server.torrust.name
    image      = hcloud_server.torrust.image
    status     = hcloud_server.torrust.status
    ip_address = hcloud_server.torrust.ipv4_address
  }
  depends_on = [hcloud_server.torrust]
}

output "connection_commands" {
  description = "Commands to connect to the server"
  value = [
    "ssh ${var.server_name}@${hcloud_server.torrust.ipv4_address}",
    "hcloud server ssh ${var.server_name}"
  ]
}

output "test_commands" {
  description = "Commands to test the server functionality"
  value = [
    "hcloud server describe ${var.server_name}",
    "hcloud server list",
    "ssh ${var.server_name}@${hcloud_server.torrust.ipv4_address} 'cat /etc/os-release'",
    "ssh ${var.server_name}@${hcloud_server.torrust.ipv4_address} 'cloud-init status'"
  ]
}
