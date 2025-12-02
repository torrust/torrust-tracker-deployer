# Provider Guides

This directory contains guides for deploying Torrust Tracker infrastructure to different cloud providers and virtualization platforms.

## Available Providers

| Provider                    | Status    | Description                                |
| --------------------------- | --------- | ------------------------------------------ |
| [LXD](lxd.md)               | ✅ Stable | Local development using LXD containers/VMs |
| [Hetzner Cloud](hetzner.md) | 🆕 New    | Cost-effective European cloud provider     |

## Choosing a Provider

### LXD (Local Development)

**Best for**:

- Local development and testing
- Learning the deployment workflow
- CI/CD pipelines
- No cloud costs

**Requirements**:

- Linux system with LXD installed
- Local storage for VMs

### Hetzner Cloud (Production)

**Best for**:

- Production deployments
- European hosting requirements
- Cost-sensitive projects
- Simple, predictable pricing

**Requirements**:

- Hetzner Cloud account
- API token with read/write access
- SSH key pair

## Provider Configuration

Each provider requires specific configuration in your environment JSON file:

### LXD Configuration

```json
{
  "provider": {
    "provider": "lxd",
    "profile_name": "my-profile"
  }
}
```

### Hetzner Configuration

```json
{
  "provider": {
    "provider": "hetzner",
    "api_token": "your-api-token",
    "server_type": "cx22",
    "location": "nbg1",
    "image": "ubuntu-24.04"
  }
}
```

## Adding New Providers

The deployer is designed to support multiple providers. To add a new provider:

1. Create OpenTofu templates in `templates/tofu/<provider>/`
2. Add provider configuration types in `src/domain/provider/`
3. Update the template renderer for provider-specific logic
4. Add documentation in `docs/user-guide/providers/`

See the [contributing guide](../../contributing/README.md) for more details.

## Common Configuration

All providers share common configuration sections:

### Environment Section

```json
{
  "environment": {
    "name": "unique-environment-name"
  }
}
```

### SSH Credentials Section

```json
{
  "ssh_credentials": {
    "private_key_path": "/path/to/private/key",
    "public_key_path": "/path/to/public/key.pub",
    "username": "torrust",
    "port": 22
  }
}
```

## Related Documentation

- [Quick Start Guide](../quick-start.md) - Getting started with the deployer
- [Commands Reference](../commands/README.md) - Available commands
- [Template Customization](../template-customization.md) - Customizing deployment templates
