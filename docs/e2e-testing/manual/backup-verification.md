# Manual Backup Service Verification

This guide provides backup-specific verification steps for manual E2E testing. For the complete deployment workflow, see the [Manual E2E Testing Guide](README.md).

## Overview

This guide covers:

- Backup configuration in environment files
- Backup template rendering verification
- Backup storage directory structure
- Manual backup execution testing
- Database-specific backup verification (SQLite vs MySQL)
- Backup file inspection and validation

## Prerequisites

Complete the standard deployment workflow first (see [Manual E2E Testing Guide](README.md)):

1. ✅ Environment created
2. ✅ Infrastructure provisioned
3. ✅ Services configured
4. ✅ Software released
5. ✅ Services running

**Your environment configuration must include backup settings**:

```json
{
  "backup": {
    "schedule": "0 3 * * *",
    "retention_days": 7
  }
}
```

### Quick Verification: Check Backup Configuration Was Loaded

Before running the deployment workflow, verify backup configuration is present in the environment state:

```bash
# After 'create environment' command
export ENV_NAME="your-environment-name"
cat data/$ENV_NAME/environment.json | jq '.Created.context.user_inputs.backup'
```

**Expected output**:

```json
{
  "schedule": "0 3 * * *",
  "retention_days": 7
}
```

If this shows `null`, backup configuration was not loaded from the config file. Verify the backup section is correctly formatted in your environment JSON file.

**Current State**: The backup service is deployed with automatic scheduled execution via crontab. The initial backup runs during the `run` command, then additional backups run automatically on the configured schedule.

## Test Scenarios

Test both database drivers to ensure comprehensive coverage:

1. **SQLite + Backup** - Tests file-based database backup
2. **MySQL + Backup** - Tests network database backup with MySQL connection

## Verification Steps

### Step 1: Verify Backup Configuration Files Were Deployed

After running the `release` command, verify backup configuration files exist on the VM:

```bash
# Set environment name and get IP from environment state
export ENV_NAME="your-environment-name"
export INSTANCE_IP=$(cat data/$ENV_NAME/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')

# Verify backup configuration is in application state
echo "Checking backup configuration in application state:"
cat data/$ENV_NAME/environment.json | jq '.Running.context.user_inputs.backup'
echo ""

# Check backup storage directory exists
echo "Checking backup storage directory on VM:"
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "ls -la /opt/torrust/storage/backup/"
```

**Expected state check**:

```json
{
  "schedule": "0 3 * * *",
  "retention_days": 7
}
```

**Expected directory structure**:

```text
drwxr-xr-x  3 torrust torrust 4096 <date> .
drwxr-xr-x  6 torrust torrust 4096 <date> ..
drwxr-xr-x  2 torrust torrust 4096 <date> etc
```

### Step 2: Verify Backup Configuration File Content

```bash
# View backup.conf
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "cat /opt/torrust/storage/backup/etc/backup.conf"
```

**Expected variables for SQLite**:

```bash
BACKUP_RETENTION_DAYS=7
BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
DB_TYPE=sqlite
DB_PATH=/data/storage/tracker/lib/tracker.db
```

**Expected variables for MySQL**:

```bash
BACKUP_RETENTION_DAYS=7
BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
DB_TYPE=mysql
DB_HOST=mysql
DB_PORT=3306
DB_USER=tracker_user
DB_PASSWORD=<your_password>
DB_NAME=torrust_tracker
```

### Step 3: Verify Backup Paths Configuration

```bash
# View backup-paths.txt
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "cat /opt/torrust/storage/backup/etc/backup-paths.txt"
```

**Expected content**:

```text
/data/storage/tracker/etc
/data/storage/prometheus/etc
/data/storage/grafana/provisioning
```

### Step 4: Verify Docker Compose Service Configuration

```bash
# Check backup service definition
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "cat /opt/torrust/docker-compose.yml | grep -A 25 'backup:'"
```

**Expected for SQLite**:

- Service name: `backup`
- Image: `torrust/tracker-backup:latest`
- Restart policy: `"no"` (runs once and exits)
- Volumes: backup storage, tracker storage, prometheus storage, grafana storage
- **No networks** (SQLite doesn't need database network)

**Expected for MySQL**:

- Same as above, plus:
- Networks: `database_network`
- Depends on: `mysql` with health condition

### Step 5: Check Backup Service Status

```bash
# View services status
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "cd /opt/torrust && docker compose ps"
```

**Expected**: Backup service should show `State: Exited (0)` - this is **correct** behavior (runs once on startup and exits).

### Step 6: Execute Manual Backup

Test running a backup manually:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP

# Navigate to application directory
cd /opt/torrust

# Run backup manually
docker compose run --rm backup
```

**Expected output**:

```text
[2026-02-03 19:05:25] Torrust Backup Container starting
[2026-02-03 19:05:25] Loading configuration from: /etc/backup/backup.conf
[2026-02-03 19:05:25] Configuration:
[2026-02-03 19:05:25]   Retention: 7 days
[2026-02-03 19:05:25]   Database: sqlite
[2026-02-03 19:05:25]   Config paths file: /etc/backup/backup-paths.txt
[2026-02-03 19:05:25] ==========================================
[2026-02-03 19:05:25] Starting backup cycle
[2026-02-03 19:05:25] ==========================================
[2026-02-03 19:05:25] Starting SQLite backup: /data/storage/tracker/lib/database/tracker.db
[2026-02-03 19:05:25] SQLite backup completed: /backups/sqlite/sqlite_20260203_190525.db.gz
[2026-02-03 19:05:25]   Size: 4.0K
[2026-02-03 19:05:25] Starting config files backup
[2026-02-03 19:05:25] Config backup completed: /backups/config/config_20260203_190525.tar.gz
[2026-02-03 19:05:25]   Files backed up: 3
[2026-02-03 19:05:25]   Size: 8.0K
[2026-02-03 19:05:25] Cleaning up backups older than 7 days
[2026-02-03 19:05:25]   No old backups to delete
[2026-02-03 19:05:25] ==========================================
[2026-02-03 19:05:25] Backup cycle completed successfully
[2026-02-03 19:05:25] ==========================================
```

**For MySQL deployments, you may see this warning** (this is **expected and not fatal**):

```text
[2026-02-03 19:47:32] Starting MySQL backup: tracker@mysql:3306
mysqldump: Error: 'Access denied; you need (at least one of) the PROCESS privilege(s) for this operation' when trying to dump tablespaces
[2026-02-03 19:47:32] MySQL backup completed: /backups/mysql/mysql_20260203_194732.sql.gz
[2026-02-03 19:47:32]   Size: 4.0K
```

The warning appears because the backup user (`tracker_user`) has all necessary permissions for table backup, but lacks the PROCESS privilege for tablespace metadata. The backup still completes successfully with all table data intact.

### Step 7: Verify Backup Files Were Created

```bash
# Check SQLite database backup files
ls -lh /opt/torrust/storage/backup/sqlite/

# Check config backup files
ls -lh /opt/torrust/storage/backup/config/

# Exit SSH
exit
```

**Expected for SQLite**:

- Database file: `sqlite_<timestamp>.db.gz` (compressed SQLite database)
- Config archive: `config_<timestamp>.tar.gz`

**Expected for MySQL**:

- Database dump: `mysql_<timestamp>.sql.gz` (compressed SQL dump)
- Config archive: `config_<timestamp>.tar.gz`

### Step 8: Inspect Backup Files (SQLite)

For SQLite deployments, verify the database backup was created:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "cd /opt/torrust/storage/backup/sqlite && \
  gunzip -c sqlite_*.db.gz | file - && \
  cd /opt/torrust/storage/backup/config && \
  tar -tzf config_*.tar.gz | head -10"
```

**Expected**:

- SQLite backup file: `sqlite_<timestamp>.db.gz` (valid SQLite 3.x database)
- Config archive contains: tracker.toml, prometheus.yml, grafana provisioning files

### Step 9: Inspect Backup Files (MySQL)

For MySQL deployments, verify the SQL dump was created with valid content:

```bash
# List MySQL backup files
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "ls -lh /opt/torrust/storage/backup/mysql/ | grep '\.sql\.gz'"

# Verify SQL structure (decompress and inspect first lines)
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "zcat /opt/torrust/storage/backup/mysql/mysql_*.sql.gz | head -20"
```

**Expected output**:

File listing shows: `mysql_<timestamp>.sql.gz` with reasonable size (typically 0.5-2 KB for test database)

SQL content preview shows valid MySQL dump headers:

```text
/*M!999999\- enable the sandbox mode */
-- MariaDB dump 10.19-11.8.3-MariaDB, for debian-linux-gnu (x86_64)
--
-- Host: mysql    Database: tracker
-- Server version       8.4.8

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
```

This confirms:

- ✅ SQL dump is valid and compressed
- ✅ Contains MySQL 8.4 database structure
- ✅ Table definitions are included
- ✅ File is restorable using `mysql < backup.sql`

### Step 10: Verify Crontab Installation

Verify the backup cron job was installed during the `release` command:

```bash
# Check if crontab exists for torrust user
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "crontab -l"
```

**Expected output** (for schedule `0 3 * * *`):

```text
# Backup cron job (managed by Torrust)
0 3 * * * cd /opt/torrust && docker compose run --rm backup > /var/log/torrust-backup.log 2>&1 || true
```

**If crontab shows nothing or different output**:

- The `release` command did not properly configure crontab
- Re-run the `release` command or configure manually

**Note**: The backup will run automatically at the scheduled time (3 AM UTC in this example). To verify automatic execution, you can either:

1. Wait for the scheduled time and check logs
2. Manually trigger a backup (see Step 6) to verify functionality
3. Check backup container logs (see Step 11 below)

### Step 11: Monitor Automatic Backup Execution

To verify automatic backups are running on schedule, monitor the backup logs:

```bash
# SSH into VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP

# Watch backup log in real-time (check after scheduled time)
tail -f /var/log/torrust-backup.log

# Or view recent backup entries
grep "Backup cycle" /var/log/torrust-backup.log

# Check backup directory for multiple backup files (evidence of automatic execution)
ls -lh /opt/torrust/storage/backup/sqlite/
ls -lh /opt/torrust/storage/backup/mysql/
```

**Expected** (after multiple scheduled runs):

- Multiple backup files with different timestamps
- Log entries showing successful backup cycles
- For example: `sqlite_20260203_030000.db.gz`, `sqlite_20260204_030000.db.gz`, `sqlite_20260205_030000.db.gz`

**Retention cleanup example**:

When backups older than the retention period exist, you'll see cleanup messages in logs:

```text
[2026-02-10 03:00:00] Cleaning up backups older than 7 days
[2026-02-10 03:00:00]   Deleted: sqlite_20260203_030000.db.gz
[2026-02-10 03:00:00]   Freed space: 4.0K
```

### Step 12: Verify Backup Container Logs

Check the backup container logs for any errors:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "cd /opt/torrust && docker compose logs backup | tail -50"
```

**Expected**: No errors, successful completion messages

## Verification Checklist

Use this checklist to track verification progress:

**Configuration & Deployment**:

- [ ] Backup storage directory exists (`/opt/torrust/storage/backup/etc/`)
- [ ] backup.conf deployed with correct database type and path
- [ ] backup-paths.txt deployed with correct paths
- [ ] Docker compose includes backup service
- [ ] Backup service has correct restart policy (`"no"`)
- [ ] Backup service has correct volumes
- [ ] Backup service has correct networks (none for SQLite, database_network for MySQL)

**Initial Backup (during `run` command)**:

- [ ] Initial backup files created after `run` command
- [ ] Database backup file exists in correct directory (`sqlite/` or `mysql/`)
- [ ] Config backup tar.gz created in `config/` directory
- [ ] Files are compressed (`.db.gz` or `.sql.gz`)

**Manual Backup Execution**:

- [ ] Manual backup executes without errors (`docker compose run --rm backup`)
- [ ] Database backup file created with new timestamp
- [ ] Config backup created with new timestamp
- [ ] Backup container logs show successful completion

**Automatic Scheduled Execution (Crontab)**:

- [ ] Crontab entry installed for torrust user
- [ ] Crontab schedule matches environment configuration
- [ ] Crontab log file exists (`/var/log/torrust-backup.log`)
- [ ] Multiple backup files present (evidence of multiple automated runs)
- [ ] Backup files have different timestamps (at least 2-3 backups)

**Data Integrity**:

- [ ] Database backup files contain valid data (checked with file/gunzip/zcat)
- [ ] Config backup tar.gz contains expected files
- [ ] No errors in backup container logs

**Retention Cleanup**:

- [ ] Retention days parameter is set correctly in backup.conf
- [ ] Old backups are cleaned up after retention period
- [ ] Cleanup messages appear in backup logs

## Troubleshooting

### Issue: Backup configuration files not found

**Symptoms**:

```bash
ls: cannot access '/opt/torrust/storage/backup/etc/': No such file or directory
```

**Cause**: Backup section might be missing from environment configuration

**Solution**:

1. Check environment state: `cat data/$ENV_NAME/environment.json | jq '.Running.context.user_inputs.backup'`
2. If null, backup was not configured - recreate environment with backup section
3. Re-run release command to deploy backup configuration

### Issue: Manual backup fails with "connection refused" (MySQL)

**Symptoms**:

```text
Error: Failed to connect to MySQL at mysql:3306
```

**Cause**: MySQL service not healthy or backup service not on database network

**Solution**:

1. Check MySQL is running: `docker compose ps mysql`
2. Check backup service has database_network: `docker compose config | grep -A 20 backup:`
3. Wait for MySQL to be healthy: `docker compose ps` should show "healthy" status

### Issue: MySQL backup fails with TLS/SSL error

**Symptoms**:

```text
mysqldump: Got error: 2026: "TLS/SSL error: self-signed certificate in certificate chain"
```

**Cause**: MySQL 8.0+ enforces SSL by default, but the backup container needs to connect without strict SSL verification

**Solution**: This is **automatically handled** by the backup container:

- The Docker image includes a MySQL client configuration file at `/etc/mysql/mysql-client.cnf` with `ssl=FALSE` setting
- The backup script references this config file via `--defaults-file=/etc/mysql/mysql-client.cnf`
- Uses `MYSQL_PWD` environment variable for secure password handling

**Status**: ✅ **FIXED** - Backup container v1.0+ includes proper SSL handling

### Issue: Backup files not created

**Symptoms**: `/opt/torrust/storage/backup/database/` is empty after manual backup

**Cause**: Backup script encountered an error during execution

**Solution**:

1. Check backup container logs: `docker compose logs backup`
2. Look for error messages in the output
3. Verify backup.conf has correct paths and credentials
4. For MySQL: verify database credentials match tracker configuration

### Issue: Backup service shows as "Exited"

**Status**: This is **NOT** an error - expected behavior

**Explanation**: The backup service is configured with `restart: "no"`, which means it runs once and exits. This is the correct behavior. The service will only run when:

1. `docker compose up` starts all services (backup runs once)
2. Manual execution: `docker compose run --rm backup`
3. (Future) Scheduled via crontab

## Current Implementation Status

**Implemented Features**:

- ✅ **Initial backup** - Created automatically during `run` command (via `docker-compose.yml`)
- ✅ **Crontab integration** - Automatic scheduled backups at configured schedule
- ✅ **Manual execution** - Can run on-demand with `docker compose run --rm backup`
- ✅ **Retention cleanup** - Automatically removes backups older than retention period
- ✅ **Database support** - Works with both SQLite and MySQL
- ✅ **Configuration backup** - Backs up tracker config, prometheus config, and Grafana provisioning

**Known Limitations**:

- ❌ **Recovery from backup** - Not yet implemented (requires manual restore process)
- ❌ **Backup verification API** - No remote endpoint to verify backup status
- ❌ **Backup encryption** - Backups are compressed but not encrypted

## Testing Workflows

### Quick Verification (10 minutes)

For rapid verification after deployment:

1. Run `provision` command
2. Run `release` command (installs crontab)
3. Run `run` command (creates initial backup)
4. SSH to VM and verify initial backup exists: `ls -lh /opt/torrust/storage/backup/sqlite/`
5. Manually run a second backup: `docker compose run --rm backup`
6. Verify second backup created: `ls -lh /opt/torrust/storage/backup/sqlite/`

**Success**: Two backup files with different timestamps exist

### Full E2E Testing (Multiple Days)

For comprehensive automated backup testing:

1. Deploy with configured backup schedule (e.g., every hour for testing)
2. Wait for scheduled backup time to pass
3. Verify automatic backup executed: `grep "Backup cycle" /var/log/torrust-backup.log`
4. Check multiple backup files created: `ls -lh /opt/torrust/storage/backup/*/`
5. Modify a configuration file, wait for next backup
6. Verify new backup contains the modification
7. Wait for retention cleanup to occur (after retention_days)
8. Verify old backups were deleted

### Retention Testing (7+ Days)

To verify retention cleanup with 7-day retention period:

1. Deploy with `retention_days: 7`
2. Create manual backups (simulating daily backups): `docker compose run --rm backup` (repeat 8 times)
3. Force manual backup on day 8
4. Check `/var/log/torrust-backup.log` for cleanup messages
5. Verify first backup was deleted, most recent 7 kept

## Next Steps

After verifying the backup service works correctly:

1. Test backup restoration (manual process) - **Future enhancement**
2. Implement automated retention testing
3. Monitor disk space usage with production workloads
4. Test backup functionality with different retention periods

## Related Documentation

- [Manual E2E Testing Guide](README.md) - Complete deployment workflow
- [Tracker Verification](tracker-verification.md) - Tracker-specific tests
- [MySQL Verification](mysql-verification.md) - MySQL-specific tests
