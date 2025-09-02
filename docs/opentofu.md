# OpenTofu Setup and Usage Guide

OpenTofu is an open-source infrastructure as code (IaC) tool that enables you to define and provision data center infrastructure using a declarative configuration language.

## 📋 Prerequisites

### Installation Verification

Check if OpenTofu is installed:

```bash
tofu version
```

Expected output format:

```text
OpenTofu v1.x.x
```

### Installation

If OpenTofu is not installed, you can install it using the official installation script:

```bash
# Download and install OpenTofu
curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
chmod +x install-opentofu.sh
./install-opentofu.sh --install-method deb

# Verify installation
tofu version
```

Alternative installation methods:

#### Ubuntu/Debian (APT)

```bash
# Add the OpenTofu repository
curl -fsSL https://packages.opentofu.org/opentofu/tofu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/opentofu.gpg
echo "deb [signed-by=/usr/share/keyrings/opentofu.gpg] https://packages.opentofu.org/opentofu/tofu/deb/ any main" | sudo tee /etc/apt/sources.list.d/opentofu.list

# Install OpenTofu
sudo apt update
sudo apt install tofu
```

#### macOS (Homebrew)

```bash
brew install opentofu
```

#### Manual Installation

Download the appropriate binary from the [OpenTofu releases page](https://github.com/opentofu/opentofu/releases) and add it to your PATH.

## 🚀 Common Commands

### Project Initialization

```bash
# Initialize a new OpenTofu project
tofu init
```

### Planning and Applying Changes

```bash
# Review planned changes (dry run)
tofu plan

# Apply changes to infrastructure
tofu apply

# Apply without interactive confirmation
tofu apply -auto-approve
```

### State Management

```bash
# Show current state
tofu show

# List resources in state
tofu state list

# Get specific resource information
tofu state show <resource_name>

# View outputs
tofu output
```

### Cleanup

```bash
# Destroy all managed infrastructure
tofu destroy

# Destroy without interactive confirmation
tofu destroy -auto-approve
```

### Validation and Formatting

```bash
# Validate configuration syntax
tofu validate

# Format configuration files
tofu fmt

# Format and show differences
tofu fmt -diff
```

## 🔧 Configuration Structure

### Basic File Structure

```text
├── main.tf              # Main configuration file
├── variables.tf         # Input variables
├── outputs.tf           # Output definitions
├── versions.tf          # Provider version constraints
└── terraform.tfvars     # Variable values (optional)
```

### Provider Configuration

```hcl
terraform {
  required_providers {
    lxd = {
      source  = "terraform-lxd/lxd"
      version = "~> 2.0"
    }
  }
  required_version = ">= 1.0"
}
```

### Resource Definition Example

```hcl
resource "lxd_container" "example" {
  name  = "my-container"
  image = "ubuntu:24.04"

  config = {
    "user.cloud-init.user-data" = file("${path.module}/cloud-init.yml")
  }
}
```

## 🎯 Best Practices

### File Organization

- Keep related resources in the same file
- Use descriptive resource names
- Separate environments with different state files
- Use modules for reusable components

### State Management

- Always use remote state for team collaboration
- Use state locking to prevent concurrent modifications
- Regularly backup state files
- Never edit state files manually

### Security

- Use variables for sensitive data
- Store sensitive variables in secure locations
- Use environment variables for secrets
- Apply principle of least privilege

### Version Control

- Always version control your configuration files
- Use `.gitignore` to exclude state files and sensitive data
- Tag releases for stable configurations
- Use meaningful commit messages

## 🐛 Troubleshooting

### Common Issues

#### Provider Not Found

```bash
# Re-initialize to download providers
tofu init -upgrade
```

#### State Lock Issues

```bash
# Force unlock (use with caution)
tofu force-unlock <lock-id>
```

#### Configuration Errors

```bash
# Validate configuration
tofu validate

# Check syntax with detailed output
tofu plan -detailed-exitcode
```

### Debugging

```bash
# Enable debug logging
export TF_LOG=DEBUG
tofu apply

# Log to file
export TF_LOG_PATH=./tofu.log
```

## 📚 Additional Resources

- [OpenTofu Documentation](https://opentofu.org/docs/)
- [OpenTofu GitHub Repository](https://github.com/opentofu/opentofu)
- [Terraform Registry](https://registry.terraform.io/) (compatible providers)
- [HCL Configuration Language](https://developer.hashicorp.com/terraform/language)
