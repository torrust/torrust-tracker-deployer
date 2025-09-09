# Simple OpenTofu configuration for testing
# This is a minimal configuration that doesn't create real resources

terraform {
  required_version = ">= 1.0"
}

# Null resource for testing - doesn't create actual infrastructure
resource "null_resource" "test" {
  provisioner "local-exec" {
    command = "echo 'OpenTofu test resource created'"
  }

  lifecycle {
    create_before_destroy = true
  }
}

# Output for testing
output "test_output" {
  value = "OpenTofu configuration is valid"
  description = "Test output to validate configuration"
}
