# Backup Management

## Overview

The Torrust Tracker Deployer includes an automated backup system that protects your tracker database and configuration files. Backups are created and managed automatically with configurable retention policies.

**What gets backed up:**

- Database (SQLite or MySQL)
- Tracker configuration file (tracker.toml)
- Prometheus configuration
- Grafana provisioning files (dashboards, datasources)

**Key features:**

- **Automatic**: Initial backup created during deployment, additional backups on configured schedule
- **Scheduled**: Configurable cron schedule (e.g., daily at 3 AM UTC)
- **Retention policy**: Automatically removes old backups after configured retention period
- **Database-aware**: Handles both SQLite and MySQL databases appropriately
- **Minimal downtime**: Backup process briefly stops tracker service (10-15 seconds)
- **Compressed**: All backups are compressed to save storage space

---

## Configuration

### Enabling Backups

Backups are configured in your environment file (e.g., `envs/my-deployment.json`):

```json
{
  "backup": {
    "schedule": "0 3 * * *",
    "retention_days": 7
  }
}
```

### Configuration Options

#### `schedule` (required)

Cron schedule for automatic backups. Specified in standard cron format: `minute hour day month day_of_week`

**Examples:**

- `0 3 * * *` - Every day at 3:00 AM UTC
- `0 2 * * 1` - Every Monday at 2:00 AM UTC
- `0 */6 * * *` - Every 6 hours
- `0 0 1 * *` - First day of every month

**Constraints:**

- Must be a valid cron expression
- Recommended: Off-peak hours to minimize tracker downtime
- Minimum frequency: Once per week (for meaningful backups)

#### `retention_days` (required)

How many days to keep backups before automatic deletion.

**Examples:**

- `7` - Keep one week of backups
- `30` - Keep one month of backups
- `90` - Keep three months of backups

**Constraints:**

- Must be between 1 and 365 days
- Recommended: 7-30 days for normal deployments
- Higher retention requires more storage space

### Configuration Examples

#### Basic Backup (Daily)

```json
{
  "backup": {
    "schedule": "0 3 * * *",
    "retention_days": 7
  }
}
```

Daily backups at 3 AM UTC, keep one week of backups.

#### Conservative Backup (Weekly)

```json
{
  "backup": {
    "schedule": "0 3 * * 0",
    "retention_days": 30
  }
}
```

Weekly backups on Sundays at 3 AM UTC, keep one month of backups.

#### Frequent Backup (Every 6 Hours)

```json
{
  "backup": {
    "schedule": "0 */6 * * *",
    "retention_days": 3
  }
}
```

Backups every 6 hours, keep 3 days (18 backup files).

#### Disable Automatic Backup

Currently, automatic backups cannot be completely disabled. If you don't want automatic backups, you can use a cron schedule that never matches:

```json
{
  "backup": {
    "schedule": "0 0 31 2 *",
    "retention_days": 7
  }
}
```

This schedule would never run (February 31st doesn't exist). Manual backups can still be triggered.

---

## How It Works

### Deployment Phases

#### Phase 1: Initial Backup (during `run` command)

When you run the `run` command, the backup service creates an initial backup:

1. Backup container starts
2. Reads configuration
3. Backs up database and config files
4. Creates compressed backup files
5. Container exits successfully

This initial backup proves the backup system is working and provides a recovery point right after deployment.

#### Phase 2: Scheduled Backups (via crontab)

After the `release` command, a system cron entry is installed at `/etc/cron.d/tracker-backup`. On your configured schedule:

1. At scheduled time, the maintenance script `/usr/local/bin/maintenance-backup.sh` executes
2. Script stops the tracker service (briefly)
3. Runs backup container: `docker compose run --rm backup`
4. Backup creates new backup files with current timestamp
5. Runs retention cleanup (deletes backups older than retention_days)
6. Tracker service restarts
7. Output logged to `/var/log/tracker-backup.log`

#### Phase 3: Retention Cleanup

After each backup, cleanup runs automatically:

1. Lists all backup files in backup directory
2. Calculates age of each backup (current_time - backup_timestamp)
3. Deletes any backups older than `retention_days`
4. Logs cleanup actions

---

## Backup File Storage

### Directory Structure

Backup files are stored in the backup container's `/backups/` directory, which is mounted to `/opt/torrust/storage/backup/` on the host:

**On the VM (host path)**:

```text
/opt/torrust/storage/backup/
├── etc/
│   ├── backup.conf                   # Backup service configuration
│   └── backup-paths.txt              # Paths to backup
├── sqlite/                           # SQLite database backups
│   ├── sqlite_20260203_030000.db.gz
│   ├── sqlite_20260204_030000.db.gz
│   └── sqlite_20260205_030000.db.gz
├── mysql/                            # MySQL database backups
│   ├── mysql_20260203_030000.sql.gz
│   ├── mysql_20260204_030000.sql.gz
│   └── mysql_20260205_030000.sql.gz
└── config/                           # Configuration backups
    ├── config_20260203_030000.tar.gz
    ├── config_20260204_030000.tar.gz
    └── config_20260205_030000.tar.gz
```

**Inside the backup container (container path)**:

```text
/backups/                            # Mounted to /opt/torrust/storage/backup/
├── sqlite/                          # SQLite database backups
├── mysql/                           # MySQL database backups
└── config/                          # Configuration backups
```

**Docker Compose volume mapping:**

```yaml
volumes:
  - ./storage/backup:/backups # Host path: /opt/torrust/storage/backup/
```

All backup files are accessible via the host path `/opt/torrust/storage/backup/` when you SSH into the VM.

### Filename Format

**SQLite database backups:**

```text
sqlite_YYYYMMDD_HHMMSS.db.gz
```

**MySQL database backups:**

```text
mysql_YYYYMMDD_HHMMSS.sql.gz
```

**Configuration backups:**

```text
config_YYYYMMDD_HHMMSS.tar.gz
```

The timestamp suffix (`YYYYMMDD_HHMMSS`) makes backups sortable and uniquely identifiable.

### File Sizes

Typical sizes for test deployments:

- **SQLite backup**: 4-10 KB (compressed)
- **MySQL backup**: 4-50 KB (compressed)
- **Config backup**: 2-5 KB (compressed)

Production deployments with active trackers may have larger backups. Monitor disk usage:

```bash
# Check backup directory size
du -sh /opt/torrust/storage/backup/
```

---

## Monitoring & Verification

### Verify Initial Backup

After deployment, verify the initial backup was created:

```bash
# SSH to your VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@<instance-ip>

# Check backup files
ls -lh /opt/torrust/storage/backup/sqlite/
ls -lh /opt/torrust/storage/backup/mysql/
ls -lh /opt/torrust/storage/backup/config/
```

You should see files like:

```text
-rw-r--r-- 1 torrust torrust 4.0K Feb  3 03:00 sqlite_20260203_030000.db.gz
-rw-r--r-- 1 torrust torrust 3.2K Feb  3 03:00 config_20260203_030000.tar.gz
```

### Check Crontab Configuration

Verify the backup system cron entry was installed during the `release` command:

```bash
# Check if system cron entry exists
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@<instance-ip> "cat /etc/cron.d/tracker-backup"
```

**Expected output** (for schedule `0 3 * * *`):

```text
# Backup Maintenance Schedule
0 3 * * * root cd /opt/torrust && /usr/local/bin/maintenance-backup.sh >> /var/log/tracker-backup.log 2>&1
```

The cron entry uses a maintenance script that:

1. Stops the tracker service
2. Runs the backup container
3. Restarts the tracker service
4. Logs all output to `/var/log/tracker-backup.log`

**If cron entry not found**:

- The `release` command did not properly install the cron entry
- Re-run the `release` command

### Monitor Automatic Backups

After the scheduled backup time passes, verify automatic backups are running:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@<instance-ip> "tail -20 /var/log/tracker-backup.log"
```

**Expected output**:

```text
[2026-02-04 16:35:01] INFO: Tracker stopped successfully
[2026-02-04 16:35:01] INFO: Running backup container (via backup profile)...
[2026-02-04 16:35:06] INFO: Backup completed successfully
[2026-02-04 16:35:06] INFO: Starting tracker container...
[2026-02-04 16:35:21] INFO: Tracker started successfully
[2026-02-04 16:35:21] Backup maintenance completed (exit code: 0)
```

You can also verify backup files were created:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@<instance-ip> "ls -lh /opt/torrust/storage/backup/sqlite/"
```

**Expected**: Multiple backup files with different timestamps (one per backup execution):

```text
-rw-r--r-- 1 root root 4.0K Feb  3 03:00 sqlite_20260203_030000.db.gz
-rw-r--r-- 1 root root 4.0K Feb  4 03:00 sqlite_20260204_030000.db.gz
-rw-r--r-- 1 root root 4.0K Feb  5 03:00 sqlite_20260205_030000.db.gz
```

Multiple files with different dates indicate automatic backups are executing on schedule.

**Note**: Backup logging to `/var/log/torrust-backup.log` is a planned enhancement for future versions. Currently, backup output is captured only when running manually via `docker compose run`.

### Verify Backup Content

For SQLite backups, verify the database backup is valid:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@<instance-ip> "cd /opt/torrust/storage/backup/sqlite && gunzip -c sqlite_*.db.gz | file -"
```

Expected: `SQLite 3.x database`

For configuration backups, list contents:

```bash
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@<instance-ip> "tar -tzf /opt/torrust/storage/backup/config/config_*.tar.gz | head -20"
```

Expected: Files like `tracker.toml`, `prometheus.yml`, etc.

---

## Troubleshooting

### No Backup Files Created

**Symptoms**: `/opt/torrust/storage/backup/` is empty or missing subdirectories

**Possible causes**:

1. Backup not configured in environment (check `environment.json`)
2. Release command failed to deploy backup configuration
3. Backup container not running

**Solutions**:

```bash
# 1. Verify backup was configured
cat data/<env-name>/environment.json | jq '.Running.context.user_inputs.backup'

# 2. Check if backup directory exists
ls -la /opt/torrust/storage/backup/

# 3. Check backup container logs
docker compose logs backup | tail -50

# 4. Try running backup manually
docker compose run --rm backup
```

### Backup Files Not Updating

**Symptoms**: Backup files exist but have old timestamps (older than cron schedule suggests)

**Possible causes**:

1. Crontab not installed or incorrect
2. Docker daemon not running
3. Insufficient disk space

**Solutions**:

```bash
# 1. Verify crontab is installed
crontab -l

# 2. Check Docker status
docker ps -a

# 3. Check disk space
df -h /opt/torrust/storage/backup/

# 4. Run backup manually to test
docker compose run --rm backup
```

### MySQL Connection Error

**Symptoms**: Backup log shows "Access denied" or "Connection refused" for MySQL

**Possible causes**:

1. MySQL service not healthy
2. Database credentials incorrect
3. Backup service not on same network as MySQL

**Solutions**:

```bash
# 1. Check MySQL service is running
docker compose ps mysql

# 2. Check MySQL is healthy
docker compose exec mysql mysql -u root -p<password> -e "SELECT 1"

# 3. Check backup service has database network
docker compose config | grep -A 30 'backup:'
```

### MySQL TLS/SSL Warning

**Symptoms**: Backup log shows "SSL error" warning but backup still completes

**Status**: ✅ **Expected and not a problem**

The warning appears because the backup user lacks the PROCESS privilege for tablespace metadata, but the backup container is configured to skip strict SSL verification. The database backup is created successfully.

### Retention Cleanup Not Running

**Symptoms**: Old backup files not being deleted after retention period

**Possible causes**:

1. Backup script not cleaning up (check manual backup output)
2. Insufficient disk permissions
3. Backup files have wrong ownership/permissions

**Solutions**:

```bash
# 1. Run backup manually and check for cleanup messages
docker compose run --rm backup

# 2. Check file permissions
ls -la /opt/torrust/storage/backup/*/

# 3. Check backup configuration
cat /opt/torrust/storage/backup/etc/backup.conf
```

### Backup Container Shows as "Exited"

**Status**: ✅ **This is normal**

The backup container is configured with `restart: no`, which means:

- It runs once (on schedule or manual trigger)
- Container exits after completing backup
- Service shows as "Exited (0)" - exit code 0 = success
- This is the correct behavior

---

## Database-Specific Notes

### SQLite Backups

**How it works:**

1. Database file located at `/data/storage/tracker/lib/tracker.db`
2. Backup compresses the entire database file: `sqlite_<timestamp>.db.gz`
3. No need to stop database (SQLite file-based)
4. Minimal downtime: Only brief lock during file read

**File format:**

- Backup is complete SQLite database file (compressed)
- Can be restored by decompressing and copying back
- Compatible with SQLite command-line tools

**Storage:**

- Typical size: 4-10 KB (compressed)
- Increases with tracker activity (larger database)

### MySQL Backups

**How it works:**

1. Uses `mysqldump` to export database structure and data
2. Creates SQL dump file: `mysql_<timestamp>.sql.gz`
3. Must connect to MySQL service
4. Backup user: `tracker_user` with full database privileges

**File format:**

- Backup is SQL dump (compressed)
- Contains `CREATE TABLE` statements and `INSERT` statements
- Compatible with MySQL command-line: `mysql < backup.sql`

**Expected warnings:**

Backup logs may show:

```text
mysqldump: Error: 'Access denied; you need (at least one of) the PROCESS privilege(s) for this operation'
```

This is **expected and not a problem** - the backup user has sufficient privileges for table backup.

**Storage:**

- Typical size: 4-50 KB (compressed)
- Depends on database size and tracker activity

---

## Manual Backup Execution

### Running a Backup On-Demand

You can trigger a backup manually anytime:

```bash
# SSH to your VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@<instance-ip>

# Navigate to application directory
cd /opt/torrust

# Run backup immediately
docker compose run --rm backup
```

This creates a new backup file with current timestamp, independent of the cron schedule.

**Use cases for manual backup:**

- Before making configuration changes
- Before deploying software updates
- Before scaling operations
- Testing backup restoration

---

## Recovery (Future Enhancement)

Backup restoration is a planned feature. Currently, recovery requires manual steps:

1. Stop the tracker service
2. Download backup file from VM
3. Decompress backup file
4. Restore database from backup
5. Restart tracker service

Recovery procedures will be documented once the feature is implemented.

---

## Best Practices

1. **Choose appropriate backup frequency**:
   - High-traffic tracker: Daily backups (or more frequent)
   - Medium-traffic tracker: Daily backups
   - Low-traffic tracker: Weekly backups are sufficient

2. **Monitor backup disk usage**:
   - Check `/var/log/torrust-backup.log` regularly
   - Use `du -sh /opt/torrust/storage/backup/` to monitor growth
   - Adjust retention_days if disk space becomes an issue

3. **Schedule backups during off-peak hours**:
   - Backup briefly stops the tracker (~10-15 seconds)
   - Schedule when user traffic is lowest
   - Avoid peak usage times

4. **Test backup restoration occasionally**:
   - Verify backups are actually restorable
   - Document restoration procedures
   - Test with staging environment first

5. **Keep configuration and database backups in sync**:
   - Both are backed up together automatically
   - Enables consistent restoration
   - Don't delete backups manually unless necessary

6. **Monitor backup execution**:

   For now, verify backups exist by checking the filesystem:

   ```bash
   # SSH to VM
   ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
     torrust@<instance-ip>

   # Check backup files
   ls -lh /opt/torrust/storage/backup/sqlite/
   ls -lh /opt/torrust/storage/backup/config/

   # When manually triggered, backup output appears on stdout
   cd /opt/torrust
   docker compose run --rm backup
   ```

   Backup logging to `/var/log/torrust-backup.log` is planned for a future release.

---

## See Also

- [Manual Backup Verification Guide](../e2e-testing/manual/backup-verification.md) - Step-by-step verification procedures
- [Create Environment Command](commands/create.md) - Backup configuration during environment creation
- [Release Command](commands/release.md) - How backup service is deployed
- [Run Command](commands/run.md) - Initial backup during deployment
