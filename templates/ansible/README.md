# Ansible Templates

This directory contains Ansible playbook templates for the Torrust Tracker Deployer project.

## Playbooks

### Core Infrastructure

- **`update-apt-cache.yml`** - Updates APT package cache with retries and network diagnostics

  - ⚠️ **Note**: This playbook contains network-sensitive operations that may fail in CI environments
  - Run this first if you need to update the package cache before installing packages

- **`install-docker.yml`** - Installs Docker CE on Ubuntu/Debian systems

  - ⚠️ **Important**: Does NOT update APT cache automatically to avoid CI issues
  - Run `update-apt-cache.yml` first if needed

- **`install-docker-compose.yml`** - Installs Docker Compose

  - Requires Docker to be installed first (`install-docker.yml`)

- **`wait-cloud-init.yml`** - Waits for cloud-init to complete on newly provisioned VMs

### System Configuration

- **`configure-security-updates.yml`** - Configures automatic security updates

  - Sets up unattended-upgrades for automatic security patches

- **`configure-firewall.yml.tera`** - Configures UFW (Uncomplicated Firewall) with SSH lockout prevention

  - ⚠️ **Critical**: This playbook configures restrictive firewall rules
  - Automatically preserves SSH access on the configured port to prevent lockout
  - **Container Limitation**: Requires kernel capabilities (CAP_NET_ADMIN, CAP_NET_RAW) not available in unprivileged containers
  - **Automatic Skip**: Container-based E2E tests automatically skip this step via `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER` environment variable
    - Accepted values: `"true"` or `"false"` (case-sensitive, lowercase only)
    - Example: `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true`
  - **VM-only**: This playbook is only executed in VM-based deployments and tests### Configuration Files

- **`ansible.cfg`** - Ansible configuration
- **`inventory.yml.tera`** - Inventory template file (processed by Tera templating engine)

## Usage Order

For a typical deployment:

1. **`wait-cloud-init.yml`** - Wait for VM to be ready
2. **`update-apt-cache.yml`** - Update package cache (if needed, skip in CI)
3. **`install-docker.yml`** - Install Docker
4. **`install-docker-compose.yml`** - Install Docker Compose (optional)
5. **`configure-security-updates.yml`** - Configure automatic security updates
6. **`configure-firewall.yml.tera`** - Configure UFW firewall (VM-only, skipped in containers)

## CI/Testing Considerations

- The `update-apt-cache.yml` playbook is separated from installation playbooks to avoid CI issues
- In E2E tests, you can skip the cache update step to avoid network timeouts
- The installation playbooks assume the cache is already up-to-date or will handle missing packages gracefully
- **Firewall configuration** is automatically skipped in container-based E2E tests because:
  - UFW/iptables require kernel-level capabilities (`CAP_NET_ADMIN`, `CAP_NET_RAW`)
  - Docker containers run unprivileged by default and lack these capabilities
  - The deployer sets `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true` for container tests (accepts `"true"` or `"false"` only)
  - VM-based tests (LXD) have full kernel access and run the firewall playbook normally

## Template Processing

These files are processed by the Tera templating engine and written to the `build/ansible/` directory during the build process.
