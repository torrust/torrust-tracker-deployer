# Provider Guides

This directory contains provider-specific configuration guides.

## Available Providers

| Provider                    | Status    | Description                                |
| --------------------------- | --------- | ------------------------------------------ |
| [LXD](lxd.md)               | ✅ Stable | Local development using LXD containers/VMs |
| [Hetzner Cloud](hetzner.md) | 🆕 New    | Cost-effective European cloud provider     |

## Choosing a Provider

### LXD (Local Development)

**Best for**: Local development, testing, CI/CD pipelines, zero cloud costs.

**Requirements**: Linux system with LXD installed.

### Hetzner Cloud (Production)

**Best for**: Production deployments, European hosting, cost-sensitive projects.

**Requirements**: Hetzner Cloud account with API token.

## Adding New Providers

To add a new provider:

1. Create OpenTofu templates in `templates/tofu/<provider>/`
2. Add provider configuration types in `src/domain/provider/`
3. Update the template renderer for provider-specific logic
4. Add documentation in `docs/user-guide/providers/<provider>.md`

See the [contributing guide](../../contributing/README.md) for more details.

## Related Documentation

- [Quick Start Guide](../quick-start.md) - Complete deployment workflow
- [Commands Reference](../commands/README.md) - Available commands
- [SSH Keys](../../tech-stack/ssh-keys.md) - SSH key generation and management
