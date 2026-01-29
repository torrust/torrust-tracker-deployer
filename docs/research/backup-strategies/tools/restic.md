# Restic Backup Tool Evaluation

**Issue**: [#310 - Research database backup strategies](https://github.com/torrust/torrust-tracker-deployer/issues/310)

## Overview

[Restic](https://restic.net/) is a modern backup program written in Go. It's designed for secure, efficient backups with built-in encryption and deduplication.

## Requirements Checklist

| Requirement        | Supported | Notes                                          |
| ------------------ | --------- | ---------------------------------------------- |
| Docker support     | ✅ Yes    | Official image: `restic/restic`                |
| SQLite backup      | ✅ Yes    | Via pre-hook script using `.backup`            |
| MySQL backup       | ✅ Yes    | Via pre-hook script using `mysqldump`          |
| Local backups      | ✅ Yes    | Local directory as repository                  |
| Remote backups     | ✅ Yes    | S3, B2, SFTP, Azure, GCS, rclone               |
| Encryption         | ✅ Yes    | AES-256, mandatory (all repos encrypted)       |
| Deduplication      | ✅ Yes    | Content-defined chunking                       |
| Retention policies | ✅ Yes    | `restic forget --keep-daily 7 --keep-weekly 4` |
| Verification       | ✅ Yes    | `restic check` for integrity                   |
| Compression        | ✅ Yes    | Built-in (since v0.14.0)                       |
| Single binary      | ✅ Yes    | Easy to install, no dependencies               |

## How It Works

Restic backs up **files**, not databases directly. For databases, we use a two-phase approach:

```text
┌─────────────────────────────────────────────────────────┐
│                  Backup Workflow                        │
│                                                         │
│  1. Pre-hook: Create database dumps                     │
│     sqlite3 db.db ".backup /dumps/tracker.db"           │
│     mysqldump ... > /dumps/mysql.sql                    │
│                                                         │
│  2. Restic: Backup the dump files                       │
│     restic backup /dumps                                │
│                                                         │
│  3. Post-hook: Cleanup temp files                       │
│     rm /dumps/*                                         │
│                                                         │
│  4. Retention: Remove old snapshots                     │
│     restic forget --keep-daily 7 --prune                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Docker Integration

### Official Docker Image

```bash
docker pull restic/restic
```

**Important**: The official image is minimal (Alpine-based) and only includes:

- `ca-certificates`, `fuse`, `openssh-client`, `tzdata`, `jq`

**It does NOT include `sqlite3` or `mysql-client`** - we need a custom image.

### Custom Dockerfile for Database Backups

```dockerfile
# Dockerfile.backup
FROM restic/restic:latest

# Install database clients for pre-hook scripts
RUN apk add --no-cache \
    sqlite \
    mysql-client

# Copy backup scripts
COPY backup-scripts/ /scripts/
RUN chmod +x /scripts/*.sh

ENTRYPOINT ["/scripts/backup-loop.sh"]
```

Build with:

```bash
docker build -f Dockerfile.backup -t torrust/backup .
```

### Example Docker Compose

```yaml
services:
  tracker:
    image: torrust/tracker
    volumes:
      - tracker-data:/var/lib/torrust

  mysql:
    image: mysql:8
    volumes:
      - mysql-data:/var/lib/mysql
    environment:
      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD}

  backup:
    # Use our custom image with sqlite3 and mysql-client
    image: torrust/backup
    # Or build inline:
    # build:
    #   context: .
    #   dockerfile: Dockerfile.backup
    volumes:
      # Database volumes (read-only)
      - tracker-data:/data/tracker:ro
      # Backup scripts
      - ./backup-scripts:/scripts:ro
      # Local backup repository
      - backup-repo:/repo
      # Temp directory for dumps
      - backup-dumps:/dumps
    environment:
      RESTIC_REPOSITORY: /repo
      RESTIC_PASSWORD: ${BACKUP_PASSWORD}
      # For remote backups (optional):
      # RESTIC_REPOSITORY: s3:s3.amazonaws.com/bucket/backups
      # AWS_ACCESS_KEY_ID: ${AWS_ACCESS_KEY_ID}
      # AWS_SECRET_ACCESS_KEY: ${AWS_SECRET_ACCESS_KEY}
    entrypoint: ["/scripts/backup-loop.sh"]
    depends_on:
      - tracker
      - mysql

volumes:
  tracker-data:
  mysql-data:
  backup-repo:
  backup-dumps:
```

### Backup Script Example

```bash
#!/bin/bash
# backup-loop.sh
set -e

BACKUP_INTERVAL=${BACKUP_INTERVAL:-21600}  # Default: 6 hours

# Initialize repository if needed
restic snapshots 2>/dev/null || restic init

echo "Backup service started. Interval: ${BACKUP_INTERVAL}s"

while true; do
    echo "$(date): Starting backup..."

    # Phase 1: Create database dumps
    # SQLite (if tracker uses SQLite)
    if [ -f /data/tracker/sqlite3.db ]; then
        echo "Dumping SQLite database..."
        sqlite3 /data/tracker/sqlite3.db ".backup /dumps/tracker.db"
    fi

    # MySQL (if configured)
    if [ -n "$MYSQL_HOST" ]; then
        echo "Dumping MySQL database..."
        mysqldump -h "$MYSQL_HOST" -u "$MYSQL_USER" -p"$MYSQL_PASSWORD" \
            "$MYSQL_DATABASE" > /dumps/mysql.sql
    fi

    # Phase 2: Restic backup
    echo "Running restic backup..."
    restic backup /dumps \
        --tag torrust \
        --tag "$(date +%Y-%m-%d)"

    # Phase 3: Apply retention policy
    echo "Applying retention policy..."
    restic forget \
        --keep-daily 7 \
        --keep-weekly 4 \
        --keep-monthly 6 \
        --prune

    # Phase 4: Cleanup dumps
    rm -f /dumps/*

    # Phase 5: Verify (weekly)
    if [ "$(date +%u)" = "1" ]; then
        echo "Running weekly integrity check..."
        restic check
    fi

    echo "$(date): Backup completed. Next backup in ${BACKUP_INTERVAL}s"
    sleep "$BACKUP_INTERVAL"
done
```

## Key Commands

```bash
# Initialize a new repository
restic init

# Backup files
restic backup /path/to/data

# List all snapshots
restic snapshots

# Restore latest snapshot
restic restore latest --target /restore/path

# Restore specific snapshot
restic restore abc123 --target /restore/path

# Check repository integrity
restic check

# Apply retention policy (keep 7 daily, 4 weekly, 6 monthly)
restic forget --keep-daily 7 --keep-weekly 4 --keep-monthly 6 --prune

# Mount repository as filesystem (for browsing)
restic mount /mnt/restic
```

## Storage Backends

Restic supports multiple storage backends:

| Backend      | Use Case             | Example                                        |
| ------------ | -------------------- | ---------------------------------------------- |
| Local        | On-disk backups      | `RESTIC_REPOSITORY=/backups`                   |
| SFTP         | Remote server        | `RESTIC_REPOSITORY=sftp:user@host:/backups`    |
| S3           | AWS S3 or compatible | `RESTIC_REPOSITORY=s3:s3.amazonaws.com/bucket` |
| Backblaze B2 | Cheap cloud storage  | `RESTIC_REPOSITORY=b2:bucket:path`             |
| Azure        | Azure Blob Storage   | `RESTIC_REPOSITORY=azure:container:/`          |
| Google Cloud | GCS                  | `RESTIC_REPOSITORY=gs:bucket:/`                |
| rclone       | 40+ cloud providers  | `RESTIC_REPOSITORY=rclone:remote:path`         |

## Deduplication Benefits

For a 17 GB database with daily backups:

| Approach              | Storage After 30 Days              |
| --------------------- | ---------------------------------- |
| Simple copies         | ~510 GB (17 GB × 30)               |
| Restic (deduplicated) | ~20-30 GB (depends on change rate) |

Restic uses content-defined chunking, so only changed blocks are stored.

## Pros and Cons

### Pros

| Advantage           | Description                                 |
| ------------------- | ------------------------------------------- |
| Battle-tested       | Mature project, active community            |
| Single binary       | Easy to install and containerize            |
| Built-in encryption | All data encrypted, no separate tool needed |
| Deduplication       | Massive storage savings for large DBs       |
| Multiple backends   | Local, cloud, SFTP - your choice            |
| Retention policies  | Built-in, easy to configure                 |
| Verification        | `restic check` ensures backup integrity     |
| Open source         | BSD license, actively maintained            |

### Cons

| Disadvantage        | Description                               |
| ------------------- | ----------------------------------------- |
| Pre-hooks required  | We still write database dump scripts      |
| Learning curve      | New tool to understand                    |
| Password management | Must secure the repository password       |
| Not database-aware  | Doesn't know about SQLite/MySQL internals |

## Conclusion

**Restic meets all our requirements.**

It doesn't backup databases directly, but this is actually a feature - we use simple, well-understood tools (`.backup`, `mysqldump`) for the database-specific part, and Restic handles the storage, encryption, deduplication, and retention.

For the Torrust Tracker Deployer:

1. ✅ Works in Docker
2. ✅ Supports SQLite (via pre-hook)
3. ✅ Supports MySQL (via pre-hook)
4. ✅ Can backup locally or to cloud
5. ✅ Encryption and deduplication included
6. ✅ Retention policies built-in

## Next Steps (Optional)

- [ ] Test Restic in a local manual environment
- [ ] Validate backup/restore cycle with SQLite
- [ ] Validate backup/restore cycle with MySQL
- [ ] Measure backup time for 17 GB database
- [ ] Measure storage savings with deduplication
- [ ] Document restore procedures

## References

- [Restic Documentation](https://restic.readthedocs.io/en/stable/)
- [Restic Docker Image](https://hub.docker.com/r/restic/restic)
- [Restic GitHub](https://github.com/restic/restic)

## Best Practices

### Stage Files Before Backing Up

**Recommended**: Stage all files to a single directory, then have Restic back up
that directory - rather than backing up files directly from multiple paths.

```text
Source Files          Staging Directory           Remote Storage
─────────────         ─────────────────           ──────────────
.env             ─┐
tracker.toml     ─┼──► /backups/TIMESTAMP/  ───► Restic repo
prometheus.yml   ─┤        │
grafana/         ─┤        ├── config/
tracker.db       ─┘        ├── grafana-data/
                           └── tracker.db
                                   │
                                   ▼
                           backup_TIMESTAMP.tar.gz
                           (kept locally for 7 days)
```

#### Why Stage First?

| Approach                 | Pros                                                                         | Cons                                                     |
| ------------------------ | ---------------------------------------------------------------------------- | -------------------------------------------------------- |
| **Stage first → Restic** | Atomic snapshot, local backup available, simpler restore, one Restic command | Requires staging disk space                              |
| **Restic directly**      | Saves disk space                                                             | Multiple paths, potential inconsistency, complex restore |

**Benefits of staging:**

1. **Atomic Consistency**: All files are captured at approximately the same
   point in time. If Restic backed up directly from multiple paths, the config
   files and database might be from different moments.

2. **Local Backup Always Available**: The staged directory (or tar.gz) provides
   instant local recovery without needing Restic:

   ```bash
   # Quick restore from local backup (no Restic needed)
   tar -xzf /backups/backup_20260129_120000.tar.gz -C /tmp/restore/
   ```

3. **Simpler Restic Command**: One path to backup instead of multiple:

   ```bash
   # One path to backup
   restic backup /backups/20260129_120000/

   # vs. multiple paths (harder to manage)
   restic backup /data/config/.env /data/tracker/etc/ /data/prometheus/etc/ ...
   ```

4. **Easier Restore**: You restore one archive and everything is in predictable
   locations:

   ```bash
   restic restore latest --target /tmp/restore/
   # Everything is in /tmp/restore/20260129_120000/
   ```

#### Recommended Backup Script Pattern

```bash
#!/bin/bash
set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
STAGING_DIR="/backups/${TIMESTAMP}"

# 1. Stage all files to single directory
mkdir -p "$STAGING_DIR/config"

cp /data/config/.env "$STAGING_DIR/config/"
cp /data/tracker/etc/tracker.toml "$STAGING_DIR/config/"
cp /data/prometheus/etc/prometheus.yml "$STAGING_DIR/config/"
cp -r /data/grafana/provisioning "$STAGING_DIR/config/grafana-provisioning"
cp -r /data/grafana/data "$STAGING_DIR/grafana-data"

# 2. Backup database to staging
sqlite3 /data/tracker/lib/database/tracker.db ".backup '$STAGING_DIR/tracker.db'"
# Or for MySQL:
# mysqldump --single-transaction ... > "$STAGING_DIR/tracker.sql"

# 3. Create local archive (optional but recommended)
cd /backups
tar -czf "backup_${TIMESTAMP}.tar.gz" "$TIMESTAMP"
rm -rf "$TIMESTAMP"  # Keep only the archive

# 4. Push to Restic (backs up the tar.gz)
restic backup "/backups/backup_${TIMESTAMP}.tar.gz" --tag "complete-backup"

# 5. Apply retention policies
restic forget --prune --keep-daily 7 --keep-weekly 4 --keep-monthly 12

# 6. Cleanup old local archives
find /backups -name "backup_*.tar.gz" -mtime +7 -delete
```

### Use Tags for Organization

Tag snapshots for easier filtering and searching:

```bash
# Add multiple tags
restic backup /backups --tag "torrust" --tag "daily" --tag "$(hostname)"

# List only tagged snapshots
restic snapshots --tag "torrust"

# Forget with tag filtering
restic forget --tag "daily" --keep-daily 7
```

### Verify Backups Regularly

Schedule integrity checks (e.g., weekly):

```bash
# Quick check (verifies metadata and structure)
restic check

# Full check (also verifies data blobs - slower)
restic check --read-data

# Check only a subset of data (faster for large repos)
restic check --read-data-subset=10%
```

### Secure the Repository Password

The Restic repository password is critical - if lost, data cannot be recovered.

```bash
# Store password in environment variable (never in scripts)
export RESTIC_PASSWORD_FILE=/run/secrets/restic-password

# Or use password command
export RESTIC_PASSWORD_COMMAND="cat /run/secrets/restic-password"
```

For Docker deployments, use Docker secrets or environment variables from a
secure `.env` file that is itself backed up separately.

### Test Restores Periodically

Backups are only useful if they can be restored. Schedule periodic restore tests:

```bash
#!/bin/bash
# test-restore.sh - Run monthly

RESTORE_DIR="/tmp/restore-test-$(date +%Y%m%d)"

# Restore latest snapshot
restic restore latest --target "$RESTORE_DIR"

# Verify key files exist
if [ -f "$RESTORE_DIR/config/.env" ] && \
   [ -f "$RESTORE_DIR/config/tracker.toml" ] && \
   [ -f "$RESTORE_DIR/tracker.db" ]; then
    echo "✅ Restore test passed"
else
    echo "❌ Restore test FAILED - missing files!"
    exit 1
fi

# Optional: Verify database integrity
sqlite3 "$RESTORE_DIR/tracker.db" "PRAGMA integrity_check;"

# Cleanup
rm -rf "$RESTORE_DIR"
```
