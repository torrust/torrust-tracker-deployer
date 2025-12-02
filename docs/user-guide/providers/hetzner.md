# Deploying to Hetzner Cloud

This guide explains how to deploy Torrust Tracker infrastructure to [Hetzner Cloud](https://www.hetzner.com/cloud), a cost-effective European cloud provider.

## Overview

Hetzner Cloud provides affordable virtual servers (VPS) with excellent performance. The deployer uses OpenTofu to provision servers and Ansible to configure them with Docker.

### Why Hetzner Cloud?

- **Cost-effective**: Competitive pricing for cloud servers
- **European data centers**: Locations in Germany and Finland
- **Simple pricing**: No hidden costs, predictable billing
- **Good performance**: NVMe storage and modern hardware

## Prerequisites

Before deploying to Hetzner Cloud, ensure you have:

1. **Hetzner Cloud Account**: Sign up at [hetzner.com/cloud](https://www.hetzner.com/cloud)
2. **API Token**: Generated from Hetzner Cloud Console
3. **SSH Key Pair**: For secure server access
4. **Deployer Dependencies**: OpenTofu and Ansible installed

### Installing Dependencies

```bash
# Verify all dependencies are installed
cargo run --bin dependency-installer check

# Install missing dependencies
cargo run --bin dependency-installer install
```

## Step 1: Create a Hetzner API Token

1. Log in to [Hetzner Cloud Console](https://console.hetzner.cloud/)
2. Select your project (or create a new one)
3. Navigate to **Security** → **API Tokens**
4. Click **Generate API Token**
5. Give it a descriptive name (e.g., "torrust-deployer")
6. Select **Read & Write** permissions
7. Click **Generate API Token**
8. **Copy the token immediately** - it won't be shown again!

> ⚠️ **Security Warning**: Keep your API token secret. Never commit it to version control. Consider using environment variables for production deployments.

## Step 2: Generate SSH Keys (if needed)

If you don't have SSH keys, generate them:

```bash
# Generate a new SSH key pair
ssh-keygen -t ed25519 -C "torrust-hetzner" -f ~/.ssh/torrust_hetzner

# Set proper permissions
chmod 600 ~/.ssh/torrust_hetzner
chmod 644 ~/.ssh/torrust_hetzner.pub
```

## Step 3: Create Environment Configuration

Create a configuration file for your Hetzner deployment:

```bash
# Create configuration in the envs directory (git-ignored)
nano envs/my-hetzner-env.json
```

**Example configuration**:

```json
{
  "environment": {
    "name": "my-hetzner-env"
  },
  "provider": {
    "provider": "hetzner",
    "api_token": "YOUR_HETZNER_API_TOKEN",
    "server_type": "cx22",
    "location": "nbg1",
    "image": "ubuntu-24.04"
  },
  "ssh_credentials": {
    "private_key_path": "/home/youruser/.ssh/torrust_hetzner",
    "public_key_path": "/home/youruser/.ssh/torrust_hetzner.pub",
    "username": "torrust",
    "port": 22
  }
}
```

### Configuration Fields

| Field                              | Description                           | Example                          |
| ---------------------------------- | ------------------------------------- | -------------------------------- |
| `environment.name`                 | Unique identifier for this deployment | `my-hetzner-env`                 |
| `provider.provider`                | Must be `"hetzner"`                   | `hetzner`                        |
| `provider.api_token`               | Your Hetzner API token                | `hcloud_xxx...`                  |
| `provider.server_type`             | Server size/type                      | `cx22`                           |
| `provider.location`                | Datacenter location                   | `nbg1`                           |
| `provider.image`                   | Operating system image                | `ubuntu-24.04`                   |
| `ssh_credentials.private_key_path` | Path to SSH private key               | `/home/user/.ssh/id_ed25519`     |
| `ssh_credentials.public_key_path`  | Path to SSH public key                | `/home/user/.ssh/id_ed25519.pub` |
| `ssh_credentials.username`         | SSH user to create                    | `torrust`                        |
| `ssh_credentials.port`             | SSH port                              | `22`                             |

### Available Server Types

Common Hetzner Cloud server types:

| Type    | vCPUs | RAM   | Storage | Use Case                    |
| ------- | ----- | ----- | ------- | --------------------------- |
| `cx22`  | 2     | 4 GB  | 40 GB   | Development, small trackers |
| `cx32`  | 4     | 8 GB  | 80 GB   | Production, medium traffic  |
| `cx42`  | 8     | 16 GB | 160 GB  | High-traffic trackers       |
| `cpx11` | 2     | 2 GB  | 40 GB   | Testing (AMD)               |
| `cpx21` | 3     | 4 GB  | 80 GB   | Development (AMD)           |

> **Tip**: Start with `cx22` for development and scale up as needed.

### Available Locations

| Location | City        | Country    |
| -------- | ----------- | ---------- |
| `fsn1`   | Falkenstein | Germany    |
| `nbg1`   | Nuremberg   | Germany    |
| `hel1`   | Helsinki    | Finland    |
| `ash`    | Ashburn     | USA (East) |
| `hil`    | Hillsboro   | USA (West) |

### Available Images

| Image          | Description                    |
| -------------- | ------------------------------ |
| `ubuntu-24.04` | Ubuntu 24.04 LTS (recommended) |
| `ubuntu-22.04` | Ubuntu 22.04 LTS               |
| `debian-12`    | Debian 12 (Bookworm)           |
| `debian-11`    | Debian 11 (Bullseye)           |

## Step 4: Create the Environment

```bash
torrust-tracker-deployer create environment --env-file envs/my-hetzner-env.json
```

**Expected output**:

```text
✓ Validating configuration...
✓ Creating environment structure...
✓ Environment created successfully: my-hetzner-env
```

## Step 5: Provision Infrastructure

Create the Hetzner Cloud server:

```bash
torrust-tracker-deployer provision my-hetzner-env
```

**Expected output**:

```text
✓ Rendering OpenTofu templates...
✓ Initializing infrastructure...
✓ Planning infrastructure changes...
✓ Applying infrastructure...
✓ Retrieving instance information...
✓ Instance IP: 203.0.113.42
✓ Rendering Ansible templates...
✓ Waiting for SSH connectivity...
✓ Waiting for cloud-init completion...
✓ Environment provisioned successfully
```

**What happens**:

1. OpenTofu creates an SSH key in Hetzner Cloud
2. A new server is provisioned with your specifications
3. Cloud-init configures the server with your SSH key
4. The deployer waits for SSH to become available

**Duration**: ~1-2 minutes

## Step 6: Configure Software

Install Docker and Docker Compose:

```bash
torrust-tracker-deployer configure my-hetzner-env
```

**Expected output**:

```text
✓ Validating prerequisites...
✓ Running Ansible playbooks...
✓ Installing Docker...
✓ Installing Docker Compose...
✓ Configuring permissions...
✓ Verifying installation...
✓ Environment configured successfully
```

**Duration**: ~3-5 minutes (depending on network speed)

## Step 7: Verify Deployment

Test that everything works:

```bash
torrust-tracker-deployer test my-hetzner-env
```

**Expected output**:

```text
✓ Validating environment state...
✓ Checking VM connectivity...
✓ Testing Docker installation...
✓ Testing Docker Compose...
✓ Verifying user permissions...
✓ Running infrastructure tests...
✓ All tests passed
```

## Step 8: Connect to Your Server

SSH into your server:

```bash
# Get the server IP from the deployment output, or:
ssh -i ~/.ssh/torrust_hetzner torrust@<server-ip>
```

Once connected, verify Docker:

```bash
docker --version
docker compose version
docker ps
```

## Step 9: Clean Up (When Done)

Destroy the infrastructure to stop billing:

```bash
torrust-tracker-deployer destroy my-hetzner-env
```

**Expected output**:

```text
✓ Stopping containers...
✓ Destroying infrastructure...
✓ Cleaning up resources...
✓ Environment destroyed successfully
```

> ⚠️ **Important**: Remember to destroy resources when not in use to avoid unnecessary charges.

## Cost Estimation

Approximate monthly costs (as of 2024):

| Server Type | Monthly Cost (EUR) |
| ----------- | ------------------ |
| `cx22`      | ~€4.35             |
| `cx32`      | ~€8.70             |
| `cx42`      | ~€17.40            |

> **Note**: Prices may vary. Check [Hetzner Cloud pricing](https://www.hetzner.com/cloud) for current rates.

## Troubleshooting

### API Token Invalid

**Error**: `Failed to authenticate with Hetzner API`

**Solution**:

1. Verify your API token is correct
2. Ensure the token has **Read & Write** permissions
3. Check the token hasn't expired

### SSH Connection Timeout

**Error**: `Failed to connect via SSH`

**Solution**:

```bash
# Check if server is running in Hetzner Console

# Verify firewall rules (if using Hetzner Firewall)
# Ensure port 22 is open for inbound SSH

# Check SSH key permissions
chmod 600 ~/.ssh/your_private_key

# Test manual SSH connection
ssh -i ~/.ssh/your_private_key -v torrust@<server-ip>
```

### Server Creation Failed

**Error**: `Failed to create server`

**Possible causes**:

1. **Quota exceeded**: Check your Hetzner project limits
2. **Invalid server type**: Verify the server type exists in your location
3. **Image not available**: Some images may not be available in all locations

### Cloud-init Timeout

**Error**: `Timeout waiting for cloud-init`

**Solution**:

```bash
# SSH into the server manually
ssh -i ~/.ssh/your_key root@<server-ip>

# Check cloud-init status
cloud-init status --wait

# View cloud-init logs
cat /var/log/cloud-init-output.log
```

## Security Best Practices

1. **Never commit API tokens**: Use environment variables or secure vaults
2. **Restrict SSH access**: Consider using Hetzner Firewall
3. **Use strong SSH keys**: Ed25519 or RSA 4096-bit minimum
4. **Regular updates**: Keep server packages updated
5. **Backups**: Enable Hetzner Cloud backups for important data

### Using Environment Variables for API Token

Instead of storing the token in the config file:

```bash
# Set environment variable
export HETZNER_API_TOKEN="your-token-here"

# In your config, use a placeholder and replace at runtime
# (This feature may be added in future versions)
```

## Next Steps

After successful deployment:

1. **Deploy Torrust Tracker**: Follow the Torrust Tracker deployment guide
2. **Configure DNS**: Point your domain to the server IP
3. **Set up TLS**: Configure SSL certificates for secure connections
4. **Monitor**: Set up monitoring and alerting

## Related Documentation

- [Quick Start Guide](../quick-start.md) - General deployment workflow
- [Command Reference](../commands/README.md) - Detailed command documentation
- [LXD Provider](lxd.md) - Local development with LXD
- [Template Customization](../template-customization.md) - Customize deployment templates
