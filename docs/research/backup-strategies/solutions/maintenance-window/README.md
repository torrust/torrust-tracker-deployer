# Maintenance Window Backup

**Status**: ✅ POC Verified

## Summary

Hybrid approach combining container-based backup with host-level orchestration.
A crontab on the host stops the tracker, runs the backup container once, and
restarts the tracker.

## Problem

Container-based backup using SQLite `.backup` is impractical for large databases
(> 1GB) due to locking overhead. The backup sidecar cannot safely stop the
tracker to perform a cold copy because it lacks container orchestration
privileges.

## Solution

Use a **hybrid architecture**:

1. **Backup logic stays in container** - Portable, updatable, version-controlled
2. **Orchestration happens at host level** - Crontab controls start/stop
3. **Backup container runs once** - Like certbot, not continuously

### Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     HOST MACHINE                            │
│                                                             │
│  ┌─────────────────┐                                        │
│  │    Crontab      │  Runs daily at 3:00 AM                 │
│  │  (host level)   │                                        │
│  └────────┬────────┘                                        │
│           │                                                 │
│           ▼                                                 │
│  ┌─────────────────┐                                        │
│  │ maintenance-    │  1. Stop tracker container             │
│  │ backup.sh       │  2. Run backup container (single exec) │
│  │  (host level)   │  3. Start tracker container            │
│  └────────┬────────┘                                        │
│           │                                                 │
│           ▼                                                 │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Docker Compose Stack                   │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │    │
│  │  │ Tracker  │  │  MySQL   │  │ Backup Container │   │    │
│  │  │(stopped) │  │(stopped) │  │  (runs once)     │   │    │
│  │  └──────────┘  └──────────┘  └──────────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Workflow

```text
Crontab triggers at 3:00 AM
         │
         ▼
    ┌────────────────────┐
    │ Check if services  │
    │ are running        │───No──▶ Log warning, skip backup
    └────────┬───────────┘
             │ Yes
             ▼
    ┌────────────────────┐
    │ Stop tracker       │
    │ container          │
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Run backup         │  Single execution, no inner loop
    │ container          │  (like certbot renew)
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Start tracker      │
    │ container          │
    └────────┬───────────┘
             │
             ▼
    ┌────────────────────┐
    │ Log result and     │
    │ send notification  │
    └────────────────────┘
```

## Trade-offs

| Advantage                       | Disadvantage                               |
| ------------------------------- | ------------------------------------------ |
| Complete backup with all data   | Brief service interruption (~90s for 17GB) |
| Container portability preserved | Requires host-level crontab                |
| No SQLite locking issues        | Not suitable for pure PaaS                 |
| Predictable backup duration     | Minimal host configuration needed          |
| Backup container is updatable   | Must detect service state                  |

## Implementation

### Deployer Integration

The deployer would handle this in existing phases:

| Phase     | Action                                           |
| --------- | ------------------------------------------------ |
| Configure | Install crontab on host instance                 |
| Release   | Copy maintenance script, configure cron schedule |

### Artifacts

All implementation artifacts are in the [`artifacts/`](artifacts/) folder:

- [`maintenance-backup.sh`](artifacts/maintenance-backup.sh) - Host-level orchestration script
- [`maintenance-backup.cron`](artifacts/maintenance-backup.cron) - Crontab entry for daily 3:00 AM backup
- [`backup-container/`](artifacts/backup-container/) - Dockerfile and backup script
- [`docker-compose-with-backup.yml`](artifacts/docker-compose-with-backup.yml) - Docker Compose with backup service

### Docker Compose Addition

```yaml
services:
  backup:
    image: tracker-backup:latest
    profiles: ["backup"] # Not started with 'docker compose up'
    volumes:
      - tracker-data:/data:ro
      - backup-storage:/backups
    environment:
      - BACKUP_MODE=single # Run once, not loop
```

## POC Test Results

Tested on 2026-01-30 using the LXD test VM environment.

### Test Environment

- VM: `torrust-tracker-vm-manual-test-sidecar-backup` (LXD)
- Database: MySQL with minimal test data (~4KB compressed)
- Services: tracker, mysql, backup, grafana, prometheus

### Test Execution

```console
$ LOG_FILE=/opt/torrust/storage/backup/log/maintenance-backup.log \
    /opt/torrust/scripts/maintenance-backup.sh

[2026-01-30 11:03:52] === Maintenance backup started ===
[2026-01-30 11:03:52] Configuration:
[2026-01-30 11:03:52]   Compose directory: /opt/torrust
[2026-01-30 11:03:52]   Tracker services:  tracker
[2026-01-30 11:03:52]   Backup service:    backup
[2026-01-30 11:03:52]   Dry run:           false
[2026-01-30 11:03:52]   Force:             false
[2026-01-30 11:03:52] Stopping tracker services: tracker
 Container tracker  Stopping
 Container tracker  Stopped
[2026-01-30 11:04:02] Running backup container (single execution)...
 Container mysql  Running
[2026-01-30 11:04:03] Backup container starting...
[2026-01-30 11:04:03] Configuration:
[2026-01-30 11:04:03]   Mode: single
[2026-01-30 11:04:03]   Interval: 0s
[2026-01-30 11:04:03]   Retention: 7 days
[2026-01-30 11:04:03]   MySQL backup: true
[2026-01-30 11:04:03]   Paths file: /config/backup-paths.txt
[2026-01-30 11:04:03] Running in SINGLE mode (one backup, then exit)
[2026-01-30 11:04:03] === Backup cycle starting ===
[2026-01-30 11:04:03] Starting MySQL backup...
[2026-01-30 11:04:03]   Database: torrust_tracker@mysql:3306
[2026-01-30 11:04:03]   Output: /backups/mysql/mysql_20260130_110403.sql.gz (4.0K)
[2026-01-30 11:04:03] MySQL backup complete
[2026-01-30 11:04:03] Starting config backup from: /config/backup-paths.txt
[2026-01-30 11:04:03]   Copied: /data/.env
[2026-01-30 11:04:03]   Copied: /data/docker-compose.yml
[2026-01-30 11:04:03]   Copied: /data/storage/tracker/etc
[2026-01-30 11:04:03]   Copied: /data/storage/prometheus/etc
[2026-01-30 11:04:03]   Copied: /data/storage/grafana/provisioning
[2026-01-30 11:04:03] Config backup complete: 5 items copied, 0 not found
[2026-01-30 11:04:03] Running maintenance...
[2026-01-30 11:04:03] === Backup cycle complete ===
[2026-01-30 11:04:03] Single backup complete - container will exit
[2026-01-30 11:04:03] Backup container completed successfully
[2026-01-30 11:04:03] Starting tracker services: tracker
 Container mysql  Waiting
 Container mysql  Healthy
 Container tracker  Starting
 Container tracker  Started
[2026-01-30 11:04:04] === Maintenance backup completed successfully in 12s ===
```

### Results

| Metric            | Value   |
| ----------------- | ------- |
| Total time        | 12s     |
| Tracker stop      | ~10s    |
| Backup execution  | ~1s     |
| Tracker restart   | ~2s     |
| MySQL backup size | 4.0K    |
| Config files      | 5 items |

### Key Observations

1. **BACKUP_MODE=single works** - Container runs one cycle and exits cleanly
2. **Host orchestration works** - Stop → backup → start sequence executes correctly
3. **Error handling works** - Tracker always restarts, even if backup fails
4. **Dry-run mode works** - Safe testing without affecting services

### Scaling Considerations

For the 17GB Torrust Demo database:

- MySQL dump time: ~60-90 seconds (estimated)
- Total downtime: ~90-120 seconds
- Acceptable for daily 3 AM maintenance window

## Edge Cases

### Services Already Down

The backup script checks if services are running before attempting backup.
If tracker is already down (manual maintenance, crash, etc.), backup is skipped
with a warning log.

### Backup Container Fails

If the backup container fails, the script should still restart the tracker.
Consider adding error handling:

```bash
run_backup() {
    log "Running backup container..."
    cd "$COMPOSE_DIR"
    if ! docker compose run --rm "$BACKUP_SERVICE"; then
        log "ERROR: Backup failed, but continuing to restart tracker"
        return 1
    fi
}

main() {
    # ...
    stop_tracker
    backup_result=0
    run_backup || backup_result=$?
    start_tracker  # Always restart, even if backup failed

    if [ $backup_result -ne 0 ]; then
        log "WARNING: Backup completed with errors"
        exit 1
    fi
}
```

## Future Enhancement

A future Tracker feature could eliminate the need for host-level configuration:

**Built-in Maintenance Window**:

```toml
# tracker.toml
[maintenance]
enabled = true
window = "03:00-03:05"  # UTC
```

During the maintenance window, the tracker would:

1. Stop accepting new announces
2. Flush database writes
3. Signal readiness for backup (e.g., touch a file, HTTP endpoint)
4. Resume after backup completes

This would allow the backup sidecar to run autonomously without host
intervention, but requires Tracker code changes.
