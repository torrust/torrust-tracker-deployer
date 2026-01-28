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
