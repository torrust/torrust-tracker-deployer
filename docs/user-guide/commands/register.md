# `register` - Register Existing Instance

Register an existing instance (VM, physical server, or container) with a deployment environment as an alternative to provisioning.

## Purpose

Allows users to deploy to pre-existing infrastructure that was not created by the deployer. This command takes an environment from the "Created" state to the "Provisioned" state by associating it with an already-running instance, enabling the full deployment workflow (`configure`, `release`, `run`) on existing infrastructure.

## Command Syntax

```bash
torrust-tracker-deployer register <ENVIRONMENT> --instance-ip <IP_ADDRESS>
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to register
- `--instance-ip <IP_ADDRESS>` (required) - IP address of the existing instance (IPv4 or IPv6)

## Prerequisites

1. **Environment created** - Must run `create environment` first with SSH credentials
2. **Existing instance** - A running instance with SSH access
3. **SSH key deployed** - The public key from the environment configuration must be installed on the instance
4. **Network reachability** - The instance must be reachable from the deployer machine

### Instance Requirements

The existing instance must meet these requirements (same as provisioned instances):

**Required**:

- Ubuntu 24.04 LTS
- SSH connectivity with credentials from `create environment`
- Public SSH key installed for access
- IP address reachable from deployer
- Username with sudo access

**Recommended** (for best compatibility):

- Cloud-init completion mark (`/var/lib/cloud/instance/boot-finished`)
- Fresh system without conflicting software

**Must NOT have**:

- Incompatible dependencies (old Docker, old systemd, etc.)
- Custom configurations preventing deployer operation

## State Transition

```text
[Created] --register--> [Provisioned]
```

This is the same target state as the `provision` command, but using existing infrastructure instead of creating new infrastructure.

## What Happens

When you register an environment:

1. **Loads environment** - Reads existing environment configuration
2. **Validates state** - Ensures environment is in "Created" state
3. **Validates SSH connectivity** - Connects to the instance using environment's SSH credentials
4. **Sets instance IP** - Records the provided IP address in runtime outputs
5. **Marks as registered** - Adds metadata flag to indicate this is registered (not provisioned) infrastructure
6. **Renders Ansible templates** - Generates configuration management files
7. **Updates environment state** - Transitions to "Provisioned"

## Examples

### Basic registration

```bash
# First, create the environment with SSH credentials
torrust-tracker-deployer create environment -f my-config.json

# Then, register the existing instance
torrust-tracker-deployer register my-environment --instance-ip 192.168.1.100

# Output:
# ✓ Loading environment...
# ✓ Validating SSH connectivity...
# ✓ Registering instance IP: 192.168.1.100
# ✓ Rendering Ansible templates...
# ✓ Environment registered successfully
```

### Register a cloud VM

```bash
# Create environment with cloud server SSH credentials
torrust-tracker-deployer create environment -f cloud-config.json

# Register your existing cloud VM
torrust-tracker-deployer register cloud-tracker --instance-ip 203.0.113.45

# Continue with deployment
torrust-tracker-deployer configure cloud-tracker
torrust-tracker-deployer test cloud-tracker
```

### Register a local Docker container

```bash
# Start a container with SSH access
docker run -d --name my-tracker -p 2222:22 my-ssh-container

# Create environment pointing to container
torrust-tracker-deployer create environment -f container-config.json

# Register the container (use host IP and mapped port)
torrust-tracker-deployer register container-test --instance-ip 127.0.0.1

# Note: SSH port should be configured in environment config
```

### Register an LXD VM created manually

```bash
# Create LXD VM manually
lxc launch ubuntu:24.04 my-manual-vm --vm
lxc exec my-manual-vm -- cloud-init status --wait

# Get VM IP
lxc list my-manual-vm -c4 --format csv | cut -d' ' -f1

# Create deployer environment
torrust-tracker-deployer create environment -f lxd-config.json

# Register the manually created VM
torrust-tracker-deployer register manual-deploy --instance-ip 10.140.190.42
```

## Workflow Comparison

### Provision Workflow (New Infrastructure)

```bash
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer provision my-env        # Creates new VM
torrust-tracker-deployer configure my-env
torrust-tracker-deployer test my-env
torrust-tracker-deployer destroy my-env          # Destroys VM
```

### Register Workflow (Existing Infrastructure)

```bash
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer register my-env --instance-ip 192.168.1.100  # Uses existing instance
torrust-tracker-deployer configure my-env
torrust-tracker-deployer test my-env
torrust-tracker-deployer destroy my-env          # Only removes environment data, preserves instance!
```

**Important**: When you destroy a registered environment, the underlying infrastructure is **preserved**. Only the deployer's environment data is removed. This is a key safety feature for registered instances.

## Output

The register command creates:

- **Runtime outputs** - Instance IP stored in environment state
- **Metadata flag** - `provision_method: Registered` to indicate external infrastructure
- **Ansible inventory** - Generated inventory in `build/<env>/ansible/`
- **Environment state update** - State file updated to "Provisioned"

## Next Steps

After registering:

```bash
# 1. Configure the infrastructure (install Docker, Docker Compose)
torrust-tracker-deployer configure my-environment

# 2. Verify infrastructure readiness
torrust-tracker-deployer test my-environment
```

## Troubleshooting

### Environment not found

**Problem**: Cannot find environment with the specified name

**Solution**: Verify the environment was created first

```bash
# Check environment data directory exists
ls -la data/my-environment/

# If not, create the environment first
torrust-tracker-deployer create environment -f config.json
```

### Environment not in Created state

**Problem**: Environment is already provisioned or in another state

**Solution**: The `register` command only works on environments in the "Created" state

```bash
# Check current environment state
cat data/my-environment/environment.json | grep state

# If already provisioned, you may need to destroy and recreate
torrust-tracker-deployer destroy my-environment
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer register my-environment --instance-ip <IP>
```

### SSH connection failed

**Problem**: Cannot establish SSH connection to the instance

**Solution**: Verify SSH configuration

```bash
# Test SSH manually with the same credentials
ssh -i <path-to-private-key> <username>@<instance-ip>

# Check if SSH service is running on the instance
# Check if the public key is in ~/.ssh/authorized_keys on the instance
# Verify the username matches the environment configuration
```

### Invalid IP address

**Problem**: The provided IP address format is invalid

**Solution**: Use a valid IPv4 or IPv6 address

```bash
# Valid IPv4 examples
--instance-ip 192.168.1.100
--instance-ip 10.0.0.1

# Valid IPv6 examples
--instance-ip 2001:db8::1
--instance-ip ::1
```

### Instance unreachable

**Problem**: Network cannot reach the instance

**Solution**: Check network connectivity

```bash
# Test network connectivity
ping <instance-ip>

# Check if firewall is blocking SSH (port 22)
nc -zv <instance-ip> 22

# Verify the instance is running
```

## Use Cases

### Deploy to spare servers

If you have existing servers that you want to use for deployment:

```bash
# Create environment with server SSH credentials
torrust-tracker-deployer create environment -f spare-server.json

# Register each server
torrust-tracker-deployer register tracker-1 --instance-ip 192.168.1.10
torrust-tracker-deployer register tracker-2 --instance-ip 192.168.1.11

# Configure and deploy
torrust-tracker-deployer configure tracker-1
torrust-tracker-deployer configure tracker-2
```

### Deploy with unsupported cloud provider

For cloud providers not yet supported by the deployer:

```bash
# 1. Manually create VM in your cloud provider (AWS, GCP, Azure, etc.)
# 2. Note the public IP address
# 3. Ensure SSH key is deployed

# Create deployer environment
torrust-tracker-deployer create environment -f cloud-config.json

# Register the cloud VM
torrust-tracker-deployer register cloud-deploy --instance-ip <cloud-vm-ip>

# Continue with standard deployment
torrust-tracker-deployer configure cloud-deploy
```

### E2E testing with containers

For fast testing using Docker containers instead of VMs:

```bash
# Start test container with SSH
docker run -d --name test-instance -p 2222:22 ubuntu-ssh:latest

# Create test environment
torrust-tracker-deployer create environment -f test-config.json

# Register the container
torrust-tracker-deployer register e2e-test --instance-ip 127.0.0.1

# Run configuration tests
torrust-tracker-deployer configure e2e-test
torrust-tracker-deployer test e2e-test

# Cleanup
torrust-tracker-deployer destroy e2e-test
docker rm -f test-instance
```

## Technical Details

### Provision Method Tracking

The environment state tracks how it was provisioned:

- `provision_method: Provisioned` - Infrastructure created by `provision` command (managed)
- `provision_method: Registered` - Infrastructure provided by user via `register` command (external)

This distinction affects the `destroy` command behavior:

- **Provisioned environments**: `destroy` removes the infrastructure (VMs, networks, etc.)
- **Registered environments**: `destroy` only removes deployer data, preserves the instance

### SSH Connectivity Validation

The `register` command validates SSH connectivity using:

- SSH credentials from the environment configuration (set during `create environment`)
- The provided instance IP address
- Connection timeout of 30 seconds

If validation fails, the command still completes (with a warning) to allow for:

- Temporary network issues
- Instance still booting
- Firewall rules pending

Subsequent commands (`configure`, `test`) will fail with clear errors if connectivity issues persist.

## See Also

- [create](create.md) - Create environment (prerequisite)
- [provision](provision.md) - Alternative: provision new infrastructure
- [configure](configure.md) - Configure infrastructure (next step)
- [test](test.md) - Verify infrastructure
- [destroy](destroy.md) - Clean up environment
