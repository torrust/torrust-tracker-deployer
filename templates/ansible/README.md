# Ansible Playbooks

This directory contains Ansible playbook templates for the Torrust Tracker Deploy project.

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

### Configuration Files

- **`ansible.cfg`** - Ansible configuration
- **`inventory.yml.tera`** - Inventory template file (processed by Tera templating engine)

## Usage Order

For a typical deployment:

1. **`wait-cloud-init.yml`** - Wait for VM to be ready
2. **`update-apt-cache.yml`** - Update package cache (if needed, skip in CI)
3. **`install-docker.yml`** - Install Docker
4. **`install-docker-compose.yml`** - Install Docker Compose (optional)

## CI/Testing Considerations

- The `update-apt-cache.yml` playbook is separated from installation playbooks to avoid CI issues
- In E2E tests, you can skip the cache update step to avoid network timeouts
- The installation playbooks assume the cache is already up-to-date or will handle missing packages gracefully

## Template Processing

These files are processed by the Tera templating engine and written to the `build/ansible/` directory during the build process.
