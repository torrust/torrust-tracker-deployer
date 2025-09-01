terraform {
  required_providers {
    multipass = {
      source  = "larstobi/multipass"
      version = "~> 1.4.2"
    }
  }
  required_version = ">= 1.0"
}

# Configure the Multipass provider
provider "multipass" {}

# Create a virtual machine instance
resource "multipass_instance" "torrust_vm" {
  name   = "torrust-vm"
  image  = "24.04"  # Ubuntu 24.04 LTS (Noble Numbat)
  cpus   = 2
  memory = "2G"
  disk   = "10G"

  # Cloud-init configuration from external file
  cloudinit_file = "${path.module}/cloud-init.yml"
}

# Data source to get the VM information including IP
data "multipass_instance" "torrust_vm" {
  name       = multipass_instance.torrust_vm.name
  depends_on = [multipass_instance.torrust_vm]
}

# Output the VM name
output "vm_name" {
  description = "The name of the Torrust VM"
  value       = multipass_instance.torrust_vm.name
}

# Output the VM's IP address
output "vm_ip_address" {
  description = "The IP address of the Torrust VM"
  value       = data.multipass_instance.torrust_vm.ipv4
}
