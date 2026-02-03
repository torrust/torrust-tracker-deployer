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

**Note**: The backup service can be deployed and executed **manually** using `docker compose run`. Automatic scheduled backups via crontab are a future enhancement.

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

For MySQL deployments, verify the SQL dump was created:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "ls -lh /opt/torrust/storage/backup/mysql/ | grep '\.sql\.gz'"

# Check file size is reasonable (should be > 0 bytes)
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "du -h /opt/torrust/storage/backup/mysql/*.sql.gz"
```

**Expected**: Compressed SQL dump file (`mysql_<timestamp>.sql.gz`) with reasonable size (> 0 bytes)

### Step 10: Verify Backup Container Logs

Check the backup container logs for any errors:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "cd /opt/torrust && docker compose logs backup | tail -50"
```

**Expected**: No errors, successful completion messages

## Verification Checklist

Use this checklist to track verification progress:

- [ ] Backup storage directory exists (`/opt/torrust/storage/backup/etc/`)
- [ ] backup.conf deployed with correct database type and path
- [ ] backup-paths.txt deployed with correct paths
- [ ] Docker compose includes backup service
- [ ] Backup service has correct restart policy (`"no"`)
- [ ] Backup service has correct volumes
- [ ] Backup service has correct networks (none for SQLite, database_network for MySQL)
- [ ] Manual backup executes without errors
- [ ] Database backup file created in correct directory (`sqlite/` for SQLite, `mysql/` for MySQL)
- [ ] Backup files are compressed (`.db.gz` for SQLite, `.sql.gz` for MySQL)
- [ ] Config backup tar.gz created in `config/` directory
- [ ] Backup files contain valid data (checked with file/gunzip/tar commands)
- [ ] No errors in backup container logs

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

## Known Limitations

- ❌ **No automatic scheduled backups** - Backups must be triggered manually
- ❌ **No crontab integration** - Future enhancement
- ✅ **Manual backup execution works** - Can run on-demand with `docker compose run`

## Next Steps

After verifying the backup service works correctly:

1. Test backup restoration (manual process)
2. Verify backup file integrity
3. Test with different retention periods
4. Monitor disk space usage with multiple backups

## Related Documentation

- [Manual E2E Testing Guide](README.md) - Complete deployment workflow
- [Tracker Verification](tracker-verification.md) - Tracker-specific tests
- [MySQL Verification](mysql-verification.md) - MySQL-specific tests
