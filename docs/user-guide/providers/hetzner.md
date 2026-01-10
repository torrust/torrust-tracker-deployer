# Hetzner Cloud Provider

This guide covers Hetzner-specific configuration for cloud deployments.

## Overview

[Hetzner Cloud](https://www.hetzner.com/cloud) provides affordable virtual servers with excellent performance. Ideal for production deployments.

**Why Hetzner?**

- Cost-effective pricing
- European data centers (Germany, Finland) + US locations
- Simple, predictable billing
- NVMe storage and modern hardware

## Prerequisites

- Hetzner Cloud account ([sign up](https://www.hetzner.com/cloud))
- API token with read/write permissions
- SSH key pair (see [SSH keys guide](../../tech-stack/ssh-keys.md))

## Create API Token

1. Log in to [Hetzner Cloud Console](https://console.hetzner.cloud/)
2. Select your project (or create a new one)
3. Navigate to **Security** → **API Tokens**
4. Click **Generate API Token**
5. Select **Read & Write** permissions
6. **Copy the token immediately** - it won't be shown again!

> ⚠️ **Security**: Never commit API tokens to version control.

## Hetzner-Specific Configuration

```json
{
  "provider": {
    "provider": "hetzner",
    "api_token": "YOUR_HETZNER_API_TOKEN",
    "server_type": "cx22",
    "location": "nbg1",
    "image": "ubuntu-24.04"
  }
}
```

| Field         | Description            | Example        |
| ------------- | ---------------------- | -------------- |
| `provider`    | Must be `"hetzner"`    | `hetzner`      |
| `api_token`   | Your Hetzner API token | `hcloud_xxx…`  |
| `server_type` | Server size/type       | `cx22`         |
| `location`    | Datacenter location    | `nbg1`         |
| `image`       | Operating system image | `ubuntu-24.04` |

### Available Server Types

| Type    | vCPUs | RAM   | Storage | Use Case                    |
| ------- | ----- | ----- | ------- | --------------------------- |
| `cx22`  | 2     | 4 GB  | 40 GB   | Development, small trackers |
| `cx32`  | 4     | 8 GB  | 80 GB   | Production, medium traffic  |
| `cx42`  | 8     | 16 GB | 160 GB  | High-traffic trackers       |
| `cpx11` | 2     | 2 GB  | 40 GB   | Testing (AMD)               |
| `cpx21` | 3     | 4 GB  | 80 GB   | Development (AMD)           |

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

## Cost Estimation

Approximate monthly costs (check [Hetzner pricing](https://www.hetzner.com/cloud) for current rates):

| Server Type | Monthly Cost (EUR) |
| ----------- | ------------------ |
| `cx22`      | ~€4.35             |
| `cx32`      | ~€8.70             |
| `cx42`      | ~€17.40            |

> ⚠️ **Important**: Remember to destroy resources when not in use to avoid charges.

## Troubleshooting

### API Token Invalid

**Error**: `Failed to authenticate with Hetzner API`

- Verify your API token is correct
- Ensure the token has **Read & Write** permissions
- Check the token hasn't been revoked

### Server Creation Failed

**Possible causes**:

- **Quota exceeded**: Check your Hetzner project limits
- **Invalid server type**: Verify the server type exists in your location
- **Image not available**: Some images may not be available in all locations

### SSH Connection Timeout

```bash
# Check if server is running in Hetzner Console
# Verify firewall rules (if using Hetzner Firewall) - port 22 must be open

# Check SSH key permissions
chmod 600 ~/.ssh/your_private_key

# Test manual SSH connection
ssh -i ~/.ssh/your_private_key -v torrust@<server-ip>
```

### Cloud-init Timeout

```bash
# SSH into the server manually
ssh -i ~/.ssh/your_key root@<server-ip>

# Check cloud-init status
cloud-init status --wait

# View cloud-init logs
cat /var/log/cloud-init-output.log
```

## Security Best Practices

1. **Never commit API tokens** - Use environment variables or secure vaults
2. **Restrict SSH access** - Consider using Hetzner Firewall
3. **Use strong SSH keys** - Ed25519 or RSA 4096-bit minimum
4. **Regular updates** - Keep server packages updated
5. **Disable root SSH access** - For production, see [SSH Root Access Guide](../../security/ssh-root-access-hetzner.md)

## SSH Key Behavior

Hetzner deployments configure SSH access through two mechanisms:

| Mechanism                 | User      | Purpose                   |
| ------------------------- | --------- | ------------------------- |
| OpenTofu `hcloud_ssh_key` | `root`    | Emergency/debug access    |
| cloud-init                | `torrust` | Normal application access |

**Why both?** If cloud-init fails, root SSH access provides a debugging path. Without it, a failed cloud-init would leave the server completely inaccessible.

**For stricter security**: You can disable root SSH access after deployment. See [SSH Root Access on Hetzner](../../security/ssh-root-access-hetzner.md) for instructions.

**Note**: The SSH key appears in your Hetzner Console under **Security** → **SSH Keys** with the name `torrust-tracker-vm-<environment>-ssh-key`.

## Related Documentation

- [Quick Start: Docker](../quick-start/docker.md) - Deploy to Hetzner using Docker
- [Quick Start: Native](../quick-start/native.md) - Deploy using native installation
- [SSH Keys Guide](../../tech-stack/ssh-keys.md) - SSH key generation
- [SSH Root Access Security](../../security/ssh-root-access-hetzner.md) - Disabling root access
- [LXD Provider](lxd.md) - Local development alternative
