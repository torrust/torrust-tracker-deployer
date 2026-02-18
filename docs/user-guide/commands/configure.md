# `configure` - Configure Provisioned Infrastructure

Configure software dependencies on provisioned infrastructure.

## Purpose

Installs and configures Docker and Docker Compose on provisioned VM infrastructure. This command takes an environment from the "Provisioned" state to the "Configured" state with all required software installed.

## Command Syntax

```bash
torrust-tracker-deployer configure <ENVIRONMENT>
```

## Verbosity Levels

Control the amount of progress detail displayed during configuration with the global `-v` flag. This helps you see what's happening under the hood when you need more visibility.

### Available Levels

| Level           | Flag      | Shows                                       | Use Case                                    |
| --------------- | --------- | ------------------------------------------- | ------------------------------------------- |
| **Normal**      | (default) | Essential progress and results              | Regular usage, clean output                 |
| **Verbose**     | `-v`      | + Detailed progress (4 configuration steps) | Understanding the configuration workflow    |
| **VeryVerbose** | `-vv`     | + Context details (versions, status)        | Troubleshooting configuration issues        |
| **Debug**       | `-vvv`    | + Technical details (Ansible commands)      | Deep troubleshooting, development debugging |

**Important**: Verbosity controls **only** progress messages. For internal diagnostic logs, use the `RUST_LOG` environment variable (see [Logging Guide](../logging.md)).

### Normal Level (Default)

Shows essential progress with minimal output:

```bash
torrust-tracker-deployer configure my-env
```

**Output**:

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: my-env (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
⏳   ✓ Infrastructure configured (took 37.3s)
✅ Environment 'my-env' configured successfully
```

### Verbose Level (`-v`)

Shows the 4 internal configuration steps:

```bash
torrust-tracker-deployer configure my-env -v
```

**Output**:

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: my-env (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
📋   [Step 1/4] Installing Docker...
📋   [Step 2/4] Installing Docker Compose...
📋   [Step 3/4] Configuring automatic security updates...
📋   [Step 4/4] Configuring firewall (UFW)...
⏳   ✓ Infrastructure configured (took 34.1s)
✅ Environment 'my-env' configured successfully
```

**When to use**: Understanding the configuration workflow, seeing which step is taking time, or confirming the command is making progress.

### VeryVerbose Level (`-vv`)

Adds contextual details like software versions, configurations, and status information:

```bash
torrust-tracker-deployer configure my-env -vv
```

**Output**:

```text
⏳ [3/3] Configuring infrastructure...
📋   [Step 1/4] Installing Docker...
📋      → Installing Docker Engine from official repository
📋      → Docker version: 24.0.7
📋   [Step 2/4] Installing Docker Compose...
📋      → Installing Docker Compose plugin
📋      → Compose version: 2.23.3
📋   [Step 3/4] Configuring automatic security updates...
📋      → Configuring unattended-upgrades for automatic security patches
📋      → Update configuration status: enabled
📋   [Step 4/4] Configuring firewall (UFW)...
📋      → Configuring UFW with restrictive default policies
📋      → Allowing SSH access before enabling firewall
📋      → Firewall status: active
⏳   ✓ Infrastructure configured (took 34.1s)
✅ Environment 'my-env' configured successfully
```

**When to use**: Troubleshooting installation issues, verifying software versions, understanding firewall configurations, or monitoring security update setup.

### Debug Level (`-vvv`)

Shows technical implementation details including Ansible commands executed:

```bash
torrust-tracker-deployer configure my-env -vvv
```

**Output**:

```text
⏳ [3/3] Configuring infrastructure...
📋   [Step 1/4] Installing Docker...
🔍      → Ansible working directory: ./build/my-env/ansible
🔍      → Executing playbook: ansible-playbook install-docker.yml -i inventory.ini
📋      → Installing Docker Engine from official repository
📋      → Docker version: 24.0.7
📋   [Step 2/4] Installing Docker Compose...
🔍      → Ansible working directory: ./build/my-env/ansible
🔍      → Executing playbook: ansible-playbook install-docker-compose.yml -i inventory.ini
📋      → Installing Docker Compose plugin
📋      → Compose version: 2.23.3
📋   [Step 3/4] Configuring automatic security updates...
🔍      → Ansible working directory: ./build/my-env/ansible
🔍      → Executing playbook: ansible-playbook configure-security-updates.yml -i inventory.ini
📋      → Configuring unattended-upgrades for automatic security patches
📋      → Update configuration status: enabled
📋   [Step 4/4] Configuring firewall (UFW)...
🔍      → Ansible working directory: ./build/my-env/ansible
🔍      → Executing playbook: ansible-playbook configure-firewall.yml -e @variables.yml -i inventory.ini
📋      → Configuring UFW with restrictive default policies
📋      → Allowing SSH access before enabling firewall
📋      → Firewall status: active
⏳   ✓ Infrastructure configured (took 36.0s)
✅ Environment 'my-env' configured successfully
```

**When to use**: Deep debugging, understanding exactly what Ansible playbooks are executed, verifying working directories, or reporting issues with detailed context.

**Symbol Legend**:

- ⏳ = Major progress milestone (all levels)
- ✅ = Success message (all levels)
- 📋 = Detailed progress (Verbose `-v` and above)
- 🔍 = Technical details (Debug `-vvv` only)

### Combining with Other Flags

Verbosity works with all other flags:

```bash
# Debug output with file and stderr logging
torrust-tracker-deployer configure my-env -vvv --log-output file-and-stderr

# Verbose output with custom log file
torrust-tracker-deployer configure my-env -v --log-output file --log-file custom.log
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to configure

## Prerequisites

1. **Environment provisioned** - Must run `provision` first
2. **VM running** - Instance must be accessible via SSH
3. **Ansible installed** - Ansible CLI available in PATH
4. **SSH connectivity** - Network access to VM

## State Transition

```text
[Provisioned] --configure--> [Configured]
```

## What Happens

When you configure an environment:

1. **Validates prerequisites** - Checks environment state and connectivity
2. **Runs Ansible playbooks** - Executes configuration management tasks
3. **Installs Docker** - Sets up Docker Engine
4. **Installs Docker Compose** - Sets up Docker Compose plugin
5. **Configures user permissions** - Adds SSH user to docker group
6. **Verifies installation** - Tests Docker and Docker Compose availability
7. **Updates environment state** - Transitions to "Configured"

## Examples

### Basic configuration

```bash
# Configure the environment
torrust-tracker-deployer configure full-stack-docs

# Output:
# ⏳ [1/3] Validating environment...
# ⏳   ✓ Environment name validated: full-stack-docs (took 0ms)
# ⏳ [2/3] Creating command handler...
# ⏳   ✓ Done (took 0ms)
# ⏳ [3/3] Configuring infrastructure...
# ⏳   ✓ Infrastructure configured (took 38.2s)
# ✅ Environment 'full-stack-docs' configured successfully
```

### Configure multiple environments

```bash
# Development
torrust-tracker-deployer configure dev-local

# Staging
torrust-tracker-deployer configure staging

# Production
torrust-tracker-deployer configure production
```

### Full workflow from start

```bash
# Complete setup sequence
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer provision my-environment
torrust-tracker-deployer configure my-environment
torrust-tracker-deployer test my-environment
```

## Output

The configure command installs:

- **Docker Engine** - Latest stable version
- **Docker Compose** - Plugin version (v2.x)
- **User permissions** - SSH user added to docker group
- **Verification results** - Docker and Compose version info

Ansible logs are written to:

- `data/logs/ansible-<timestamp>.log`

## Next Steps

After configuration:

```bash
# Verify the infrastructure is ready
torrust-tracker-deployer test my-environment

# Expected output:
# ✓ All infrastructure tests passed
```

## Troubleshooting

### Environment not provisioned

**Problem**: Cannot configure an environment that hasn't been provisioned

**Solution**: Provision the environment first

```bash
# Check environment state
cat data/my-environment/state.json

# If state is "Created", provision first
torrust-tracker-deployer provision my-environment
```

### Ansible not found

**Problem**: Ansible CLI is not installed or not in PATH

**Solution**: Install Ansible

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install ansible

# Or use pip
pip install ansible

# Verify installation
ansible --version
```

### SSH connection failed

**Problem**: Cannot connect to VM via SSH

**Solution**: Verify VM is running and SSH is accessible

```bash
# Check VM status
lxc list

# Get VM IP
lxc list my-environment --format json | jq -r '.[0].state.network.eth0.addresses[0].address'

# Try manual SSH connection
ssh -i <private-key> torrust@<vm-ip>

# If cloud-init is still running, wait for it
lxc exec <instance-name> -- cloud-init status --wait
```

### Docker installation failed

**Problem**: Ansible playbook fails during Docker installation

**Solution**: Check Ansible logs and VM network connectivity

```bash
# Review Ansible logs
tail -f data/logs/ansible-*.log

# Manually check VM network
lxc exec <instance-name> -- ping -c 3 8.8.8.8

# Check VM DNS
lxc exec <instance-name> -- cat /etc/resolv.conf

# Retry configuration
torrust-tracker-deployer configure my-environment
```

### Permission denied after install

**Problem**: User cannot run Docker commands without sudo

**Solution**: The configure command should handle this, but if it fails:

```bash
# SSH into the VM
ssh -i <private-key> torrust@<vm-ip>

# Add user to docker group (done by playbook normally)
sudo usermod -aG docker $USER

# Log out and log back in for group changes to take effect
exit
ssh -i <private-key> torrust@<vm-ip>

# Verify
docker ps
```

## Common Use Cases

### Automated testing pipeline

```bash
#!/bin/bash
set -e

ENV_NAME="test-${BUILD_ID}"

# Setup
torrust-tracker-deployer create environment -f test.json
torrust-tracker-deployer provision ${ENV_NAME}
torrust-tracker-deployer configure ${ENV_NAME}

# Verify
torrust-tracker-deployer test ${ENV_NAME}

# Your tests here...

# Cleanup
torrust-tracker-deployer destroy ${ENV_NAME}
```

### Manual development setup

```bash
# Set up infrastructure
torrust-tracker-deployer create environment -f dev.json
torrust-tracker-deployer provision dev-local
torrust-tracker-deployer configure dev-local

# SSH into VM for manual work
ssh -i fixtures/testing_rsa torrust@$(lxc list --format json | jq -r '.[0].state.network.eth0.addresses[0].address')

# Inside VM: verify Docker
docker --version
docker compose version
```

### Reconfiguration

If you need to reconfigure without reprovisioning:

```bash
# Just run configure again (idempotent)
torrust-tracker-deployer configure my-environment

# Ansible playbooks are designed to be idempotent
# Safe to run multiple times
```

## Technical Details

### Ansible Playbooks

The configure command runs these playbooks in order:

1. **install-docker.yml** - Installs Docker Engine
   - Adds Docker GPG key
   - Adds Docker repository
   - Installs docker-ce, docker-ce-cli, containerd.io
   - Starts and enables Docker service

2. **install-docker-compose.yml** - Installs Docker Compose
   - Downloads Docker Compose plugin
   - Installs to `/usr/local/lib/docker/cli-plugins/docker-compose`
   - Sets executable permissions

3. **configure-docker-permissions.yml** - Sets up user permissions
   - Adds SSH user to docker group
   - Applies group changes

### Generated Files

Configuration generates:

- **Ansible inventory** - `build/<env>/ansible/inventory.yml`
- **Ansible logs** - `data/logs/ansible-<timestamp>.log`
- **Environment state** - Updated with "Configured" status

### Verification Steps

After configuration, the command verifies:

- Docker daemon is running
- Docker CLI is accessible
- Docker Compose is installed
- User has docker group permissions

## See Also

- [provision](provision.md) - Provision infrastructure (prerequisite)
- [test](test.md) - Verify configuration (next step)
- [destroy](destroy.md) - Clean up infrastructure
- [create](create.md) - Create environment
