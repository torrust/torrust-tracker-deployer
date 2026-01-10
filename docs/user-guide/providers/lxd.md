# LXD Provider

This guide covers LXD-specific configuration for deploying locally.

## Overview

LXD provides lightweight virtual machines that run on your local system. Ideal for development, testing, and CI/CD pipelines.

**Why LXD?**

- Zero cloud costs
- Fast iteration
- CI/CD friendly (works in GitHub Actions)

## Prerequisites

- LXD installed and initialized (see [LXD tech guide](../../tech-stack/lxd.md))
- SSH key pair (see [SSH keys guide](../../tech-stack/ssh-keys.md))

## LXD-Specific Configuration

```json
{
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-local"
  }
}
```

| Field          | Description                     | Example                 |
| -------------- | ------------------------------- | ----------------------- |
| `provider`     | Must be `"lxd"`                 | `lxd`                   |
| `profile_name` | LXD profile name (auto-created) | `torrust-profile-local` |

## LXD-Specific Operations

### Check VM Status

```bash
lxc list
```

### Direct Console Access

```bash
lxc exec torrust-tracker-vm-<environment-name> -- bash
```

### Manual Cleanup

If you need to manually clean up:

```bash
# Delete an instance
lxc delete <instance-name> --force

# Delete a profile
lxc profile delete <profile-name>
```

## Troubleshooting

### LXD Not Running

```bash
# Check LXD status
sudo systemctl status snap.lxd.daemon

# Restart LXD
sudo systemctl restart snap.lxd.daemon
```

### Permission Denied

See the [LXD Group Setup](../../tech-stack/lxd.md#proper-lxd-group-setup) section in the LXD tech guide.

### Network Issues

```bash
# Check network bridge
lxc network list

# Recreate default bridge if needed
lxc network delete lxdbr0
lxc network create lxdbr0
```

## Resource Requirements

| Resource | Minimum | Recommended   |
| -------- | ------- | ------------- |
| RAM      | 4 GB    | 8+ GB         |
| CPU      | 2 cores | 4+ cores      |
| Storage  | 20 GB   | 50+ GB        |
| OS       | Linux   | Ubuntu 22.04+ |

## Related Documentation

- [LXD Tech Guide](../../tech-stack/lxd.md) - Installation and detailed LXD operations
- [Quick Start: Native](../quick-start/native.md) - LXD deployment workflow
- [Hetzner Provider](hetzner.md) - Cloud deployment alternative
