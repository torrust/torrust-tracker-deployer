# Sidecar Container Backup Solution

## Overview

This solution uses a dedicated **sidecar container** within the Docker Compose
stack to handle all database backup operations. The backup container runs
alongside the tracker and database containers, providing a clean separation
of concerns while remaining fully portable with the deployment.

## Why This Approach?

### The Problem

When considering where to execute backup commands, we identified three options:

| Option               | Description                                  | Issues                                                 |
| -------------------- | -------------------------------------------- | ------------------------------------------------------ |
| Inside app container | Add cron + backup tools to tracker container | Violates single-process principle, needs supervisord   |
| On host VM           | Run backup scripts directly on the VM        | Not portable, tied to VM, code doesn't move with stack |
| Sidecar container    | Dedicated container for backups              | ✅ Clean, portable, follows Docker best practices      |

### Why Not Inside the App Container?

Docker containers are designed to run a single process. Adding backup
functionality to the tracker container would require:

- A process supervisor (supervisord, s6, etc.)
- Installing backup tools (sqlite3, mysqldump, restic)
- Managing cron within the container
- Increased attack surface and complexity

### Why Not On the Host VM?

The current Torrust Demo uses host-based backup scripts. While functional,
this approach has significant drawbacks:

- **Not portable**: Scripts are tied to the specific VM
- **Not version controlled with the stack**: Backup logic lives outside
  the docker-compose.yml
- **Harder to test**: Cannot easily test backup in development environments
- **Different environments diverge**: Each deployment may have different
  backup implementations

### The Sidecar Solution

A dedicated backup container solves all these issues:

- ✅ **Portable**: Moves with the Docker Compose stack
- ✅ **Version controlled**: Part of the deployment configuration
- ✅ **Testable**: Same backup runs in dev, staging, and production
- ✅ **Single responsibility**: Container does one thing - backups
- ✅ **Clean separation**: Tracker focuses on tracking, backup focuses on backups

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                     Docker Compose Stack                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   tracker    │    │    mysql     │    │   backup     │       │
│  │              │    │              │    │  (sidecar)   │       │
│  │  Port: 6969  │    │  Port: 3306  │    │              │       │
│  │  Port: 7070  │    │              │    │  - cron      │       │
│  │  Port: 1212  │    │              │    │  - sqlite3   │       │
│  └──────┬───────┘    └──────┬───────┘    │  - mysqldump │       │
│         │                   │            │  - restic    │       │
│         │                   │            └──────┬───────┘       │
│         │                   │                   │               │
│         ▼                   │                   │               │
│  ┌──────────────┐           │            ┌──────┴───────┐       │
│  │ tracker_data │◄──────────┼────────────│   Volumes    │       │
│  │   (volume)   │           │            │              │       │
│  └──────────────┘           │            │ - tracker:ro │       │
│                             ▼            │ - backups:rw │       │
│                      ┌──────────────┐    └──────────────┘       │
│                      │  mysql_data  │                           │
│                      │   (volume)   │    Network connection     │
│                      └──────────────┘    for mysqldump ─────────┤
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Remote Storage  │
                    │  (via Restic)    │
                    │                  │
                    │  - S3            │
                    │  - B2            │
                    │  - SFTP          │
                    │  - Local         │
                    └──────────────────┘
```

## Key Design Decisions

### SQLite vs MySQL Access Methods

| Database | Access Method            | Why                                                     |
| -------- | ------------------------ | ------------------------------------------------------- |
| SQLite   | Volume mount (read-only) | File-based database, `.backup` needs direct file access |
| MySQL    | Network connection       | Client/server model, `mysqldump` connects via TCP       |

**Important**: For MySQL, the backup container does NOT need to mount the
mysql_data volume. It connects over the Docker network to the mysql service
and uses `mysqldump` to create logical backups.

### Read-Only Volume Mounts

The tracker_data volume is mounted as **read-only** (`:ro`) in the backup
container. This:

- Prevents accidental data corruption
- Provides an additional security layer
- Clearly documents the backup container's read-only intent

SQLite's `.backup` command works fine with read-only source access.

## Implementation

### Docker Compose Configuration

```yaml
services:
  tracker:
    image: torrust/tracker:latest
    volumes:
      - tracker_data:/var/lib/torrust/tracker
    # ... other tracker configuration

  mysql:
    image: mysql:8.0
    volumes:
      - mysql_data:/var/lib/mysql
    environment:
      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD}
      MYSQL_DATABASE: ${MYSQL_DATABASE}
      MYSQL_USER: ${MYSQL_USER}
      MYSQL_PASSWORD: ${MYSQL_PASSWORD}
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 10s
      timeout: 5s
      retries: 5

  backup:
    build:
      context: ./backup
      dockerfile: Dockerfile
    volumes:
      # SQLite: Mount tracker data (read-only)
      - tracker_data:/data/tracker:ro
      # Backup output directory
      - ./backups:/backups
    environment:
      # MySQL connection (network-based, no volume mount needed)
      MYSQL_HOST: mysql
      MYSQL_PORT: 3306
      MYSQL_DATABASE: ${MYSQL_DATABASE}
      MYSQL_USER: ${MYSQL_USER}
      MYSQL_PASSWORD: ${MYSQL_PASSWORD}
      # Restic configuration
      RESTIC_REPOSITORY: ${RESTIC_REPOSITORY}
      RESTIC_PASSWORD: ${RESTIC_PASSWORD}
      # Backup schedule (cron format)
      BACKUP_SCHEDULE: "0 * * * *" # Every hour
    depends_on:
      mysql:
        condition: service_healthy

volumes:
  tracker_data:
  mysql_data:
```

### Backup Container Dockerfile

```dockerfile
FROM alpine:3.19

# Install backup tools
RUN apk add --no-cache \
    sqlite \
    mysql-client \
    restic \
    bash \
    dcron \
    tzdata

# Create backup directories
RUN mkdir -p /backups/sqlite /backups/mysql /scripts

# Copy backup scripts
COPY scripts/ /scripts/
RUN chmod +x /scripts/*.sh

# Copy entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
```

### Entrypoint Script

```bash
#!/bin/bash
set -e

# Generate crontab from environment variable
echo "${BACKUP_SCHEDULE} /scripts/backup-all.sh >> /var/log/backup.log 2>&1" > /etc/crontabs/root

# Start cron in foreground
exec crond -f -d 8
```

### SQLite Backup Script

```bash
#!/bin/bash
# /scripts/backup-sqlite.sh

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/sqlite"
SOURCE_DB="/data/tracker/tracker.db"
BACKUP_FILE="${BACKUP_DIR}/tracker_${TIMESTAMP}.db"

echo "[$(date)] Starting SQLite backup..."

# Use SQLite's .backup command for consistent backup
sqlite3 "$SOURCE_DB" ".backup '$BACKUP_FILE'"

# Verify backup
if [ -s "$BACKUP_FILE" ]; then
    echo "[$(date)] SQLite backup successful: $BACKUP_FILE"
    echo "[$(date)] Size: $(du -h "$BACKUP_FILE" | cut -f1)"
else
    echo "[$(date)] ERROR: SQLite backup failed or empty!"
    exit 1
fi

# Clean old backups (keep 7 days locally)
find "$BACKUP_DIR" -name "tracker_*.db" -mtime +7 -delete
```

### MySQL Backup Script

```bash
#!/bin/bash
# /scripts/backup-mysql.sh

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/mysql"
BACKUP_FILE="${BACKUP_DIR}/mysql_${TIMESTAMP}.sql.gz"

echo "[$(date)] Starting MySQL backup..."

# Use mysqldump with --single-transaction for lock-free InnoDB backup
mysqldump \
    --host="$MYSQL_HOST" \
    --port="$MYSQL_PORT" \
    --user="$MYSQL_USER" \
    --password="$MYSQL_PASSWORD" \
    --single-transaction \
    --routines \
    --triggers \
    "$MYSQL_DATABASE" | gzip > "$BACKUP_FILE"

# Verify backup
if [ -s "$BACKUP_FILE" ]; then
    echo "[$(date)] MySQL backup successful: $BACKUP_FILE"
    echo "[$(date)] Size: $(du -h "$BACKUP_FILE" | cut -f1)"
else
    echo "[$(date)] ERROR: MySQL backup failed or empty!"
    exit 1
fi

# Clean old backups (keep 7 days locally)
find "$BACKUP_DIR" -name "mysql_*.sql.gz" -mtime +7 -delete
```

### Unified Backup Script with Restic

```bash
#!/bin/bash
# /scripts/backup-all.sh

set -e

echo "============================================"
echo "[$(date)] Starting backup job..."
echo "============================================"

# Detect which database is in use
if [ -f "/data/tracker/tracker.db" ]; then
    echo "[$(date)] SQLite database detected"
    /scripts/backup-sqlite.sh
    BACKUP_SOURCE="/backups/sqlite"
elif [ -n "$MYSQL_HOST" ]; then
    echo "[$(date)] MySQL database detected"
    /scripts/backup-mysql.sh
    BACKUP_SOURCE="/backups/mysql"
else
    echo "[$(date)] ERROR: No database detected!"
    exit 1
fi

# Push to remote storage with Restic (if configured)
if [ -n "$RESTIC_REPOSITORY" ]; then
    echo "[$(date)] Pushing to Restic repository..."

    # Initialize repo if needed (idempotent)
    restic init 2>/dev/null || true

    # Backup to Restic
    restic backup "$BACKUP_SOURCE" --tag "tracker-backup"

    # Prune old backups (keep 7 daily, 4 weekly, 12 monthly)
    restic forget --prune \
        --keep-daily 7 \
        --keep-weekly 4 \
        --keep-monthly 12

    echo "[$(date)] Restic backup complete"
else
    echo "[$(date)] Restic not configured, keeping local backups only"
fi

echo "============================================"
echo "[$(date)] Backup job complete"
echo "============================================"
```

## Advantages of This Solution

### 1. Portability

The entire backup solution is defined in:

- `docker-compose.yml` - Service definition
- `backup/Dockerfile` - Container image
- `backup/scripts/` - Backup logic

All of this moves with the deployment. Clone the repo, run `docker compose up`,
and backups are automatically configured.

### 2. Consistency Across Environments

The same backup container runs in:

- Development (your laptop)
- Staging
- Production

No environment-specific backup scripts to maintain.

### 3. Testability

```bash
# Test backup manually
docker compose exec backup /scripts/backup-all.sh

# Check backup logs
docker compose logs backup

# Verify backup files
ls -la ./backups/
```

### 4. Clean Separation

| Container | Responsibility                    |
| --------- | --------------------------------- |
| tracker   | Serve BitTorrent tracker requests |
| mysql     | Store persistent data             |
| backup    | Create and manage backups         |

Each container does one thing well.

### 5. Security

- Backup container has **read-only** access to tracker data
- MySQL credentials are passed via environment variables
- Restic encrypts all data at rest
- No backup tools installed in production containers

## Comparison with Current Demo Approach

| Aspect          | Host VM Scripts (Current) | Sidecar Container (Proposed)      |
| --------------- | ------------------------- | --------------------------------- |
| Portability     | ❌ Tied to VM             | ✅ Moves with stack               |
| Version Control | ❌ Scripts outside repo   | ✅ Part of deployment             |
| Testing         | ❌ Hard to test           | ✅ Same in all environments       |
| Maintenance     | ❌ Manual updates per VM  | ✅ Update once, deploy everywhere |
| Complexity      | ✅ Simple for single VM   | ✅ Simple for Docker deployments  |

## When NOT to Use This

This solution is designed for Docker Compose deployments. It may not be
appropriate for:

- Kubernetes deployments (use CronJobs or backup operators instead)
- Non-containerized deployments (use host scripts)
- Managed database services (use provider's backup features)

## Files to Backup

A complete backup must include both **database data** and **configuration files**.
The deployer creates a specific directory structure on the host that maps to
container paths via Docker bind mounts.

### Host Directory Structure

The deployer creates the following structure at `/opt/torrust/`:

```text
/opt/torrust/                              # Base deployment directory
├── .env                                   # ⚠️ CRITICAL - Environment variables (secrets!)
├── docker-compose.yml                     # Docker Compose configuration
└── storage/                               # All persistent data
    ├── tracker/
    │   ├── etc/
    │   │   └── tracker.toml               # Tracker configuration
    │   ├── lib/
    │   │   └── database/
    │   │       └── tracker.db             # SQLite database (if using SQLite)
    │   └── log/                           # Tracker logs (optional backup)
    ├── mysql/
    │   └── data/                          # MySQL data directory (if using MySQL)
    │       └── ...                        # (backed up via mysqldump, not file copy)
    ├── prometheus/
    │   └── etc/
    │       └── prometheus.yml             # Prometheus configuration
    └── grafana/
        ├── data/                          # Grafana database and state
        └── provisioning/
            ├── datasources/
            │   └── prometheus.yml         # Datasource configuration
            └── dashboards/
                ├── torrust.yml            # Dashboard provider config
                └── torrust/
                    ├── metrics.json       # Metrics dashboard
                    └── stats.json         # Stats dashboard
```

### Backup Categories

#### 1. Critical - Must Backup

| Path                                                   | Description                        | Method            |
| ------------------------------------------------------ | ---------------------------------- | ----------------- |
| `/opt/torrust/.env`                                    | Environment variables with secrets | File copy         |
| `/opt/torrust/storage/tracker/lib/database/tracker.db` | SQLite database                    | `.backup` command |
| `/opt/torrust/storage/tracker/etc/tracker.toml`        | Tracker configuration              | File copy         |
| MySQL database (if enabled)                            | Tracker data in MySQL              | `mysqldump`       |

#### 2. Important - Recommended Backup

| Path                                                 | Description                         | Method                |
| ---------------------------------------------------- | ----------------------------------- | --------------------- |
| `/opt/torrust/storage/prometheus/etc/prometheus.yml` | Prometheus config                   | File copy             |
| `/opt/torrust/storage/grafana/provisioning/`         | Dashboard definitions               | File copy (recursive) |
| `/opt/torrust/storage/grafana/data/`                 | Grafana state and custom dashboards | File copy             |

#### 3. Regenerable - No Backup Needed

| Path                              | Description              | Why Skip                      |
| --------------------------------- | ------------------------ | ----------------------------- |
| `/opt/torrust/docker-compose.yml` | Compose configuration    | Generated by deployer         |
| Ansible playbooks                 | Configuration management | Never copied to instance      |
| Container images                  | Docker images            | Pulled from registry          |
| Prometheus data                   | Metrics time series      | Ephemeral, can be regenerated |

### Host Path vs Container Path Mapping

The backup container must understand the mapping between host paths
(where files exist on the VM) and container paths (where files are mounted):

| Host Path                                   | Container Mount             | Purpose              |
| ------------------------------------------- | --------------------------- | -------------------- |
| `/opt/torrust/storage/tracker/lib`          | `/var/lib/torrust/tracker`  | Tracker data         |
| `/opt/torrust/storage/tracker/etc`          | `/etc/torrust/tracker`      | Tracker config       |
| `/opt/torrust/storage/tracker/log`          | `/var/log/torrust/tracker`  | Tracker logs         |
| `/opt/torrust/storage/prometheus/etc`       | `/etc/prometheus`           | Prometheus config    |
| `/opt/torrust/storage/grafana/data`         | `/var/lib/grafana`          | Grafana data         |
| `/opt/torrust/storage/grafana/provisioning` | `/etc/grafana/provisioning` | Grafana provisioning |
| `/opt/torrust/storage/mysql/data`           | `/var/lib/mysql`            | MySQL data           |

**Important**: The backup container operates on **host paths**, not container
paths. It mounts the same host directories that other containers use.

### Updated Docker Compose for Complete Backup

```yaml
services:
  backup:
    build:
      context: ./backup
      dockerfile: Dockerfile
    volumes:
      # Configuration files (read-only)
      - /opt/torrust/.env:/data/config/.env:ro
      - /opt/torrust/storage/tracker/etc:/data/tracker/etc:ro
      - /opt/torrust/storage/prometheus/etc:/data/prometheus/etc:ro
      - /opt/torrust/storage/grafana/provisioning:/data/grafana/provisioning:ro

      # Database files (read-only for backup)
      - /opt/torrust/storage/tracker/lib:/data/tracker/lib:ro
      - /opt/torrust/storage/grafana/data:/data/grafana/data:ro

      # Backup output directory
      - /opt/torrust/backups:/backups
    environment:
      # MySQL connection (network-based, not volume)
      MYSQL_HOST: mysql
      MYSQL_PORT: 3306
      MYSQL_DATABASE: ${MYSQL_DATABASE}
      MYSQL_USER: ${MYSQL_USER}
      MYSQL_PASSWORD: ${MYSQL_PASSWORD}
      # Database type detection
      DATABASE_DRIVER: ${TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER}
      # Restic configuration
      RESTIC_REPOSITORY: ${RESTIC_REPOSITORY}
      RESTIC_PASSWORD: ${RESTIC_PASSWORD}
```

### Complete Backup Script

This script follows the **staging pattern** recommended for Restic backups.
All files are first copied to a staging directory, then archived, and finally
backed up to Restic. This ensures atomic consistency and provides local backups.

> **📖 See [Restic Best Practices](../tools/restic.md#best-practices)** for
> detailed rationale on why staging files before Restic backup is recommended
> over backing up directly from multiple source paths.

```bash
#!/bin/bash
# /scripts/backup-all.sh - Complete backup of database and configuration

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/${TIMESTAMP}"

echo "[$(date)] Starting complete backup..."
mkdir -p "$BACKUP_DIR"

# 1. Backup configuration files
echo "[$(date)] Backing up configuration files..."
mkdir -p "$BACKUP_DIR/config"

cp /data/config/.env "$BACKUP_DIR/config/.env"
cp /data/tracker/etc/tracker.toml "$BACKUP_DIR/config/tracker.toml"
cp /data/prometheus/etc/prometheus.yml "$BACKUP_DIR/config/prometheus.yml"
cp -r /data/grafana/provisioning "$BACKUP_DIR/config/grafana-provisioning"

# 2. Backup Grafana data (custom dashboards, users, etc.)
echo "[$(date)] Backing up Grafana data..."
cp -r /data/grafana/data "$BACKUP_DIR/grafana-data"

# 3. Backup database
echo "[$(date)] Backing up database..."

if [ "$DATABASE_DRIVER" = "sqlite3" ]; then
    # SQLite backup using .backup command
    sqlite3 /data/tracker/lib/database/tracker.db ".backup '$BACKUP_DIR/tracker.db'"
    echo "[$(date)] SQLite backup complete"
elif [ "$DATABASE_DRIVER" = "mysql" ]; then
    # MySQL backup using mysqldump
    mysqldump \
        --host="$MYSQL_HOST" \
        --port="$MYSQL_PORT" \
        --user="$MYSQL_USER" \
        --password="$MYSQL_PASSWORD" \
        --single-transaction \
        --routines \
        --triggers \
        "$MYSQL_DATABASE" > "$BACKUP_DIR/tracker.sql"
    echo "[$(date)] MySQL backup complete"
fi

# 4. Create archive
echo "[$(date)] Creating backup archive..."
cd /backups
tar -czf "backup_${TIMESTAMP}.tar.gz" "$TIMESTAMP"
rm -rf "$TIMESTAMP"

echo "[$(date)] Backup archive created: backup_${TIMESTAMP}.tar.gz"

# 5. Push to remote storage (if Restic configured)
if [ -n "$RESTIC_REPOSITORY" ]; then
    echo "[$(date)] Pushing to Restic repository..."
    restic init 2>/dev/null || true
    restic backup "/backups/backup_${TIMESTAMP}.tar.gz" --tag "complete-backup"
    restic forget --prune --keep-daily 7 --keep-weekly 4 --keep-monthly 12
fi

# 6. Cleanup old local backups
find /backups -name "backup_*.tar.gz" -mtime +7 -delete

echo "[$(date)] Backup complete!"
```

### Backup Archive Contents

A complete backup archive contains:

```text
backup_20260129_120000.tar.gz
└── 20260129_120000/
    ├── config/
    │   ├── .env                           # Environment variables
    │   ├── tracker.toml                   # Tracker configuration
    │   ├── prometheus.yml                 # Prometheus configuration
    │   └── grafana-provisioning/          # Grafana dashboards
    │       ├── datasources/
    │       │   └── prometheus.yml
    │       └── dashboards/
    │           ├── torrust.yml
    │           └── torrust/
    │               ├── metrics.json
    │               └── stats.json
    ├── grafana-data/                      # Grafana state
    │   └── grafana.db                     # (if exists)
    └── tracker.db                         # SQLite database
        (or tracker.sql for MySQL)
```

## Next Steps

1. [ ] Create backup container Dockerfile
2. [ ] Implement backup scripts
3. [ ] Add to Docker Compose templates
4. [ ] Test with SQLite configuration
5. [ ] Test with MySQL configuration
6. [ ] Document restore procedures
7. [ ] Add monitoring/alerting for backup failures

## References

- [SQLite Backup API](https://www.sqlite.org/backup.html)
- [MySQL --single-transaction](https://dev.mysql.com/doc/refman/8.0/en/mysqldump.html#option_mysqldump_single-transaction)
- [Restic Documentation](https://restic.readthedocs.io/)
- [Docker Compose Volumes](https://docs.docker.com/compose/compose-file/07-volumes/)
