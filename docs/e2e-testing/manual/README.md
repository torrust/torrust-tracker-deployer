# Manual E2E Testing Guide

This guide explains how to manually run a complete end-to-end test of the Torrust Tracker Deployer using CLI commands. This is useful for testing new features, debugging issues, or validating changes before running automated tests.

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Complete Manual Test Workflow](#complete-manual-test-workflow)
- [Service-Specific Verification](#service-specific-verification)
- [Handling Interrupted Commands](#handling-interrupted-commands)
- [State Recovery](#state-recovery)
- [Troubleshooting Manual Tests](#troubleshooting-manual-tests)
- [Cleanup Procedures](#cleanup-procedures)
- [Advanced Manual Testing](#advanced-manual-testing)

## Prerequisites

Before starting, ensure all dependencies are installed:

```bash
# Check dependencies
cargo run --bin dependency-installer check

# Install missing dependencies
cargo run --bin dependency-installer install
```

Required tools:

- **LXD** - For VM provisioning
- **OpenTofu** - Infrastructure as code
- **Ansible** - Configuration management
- **Docker** - For containerized tracker deployment

## Complete Manual Test Workflow

This section walks through a complete manual E2E test from start to finish.

### Step 1: Create Environment Configuration

Generate a template configuration file using the `create template` command:

```bash
# Generate template for LXD provider
cargo run -- create template --provider lxd envs/manual-test.json
```

**Expected output**:

```text
✓ Template generated: envs/manual-test.json
```

This creates a pre-filled template with the correct structure and default values. The template command ensures you always get the latest configuration format.

**Customize the generated template**:

```bash
# Edit the template to customize values
nano envs/manual-test.json
```

**Key fields to customize**:

- `environment.name` - Change to a unique name if needed (default: derived from filename)
- `ssh_credentials.private_key_path` - Use `fixtures/testing_rsa` for testing
- `ssh_credentials.public_key_path` - Use `fixtures/testing_rsa.pub` for testing
- `provider.profile_name` - Ensure it's unique (e.g., `torrust-profile-manual-test`)

**Example template structure** (for reference):

<details>
<summary>Click to expand example configuration</summary>

```json
{
  "environment": {
    "name": "manual-test",
    "instance_name": null
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-manual-test"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      },
      "private": false
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    }
  }
}
```

</details>

**Using MySQL Instead of SQLite**:

The default template uses SQLite (`driver: "sqlite3"`), which is suitable for testing and small deployments. To use MySQL instead, you need to provide additional database configuration fields:

<details>
<summary>Click to expand MySQL configuration example</summary>

```json
{
  "environment": {
    "name": "manual-test-mysql",
    "instance_name": null
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-manual-test-mysql"
  },
  "tracker": {
    "core": {
      "database": {
        "driver": "mysql",
        "host": "mysql",
        "port": 3306,
        "database_name": "torrust_tracker",
        "username": "tracker_user",
        "password": "tracker_password"
      },
      "private": false
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    }
  }
}
```

</details>

**Required MySQL Fields**:

When `driver` is set to `"mysql"`, you must provide:

- `host` - MySQL hostname (use `"mysql"` for Docker Compose service name)
- `port` - MySQL port (typically `3306`)
- `database_name` - Name of the database to create
- `username` - MySQL user for tracker connection
- `password` - Password for the MySQL user

These credentials are used to:

1. Configure the MySQL Docker container (via docker-compose.yml)
2. Configure the tracker to connect to MySQL
3. Initialize the database schema

> **💡 Tip**: Always use `create template` to generate configuration files. This ensures you get the latest schema and prevents issues with outdated examples in documentation.

### Step 2: Create Environment

Initialize the environment structure:

```bash
cargo run -- create environment --env-file envs/manual-test.json
```

**Expected Output**:

```text
⏳ [1/3] Loading configuration...
  ✓ Configuration loaded: manual-test (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Creating environment...
  ✓ Environment created: manual-test (took 1ms)
✅ Environment 'manual-test' created successfully

Environment Details:
1. Environment name: manual-test
2. Instance name: torrust-tracker-vm-manual-test
3. Data directory: ./data/manual-test
4. Build directory: ./build/manual-test
```

**What This Does**:

- Creates `data/manual-test/` directory
- Creates `build/manual-test/` directory
- Initializes environment state file
- Validates configuration

**Verify Success**:

```bash
# Check environment was created
ls -la data/manual-test/
cat data/manual-test/environment.json | grep -A 1 '"Created"'
```

### Step 3: Provision Infrastructure

Create the LXD VM and network infrastructure:

```bash
cargo run -- provision manual-test --log-output file-and-stderr
```

**Expected Output**:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: manual-test (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
  ✓ Infrastructure provisioned (took 70.6s)
✅ Environment 'manual-test' provisioned successfully
```

**Duration**: ~60-90 seconds

**What This Does**:

- Renders OpenTofu templates
- Initializes OpenTofu
- Creates LXD profile
- Creates LXD VM instance
- Waits for SSH connectivity
- Waits for cloud-init completion

**Verify Success**:

```bash
# Check VM is running
lxc list | grep manual-test

# Check environment state changed to Provisioned
cat data/manual-test/environment.json | grep -A 1 '"Provisioned"'

# Get the VM IP address
cat data/manual-test/environment.json | grep instance_ip
```

**Example Output**:

```text
"instance_ip": "10.140.190.215"
```

### Step 4: Configure Software

Install Docker and Docker Compose on the provisioned VM:

```bash
cargo run -- configure manual-test
```

**Expected Output**:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: manual-test (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
  ✓ Infrastructure configured (took 43.1s)
✅ Environment 'manual-test' configured successfully
```

**Duration**: ~40-60 seconds (installs Docker, Docker Compose, security updates, firewall configuration)

**What This Does**:

- Installs Docker Engine
- Installs Docker Compose plugin
- Adds SSH user to docker group
- Verifies installation

**Verify Success**:

```bash
# Check environment state changed to Configured
cat data/manual-test/environment.json | jq -r 'keys[0]'  # Should show "Configured"

# Verify Docker is installed
export INSTANCE_IP=$(cat data/manual-test/environment.json | jq -r '.Configured.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null torrust@$INSTANCE_IP "docker --version"

# Verify Docker Compose is installed
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null torrust@$INSTANCE_IP "docker compose version"
```

### Step 5: Release Tracker

Pull the Docker image and prepare for running:

```bash
cargo run -- release manual-test
```

**Expected Output**:

```text
⏳ [1/2] Validating environment...
  ✓ Environment name validated: manual-test (took 0ms)
⏳ [2/2] Releasing application...
  ✓ Application released successfully (took 7.1s)
✅ Release command completed successfully for 'manual-test'
```

**Duration**: ~7-10 seconds (depending on network speed for Docker image pull)

**What This Does**:

- Pulls tracker Docker image from registry
- Prepares Docker container configuration
- Sets up runtime environment

**Verify Success**:

```bash
# Check environment state changed to Released
cat data/manual-test/environment.json | jq -r 'keys[0]'  # Should show "Released"

# Check Docker images were pulled
export INSTANCE_IP=$(cat data/manual-test/environment.json | jq -r '.Released.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null torrust@$INSTANCE_IP "docker images | grep torrust/tracker"
```

### Step 6: Run Tracker

Start the tracker service:

```bash
cargo run -- run manual-test
```

**Expected Output**:

```text
⏳ [1/2] Validating environment...
  ✓ Environment name validated: manual-test (took 0ms)
⏳ [2/2] Running application services...
  ✓ Services started (took 10.3s)
✅ Run command completed for 'manual-test'
```

**Duration**: ~10-15 seconds

**What This Does**:

- Starts tracker Docker container
- Waits for health checks to pass
- Verifies tracker is accessible

**Verify Success**:

```bash
# Check environment state changed to Running
cat data/manual-test/environment.json | grep -A 1 '"Running"'

# Check Docker container is running
IP=$(cat data/manual-test/environment.json | grep instance_ip | cut -d'"' -f4)
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$IP \
  "docker ps | grep tracker"

# Test tracker HTTP API
curl http://$IP:7070/health_check | jq
```

**Expected Health Check Response**:

```json
{
  "status": "ok"
}
```

### Step 7: Test Tracker (Optional)

Verify the tracker is working correctly:

```bash
# Get the VM IP
export INSTANCE_IP=$(cat data/manual-test/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')

# Test HTTP tracker health endpoint
curl http://$INSTANCE_IP:7070/health_check

# Test HTTP API health endpoint
curl http://$INSTANCE_IP:1212/api/health_check

# Check container logs
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null torrust@$INSTANCE_IP \
  "docker logs tracker"
```

### Step 8: Clean Up

Destroy the environment and all resources:

```bash
cargo run -- destroy manual-test
```

**Expected Output**:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: manual-test (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Tearing down infrastructure...
  ✓ Infrastructure torn down (took 96ms)
✅ Environment 'manual-test' destroyed successfully
```

**Duration**: ~1-2 seconds

**What This Does**:

- Stops and removes Docker containers
- Destroys LXD VM instance
- Removes LXD profile
- Cleans up OpenTofu state
- Removes environment directories

**Verify Cleanup**:

```bash
# Check VM is gone
lxc list | grep manual-test

# Check profile is gone
lxc profile list | grep manual-test

# Check environment directories are gone
ls data/manual-test 2>/dev/null || echo "Cleaned up successfully"
```

## Service-Specific Verification

After deploying your environment, you may want to verify that specific services are working correctly. The following guides provide detailed verification steps for each supported service:

### Torrust Tracker

The tracker is the core service deployed by this tool. See the [Tracker Verification Guide](tracker-verification.md) for detailed steps to:

- Test HTTP tracker announce and scrape endpoints
- Test UDP tracker functionality (overview and tooling)
- Verify tracker REST API endpoints
- Check health endpoints
- Troubleshoot tracker-specific issues
- Monitor tracker logs and performance

### MySQL Database

If your deployment includes MySQL as the database backend, see the [MySQL Verification Guide](mysql-verification.md) for detailed steps to:

- Verify MySQL container health and connectivity
- Check database tables and schema
- Validate tracker-to-MySQL connectivity
- Troubleshoot MySQL-specific issues
- Compare MySQL behavior vs SQLite

### Prometheus Metrics Collection

If your deployment includes Prometheus for metrics collection (enabled by default), see the [Prometheus Verification Guide](prometheus-verification.md) for detailed steps to:

- Verify Prometheus container is running
- Check configuration file deployment
- Validate target scraping (both `/api/v1/stats` and `/api/v1/metrics`)
- Access Prometheus web UI
- Query collected metrics
- Troubleshoot Prometheus-specific issues

### Grafana Dashboards

If your deployment includes Grafana for metrics visualization, see the [Grafana Verification Guide](grafana-verification.md) for detailed steps to:

- Verify Grafana container health and connectivity
- Check dashboard and datasource provisioning
- Validate Prometheus datasource connection
- Test end-to-end data flow (Tracker → Prometheus → Grafana)
- Troubleshoot Grafana-specific issues

### Basic Tracker Verification

For quick basic tracker functionality checks without the detailed guide:

```bash
# Get the VM IP
export INSTANCE_IP=$(cat data/manual-test/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')

# Test HTTP tracker health endpoint
curl http://$INSTANCE_IP:7070/health_check

# Test HTTP API health endpoint
curl http://$INSTANCE_IP:1212/api/health_check

# Check tracker container logs
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null torrust@$INSTANCE_IP \
  "docker logs tracker"
```

## Handling Interrupted Commands

Commands can be interrupted (Ctrl+C) during execution, leaving the environment in an intermediate state.

### Identifying the Current State

Check the current environment state:

```bash
cat data/<env-name>/environment.json | head -n 3
```

**Possible States**:

- `Created` - Environment initialized, ready for provisioning
- `Provisioning` - Infrastructure creation in progress (INTERRUPTED)
- `Provisioned` - Infrastructure ready, waiting for configuration
- `Configuring` - Configuration in progress (INTERRUPTED)
- `Configured` - Configuration complete, ready for release
- `Releasing` - Release preparation in progress (INTERRUPTED)
- `Released` - Ready to run
- `Running` - Tracker is running
- `Destroying` - Cleanup in progress (INTERRUPTED)

### Recovering from Intermediate States

#### If Interrupted During Provisioning

#### Option 1: Destroy and Retry

```bash
# Clean up partial infrastructure
cargo run -- destroy <env-name>

# If destroy fails, manually clean up
lxc delete torrust-tracker-vm-<env-name> --force 2>/dev/null
lxc profile delete torrust-profile-<env-name> 2>/dev/null

# Remove state
rm -rf data/<env-name> build/<env-name>

# Start fresh
cargo run -- create environment --env-file envs/<env-name>.json
cargo run -- provision <env-name> --log-output file-and-stderr
```

#### Option 2: Manual State Reset

```bash
# Edit the environment state file
nano data/<env-name>/environment.json

# Change "Provisioning" to "Created"
# Save and retry provision
cargo run -- provision <env-name> --log-output file-and-stderr
```

#### If Interrupted During Configure/Release

```bash
# Check if VM is still running
lxc list | grep <env-name>

# If VM exists, manually reset state
nano data/<env-name>/environment.json
# Change state from "Configuring" to "Provisioned" (or appropriate previous state)

# Retry the command
cargo run -- configure <env-name>
```

#### If Interrupted During Destroy

```bash
# Complete manual cleanup
lxc delete torrust-tracker-vm-<env-name> --force 2>/dev/null
lxc profile delete torrust-profile-<env-name> 2>/dev/null
rm -rf data/<env-name> build/<env-name>
```

### Prevention: Don't Interrupt Commands

**Best Practice**: Let commands complete. If you must interrupt:

1. Note which command was interrupted
2. Check the state immediately: `cat data/<env-name>/environment.json`
3. Follow recovery procedures above
4. Use `--log-output file-and-stderr` to see detailed progress

## State Recovery

> **⚠️ WARNING: Manual State Editing Is Dangerous**
>
> Manually editing the state file in `data/<env-name>/environment.json` can cause **system inconsistencies** and **unpredictable behavior**. The application state may not match the actual infrastructure state, leading to:
>
> - Failed commands with cryptic errors
> - Resources not being properly cleaned up
> - Ansible playbooks running on inconsistent system state
> - Difficulty troubleshooting issues
>
> **Recommended Approach**: Destroy the environment and recreate it from scratch:
>
> ```bash
> # Stop the VM if running
> lxc stop torrust-tracker-vm-<env-name> --force
>
> # Destroy the environment
> cargo run -- destroy <env-name>
>
> # If destroy fails, manually clean up
> lxc delete torrust-tracker-vm-<env-name> --force 2>/dev/null
> lxc profile delete torrust-profile-<env-name> 2>/dev/null
> rm -rf data/<env-name> build/<env-name>
>
> # Start fresh
> cargo run -- create environment --env-file envs/<env-name>.json
> cargo run -- provision <env-name>
> # ... continue with configure, release, run
> ```
>
> **Only edit state manually as a last resort for testing or development purposes.**

### Checking Logs for Diagnosis

Before manually editing state or destroying the environment, always check the application logs to understand what actually happened:

```bash
# View recent logs for your environment
tail -100 data/logs/log.txt | grep -A 5 -B 5 "<env-name>"

# Check specific state transitions
tail -200 data/logs/log.txt | grep "<env-name>" | grep "transition"

# View complete workflow history
cat data/logs/log.txt | grep "<env-name>"
```

**Key information in logs**:

- **State transitions**: Shows actual state changes (e.g., `Provisioned → Configuring`)
- **Command completion**: Look for "took Xs" messages indicating successful completion
- **Timestamps**: Helps identify when commands were interrupted vs completed
- **Error details**: Full error messages with context

**Example log analysis**:

```text
# Command completed successfully:
2025-01-11T12:15:51.525383Z INFO Transition completed: Configuring → Configured (took 43.1s)

# Command was interrupted:
2025-01-11T12:21:27.352044Z INFO Transition started: Provisioned → Configuring
# (no completion message after this = interrupted)
```

### Understanding Environment States

The environment state machine follows this progression:

```text
Created → Provisioning → Provisioned → Configuring → Configured →
Releasing → Released → Running
                           ↓
                      Destroying
```

**Terminal States**:

- `Created` - Can provision
- `Provisioned` - Can configure or destroy
- `Configured` - Can release or destroy
- `Released` - Can run or destroy
- `Running` - Can stop or destroy
- `Destroyed` - Final state (environment removed)

**Intermediate States** (should not persist):

- `Provisioning`, `Configuring`, `Releasing`, `Destroying`

### When to Manually Edit State

**Safe to Edit**:

- Recovering from interrupted commands (intermediate states)
- Resetting to previous stable state after failure
- Testing state transitions

**Never Edit**:

- Runtime outputs (instance_ip, provision_method)
- User inputs (changing these requires destroy + recreate)
- Internal config paths

### Manual State Reset Procedure

```bash
# 1. Back up current state
cp data/<env-name>/environment.json data/<env-name>/environment.json.backup

# 2. Edit the state file
nano data/<env-name>/environment.json

# 3. Change the state (first line):
# From: "Provisioning": {
# To:   "Created": {
# Or:   "Provisioned": {

# 4. Save and verify
cat data/<env-name>/environment.json | head -n 3

# 5. Retry the command
cargo run -- provision <env-name>
```

## Troubleshooting Manual Tests

### Environment Already Exists

**Error**: `Environment 'manual-test' already exists`

**Cause**: Environment was not properly cleaned up from previous test

**Solution**:

```bash
# Try normal destroy first
cargo run -- destroy manual-test

# If that fails, manually clean up
rm -rf data/manual-test build/manual-test

# Clean up LXD resources if they exist
lxc delete torrust-tracker-vm-manual-test --force 2>/dev/null
lxc profile delete torrust-profile-manual-test 2>/dev/null

# Start fresh
cargo run -- create environment --env-file envs/manual-test.json
```

### LXD Profile Already Exists

**Error**: `Error inserting "torrust-profile-manual-test" into database: The profile already exists`

**Cause**: Previous test left LXD profile behind

**Solution**:

```bash
# Check profile exists
lxc profile list | grep manual-test

# Check if it's in use
lxc profile show torrust-profile-manual-test

# Delete profile
lxc profile delete torrust-profile-manual-test

# Retry provision
cargo run -- provision manual-test
```

### LXD Instance Already Exists

**Error**: VM creation fails with "instance already exists"

**Solution**:

```bash
# List instances
lxc list | grep manual-test

# Force delete the instance
lxc delete torrust-tracker-vm-manual-test --force

# Retry provision
cargo run -- provision manual-test
```

### SSH Connection Timeout

**Error**: `Failed to connect via SSH` or SSH hangs

**Solution**:

```bash
# Check VM is running
lxc list

# Check VM IP is reachable
IP=$(cat data/manual-test/environment.json | grep instance_ip | cut -d'"' -f4)
ping -c 3 $IP

# Check cloud-init completed
lxc exec torrust-tracker-vm-manual-test -- cloud-init status

# Check SSH is listening
lxc exec torrust-tracker-vm-manual-test -- systemctl status ssh

# Verify SSH key permissions
chmod 600 fixtures/testing_rsa
```

### Docker Not Accessible

**Error**: `docker: command not found` or permission denied

**Solution**:

```bash
# SSH into VM
IP=$(cat data/manual-test/environment.json | grep instance_ip | cut -d'"' -f4)
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$IP

# Check Docker is installed
docker --version

# Check Docker daemon is running
sudo systemctl status docker

# Check user is in docker group
groups | grep docker

# If not in docker group, re-run configure
exit
cargo run -- configure manual-test
```

### Invalid State Transition

**Error**: `Expected state 'provisioned', but found 'provisioning'`

**Cause**: Command was interrupted and left intermediate state

**Solution**: See [State Recovery](#state-recovery) section above

### Ports Already in Use

**Error**: Port binding errors in Docker logs

**Cause**: Another tracker instance is running

**Solution**:

```bash
# SSH into VM
IP=$(cat data/manual-test/environment.json | grep instance_ip | cut -d'"' -f4)
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$IP

# Check running containers
docker ps

# Stop conflicting container
docker stop tracker

# Remove container
docker rm tracker

# Exit and retry run
exit
cargo run -- run manual-test
```

## Debugging with Application Logs

If you encounter any issues during the workflow, the application maintains detailed logs that can help diagnose problems:

### Log Location

All application execution logs are stored in:

```bash
data/logs/log.txt
```

This file contains **all** operations performed by the deployer, including:

- Command execution traces with timestamps
- State transitions (Created → Provisioned → Configured → Released → Running)
- Ansible playbook executions with full command details
- Template rendering operations
- Error messages with context
- Step-by-step progress through each command

### Viewing Logs

**View recent operations**:

```bash
# Last 100 lines
tail -100 data/logs/log.txt

# Follow logs in real-time
tail -f data/logs/log.txt
```

**Search for specific command**:

```bash
# View all release command operations
grep -A5 -B5 'release' data/logs/log.txt

# View provision operations
grep -A5 -B5 'provision' data/logs/log.txt

# View Ansible playbook executions
grep 'ansible-playbook' data/logs/log.txt
```

**Check state transitions**:

```bash
# View all state transitions for your environment
grep 'Environment state transition' data/logs/log.txt | grep manual-test
```

**Find errors**:

```bash
# Search for ERROR level logs
grep 'ERROR' data/logs/log.txt

# Search for WARN level logs
grep 'WARN' data/logs/log.txt
```

### Example: Debugging Release Command

If the release command completes but files aren't on the VM:

```bash
# Check what actually happened during release
grep -A10 'release_command' data/logs/log.txt | tail -50

# Verify Ansible playbooks were executed
grep 'deploy-tracker-config\|deploy-compose-files' data/logs/log.txt

# Check for any Ansible errors
grep -A5 'Ansible playbook.*failed' data/logs/log.txt
```

### Log Format

Logs are structured with:

- **Timestamp**: ISO 8601 format (e.g., `2025-12-14T11:52:16.232160Z`)
- **Level**: INFO, WARN, ERROR
- **Span**: Command and step context (e.g., `release_command:deploy_tracker_config`)
- **Module**: Rust module path
- **Message**: Human-readable description
- **Fields**: Structured data (environment name, step name, status, etc.)

**Example log entry**:

```text
2025-12-14T11:52:21.495109Z  INFO release_command: torrust_tracker_deployer_lib::application::command_handlers::release::handler:
Tracker configuration deployed successfully command="release" step=Deploy Tracker Config to Remote command_type="release"
environment=manual-test
```

### Common Issues in Logs

1. **"Ansible playbook failed"**: Check the Ansible command that was executed and verify SSH connectivity
2. **"Template rendering failed"**: Check template syntax and context data
3. **"State persistence failed"**: Check file permissions in `data/` directory
4. **"Instance IP not found"**: Environment wasn't provisioned correctly

### Tips

- The log file grows with each command execution
- Consider searching for your environment name to filter relevant logs
- Timestamps help correlate logs with command execution times
- All Ansible playbook commands are logged with full paths and arguments

## Cleanup Procedures

### Application-Level Cleanup (Recommended)

Use the destroy command to clean up everything:

```bash
cargo run -- destroy <env-name>
```

This handles:

- Stopping Docker containers
- Destroying LXD VM
- Removing LXD profile
- Cleaning OpenTofu state
- Removing directories

### Manual LXD Cleanup (When Destroy Fails)

If `destroy` command fails or hangs:

```bash
# Step 1: List all resources
lxc list
lxc profile list

# Step 2: Force delete VM instance
lxc delete torrust-tracker-vm-<env-name> --force

# Step 3: Delete profile (only if no other VMs use it)
lxc profile delete torrust-profile-<env-name>

# Step 4: Clean up directories
rm -rf data/<env-name> build/<env-name>

# Step 5: Verify cleanup
lxc list | grep <env-name>
```

### Complete System Cleanup

Clean up all test environments:

```bash
# List all test VMs
lxc list | grep torrust-tracker-vm

# Delete all test VMs
for vm in $(lxc list -c n --format csv | grep torrust-tracker-vm); do
  lxc delete $vm --force
done

# List all test profiles
lxc profile list | grep torrust-profile

# Delete all test profiles
for profile in $(lxc profile list --format csv | cut -d',' -f1 | grep torrust-profile); do
  lxc profile delete $profile
done

# Clean up all environment data
rm -rf data/manual-test* data/*-e2e
rm -rf build/manual-test* build/*-e2e
```

### Emergency Cleanup Script

Save this as `scripts/emergency-cleanup.sh`:

```bash
#!/bin/bash
set -e

ENV_NAME=${1:-manual-test}

echo "🧹 Emergency cleanup for environment: $ENV_NAME"

echo "→ Stopping Docker containers..."
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no \
  torrust@$(cat data/$ENV_NAME/environment.json | grep instance_ip | cut -d'"' -f4) \
  "docker stop tracker 2>/dev/null || true" 2>/dev/null || true

echo "→ Deleting LXD VM..."
lxc delete torrust-tracker-vm-$ENV_NAME --force 2>/dev/null || true

echo "→ Deleting LXD profile..."
lxc profile delete torrust-profile-$ENV_NAME 2>/dev/null || true

echo "→ Removing directories..."
rm -rf data/$ENV_NAME build/$ENV_NAME

echo "✅ Emergency cleanup complete"
```

Usage:

```bash
chmod +x scripts/emergency-cleanup.sh
./scripts/emergency-cleanup.sh manual-test
```

## Advanced Manual Testing

### Testing Specific Commands

Test individual commands without full workflow:

```bash
# Test only provision (assumes environment exists)
cargo run -- provision manual-test

# Test only configure (assumes provisioned)
cargo run -- configure manual-test

# Test release (assumes configured)
cargo run -- release manual-test

# Test run (assumes released)
cargo run -- run manual-test
```

### Multiple Environment Testing

Run multiple environments simultaneously:

```bash
# Create three environments
for i in 1 2 3; do
  cat envs/manual-test.json | \
    sed "s/manual-test/manual-test-$i/g" > envs/manual-test-$i.json
  cargo run -- create environment --env-file envs/manual-test-$i.json
done

# Provision all (can run in parallel)
cargo run -- provision manual-test-1 &
cargo run -- provision manual-test-2 &
cargo run -- provision manual-test-3 &
wait

# Continue with configure, release, run...
for i in 1 2 3; do
  cargo run -- configure manual-test-$i
  cargo run -- release manual-test-$i
  cargo run -- run manual-test-$i
done
```

### Testing with Different Configurations

Test different tracker configurations:

```bash
# Create environment with MySQL instead of SQLite
cat > envs/manual-test-mysql.json <<EOF
{
  "environment": { "name": "manual-test-mysql" },
  "tracker": {
    "core": {
      "database": {
        "driver": "mysql",
        "host": "localhost",
        "port": 3306,
        "database_name": "tracker"
      }
    }
  }
}
EOF

# Run full workflow
cargo run -- create environment --env-file envs/manual-test-mysql.json
cargo run -- provision manual-test-mysql
cargo run -- configure manual-test-mysql
cargo run -- release manual-test-mysql
cargo run -- run manual-test-mysql
```

### Debugging with SSH Access

Keep the environment running and debug interactively:

```bash
# Get VM IP
IP=$(cat data/manual-test/environment.json | grep instance_ip | cut -d'"' -f4)

# SSH into VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$IP

# Inside VM: Check Docker
docker ps
docker logs tracker
docker exec -it tracker /bin/bash

# Inside container: Check tracker
cat /etc/torrust/tracker/tracker.toml
ls -la /var/lib/torrust/tracker/
tail -f /var/log/torrust/tracker/*

# Test tracker locally
curl http://localhost:7070/health_check
curl http://localhost:1212/api/health_check
```

### Performance Testing

Measure command execution times:

```bash
# Time each command
time cargo run -- create environment --env-file envs/manual-test.json
time cargo run -- provision manual-test
time cargo run -- configure manual-test
time cargo run -- release manual-test
time cargo run -- run manual-test
time cargo run -- destroy manual-test
```

## Summary

This guide covered:

✅ **Complete Workflow** - Step-by-step manual E2E testing
✅ **Error Recovery** - Handling interrupted commands and state issues
✅ **Troubleshooting** - Common problems and solutions
✅ **Cleanup** - Proper resource cleanup procedures
✅ **Advanced Testing** - Multiple environments and debugging techniques

**Key Takeaways**:

1. Always let commands complete (avoid Ctrl+C)
2. Check state after any failure: `cat data/<env>/environment.json`
3. Use `--log-output file-and-stderr` for detailed logging
4. Manual state reset is safe for intermediate states only
5. Use `destroy` first, manual cleanup as fallback

For automated E2E testing, see [running-tests.md](running-tests.md).
