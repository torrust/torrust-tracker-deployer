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

- **`configure-firewall.yml`** - Configures UFW (Uncomplicated Firewall) with SSH lockout prevention
  - ⚠️ **Critical**: This playbook configures restrictive firewall rules
  - Automatically preserves SSH access on the configured port to prevent lockout
  - Uses centralized variables from `variables.yml` (loaded via `vars_files`)
  - **Container Limitation**: Requires kernel capabilities (CAP_NET_ADMIN, CAP_NET_RAW) not available in unprivileged containers
  - **Automatic Skip**: Container-based E2E tests automatically skip this step via `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER` environment variable
    - Accepted values: `"true"` or `"false"` (case-sensitive, lowercase only)
    - Example: `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true`
  - **VM-only**: This playbook is only executed in VM-based deployments and tests

### Configuration Files

- **`ansible.cfg`** - Ansible configuration (static)
- **`inventory.yml.tera`** - Inventory template file (processed by Tera templating engine)
- **`variables.yml.tera`** - Centralized variables template (processed by Tera templating engine)

## Usage Order

For a typical deployment:

1. **`wait-cloud-init.yml`** - Wait for VM to be ready
2. **`update-apt-cache.yml`** - Update package cache (if needed, skip in CI)
3. **`install-docker.yml`** - Install Docker
4. **`install-docker-compose.yml`** - Install Docker Compose (optional)
5. **`configure-security-updates.yml`** - Configure automatic security updates
6. **`configure-firewall.yml`** - Configure UFW firewall (VM-only, skipped in containers)

## CI/Testing Considerations

- The `update-apt-cache.yml` playbook is separated from installation playbooks to avoid CI issues
- In E2E tests, you can skip the cache update step to avoid network timeouts
- The installation playbooks assume the cache is already up-to-date or will handle missing packages gracefully
- **Firewall configuration** is automatically skipped in container-based E2E tests because:
  - UFW/iptables require kernel-level capabilities (`CAP_NET_ADMIN`, `CAP_NET_RAW`)
  - Docker containers run unprivileged by default and lack these capabilities
  - The deployer sets `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true` for container tests (accepts `"true"` or `"false"` only)
  - VM-based tests (LXD) have full kernel access and run the firewall playbook normally

## Variables Pattern

This directory uses a **centralized variables pattern**:

- **`variables.yml.tera`** - Centralized variables (rendered at runtime with Tera)
- **`inventory.yml.tera`** - Connection variables (rendered at runtime with Tera)
- **`*.yml`** - Static playbooks that load `variables.yml` via `vars_files` directive

### When Adding New Playbooks

1. **Add variables** to `variables.yml.tera`
2. **Create static** `.yml` playbook (not `.tera`)
3. **Add `vars_files: [variables.yml]`** to playbook
4. **Register** in `copy_static_templates()` if new static playbook

This pattern reduces Rust boilerplate (no per-playbook renderer/wrapper/context needed) while providing centralized variable management.

## Red Flags (stop and reconsider)

- Adding tasks to an existing playbook instead of creating a new atomic playbook
- Using Ansible `when:` to decide if a service/feature is enabled (gating belongs in Rust commands/steps; `when:` is only for host facts)
- Playbook names with "and" or multiple unrelated actions
- Multiple unrelated `ansible.builtin.file`/copy tasks bundled together

**Correct pattern**: New feature → new atomic playbook + new Rust step that decides whether to run it. See [atomic-ansible-playbooks.md](../../decisions/atomic-ansible-playbooks.md) and AGENTS rule #8.

## Before adding Ansible functionality checklist

- [ ] Am I adding tasks to an existing playbook? → Create a new atomic playbook
- [ ] Does the playbook do more than one conceptual thing? → Split it
- [ ] Am I using `when:` for feature/service enablement? → Move gating to Rust command/step
- [ ] Did I add a corresponding Rust step that conditionally runs this playbook?
- [ ] Did I register the static playbook in `copy_static_templates()`?
- [ ] Does the playbook name describe a single action?

## Template Processing

Files with `.tera` extension are processed by the Tera templating engine. All files are written to the `build/ansible/` directory during the build process.
