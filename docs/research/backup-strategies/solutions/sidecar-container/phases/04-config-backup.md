# Phase 4: Configuration Files Backup

**Status**: ✅ Completed
**Date**: 2026-01-29

## Goal

Add configuration files to the backup with a flexible, configuration-driven approach.

## Design Decisions

### Unified Backup Script

Instead of separate scripts (`backup-mysql.sh`, `entrypoint.sh`, `backup-config.sh`), we created a single `backup.sh` that handles both MySQL and config backups. This eliminates code duplication and simplifies maintenance.

### Paths-File Approach (like restic)

Configuration files to backup are specified in a `backup-paths.txt` file, similar to restic's `--files-from` option. This allows:

- Adding/removing backup paths without rebuilding the container
- Clear documentation of what's being backed up
- Easy customization per deployment

### Standardized Storage Structure

Following the tracker's convention, the backup service uses:

- `etc/` - Configuration (backup-paths.txt)
- `lib/` - Backup data output (mysql/, config/)
- `log/` - Logs (future use)

### Environment-Driven Behavior

All behavior is controlled via environment variables, enabling the same Docker image to be used across different deployments without rebuilding:

- `BACKUP_INTERVAL` - Seconds between backups
- `BACKUP_MYSQL_ENABLED` - Enable/disable MySQL backup
- `BACKUP_PATHS_FILE` - Path to the paths configuration file
- MySQL connection variables

## Checklist

- [x] Create unified backup.sh script
- [x] Implement paths-file parsing
- [x] Mount config directories as read-only
- [x] Copy `.env` and `docker-compose.yml`
- [x] Copy tracker, Prometheus, and Grafana configs
- [x] Preserve directory structure in backups
- [x] Standardize storage structure (etc/lib/log)

## Artifacts

- Unified backup script: [../artifacts/backup-container/backup.sh](../artifacts/backup-container/backup.sh)
- Paths configuration: [../artifacts/backup-storage/etc/backup-paths.txt](../artifacts/backup-storage/etc/backup-paths.txt)
- Docker Compose: [../artifacts/docker-compose-with-backup.yml](../artifacts/docker-compose-with-backup.yml)

## Backup Paths Configuration

The `backup-paths.txt` file specifies what to backup:

```text
# Docker Compose deployment configuration
/data/.env
/data/docker-compose.yml

# Tracker configuration (entire etc directory for future additions)
/data/storage/tracker/etc

# Prometheus configuration (entire etc directory for future additions)
/data/storage/prometheus/etc

# Grafana provisioning (dashboards and datasources)
/data/storage/grafana/provisioning
```

## Volume Mounts

```yaml
volumes:
  # Backup configuration (etc)
  - ./storage/backup/etc:/config:ro
  # Backup output directory (lib)
  - ./storage/backup/lib:/backups
  # Backup logs (log) - future use
  - ./storage/backup/log:/logs
  # Source data directory (read-only)
  - ./:/data:ro
```

## Commands Executed

```bash
# Copy backup container files
scp -i fixtures/testing_rsa backup-container/* torrust@10.140.190.35:/opt/torrust/backup/

# Create storage directories
ssh torrust@10.140.190.35 "mkdir -p /opt/torrust/storage/backup/{etc,lib,log}"

# Copy paths configuration
scp -i fixtures/testing_rsa backup-paths.txt torrust@10.140.190.35:/opt/torrust/storage/backup/etc/

# Rebuild and restart
ssh torrust@10.140.190.35 "cd /opt/torrust && docker compose stop backup && docker compose build backup && docker compose up -d backup"
```

## Validation

### Test: Config Backup Works

```bash
ssh torrust@10.140.190.35 "docker logs backup 2>&1 | tail -20"
```

**Output**:

```text
[2026-01-29 19:53:08] Backup sidecar starting...
[2026-01-29 19:53:08] Configuration:
[2026-01-29 19:53:08]   Interval: 120s
[2026-01-29 19:53:08]   MySQL backup: true
[2026-01-29 19:53:08]   Paths file: /config/backup-paths.txt
[2026-01-29 19:53:08] === Backup cycle starting ===
[2026-01-29 19:53:08] Starting MySQL backup...
[2026-01-29 19:53:08] Database: torrust_tracker@mysql:3306
[2026-01-29 19:53:08] MySQL backup complete: /backups/mysql/mysql_20260129_195308.sql.gz (4.0K)
[2026-01-29 19:53:08] Starting config backup from: /config/backup-paths.txt
[2026-01-29 19:53:08]   Copied: /data/.env
[2026-01-29 19:53:08]   Copied: /data/docker-compose.yml
[2026-01-29 19:53:08]   Copied: /data/storage/tracker/etc
[2026-01-29 19:53:08]   Copied: /data/storage/prometheus/etc
[2026-01-29 19:53:08]   Copied: /data/storage/grafana/provisioning
[2026-01-29 19:53:08] Config backup complete: 5 items copied, 0 not found
[2026-01-29 19:53:08] === Backup cycle complete ===
```

### Test: Verify Backup Structure

```bash
ssh torrust@10.140.190.35 "sudo find /opt/torrust/storage/backup/lib/config -type f"
```

**Output**:

```text
/opt/torrust/storage/backup/lib/config/.env
/opt/torrust/storage/backup/lib/config/docker-compose.yml
/opt/torrust/storage/backup/lib/config/storage/tracker/etc/tracker.toml
/opt/torrust/storage/backup/lib/config/storage/prometheus/etc/prometheus.yml
/opt/torrust/storage/backup/lib/config/storage/grafana/provisioning/dashboards/torrust.yml
/opt/torrust/storage/backup/lib/config/storage/grafana/provisioning/dashboards/torrust/metrics.json
/opt/torrust/storage/backup/lib/config/storage/grafana/provisioning/dashboards/torrust/stats.json
/opt/torrust/storage/backup/lib/config/storage/grafana/provisioning/datasources/prometheus.yml
```

**Result**: ✅ All configuration files backed up with directory structure preserved.

## Issues Encountered

### Issue: Script Crashing on First File

**Problem**: `set -e` caused the script to exit when `((count++))` returned exit code 1 (when count starts at 0).

**Solution**: Changed from `((count++))` to `count=$((count + 1))` which doesn't have this issue.

### Issue: Shellcheck Warnings

**Problem**: `local var=$(command)` masks return values.

**Solution**: Declare and assign separately:

```bash
local var
var=$(command)
```

## Next Steps

Proceed to [Phase 5: Archive Creation](05-archive-creation.md).
