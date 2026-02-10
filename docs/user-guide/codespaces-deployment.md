# Using GitHub Codespaces

This guide explains how to use the Torrust Tracker Deployer in GitHub Codespaces without installing any dependencies locally.

## Overview

GitHub Codespaces provides a cloud-based development environment that comes pre-configured with all the tools needed to run the Torrust Tracker Deployer:

- ✅ OpenTofu (Terraform alternative)
- ✅ Ansible
- ✅ Rust toolchain
- ✅ All project dependencies

This means you can use the deployer directly from your browser without installing anything on your local machine.

## Creating a Codespace

1. Navigate to the [torrust-tracker-deployer](https://github.com/torrust/torrust-tracker-deployer) repository
2. Click the **Code** button
3. Select the **Codespaces** tab
4. Click **Create codespace on main**

The environment will initialize automatically (takes 2-3 minutes):

- Downloads the Docker image with all dependencies
- Builds the project with `cargo build`
- Configures VS Code extensions

## What's Included

The Codespace comes pre-configured with:

- **VS Code Extensions**:
  - Rust Analyzer (Rust language support)
  - Even Better TOML (TOML formatting)
  - YAML (YAML validation)
  - GitHub Copilot (AI assistance)
- **Settings**:
  - Agent skills enabled for Copilot
  - JSON schema validation for environment files
  - TOML formatter configuration

## Using the Deployer

Once your Codespace is running, use the deployer normally. See the main documentation:

## Using the Deployer

Once your Codespace is running, use the deployer normally. See the main documentation:

- **[User Guide](README.md)** - Complete deployer documentation
- **[Quick Start Guides](quick-start/README.md)** - Step-by-step deployment guides
- **[Command Reference](../console-commands.md)** - All available commands

## Supported Features

### ✅ Supported

- **Cloud Providers**: All cloud provider deployments (Hetzner, AWS, Azure, GCP, etc.)
- **Databases**: SQLite, MySQL, PostgreSQL
- **All CLI Commands**: `create`, `provision`, `configure`, `release`, `run`, `destroy`, etc.
- **E2E Tests**: All tests except those requiring LXD
- **Linting**: All linters (`cargo run --bin linter all`)
- **Documentation**: View and edit all project documentation

### ❌ Not Supported

- **Local LXD Provider**: Codespaces runs in containers, not VMs
  - Nested virtualization is not available
  - Cannot use `provider.type = "lxd"` in environment configs
  - Use cloud providers instead

## Managing Secrets

When using cloud provider credentials, use GitHub Codespaces secrets instead of committing them to files:

1. Go to your [Codespaces settings](https://github.com/settings/codespaces)
2. Under **Secrets**, click **New secret**
3. Add your secrets (API tokens, SSH keys, passwords)
4. Reference them as environment variables in your configuration files

**Documentation**: [Managing Codespaces Secrets](https://docs.github.com/en/codespaces/managing-your-codespaces/managing-secrets-for-your-codespaces)

## GitHub Copilot Integration

If you have GitHub Copilot enabled, it can assist with deployment tasks. Copilot has access to:

- Project documentation (`AGENTS.md`)
- Agent skills (`.github/skills/`)
- Architecture guides and ADRs

## Cleanup

### Stop Codespace

Codespaces auto-stop after 30 minutes of inactivity. To stop manually:

1. Go to https://github.com/codespaces
2. Find your Codespace
3. Click **Stop codespace**

### Delete Codespace

To completely remove the Codespace and its data:

1. Go to https://github.com/codespaces
2. Find your Codespace
3. Click **Delete**

## Cost Considerations

GitHub Codespaces usage is billed based on compute time and storage. See [GitHub Codespaces pricing](https://docs.github.com/en/billing/managing-billing-for-github-codespaces/about-billing-for-github-codespaces) for current rates.

**Tips**:

- Auto-stop is enabled by default (30 minutes)
- Delete Codespaces after use to avoid storage costs
- Use smaller machine types when sufficient

## Related Documentation

- [GitHub Codespaces Documentation](https://docs.github.com/en/codespaces)
- [Devcontainer Specification](https://containers.dev/)
- [User Guide](README.md) - Complete deployer documentation
- [Command Reference](../console-commands.md) - All available commands
