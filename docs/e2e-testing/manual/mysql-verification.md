# Manual MySQL Service Verification

This guide provides MySQL-specific verification steps for manual E2E testing. For the complete deployment workflow, see the [Manual E2E Testing Guide](README.md).

## Overview

This guide covers:

- MySQL container health and connectivity
- Database schema verification
- Tracker-to-MySQL connection validation
- MySQL-specific troubleshooting
- Performance comparison with SQLite

## Prerequisites

Complete the standard deployment workflow first (see [Manual E2E Testing Guide](README.md)):

1. ✅ Environment created with MySQL configuration
2. ✅ Infrastructure provisioned
3. ✅ Services configured
4. ✅ Software released
5. ✅ Services running

**Your environment configuration must include MySQL**:

```json
{
  "tracker": {
    "core": {
      "database": {
        "driver": "mysql",
        "database_name": "torrust_tracker"
      }
    }
  },
  "database": {
    "driver": "mysql",
    "host": "mysql",
    "port": 3306,
    "database_name": "torrust_tracker",
    "username": "tracker_user",
    "password": "tracker_password",
    "root_password": "root_password"
  }
}
```

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

## MySQL-Specific Verification

This section provides detailed MySQL verification steps that should be performed after completing the standard deployment workflow.

### 1. Get the VM IP Address

Extract the instance IP from the environment state:

```bash
export INSTANCE_IP=$(cat data/your-env/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
echo "VM IP: $INSTANCE_IP"
```

### 2. Verify MySQL Container Health

Check that the MySQL container is running and healthy:

```bash
# Check both containers are running
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker ps --format 'table {{.Names}}\t{{.Status}}'"
```

**Expected output:**

```text
NAMES     STATUS
tracker   Up X seconds (healthy)
mysql     Up X seconds (healthy)
```

**Key verification points:**

- ✅ MySQL container status shows `(healthy)`
- ✅ Tracker container also shows `(healthy)` indicating it connected to MySQL successfully
- ✅ Both containers have been up for some time (not restarting)

### 3. Verify MySQL Database and Schema

Check that the database was created and tables exist:

```bash
# List all databases
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec mysql mysql -u tracker_user -p'tracker_password' -e 'SHOW DATABASES;'"
```

**Expected databases:**

```text
Database
information_schema
performance_schema
torrust_tracker        ← Your tracker database
```

**Check tracker tables:**

```bash
# List tables in tracker database
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec mysql mysql -u tracker_user -p'tracker_password' torrust_tracker -e 'SHOW TABLES;'"
```

**Expected tables:**

```text
Tables_in_torrust_tracker
keys
torrent_aggregate_metrics
torrents
whitelist
```

### 4. Verify Docker-to-MySQL Network Connectivity

Test that the tracker container can reach MySQL over the Docker network:

```bash
# Ping MySQL from tracker container
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec tracker ping -c 2 mysql"
```

**Expected output:**

```text
PING mysql (172.18.0.2): 56 data bytes
64 bytes from 172.18.0.2: seq=0 ttl=64 time=0.052 ms
64 bytes from 172.18.0.2: seq=1 ttl=64 time=0.081 ms
2 packets transmitted, 2 packets received, 0% packet loss
```

### 5. Verify Tracker Configuration

Check that the tracker is configured to use MySQL:

```bash
# Check tracker configuration
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec tracker cat /etc/torrust/tracker/tracker.toml | grep -A 5 '\[core.database\]'"
```

**Expected output:**

```toml
[core.database]
driver = "mysql"
path = "mysql://tracker_user:tracker_password@mysql:3306/torrust_tracker"
```

**Key verification points:**

- ✅ `driver = "mysql"` (not "sqlite3")
- ✅ Connection string uses MySQL format
- ✅ Hostname is `mysql` (Docker service name)
- ✅ Port is `3306` (MySQL default)
- ✅ Database name matches configuration

### 6. Verify Tracker Startup (No Connection Errors)

**IMPORTANT**: The docker-compose template includes `depends_on` with `condition: service_healthy` for the tracker service. This ensures the tracker waits for MySQL to be healthy before starting.

Check tracker logs for clean startup:

```bash
# Check for database connection errors
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker logs tracker 2>&1 | grep -i 'database\|mysql\|error' | head -20"
```

**What to look for:**

- ✅ **GOOD**: Clean startup with no "Connection refused" errors
- ✅ **GOOD**: Configuration shows `"driver": "mysql"`
- ❌ **BAD**: "Could not connect to address `mysql:3306': Connection refused"

**Note**: With proper `depends_on` configuration, you should NOT see connection refused errors. The tracker waits for MySQL's healthcheck to pass before starting.

### 7. Verify Environment Variables

Check that MySQL credentials are properly configured in the environment file:

```bash
# Check .env file contains MySQL variables
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/.env | grep MYSQL"
```

**Expected variables:**

```env
MYSQL_ROOT_PASSWORD=root_password
MYSQL_DATABASE=torrust_tracker
MYSQL_USER=tracker_user
MYSQL_PASSWORD=tracker_password
```

**Verify docker-compose.yml references:**

```bash
# Check docker-compose.yml uses environment variables
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/docker-compose.yml | grep -A 10 'mysql:'"
```

Should show environment variable references like `${MYSQL_ROOT_PASSWORD}`, not hardcoded values.

### 8. Test Tracker Functionality with MySQL

Make an announce request and verify stats are collected:

```bash
# Get initial stats
INITIAL_STATS=$(curl -s -H "Authorization: Bearer MyAccessToken" \
  http://$INSTANCE_IP:1212/api/v1/stats)
echo "Initial stats: $INITIAL_STATS"

# Make an announce request (from outside VM - more realistic)
curl -H "X-Forwarded-For: 203.0.113.45" \
  "http://$INSTANCE_IP:7070/announce?info_hash=%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C%3C&peer_id=-qB00000000000000001&port=17548&uploaded=0&downloaded=0&left=0&event=started"

# Get updated stats
UPDATED_STATS=$(curl -s -H "Authorization: Bearer MyAccessToken" \
  http://$INSTANCE_IP:1212/api/v1/stats)
echo "Updated stats: $UPDATED_STATS"
```

**Expected behavior:**

- ✅ Announce request returns HTTP 200 with tracker response
- ✅ Stats counters increment (e.g., `tcp4_announces_handled`, `tcp4_connections_handled`)
- ✅ MySQL connection remains stable (no errors in tracker logs)

**Note**: The tracker uses in-memory storage for active torrents by default. Torrents are only persisted to MySQL in specific cases:

- In private mode: all torrents are persisted
- In public mode: only whitelisted torrents are persisted

The key verification is that MySQL is accessible and the tracker functions correctly.

## MySQL-Specific Troubleshooting

### Common Verification Commands

```bash
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

## Step 4: Release Application

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

## MySQL-Specific Troubleshooting

This section covers common MySQL-specific issues. For general troubleshooting, see the [Manual E2E Testing Guide](README.md#troubleshooting-manual-tests).

### MySQL Container Not Healthy

If the MySQL container fails to start or shows unhealthy status:

```bash
# Check MySQL container logs for errors
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker logs mysql 2>&1 | tail -50"

# Check container status
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker ps --filter 'name=mysql' --format '{{.Names}}\t{{.Status}}'"
```

**Common issues:**

- **Port 3306 already in use**: Another MySQL instance running on host
- **Permission denied**: Volume mount permissions incorrect
- **Initialization failed**: Database name or credentials invalid

### Tracker Connection Refused Errors

If you see "Connection refused" errors when tracker tries to connect to MySQL:

```bash
# Check if MySQL healthcheck is properly configured
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/docker-compose.yml | grep -A 10 'mysql:' | grep healthcheck -A 5"
```

**Expected healthcheck configuration:**

```yaml
healthcheck:
  test:
    [
      "CMD",
      "mysqladmin",
      "ping",
      "-h",
      "localhost",
      "-u",
      "root",
      "-p$$MYSQL_ROOT_PASSWORD",
    ]
  interval: 10s
  timeout: 5s
  retries: 5
```

**Verify tracker depends_on configuration:**

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/docker-compose.yml | grep -A 5 'tracker:' | grep depends_on -A 3"
```

**Expected tracker depends_on:**

```yaml
depends_on:
  mysql:
    condition: service_healthy
```

### Database Connection Errors in Tracker Logs

**Note**: You may see errors like "unable to open database file: mysql://..." in the tracker logs. This is a known issue being investigated. The tracker may still function correctly despite these errors.

Check tracker logs for MySQL connection issues:

```bash
# Filter tracker logs for database/MySQL errors
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker logs tracker 2>&1 | grep -i 'database\|mysql\|r2d2\|connection' | tail -30"
```

**Verify tracker can connect to MySQL:**

```bash
# Test MySQL connection from inside tracker container
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec tracker sh -c 'nc -zv mysql 3306'"

# Expected output:
# mysql (172.18.0.2:3306) open
```

### Environment Variables Not Applied

If MySQL credentials don't match configuration:

```bash
# Check .env file contains correct MySQL variables
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/.env | grep MYSQL"

# Verify docker-compose.yml references variables (not hardcoded values)
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "cat /opt/torrust/docker-compose.yml | grep -A 15 'mysql:' | grep environment -A 5"
```

**Expected in docker-compose.yml:**

```yaml
environment:
  - MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}
  - MYSQL_DATABASE=${MYSQL_DATABASE}
  - MYSQL_USER=${MYSQL_USER}
  - MYSQL_PASSWORD=${MYSQL_PASSWORD}
```

**NOT like this (hardcoded):**

```yaml
environment:
  - MYSQL_ROOT_PASSWORD=hardcoded_password # ❌ WRONG
```

### Tables Not Created

If tracker tables don't exist in MySQL:

```bash
# Check if tracker has created tables
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@$INSTANCE_IP \
  "docker exec mysql mysql -u tracker_user -p'tracker_password' torrust_tracker -e 'SHOW TABLES;'"
```

**If tables are missing:**

1. Check tracker logs for migration errors
2. Verify tracker has correct database permissions
3. Ensure tracker started successfully after MySQL was healthy

## Comparison: MySQL vs SQLite

### Performance Characteristics

**SQLite:**

- ✅ Simpler setup (no separate container)
- ✅ Faster for small-scale deployments
- ✅ Lower memory footprint
- ❌ Limited concurrency
- ❌ Single file - no network access

**MySQL:**

- ✅ Better concurrency for multiple clients
- ✅ Network accessible (can query from other services)
- ✅ Better for high-traffic deployments
- ✅ Advanced features (replication, clustering)
- ❌ More complex setup (requires container/service)
- ❌ Higher memory usage

### When to Use MySQL

Choose MySQL when:

- **High concurrency**: Multiple clients accessing tracker simultaneously
- **Network access**: Need to query database from external tools/services
- **Production deployments**: Long-term stable deployments with scaling needs
- **Replication needs**: Want database backup/replication features

Choose SQLite when:

- **Development/testing**: Quick local testing
- **Low traffic**: Personal or small-scale deployments
- **Simplicity**: Prefer simpler setup without database container
- **Single-instance**: No need for network database access

## Related Documentation

- [Manual E2E Testing Guide](README.md) - Complete deployment workflow
- [Prometheus Verification Guide](prometheus-verification.md) - Metrics collection verification
- [MySQL Configuration Schema](../../user-guide/README.md) - Configuration file format
- [Troubleshooting Guide](../README.md) - General troubleshooting tips
