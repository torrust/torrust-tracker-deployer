# `provision` - Provision VM Infrastructure

Provision virtual machine infrastructure for a deployment environment.

## Purpose

Creates and configures VM infrastructure using OpenTofu (Terraform). This command takes an environment from the "Created" state to the "Provisioned" state with running VM instances.

The provision command works with all supported providers:

- **LXD** - Creates local VMs for development and testing
- **Hetzner Cloud** - Creates cloud servers for production deployments

## Command Syntax

```bash
torrust-tracker-deployer provision <ENVIRONMENT>
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to provision

## Output Formats

The `provision` command supports two output formats for command results:

- **Text** (default) - Human-readable formatted output
- **JSON** - Machine-readable JSON for automation

Use the global `--output-format` flag to control the format.

### Text Output (Default)

The default output format provides human-readable information with visual formatting:

```bash
torrust-tracker-deployer provision my-environment
```

**Output**:

```text
✓ Rendering OpenTofu templates...
✓ Initializing infrastructure...
✓ Planning infrastructure changes...
✓ Applying infrastructure...
✓ Retrieving instance information...
✓ Instance IP: 10.140.190.42
✓ Rendering Ansible templates...
✓ Waiting for SSH connectivity...
✓ Waiting for cloud-init completion...
✓ Environment provisioned successfully

Provisioning Details:
 1. Environment name: my-environment
 2. Instance name: torrust-tracker-vm-my-environment
 3. Instance IP: 10.140.190.42
 4. SSH credentials:
    - Private key: /home/user/.ssh/id_rsa
    - Public key: /home/user/.ssh/id_rsa.pub
    - Username: torrust
    - Port: 22
 5. Provider: lxd
 6. Domains: (none)
```

**Features**:

- Progress indicators (✓)
- Numbered list format
- Clear section organization
- Color-coded status messages

### JSON Output

Use `--output-format json` for machine-readable output ideal for automation, scripts, and programmatic processing:

```bash
torrust-tracker-deployer provision my-environment --output-format json
```

**Output**:

```json
{
  "environment_name": "my-environment",
  "instance_name": "torrust-tracker-vm-my-environment",
  "instance_ip": "10.140.190.42",
  "ssh_private_key_path": "/home/user/.ssh/id_rsa",
  "ssh_public_key_path": "/home/user/.ssh/id_rsa.pub",
  "ssh_username": "torrust",
  "ssh_port": 22,
  "provider": "lxd",
  "domains": [],
  "provisioned_at": "2026-02-16T13:38:02.446056727Z"
}
```

**Features**:

- Valid, parseable JSON
- Pretty-printed for readability
- ISO 8601 timestamps
- Consistent field ordering
- Ready for immediate SSH automation

#### JSON Schema

| Field                  | Type     | Description                        | Example                            |
| ---------------------- | -------- | ---------------------------------- | ---------------------------------- |
| `environment_name`     | string   | Name of the environment            | `"production"`                     |
| `instance_name`        | string   | Full VM instance name              | `"torrust-tracker-vm-production"`  |
| `instance_ip`          | string   | IP address of provisioned VM       | `"10.140.190.42"`                  |
| `ssh_private_key_path` | string   | Path to SSH private key            | `"/home/user/.ssh/id_rsa"`         |
| `ssh_public_key_path`  | string   | Path to SSH public key             | `"/home/user/.ssh/id_rsa.pub"`     |
| `ssh_username`         | string   | SSH username for VM access         | `"torrust"`                        |
| `ssh_port`             | number   | SSH port number                    | `22`                               |
| `provider`             | string   | Provider used ("lxd" or "hetzner") | `"lxd"`                            |
| `domains`              | string[] | Configured domains (HTTPS only)    | `["tracker.example.com"]`          |
| `provisioned_at`       | string   | ISO 8601 timestamp of provisioning | `"2026-02-16T13:38:02.446056727Z"` |

#### Short Form

Use the `-o` alias for shorter commands:

```bash
torrust-tracker-deployer provision my-environment -o json
```

### Automation Examples

#### Extract IP Address for SSH Automation

```bash
#!/bin/bash

# Provision environment and capture JSON output
JSON_OUTPUT=$(torrust-tracker-deployer provision my-environment \
  --output-format json \
  --log-output file-only)

# Extract instance IP and SSH credentials using jq
INSTANCE_IP=$(echo "$JSON_OUTPUT" | jq -r '.instance_ip')
SSH_KEY=$(echo "$JSON_OUTPUT" | jq -r '.ssh_private_key_path')
SSH_USER=$(echo "$JSON_OUTPUT" | jq -r '.ssh_username')

echo "VM provisioned at: $INSTANCE_IP"

# Connect to the instance
ssh -i "$SSH_KEY" "$SSH_USER@$INSTANCE_IP"
```

#### CI/CD Pipeline Integration (GitHub Actions)

```yaml
name: Deploy Tracker

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Provision Infrastructure
        id: provision
        run: |
          OUTPUT=$(torrust-tracker-deployer provision ${{ env.ENVIRONMENT_NAME }} \
            --output-format json \
            --log-output file-only)

          echo "ip=$(echo $OUTPUT | jq -r '.instance_ip')" >> $GITHUB_OUTPUT
          echo "ssh_key=$(echo $OUTPUT | jq -r '.ssh_private_key_path')" >> $GITHUB_OUTPUT
          echo "ssh_user=$(echo $OUTPUT | jq -r '.ssh_username')" >> $GITHUB_OUTPUT

      - name: Configure DNS
        if: ${{ steps.provision.outputs.ip != '' }}
        run: |
          # Update DNS A record with provisioned IP
          curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
            -H "Authorization: Bearer ${{ secrets.CLOUDFLARE_API_TOKEN }}" \
            -H "Content-Type: application/json" \
            --data '{"type":"A","name":"tracker","content":"${{ steps.provision.outputs.ip }}"}'

      - name: Deploy Application
        run: |
          ssh -i "${{ steps.provision.outputs.ssh_key }}" \
              "${{ steps.provision.outputs.ssh_user }}@${{ steps.provision.outputs.ip }}" \
              'cd /opt/torrust && docker compose up -d'
```

#### Python Script for Multi-Region Deployment

```python
import json
import subprocess
from typing import Dict, List

def provision_environment(env_name: str) -> Dict[str, any]:
    """Provision an environment and return provisioning details."""
    result = subprocess.run(
        [
            "torrust-tracker-deployer",
            "provision",
            env_name,
            "--output-format", "json",
            "--log-output", "file-only"
        ],
        capture_output=True,
        text=True,
        check=True
    )
    return json.loads(result.stdout)

def deploy_multi_region(regions: List[str]):
    """Deploy tracker to multiple regions in parallel."""
    deployments = []

    for region in regions:
        env_name = f"tracker-{region}"
        print(f"Provisioning {env_name}...")

        details = provision_environment(env_name)
        deployments.append({
            "region": region,
            "ip": details["instance_ip"],
            "ssh_user": details["ssh_username"],
            "ssh_key": details["ssh_private_key_path"],
            "provisioned_at": details["provisioned_at"]
        })

    # Save deployment manifest
    with open("deployments.json", "w") as f:
        json.dump(deployments, f, indent=2)

    print(f"\nDeployed to {len(deployments)} regions")
    for deployment in deployments:
        print(f"  {deployment['region']}: {deployment['ip']}")

# Deploy to multiple regions
deploy_multi_region(["us-east", "eu-west", "ap-southeast"])
```

#### Terraform/OpenTofu Integration

```hcl
# external-data.tf
# Use provisioned VM in downstream Terraform configuration

data "external" "provision_vm" {
  program = ["bash", "-c", <<-EOF
    torrust-tracker-deployer provision my-environment \
      --output-format json \
      --log-output file-only
  EOF
  ]
}

resource "cloudflare_record" "tracker_a_record" {
  zone_id = var.cloudflare_zone_id
  name    = "tracker"
  value   = data.external.provision_vm.result.instance_ip
  type    = "A"
  ttl     = 300
}

output "vm_ip" {
  value = data.external.provision_vm.result.instance_ip
}

output "ssh_command" {
  value = "ssh -i ${data.external.provision_vm.result.ssh_private_key_path} ${data.external.provision_vm.result.ssh_username}@${data.external.provision_vm.result.instance_ip}"
}
```

#### Extract Domains for HTTPS Configuration

```bash
#!/bin/bash

# Provision HTTPS environment
JSON_OUTPUT=$(torrust-tracker-deployer provision https-tracker \
  --output-format json \
  --log-output file-only)

# Extract domains array
DOMAINS=$(echo "$JSON_OUTPUT" | jq -r '.domains[]')

if [ -z "$DOMAINS" ]; then
  echo "No domains configured - HTTP-only deployment"
else
  echo "Configured domains:"
  echo "$DOMAINS"

  # Generate SSL certificates for each domain
  for domain in $DOMAINS; do
    echo "Requesting certificate for $domain..."
    certbot certonly --standalone -d "$domain" --non-interactive --agree-tos
  done
fi
```

## Prerequisites

1. **Environment created** - Must run `create environment` first
2. **Provider-specific requirements**:
   - **LXD**: Local LXD installation configured
   - **Hetzner**: Valid API token in environment configuration
3. **OpenTofu installed** - OpenTofu CLI available in PATH
4. **SSH keys** - SSH key pair referenced in environment configuration

📖 **See [Provider Guides](../providers/README.md)** for provider-specific setup.

## State Transition

```text
[Created] --provision--> [Provisioned]
```

## What Happens

When you provision an environment:

1. **Renders OpenTofu templates** - Generates provider-specific infrastructure-as-code files
2. **Initializes OpenTofu** - Sets up backend and providers (`tofu init`)
3. **Creates execution plan** - Validates configuration (`tofu plan`)
4. **Applies infrastructure** - Creates VM resources (`tofu apply`)
5. **Retrieves instance info** - Gets IP address and instance details
6. **Renders Ansible templates** - Generates configuration management files
7. **Waits for SSH** - Verifies network connectivity
8. **Waits for cloud-init** - Ensures VM initialization is complete
9. **Updates environment state** - Transitions to "Provisioned"

## Examples

### Basic provisioning (Text Output)

```bash
# Provision the environment with human-readable output
torrust-tracker-deployer provision my-environment

# Output:
# ✓ Rendering OpenTofu templates...
# ✓ Initializing infrastructure...
# ✓ Planning infrastructure changes...
# ✓ Applying infrastructure...
# ✓ Retrieving instance information...
# ✓ Instance IP: 10.140.190.42
# ✓ Rendering Ansible templates...
# ✓ Waiting for SSH connectivity...
# ✓ Waiting for cloud-init completion...
# ✓ Environment provisioned successfully
#
# Provisioning Details:
#  1. Environment name: my-environment
#  2. Instance name: torrust-tracker-vm-my-environment
#  3. Instance IP: 10.140.190.42
#  ...
```

### Basic provisioning (JSON Output)

```bash
# Provision the environment with JSON output for automation
torrust-tracker-deployer provision my-environment --output-format json

# Output (structured JSON):
# {
#   "environment_name": "my-environment",
#   "instance_name": "torrust-tracker-vm-my-environment",
#   "instance_ip": "10.140.190.42",
#   "ssh_private_key_path": "/home/user/.ssh/id_rsa",
#   "ssh_public_key_path": "/home/user/.ssh/id_rsa.pub",
#   "ssh_username": "torrust",
#   "ssh_port": 22,
#   "provider": "lxd",
#   "domains": [],
#   "provisioned_at": "2026-02-16T13:38:02.446056727Z"
# }
```

### Provision and extract IP for automation

```bash
# Provision and immediately extract IP address
IP=$(torrust-tracker-deployer provision my-environment \
  --output-format json \
  --log-output file-only | jq -r '.instance_ip')

echo "Provisioned VM at: $IP"

# Use IP in subsequent automation
ansible-playbook -i "$IP," deploy.yml
```

### Provision multiple environments

```bash
# Development (local)
torrust-tracker-deployer provision dev-local

# Staging (cloud)
torrust-tracker-deployer provision staging --output-format json

# Production (Hetzner with JSON for automation)
torrust-tracker-deployer provision production -o json
```

## Output

The provision command creates provider-specific resources:

### LXD Provider

- **VM instance** - LXD virtual machine (`torrust-tracker-vm-<env-name>`)
- **LXD profile** - Custom profile with cloud-init configuration
- **Network configuration** - Bridged network with IP assignment
- **OpenTofu state** - Infrastructure state in `build/<env>/tofu/lxd/`

### Hetzner Provider

- **Cloud server** - Hetzner Cloud server instance
- **Firewall** - Hetzner firewall with SSH access
- **SSH key** - Uploaded SSH public key
- **OpenTofu state** - Infrastructure state in `build/<env>/tofu/hetzner/`

### Common Outputs (All Providers)

- **Ansible inventory** - Generated inventory in `build/<env>/ansible/`
- **Environment state update** - State file updated to "Provisioned"

## Next Steps

After provisioning:

```bash
# 1. Configure the infrastructure (install Docker, Docker Compose)
torrust-tracker-deployer configure my-environment

# 2. Verify infrastructure readiness
torrust-tracker-deployer test my-environment
```

## Troubleshooting

### Environment not found

**Problem**: Cannot find environment with the specified name

**Solution**: Verify the environment was created

```bash
# Check environment data directory exists
ls -la data/my-environment/

# If not, create the environment first
torrust-tracker-deployer create environment -f config.json
```

### LXD not initialized (LXD provider only)

**Problem**: LXD is not properly initialized

**Solution**: Initialize LXD

```bash
# Initialize LXD with default settings
sudo lxd init --auto

# Add your user to lxd group
sudo usermod -a -G lxd $USER
newgrp lxd
```

### OpenTofu not found

**Problem**: OpenTofu CLI is not installed or not in PATH

**Solution**: Install OpenTofu

```bash
# Install OpenTofu
curl -fsSL https://get.opentofu.org/install-opentofu.sh | sudo bash

# Verify installation
tofu version
```

### SSH connection timeout

**Problem**: Cannot establish SSH connection to provisioned VM

**Solution**: Check network connectivity and cloud-init status

```bash
# Get VM IP address
lxc list

# Try to connect manually
ssh -i <path-to-private-key> torrust@<vm-ip>

# Check cloud-init status
lxc exec <instance-name> -- cloud-init status
```

### Port already in use

**Problem**: LXD profile or instance name already exists

**Solution**: Clean up existing resources

```bash
# List existing instances
lxc list

# Delete old instance if needed
lxc delete <instance-name> --force

# List profiles
lxc profile list

# Delete old profile if needed
lxc profile delete <profile-name>
```

## Common Use Cases

### Quick local development

```bash
# Create, provision, and configure in sequence
torrust-tracker-deployer create environment -f dev.json
torrust-tracker-deployer provision dev-local
torrust-tracker-deployer configure dev-local
```

### CI/CD pipeline

```bash
#!/bin/bash
set -e

ENV_NAME="ci-${CI_JOB_ID}"

# Create environment
torrust-tracker-deployer create environment -f ci-config.json

# Provision infrastructure
torrust-tracker-deployer provision ${ENV_NAME}

# Run tests...
# Cleanup is handled by destroy command
```

### Reprovisioning

If you need to reprovision (destroy and create again):

```bash
# Destroy existing environment
torrust-tracker-deployer destroy my-environment

# Create fresh environment
torrust-tracker-deployer create environment -f config.json

# Provision again
torrust-tracker-deployer provision my-environment
```

## Technical Details

### Generated Resources

**LXD Resources**:

- Instance: `torrust-tracker-vm-<environment-name>`
- Profile: `torrust-profile-<environment-name>`
- Network: Bridged network with DHCP

**File Artifacts**:

- OpenTofu files: `build/<env>/tofu/lxd/`
- Ansible inventory: `build/<env>/ansible/inventory.yml`
- Instance info: Stored in environment state

### Cloud-Init Configuration

The provisioned VM includes cloud-init configuration for:

- User account creation (SSH username from config)
- SSH key deployment (public key from config)
- Network configuration
- Initial system setup

## See Also

- [create](create.md) - Create environment (prerequisite)
- [configure](configure.md) - Configure infrastructure (next step)
- [test](test.md) - Verify infrastructure
- [destroy](destroy.md) - Clean up infrastructure
