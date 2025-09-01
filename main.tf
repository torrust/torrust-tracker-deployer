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
  image  = "22.04"  # Ubuntu 22.04 LTS
  cpus   = 2
  memory = "2G"
  disk   = "10G"

  # Cloud-init configuration from external file
  cloud_init = file("${path.module}/cloud-init.yml")
}

# Output the VM's IP address
output "vm_ip_address" {
  description = "The IP address of the Torrust VM"
  value       = multipass_instance.torrust_vm.ipv4
}

# Output the VM name
output "vm_name" {
  description = "The name of the Torrust VM"
  value       = multipass_instance.torrust_vm.name
}
