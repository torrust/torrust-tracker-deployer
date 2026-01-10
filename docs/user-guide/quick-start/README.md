# Quick Start Guides

Choose the guide that matches your deployment scenario.

## Available Guides

| Guide                            | Best For                        | Provider Support |
| -------------------------------- | ------------------------------- | ---------------- |
| [Docker Deployment](docker.md)   | Fast setup, cloud deployments   | Hetzner only     |
| [Native Installation](native.md) | Local development, full control | LXD, Hetzner     |

## Which Guide Should I Use?

### Use Docker Deployment if

- You want the **fastest setup** with minimal dependencies
- You're deploying to **Hetzner Cloud** (or other cloud providers in the future)
- You prefer **containerized tools** for reproducibility
- You don't need LXD local development

### Use Native Installation if

- You need **LXD support** for local development or CI/CD
- You want to **contribute** to the project
- You prefer tools installed directly on your system
- You need the **full feature set** including all providers

## Quick Decision Tree

```text
Do you need LXD (local VMs)?
├── Yes → Native Installation
└── No → Are you deploying to Hetzner Cloud?
         ├── Yes → Docker Deployment (recommended)
         └── No → Native Installation
```

## Future Guides (Planned)

- **Pre-provisioned Server** - Deploy to an existing server you manage
- **Templates Only** - Generate configuration files without automated deployment
- **CI/CD Integration** - Automated deployments in pipelines

## Prerequisites Overview

| Requirement | Docker           | Native           |
| ----------- | ---------------- | ---------------- |
| Docker      | ✅ Required      | ❌ Not needed    |
| Rust/Cargo  | ❌ Not needed    | ✅ Required      |
| OpenTofu    | ❌ Included      | ✅ Required      |
| Ansible     | ❌ Included      | ✅ Required      |
| LXD         | ❌ Not supported | ✅ For local dev |
| SSH Keys    | ✅ Required      | ✅ Required      |

## Next Steps

After completing a quick start guide:

1. **[Provider Guides](../providers/README.md)** - Detailed provider configuration
2. **[Command Reference](../commands/README.md)** - All available commands
3. **[Troubleshooting](../../contributing/known-issues.md)** - Common issues and solutions
