# Torrust Live Demo Backup Implementation

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

This document analyzes the current SQLite backup implementation used in the Torrust Live Demo production environment.

## Current Implementation

**Source**: [tracker-db-backup.sh](https://github.com/torrust/torrust-demo/blob/main/share/bin/tracker-db-backup.sh)

```bash
#!/bin/bash

# Backup the Index SQLite database

# Define the directory where backups will be stored
BACKUP_DIR="/home/torrust/backups"

# Define the SQLite database file's path
DATABASE_FILE="/home/torrust/github/torrust/torrust-demo/storage/tracker/lib/database/sqlite3.db"

# Create a timestamped backup filename
BACKUP_FILE="$BACKUP_DIR/tracker_backup_$(date +%Y-%m-%d_%H-%M-%S).db"

# Copy the SQLite database file to create a backup
cp $DATABASE_FILE "$BACKUP_FILE"

# Find and remove backups older than 7 days
find $BACKUP_DIR -type f -name "tracker_backup_*.db" -mtime +7 -exec rm -f {} \;
```

## Implementation Analysis

### Configuration

| Setting          | Value                          | Notes                          |
| ---------------- | ------------------------------ | ------------------------------ |
| Backup Directory | `/home/torrust/backups`        | Local directory on same server |
| Database Path    | `/home/torrust/.../sqlite3.db` | Main tracker database          |
| Timestamp Format | `%Y-%m-%d_%H-%M-%S`            | Per-second granularity         |
| Retention Period | 7 days                         | Uses `find -mtime +7`          |

### Backup Method

**Method**: Simple file copy using `cp`

```bash
cp $DATABASE_FILE "$BACKUP_FILE"
```

**Characteristics:**

- No SQLite-aware backup mechanism
- No locking or coordination with the tracker
- Tracker continues running during backup
- Relies on filesystem copy atomicity

### Schedule

The script is run via cron (assumed daily based on context):

```cron
# Example cron entry (not in script, configured separately)
0 3 * * * /path/to/tracker-db-backup.sh
```

### Retention Policy

```bash
find $BACKUP_DIR -type f -name "tracker_backup_*.db" -mtime +7 -exec rm -f {} \;
```

- Keeps backups for 7 days
- Uses `find -mtime +7` to identify old files
- Deletes files matching the backup naming pattern

## Strengths

| Strength                | Description                                  |
| ----------------------- | -------------------------------------------- |
| **Simplicity**          | Very simple, easy to understand and maintain |
| **Non-intrusive**       | Tracker doesn't need to be aware of backups  |
| **Fast**                | Simple file copy is very fast                |
| **Reliable scheduling** | cron-based, well-understood mechanism        |
| **Automatic cleanup**   | Old backups are automatically removed        |

## Limitations and Risks

### 1. No Consistency Guarantee

**Risk**: If the tracker writes to the database during the `cp` operation, the backup may be inconsistent or corrupted.

**Mitigation in current setup**:

- Low write frequency on demo tracker
- Backup runs during low-activity period (likely early morning)
- Demo data is not critical - occasional corruption acceptable

**Better approach**: Use `sqlite3 database.db ".backup backup.db"`

### 2. No Compression

**Issue**: Backups are stored uncompressed, using more disk space than necessary.

**Current state**: Each backup is full size of database

**Improvement**: Add compression (e.g., `gzip` or `zstd`)

```bash
cp $DATABASE_FILE "$BACKUP_FILE" && gzip "$BACKUP_FILE"
```

### 3. No Error Handling

**Issue**: Script doesn't check if operations succeed.

**Current state**:

- No check if `cp` succeeds
- No check if `find` cleanup succeeds
- No logging of success/failure
- No alerting on failure

**Improvement**:

```bash
if ! cp "$DATABASE_FILE" "$BACKUP_FILE"; then
    echo "Backup failed: $(date)" >> /var/log/backup-errors.log
    exit 1
fi
```

### 4. No Backup Verification

**Issue**: No check that the backup file is valid SQLite database.

**Improvement**:

```bash
if ! sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;" | grep -q "ok"; then
    echo "Backup integrity check failed" >&2
    rm "$BACKUP_FILE"
    exit 1
fi
```

### 5. WAL Mode Not Handled

**Issue**: If the database uses WAL mode, the `-wal` and `-shm` files are not backed up.

**Current state**: Only the main `.db` file is copied

**Improvement**: Use `.backup` command which handles WAL automatically, or explicitly copy WAL files:

```bash
cp "${DATABASE_FILE}-wal" "${BACKUP_FILE}-wal" 2>/dev/null || true
cp "${DATABASE_FILE}-shm" "${BACKUP_FILE}-shm" 2>/dev/null || true
```

### 6. No Off-Site Redundancy

**Issue**: Backups stored on same server as the database.

**Risk**: Server failure loses both database and all backups

**Current mitigation**: The droplet has daily infrastructure snapshots

**Better approach**: Copy backups to separate storage (S3, attached volume, etc.)

### 7. No Quoting in Variables

**Issue**: Variables are not quoted, which could cause issues with paths containing spaces.

**Current**:

```bash
cp $DATABASE_FILE "$BACKUP_FILE"
```

**Better**:

```bash
cp "$DATABASE_FILE" "$BACKUP_FILE"
```

## Why This Works for Live Demo

Despite the limitations, this approach is acceptable for the Live Demo because:

1. **Demo data is not critical** - Loss of peer statistics is inconvenient but not catastrophic
2. **Low write frequency** - Demo tracker has minimal concurrent writes
3. **Infrastructure backup** - Droplet snapshots provide secondary protection
4. **Simple maintenance** - Easy to understand and modify
5. **Proven track record** - Has been working reliably in production

## Improved Scripts (Exploration)

These scripts explore how the identified limitations could be addressed:

### Minimal Improvements

```bash
#!/bin/bash
set -e  # Exit on error

BACKUP_DIR="/home/torrust/backups"
DATABASE_FILE="/path/to/sqlite3.db"
BACKUP_FILE="$BACKUP_DIR/tracker_backup_$(date +%Y-%m-%d_%H-%M-%S).db"

# Use SQLite backup command for consistency
sqlite3 "$DATABASE_FILE" ".backup '$BACKUP_FILE'"

# Compress the backup
gzip "$BACKUP_FILE"

# Cleanup old backups
find "$BACKUP_DIR" -type f -name "tracker_backup_*.db.gz" -mtime +7 -delete
```

### Full Improvements

```bash
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/home/torrust/backups"
DATABASE_FILE="/path/to/sqlite3.db"
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
BACKUP_FILE="$BACKUP_DIR/tracker_backup_${TIMESTAMP}.db"
LOG_FILE="/var/log/tracker-backup.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$LOG_FILE"
}

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Use SQLite backup command for consistency
log "Starting backup..."
if ! sqlite3 "$DATABASE_FILE" ".backup '$BACKUP_FILE'"; then
    log "ERROR: Backup command failed"
    exit 1
fi

# Verify backup integrity
if ! sqlite3 "$BACKUP_FILE" "PRAGMA integrity_check;" | grep -q "ok"; then
    log "ERROR: Backup integrity check failed"
    rm -f "$BACKUP_FILE"
    exit 1
fi

# Compress with zstd
log "Compressing backup..."
zstd -q --rm "$BACKUP_FILE" -o "${BACKUP_FILE}.zst"

# Cleanup old backups (keep 7 days)
log "Cleaning up old backups..."
find "$BACKUP_DIR" -type f -name "tracker_backup_*.db.zst" -mtime +7 -delete

log "Backup completed: ${BACKUP_FILE}.zst"
```

## Journal Mode Investigation

### Checking the Current Mode

The Torrust Live Demo database uses the traditional **delete** journal mode:

```bash
$ sqlite3 ./storage/tracker/lib/database/sqlite3.db "PRAGMA journal_mode;"
delete
```

### Journal Modes Explained

| Mode       | Description                                        | Backup Implications                      |
| ---------- | -------------------------------------------------- | ---------------------------------------- |
| `delete`   | Traditional rollback journal, deleted after commit | Single `.db` file, simpler backup        |
| `wal`      | Write-Ahead Logging, journal persists              | Three files: `.db`, `.db-wal`, `.db-shm` |
| `truncate` | Journal truncated instead of deleted               | Single `.db` file                        |
| `persist`  | Journal header zeroed instead of deleted           | Single `.db` file                        |

### Changing to WAL Mode

To change a database to WAL mode:

```sql
PRAGMA journal_mode=WAL;
```

From the command line:

```bash
sqlite3 database.db "PRAGMA journal_mode=WAL;"
```

**Notes on WAL mode**:

- Better concurrency (readers don't block writers)
- Slightly better performance for most workloads
- Requires backing up three files instead of one (unless using `.backup` command)
- The `.backup` command handles WAL automatically

### Implications for Backup

Since the demo uses `delete` mode:

- Only the `.db` file needs to be backed up
- Simple `cp` is slightly safer than with WAL (no extra files to coordinate)
- However, `.backup` command is still safer regardless of journal mode

## Summary

The current Torrust Live Demo backup implementation is:

- **Simple and functional** for a demo environment
- **Has known limitations** around consistency and redundancy
- **Uses delete journal mode** (single file, no WAL complications)
- **Acceptable risk** given the non-critical nature of demo data

The "Improved Scripts" section above shows potential enhancements that address the identified limitations.
