terraform {
  required_providers {
    lxd = {
      source  = "terraform-lxd/lxd"
      version = "~> 2.0"
    }
  }
  required_version = ">= 1.0"
}

# Configure the LXD Provider
provider "lxd" {
  # Use local LXD daemon via unix socket
}

# Variables
variable "container_name" {
  description = "Name of the LXD container"
  type        = string
  default     = "torrust-vm"
}

variable "image" {
  description = "LXD image to use"
  type        = string
  default     = "ubuntu:24.04"
}

# Create a profile for our container with cloud-init support
resource "lxd_profile" "torrust_profile" {
  name = "torrust-profile"

  config = {
    "user.user-data" = file("${path.module}/cloud-init.yml")
  }

  device {
    name = "root"
    type = "disk"
    properties = {
      path = "/"
      pool = "default"
      size = "10GB"
    }
  }

  device {
    name = "eth0"
    type = "nic"
    properties = {
      network = "lxdbr0"
      name    = "eth0"
    }
  }
}

# Create the LXD container (VM-like system container)
resource "lxd_instance" "torrust_vm" {
  name      = var.container_name
  image     = var.image
  type      = "container"
  profiles  = [lxd_profile.torrust_profile.name]

  config = {
    "boot.autostart"      = "true"
    "security.nesting"    = "true"
    "security.privileged" = "false"
  }
}

# Output information about the container
output "container_info" {
  description = "Information about the created container"
  value = {
    name       = lxd_instance.torrust_vm.name
    image      = lxd_instance.torrust_vm.image
    status     = lxd_instance.torrust_vm.status
    ip_address = lxd_instance.torrust_vm.ipv4_address
  }
  depends_on = [lxd_instance.torrust_vm]
}

output "connection_commands" {
  description = "Commands to connect to the container"
  value = [
    "lxc exec ${var.container_name} -- /bin/bash",
    "lxc exec ${var.container_name} -- whoami",
    "lxc exec ${var.container_name} -- systemctl status",
    "lxc list ${var.container_name}"
  ]
}

output "test_commands" {
  description = "Commands to test the container functionality"
  value = [
    "lxc exec ${var.container_name} -- cat /etc/os-release",
    "lxc exec ${var.container_name} -- df -h",
    "lxc exec ${var.container_name} -- free -h",
    "lxc exec ${var.container_name} -- systemctl list-units --type=service --state=running",
    "lxc exec ${var.container_name} -- cloud-init status"
  ]
}
