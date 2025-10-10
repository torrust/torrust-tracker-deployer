# Setup Scripts

This directory contains installation and configuration scripts for the tools required by the Torrust Tracker Deployer project.

## Available Scripts

### Core Infrastructure Tools

- **`install-opentofu.sh`** - Install OpenTofu (Terraform alternative)
- **`install-ansible.sh`** - Install Ansible automation platform

### Container/VM Providers

- **`install-lxd-ci.sh`** - Install and configure LXD (CI-optimized)

## Usage

### Individual Installation

```bash
# Install OpenTofu
./scripts/setup/install-opentofu.sh

# Install Ansible
./scripts/setup/install-ansible.sh

# Install LXD (CI environment)
./scripts/setup/install-lxd-ci.sh
```

### Batch Installation

```bash
# Install all core tools
./scripts/setup/install-opentofu.sh
./scripts/setup/install-ansible.sh

# Install container provider
./scripts/setup/install-lxd-ci.sh
```

## CI vs Local Development

### CI Environment Detection

The scripts automatically detect CI environments and apply appropriate configurations:

- **CI Detection**: Checks for `CI=true`, `GITHUB_ACTIONS=true`, or `RUNNER_USER` environment variables
- **CI Optimizations**: Uses `sudo` commands and socket permission fixes
- **Logging**: Provides clear feedback about environment detection

### Local Development

For local development, see the dedicated documentation:

- **LXD**: [templates/tofu/lxd/README.md](../../templates/tofu/lxd/README.md)
- **LXD Tech Stack**: [docs/tech-stack/lxd.md](../../docs/tech-stack/lxd.md)

**Important**: The CI scripts use `sudo` commands and socket permission modifications that are **NOT recommended** for local development. Use proper group membership for local setups.

## Script Features

### Common Features

- **Idempotent**: Safe to run multiple times
- **Verbose Logging**: Clear progress indicators with colored output
- **Error Handling**: Proper error checking and meaningful messages
- **Version Detection**: Skips installation if tool is already installed

### Error Handling

All scripts use `set -euo pipefail` for strict error handling:

- **`-e`**: Exit on any command failure
- **`-u`**: Exit on undefined variable usage
- **`-o pipefail`**: Exit on pipe command failures

### Logging Levels

- **🟢 INFO**: Normal progress messages
- **🟡 WARN**: Important notices or CI-specific actions
- **🔴 ERROR**: Failure messages

## Integration with Workflows

These scripts replace duplicated installation code in GitHub Actions workflows:

- **`.github/workflows/test-e2e.yml`**
- **`.github/workflows/test-lxd-provision.yml`**

## Troubleshooting

### Permission Issues

If you encounter permission errors:

1. **For CI**: Scripts should handle permissions automatically
2. **For Local**: Follow the group membership setup in the tech stack documentation

### Installation Failures

1. Check internet connectivity for download-based installations
2. Verify system requirements (Ubuntu/Debian for apt-based installs)
3. Check available disk space
4. Review script output for specific error messages

### Tool-Specific Issues

- **OpenTofu**: Requires `curl` and package management tools
- **Ansible**: Requires Python and pip (usually pre-installed)
- **LXD**: Requires snap and sufficient privileges

## Future Enhancements

These bash scripts are designed to be simple and maintainable. For more complex installation logic, they may be replaced by Rust utilities in the future while maintaining the same interface.
