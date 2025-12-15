# Manual E2E Testing Guide for MySQL Support

This guide provides step-by-step instructions for manually testing the complete MySQL deployment workflow.

## ⚠️ CRITICAL: Understanding File Locations

**There are TWO completely different JSON files with different purposes:**

1. **User Environment Configuration** (`envs/manual-test-mysql.json`):

   - **Purpose**: User-provided input configuration for creating environments
   - **Location**: `envs/` directory (user-managed)
   - **Format**: Environment creation schema (see `envs/environment-schema.json`)
   - **Contains**: Provider config, SSH credentials, tracker settings, database config
   - **Usage**: Passed to `create environment --env-file` command
   - **Version Control**: Gitignored (user-specific configurations)

2. **Internal Application State** (`data/manual-test-mysql/environment.json`):
   - **Purpose**: Internal serialization of Rust types representing current deployment state
   - **Location**: `data/` directory (application-managed, DO NOT EDIT)
   - **Format**: Rust struct serialization (Domain model: `Environment<State>`)
   - **Contains**: State machine data, runtime outputs, trace IDs, timestamps
   - **Usage**: Read-only inspection for debugging/verification
   - **Version Control**: Gitignored (runtime-generated state)

**NEVER confuse these two files!** The user creates configurations in `envs/`, and the application manages state in `data/`.

## Test Environment

- **Environment Name**: `manual-test-mysql`
- **Database**: MySQL 8.0
- **Provider**: LXD
- **User Configuration File**: `envs/manual-test-mysql.json`
- **Internal State Directory**: `data/manual-test-mysql/`

## Prerequisites

Before starting the test, ensure:

1. LXD is installed and configured
2. The `torrust-profile-manual-test-mysql` LXD profile exists (will be created automatically)
3. All dependencies are installed: `cargo run --bin dependency-installer install`
4. Pre-commit checks pass: `./scripts/pre-commit.sh`
5. The environment configuration exists: `envs/manual-test-mysql.json`

## Complete MySQL Deployment Workflow

### Step 1: Create Environment

Create the deployment environment from the MySQL configuration:

```bash
cargo run -- create environment --env-file envs/manual-test-mysql.json
```

**Expected Output**:

```text
✓ Environment 'manual-test-mysql' created successfully
```

**Verification**:

```bash
# Check internal state file was created by the application
ls -la data/manual-test-mysql/environment.json

# Inspect internal state (Rust struct serialization) - shows current deployment state
cat data/manual-test-mysql/environment.json | jq '.state.type'
# Expected: "Created" (note: capitalized, this is the Rust enum variant name)
```

**Note**: The state file in `data/` is the application's internal representation. Do NOT edit it manually.

### Step 2: Provision Infrastructure

Provision the LXD VM instance:

```bash
cargo run -- provision manual-test-mysql
```

**Expected Output**:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: manual-test-mysql (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Provisioning infrastructure...
  ✓ Infrastructure provisioned (took 28.4s)
✅ Environment 'manual-test-mysql' provisioned successfully
```

**Verification**:

```bash
# Check instance is running
lxc list | grep torrust-tracker-manual-test-mysql

# Check internal state transitioned to Provisioned
cat data/manual-test-mysql/environment.json | jq 'keys | .[]'
# Expected: "Provisioned" (this is the top-level key - Rust enum variant)

# Check instance IP was recorded in runtime outputs
cat data/manual-test-mysql/environment.json | jq '.Provisioned.context.runtime_outputs.instance_ip'
# Expected: "10.x.x.x" (actual IP assigned by LXD)
```

### Step 3: Configure Instance

Install Docker and configure the instance:

```bash
cargo run -- configure manual-test-mysql
```

**Expected Output**:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: manual-test-mysql (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
  ✓ Infrastructure configured (took 34.0s)
✅ Environment 'manual-test-mysql' configured successfully
```

**Verification**:

```bash
# Check internal state transitioned to Configured
cat data/manual-test-mysql/environment.json | jq 'keys | .[]'
# Expected: "Configured" (top-level key)

# Extract instance IP from internal state for SSH access
INSTANCE_IP=$(cat data/manual-test-mysql/environment.json | jq -r '.Configured.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP "docker --version"
# Expected: Docker version 20.x or higher

ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "docker ps"
# Expected: Empty list (no containers running yet)
```

### Step 4: Release Application

Deploy tracker configuration and Docker Compose files:

```bash
cargo run -- release manual-test-mysql
```

**Expected Output**:

```text
⏳ [1/2] Validating environment...
  ✓ Environment name validated: manual-test-mysql (took 0ms)
⏳ [2/2] Releasing application...
  ✓ Application released successfully (took 7.3s)
✅ Release command completed successfully for 'manual-test-mysql'
```

**Verification**:

```bash
# Check internal state transitioned to Released
cat data/manual-test-mysql/environment.json | jq 'keys | .[]'
# Expected: "Released" (top-level key)

# Extract instance IP from internal state for verification
INSTANCE_IP=$(cat data/manual-test-mysql/environment.json | jq -r '.Released.context.runtime_outputs.instance_ip')

# Verify tracker configuration was deployed to correct location
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/storage/tracker/etc/tracker.toml | grep -A 5 '\[core.database\]'"

# Expected output:
# [core.database]
# driver = "mysql"
# path = "mysql://tracker_user:tracker_password@mysql:3306/torrust_tracker"

# Verify .env file was deployed with standardized variable name
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/.env | grep DATABASE_DRIVER"

# Expected output:
# TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER='mysql'

# Verify MySQL credentials in .env
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/.env | grep MYSQL"

# Expected output:
# MYSQL_ROOT_PASSWORD='tracker_password_root'
# MYSQL_DATABASE='torrust_tracker'
# MYSQL_USER='tracker_user'
# MYSQL_PASSWORD='tracker_password'

# Verify docker-compose.yml references environment variables
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/docker-compose.yml | grep -A 10 'mysql:'"

# Should show environment variable references like ${MYSQL_ROOT_PASSWORD}
```

### Step 5: Run Services

Start the tracker and MySQL services:

```bash
cargo run -- run manual-test-mysql
```

**Expected Output**:

```text
⏳ [1/2] Validating environment...
  ✓ Environment name validated: manual-test-mysql (took 0ms)
⏳ [2/2] Running application services...
  ✓ Services started (took 19.1s)
✅ Run command completed for 'manual-test-mysql'
```

**Initial Verification**:

```bash
# Check internal state transitioned to Running
cat data/manual-test-mysql/environment.json | jq 'keys | .[]'
# Expected: "Running" (top-level key)

# Extract instance IP from internal state
INSTANCE_IP=$(cat data/manual-test-mysql/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')

# Wait for containers to fully start (30 seconds)
sleep 30

# Check both containers are running and healthy
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP "docker ps --format 'table {{.Names}}\t{{.Status}}'"
# Expected output:
# NAMES     STATUS
# tracker   Up X seconds (healthy)
# mysql     Up X seconds (healthy)
```

**Deep Verification - MySQL Database**:

```bash
# 1. Verify MySQL container is healthy
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker ps --filter 'name=mysql' --format '{{.Names}}\t{{.Status}}'"
# Expected: mysql    Up X seconds (healthy)

# 2. Connect to MySQL and verify database exists
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec mysql mysql -u tracker_user -p'tracker_password' -e 'SHOW DATABASES;'"
# Expected output:
# mysql: [Warning] Using a password on the command line interface can be insecure.
# Database
# information_schema
# performance_schema
# torrust_tracker

# 3. Check if tracker tables were created
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec mysql mysql -u tracker_user -p'tracker_password' torrust_tracker -e 'SHOW TABLES;'"
# Expected output:
# mysql: [Warning] Using a password on the command line interface can be insecure.
# Tables_in_torrust_tracker
# keys
# torrent_aggregate_metrics
# torrents
# whitelist

# 4. Verify MySQL is accessible from within Docker network
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec tracker ping -c 2 mysql"
# Expected output:
# PING mysql (172.18.0.2): 56 data bytes
# 64 bytes from 172.18.0.2: seq=0 ttl=64 time=0.052 ms
# 64 bytes from 172.18.0.2: seq=1 ttl=64 time=0.081 ms
# 2 packets transmitted, 2 packets received, 0% packet loss
```

**Deep Verification - Tracker Container**:

**IMPORTANT**: The docker-compose template includes `depends_on` with `condition: service_healthy` for the tracker service. This ensures the tracker container waits for MySQL to be healthy before starting, preventing "Connection refused" errors at startup.

```bash
# 1. Check tracker container status
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker ps --filter 'name=tracker' --format '{{.Names}}\t{{.Status}}'"
# Expected output:
# tracker   Up X minutes (healthy)

# 2. Check tracker startup logs for database connection
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker logs tracker 2>&1 | grep -i 'database\|mysql\|error' | head -20"
# IMPORTANT: With depends_on configured correctly, you should NOT see:
#   "ERROR r2d2: DriverError { Could not connect to address `mysql:3306': Connection refused"
# The tracker waits for MySQL healthcheck to pass before starting
# Look for:
#   - "database": { "driver": "mysql" }
#   - No connection refused errors
#   - Clean startup sequence

# 3. Check tracker configuration inside container
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec tracker cat /etc/torrust/tracker/tracker.toml | grep -A 5 '\[core.database\]'"
# Expected:
# [core.database]
# driver = "mysql"
# path = "mysql://tracker_user:tracker_password@mysql:3306/torrust_tracker"
```

**Deep Verification - HTTP API**:

```bash
# 1. Get initial stats from the API
INSTANCE_IP=$(cat data/manual-test-mysql/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
INITIAL_STATS=$(curl -s -H "Authorization: Bearer MyAccessToken" http://$INSTANCE_IP:1212/api/v1/stats)
echo "Initial stats: $INITIAL_STATS"
# Expected: JSON with torrents, seeders, leechers counts

# 2. Make an announce request to increment stats
# Note: The tracker is configured for reverse proxy mode, so we need to send the X-Forwarded-For header
# Making the request from outside the VM (from host) is more realistic and simulates a real client

# From the host machine (outside VM):
curl -H "X-Forwarded-For: 203.0.113.45" \
  "http://$INSTANCE_IP:7070/announce?info_hash=%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C&peer_id=-qB00000000000000001&port=17548&uploaded=0&downloaded=0&left=0&event=started"
# Expected: HTTP 200 OK with tracker response (compact or full format)

# Alternative: Make request from inside VM (less realistic, but useful for testing)
# ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
#   "curl -s 'http://localhost:7070/announce?info_hash=%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C&peer_id=-qB00000000000000001&port=17548&uploaded=0&downloaded=0&left=0&event=started'"
# Note: This will fail with "Error resolving peer IP: missing or invalid the right most X-Forwarded-For IP"

# 3. Get updated stats and compare
UPDATED_STATS=$(curl -s -H "Authorization: Bearer MyAccessToken" http://$INSTANCE_IP:1212/api/v1/stats)
echo "Updated stats: $UPDATED_STATS"
# Expected: Counters incremented (e.g., tcp4_announces_handled, tcp4_connections_handled)
# Example output showing successful announce:
# {
#   "torrents": 1,
#   "seeders": 1,
#   "tcp4_announces_handled": 1,
#   "tcp4_connections_handled": 1,
#   ...
# }

# 4. Verify MySQL connection by checking database tables are accessible
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec mysql mysql -u tracker_user -p'tracker_password' torrust_tracker -e 'SHOW TABLES;'"
# Expected: List of tables (keys, torrent_aggregate_metrics, torrents, whitelist)
# This confirms MySQL connectivity is working

# Note: The tracker uses in-memory storage for active torrents by default.
# Torrents are only persisted to MySQL in specific cases:
# - In private mode: all torrents are persisted
# - In public mode: only whitelisted torrents are persisted
# The key verification is that MySQL is accessible and stats counters increase after announces.
```

### Step 6: Cleanup

After completing all verification steps above, destroy the test environment:

```bash
cargo run -- destroy manual-test-mysql
```

**Expected Output**:

```text
⏳ [1/3] Validating environment...
  ✓ Environment name validated: manual-test-mysql (took 0ms)
⏳ [2/3] Creating command handler...
  ✓ Done (took 0ms)
⏳ [3/3] Tearing down infrastructure...
  ✓ Infrastructure torn down (took 0ms)
✅ Environment 'manual-test-mysql' destroyed successfully
```

**Verification**:

```bash
# Verify LXD instance was destroyed
lxc list | grep torrust-tracker-manual-test-mysql
# Expected: No output (instance removed)

# Check internal state transitioned to Destroyed
cat data/manual-test-mysql/environment.json | jq 'keys | .[]'
# Expected: "Destroyed" (top-level key)
```

## Key Verification Points

### MySQL Configuration in Templates

**1. Tracker Config (`tracker.toml`)**:

- Driver should be `"mysql"`
- Path should be MySQL connection string format: `mysql://user:pass@host:port/database`

**2. Docker Compose `.env` file**:

- Should contain `MYSQL_ROOT_PASSWORD`, `MYSQL_DATABASE`, `MYSQL_USER`, `MYSQL_PASSWORD`
- Values should match environment configuration

**3. Docker Compose `docker-compose.yml`**:

- MySQL service should use `${MYSQL_*}` environment variable references
- NOT hardcoded values

### Runtime Verification

**1. Containers**:

- Both `torrust-tracker` and `mysql` containers should be running
- MySQL container should show `(healthy)` status

**2. Database**:

- MySQL database tables should be created
- Tracker should be able to read/write to database

**3. API**:

- Tracker API should respond on port 1212
- Stats endpoint should return valid JSON

## Troubleshooting

### MySQL container not healthy

```bash
# Check MySQL container logs
INSTANCE_IP=$(cat data/manual-test-mysql/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP "docker logs mysql"

# Verify MySQL service status
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker ps --filter 'name=mysql' --format '{{.Names}}\t{{.Status}}'"
```

### Tracker shows database connection errors

**Note**: You may see errors like "unable to open database file: mysql://..." in the tracker logs. This is a known issue being investigated. The tracker may still function correctly despite these errors.

```bash
# Check tracker logs
INSTANCE_IP=$(cat data/manual-test-mysql/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker logs tracker 2>&1 | tail -50"

# Verify tracker configuration inside container
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec tracker cat /etc/torrust/tracker/tracker.toml | grep -A 3 'database'"

# Check if MySQL database is accessible
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec mysql mysql -u tracker_user -p'tracker_password' torrust_tracker -e 'SELECT 1;'"
```

### Environment variables not applied

```bash
# Verify .env file exists and has MySQL variables
INSTANCE_IP=$(cat data/manual-test-mysql/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP "cat /opt/torrust/.env"

# Check docker-compose.yml references variables correctly
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/docker-compose.yml | grep -A 15 'mysql:'"
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
grep 'Environment state transition' data/logs/log.txt | grep manual-test-mysql
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
environment=manual-test-mysql
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

## Success Criteria

The MySQL implementation is successful if:

1. ✅ All commands complete without errors
2. ✅ Tracker config contains MySQL connection string (not SQLite path)
3. ✅ `.env` file contains all MySQL credentials AND standardized `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER` variable
4. ✅ `docker-compose.yml` uses `${MYSQL_*}` and `${TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER}` references (not hardcoded)
5. ✅ Both containers (tracker + MySQL) start and run healthy
6. ✅ Tracker API responds with valid JSON
7. ✅ MySQL database tables are created
8. ✅ No connection errors in tracker logs
9. ✅ Application logs in `data/logs/log.txt` show successful state transitions

## Comparison with SQLite

For comparison, test the same workflow with SQLite configuration:

```bash
# Use existing SQLite config
cargo run -- create environment --env-file data/e2e-deployment/environment.json
cargo run -- provision e2e-deployment
cargo run -- configure e2e-deployment
cargo run -- release e2e-deployment
cargo run -- run e2e-deployment
cargo run -- test e2e-deployment
cargo run -- destroy e2e-deployment
```

**Key Differences**:

- SQLite: `driver = "sqlite3"`, `path = "/var/lib/torrust/tracker/database/sqlite3.db"`
- MySQL: `driver = "mysql"`, `path = "mysql://user:pass@host:port/database"`
- SQLite: No MySQL service in docker-compose
- MySQL: MySQL service with healthcheck in docker-compose

## Related Documentation

- [Environment Configuration](../user-guide/configuration/environment.md)
- [Release Command](../user-guide/commands/release.md)
- [Run Command](../user-guide/commands/run.md)
- [ADR: Database Configuration Structure in Templates](../decisions/database-configuration-structure-in-templates.md)
- [ADR: Environment Variable Injection in Docker Compose](../decisions/environment-variable-injection-in-docker-compose.md)
