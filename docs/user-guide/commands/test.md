# `test` - Verify Infrastructure Configuration

Verify that provisioned and configured infrastructure meets all requirements.

## Purpose

Validates infrastructure readiness by running comprehensive tests against the configured environment. This command confirms the environment is in a working state and ready for deployment.

## Command Syntax

```bash
torrust-tracker-deployer test <ENVIRONMENT>
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to test

## Prerequisites

1. **Environment configured** - Must run `configure` first
2. **VM running** - Instance must be accessible
3. **Docker installed** - Docker and Docker Compose must be available
4. **SSH connectivity** - Network access to VM

## State Validation

Tests verify the environment is in "Configured" state and all components work correctly.

## What Happens

When you test an environment:

1. **Validates environment state** - Ensures environment is configured
2. **Checks VM connectivity** - Verifies SSH access
3. **Tests Docker installation** - Validates Docker daemon
4. **Tests Docker Compose** - Validates Docker Compose plugin
5. **Verifies user permissions** - Confirms non-root Docker access
6. **Runs infrastructure tests** - Executes comprehensive test suite
7. **Reports results** - Provides detailed pass/fail status

## Examples

### Basic testing

```bash
# Test the environment
torrust-tracker-deployer test full-stack-docs

# Output:
# ⏳ [1/3] Validating environment...
# ⏳   ✓ Environment name validated: full-stack-docs (took 0ms)
# ⏳ [2/3] Creating command handler...
# ⏳   ✓ Done (took 0ms)
# ⏳ [3/3] Testing infrastructure...
# ❌ Test command failed: Validation failed for environment 'full-stack-docs': Remote action failed: Action 'running-services-validation' validation failed: HTTPS request to 'https://api.example.com/api/health_check' failed: error sending request for url (https://api.example.com/api/health_check). Check that Caddy is running and port 443 is open. Domain 'api.example.com' was resolved to 10.140.190.211 for testing.
# Tip: Check logs and try running with --log-output file-and-stderr for more details
#
# For detailed troubleshooting:
# Validation Failed - Detailed Troubleshooting:
#
# 1. Check validation logs for specific failure:
#    - Re-run with verbose logging:
#      torrust-tracker-deployer test <environment-name> --log-output file-and-stderr
#
# 2. Common validation failures:
#    - Cloud-init not completed: Wait for instance initialization
#    - Docker not installed: Run configure command
#    - Docker Compose not installed: Run configure command
#
# 3. Remediation steps:
#    - If cloud-init failed: Destroy and re-provision
#    - If Docker/Compose missing: Run configure command
#      torrust-tracker-deployer configure <environment-name>
#
# 4. Check instance status:
#    - Verify instance is running
#    - Check SSH connectivity
#    - Review system logs on the instance
```

### Complete workflow

```bash
# Full setup and verification
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer provision my-environment
torrust-tracker-deployer configure my-environment
torrust-tracker-deployer test my-environment

# If all tests pass, environment is ready
```

### CI/CD verification

```bash
#!/bin/bash
set -e

ENV_NAME="ci-${BUILD_ID}"

# Setup
torrust-tracker-deployer create environment -f ci.json
torrust-tracker-deployer provision ${ENV_NAME}
torrust-tracker-deployer configure ${ENV_NAME}

# Verify infrastructure readiness
if torrust-tracker-deployer test ${ENV_NAME}; then
    echo "Infrastructure ready for deployment"
    # Deploy your application...
else
    echo "Infrastructure tests failed"
    exit 1
fi

# Cleanup
torrust-tracker-deployer destroy ${ENV_NAME}
```

## Test Categories

### Connectivity Tests

- **SSH Access** - Verifies SSH connection to VM
- **Network Reachability** - Confirms VM can reach internet
- **DNS Resolution** - Validates DNS configuration

### Docker Tests

- **Docker Daemon** - Checks Docker service is running
- **Docker CLI** - Verifies Docker commands work
- **Docker Info** - Validates Docker system information
- **Docker Version** - Confirms Docker version

### Docker Compose Tests

- **Compose Plugin** - Verifies Docker Compose is installed
- **Compose Version** - Confirms Docker Compose version
- **Compose Functionality** - Tests basic Compose operations

### Permission Tests

- **Non-root Access** - Confirms user can run Docker without sudo
- **Docker Group** - Verifies user is in docker group
- **Socket Access** - Validates docker socket permissions

## Output

The test command provides:

- **Test results** - Pass/fail status for each test
- **Detailed logs** - Information about any failures
- **Exit code** - 0 for success, non-zero for failure

Test logs are written to:

- `data/logs/test-<timestamp>.log`

## Next Steps

After testing:

```bash
# If tests pass, environment is ready
# Deploy your application

# SSH into the environment
ssh -i <private-key> torrust@<vm-ip>

# Or proceed with application deployment
# (application deployment features coming in future releases)
```

## Troubleshooting

### Environment not configured

**Problem**: Cannot test an environment that hasn't been configured

**Solution**: Configure the environment first

```bash
# Check environment state
cat data/my-environment/state.json

# If state is "Provisioned", configure first
torrust-tracker-deployer configure my-environment
```

### SSH connectivity test fails

**Problem**: Cannot establish SSH connection to VM

**Solution**: Check VM status and network

```bash
# Verify VM is running
lxc list

# Get VM IP
lxc list --format json | jq -r '.[].state.network.eth0.addresses[0].address'

# Try manual SSH
ssh -i <private-key> torrust@<vm-ip>

# Check SSH key permissions
ls -l <private-key>
chmod 600 <private-key>
```

### Docker daemon test fails

**Problem**: Docker service is not running

**Solution**: Check Docker service status on VM

```bash
# SSH into VM
ssh -i <private-key> torrust@<vm-ip>

# Check Docker status
sudo systemctl status docker

# Start Docker if needed
sudo systemctl start docker

# Enable Docker to start on boot
sudo systemctl enable docker
```

### Docker Compose test fails

**Problem**: Docker Compose is not installed or not in PATH

**Solution**: Verify Docker Compose installation

```bash
# SSH into VM
ssh -i <private-key> torrust@<vm-ip>

# Check Docker Compose
docker compose version

# If not found, reconfigure
exit
torrust-tracker-deployer configure my-environment
```

### Permission test fails

**Problem**: User cannot run Docker without sudo

**Solution**: Verify user is in docker group

```bash
# SSH into VM
ssh -i <private-key> torrust@<vm-ip>

# Check groups
groups

# If docker group missing, add it
sudo usermod -aG docker $USER

# Log out and back in
exit
ssh -i <private-key> torrust@<vm-ip>

# Verify
docker ps
```

### Network connectivity test fails

**Problem**: VM cannot reach external networks

**Solution**: Check LXD network configuration

```bash
# Check LXD network
lxc network list

# Check instance network config
lxc config device show <instance-name>

# Check VM network inside
lxc exec <instance-name> -- ip addr
lxc exec <instance-name> -- ping -c 3 8.8.8.8
```

## Common Use Cases

### Automated validation

```bash
#!/bin/bash
set -e

# Setup infrastructure
./scripts/setup-environment.sh my-environment

# Run comprehensive tests
if torrust-tracker-deployer test my-environment; then
    echo "✓ Infrastructure validated successfully"

    # Deploy application
    ./scripts/deploy-app.sh my-environment
else
    echo "✗ Infrastructure validation failed"

    # Collect diagnostics
    ./scripts/collect-logs.sh my-environment

    exit 1
fi
```

### Development workflow

```bash
# After making infrastructure changes
torrust-tracker-deployer configure my-environment

# Verify changes didn't break anything
torrust-tracker-deployer test my-environment

# If tests pass, continue development
```

### Pre-deployment checklist

```bash
# Staging environment verification
torrust-tracker-deployer test staging

# Production environment verification
torrust-tracker-deployer test production

# Only deploy if both pass
```

## Technical Details

### Test Implementation

Tests are implemented using:

- **SSH Client** - Remote command execution
- **Assertions** - Validates expected outcomes
- **Error Handling** - Provides detailed failure messages

### Test Execution Flow

1. **State Validation** - Checks environment JSON state
2. **SSH Connection** - Establishes connection to VM
3. **Docker Checks** - Runs docker commands remotely
4. **Compose Checks** - Runs docker compose commands remotely
5. **Permission Checks** - Validates non-root access
6. **Result Aggregation** - Collects all test results
7. **Reporting** - Outputs pass/fail status

### Exit Codes

- **0** - All tests passed
- **1** - One or more tests failed
- **2** - Invalid environment state
- **3** - SSH connection failed

## See Also

- [configure](configure.md) - Configure infrastructure (prerequisite)
- [provision](provision.md) - Provision infrastructure
- [destroy](destroy.md) - Clean up infrastructure
- [create](create.md) - Create environment
