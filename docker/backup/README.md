# Torrust Backup Container

Production backup container for Torrust Tracker deployments. This container performs automated backups of MySQL/SQLite databases and configuration files.

## Features

- **Database Backup**: Supports MySQL (via mysqldump) and SQLite (via .backup command)
- **Config File Backup**: Archives specified configuration files and directories
- **Compression**: All backups are gzip-compressed to save storage
- **Retention Management**: Automatic cleanup of backups older than configured retention period
- **Config-Driven**: All behavior controlled via mounted configuration files (no environment variables)
- **Tested**: 58 unit tests run during container build

## Configuration

The container is configured via two mounted files:

### 1. Main Configuration (`/etc/backup/backup.conf`)

Bash-sourceable configuration file with key=value pairs:

```bash
# Days to keep old backups before deletion
BACKUP_RETENTION_DAYS=7

# Path to file containing list of paths to backup
BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt

# Database type: mysql, sqlite, or none
DB_TYPE=mysql

# MySQL configuration (required if DB_TYPE=mysql)
DB_HOST=mysql
DB_PORT=3306
DB_USER=tracker_user
DB_PASSWORD=tracker_password
DB_NAME=torrust_tracker

# SQLite configuration (required if DB_TYPE=sqlite)
DB_PATH=/data/storage/tracker/lib/tracker.db
```

### 2. Backup Paths File (`/etc/backup/backup-paths.txt`)

Plain text file listing paths to backup (one per line):

```text
# Comments start with #
/data/storage/tracker/etc/tracker.toml
/data/storage/prometheus/etc/prometheus.yml
/data/storage/grafana/provisioning
/data/storage/caddy/etc/Caddyfile
```

## Volume Mounts

| Mount Point                    | Purpose                             | Mode       |
| ------------------------------ | ----------------------------------- | ---------- |
| `/backups`                     | Output directory for all backups    | Read-Write |
| `/data`                        | Source data directory (app storage) | Read-Only  |
| `/etc/backup/backup.conf`      | Main configuration file             | Read-Only  |
| `/etc/backup/backup-paths.txt` | Paths to backup                     | Read-Only  |

## Output Structure

```text
/backups/
├── mysql/
│   ├── mysql_20260201_030000.sql.gz
│   └── mysql_20260202_030000.sql.gz
├── sqlite/
│   ├── sqlite_20260201_030000.db.gz
│   └── sqlite_20260202_030000.db.gz
└── config/
    ├── config_20260201_030000.tar.gz
    └── config_20260202_030000.tar.gz
```

## Usage

### Running a Backup

The container runs once and exits:

```bash
docker run --rm \
  -v /path/to/backup.conf:/etc/backup/backup.conf:ro \
  -v /path/to/backup-paths.txt:/etc/backup/backup-paths.txt:ro \
  -v /opt/torrust/storage:/data:ro \
  -v /opt/torrust/storage/backup:/backups \
  torrust/backup:latest
```

### With Docker Compose

```yaml
services:
  backup:
    image: torrust/backup:latest
    container_name: backup
    restart: "no"
    volumes:
      - ./backup/backup.conf:/etc/backup/backup.conf:ro
      - ./backup/backup-paths.txt:/etc/backup/backup-paths.txt:ro
      - ./storage:/data:ro
      - ./storage/backup:/backups
    networks:
      - database_network # Only if MySQL backup enabled
    depends_on:
      mysql:
        condition: service_healthy # Only if MySQL backup enabled
```

## Database-Specific Notes

### MySQL Backup

- Uses `mysqldump` with `--single-transaction` for consistent snapshots
- Requires network connectivity to MySQL service
- Container must wait for MySQL to be healthy before starting

### SQLite Backup

- Uses SQLite's `.backup` command for online backup
- No network required (file-based access via volume)
- Safe to run while tracker is accessing the database

## Security

- Container runs as non-root user (UID 1000, username: `torrust`)
- Backup files inherit the same ownership as application files
- Database credentials stored in config file (mounted read-only)

## Building the Container

### Docker Build Context

The backup container build uses a specific build context strategy to maintain consistency between local development and CI environments. This prevents subtle build failures caused by context/path mismatches.

**Critical Concept**: In Docker, `COPY` and `ADD` commands resolve paths **relative to the build context**, NOT relative to the Dockerfile location. Understanding this is essential to prevent regression.

### Local Build

```bash
cd /path/to/docker/backup
docker build -t torrust/backup:latest .
```

Or from repository root:

```bash
# Correct: context is ./docker/backup directory
docker build -f docker/backup/Dockerfile -t torrust/backup:latest docker/backup

# Incorrect: would break COPY commands
docker build -f docker/backup/Dockerfile -t torrust/backup:latest .
```

**Key Point**: When you run `docker build -t image:tag <CONTEXT>`, the `<CONTEXT>` path becomes the root for all COPY/ADD commands in the Dockerfile. Our Dockerfile uses simple relative paths (`COPY backup.sh`) which work when context is `./docker/backup`.

### CI/GitHub Workflow Build

The GitHub Actions workflow specifies build context explicitly:

```yaml
uses: docker/build-push-action@v6
with:
  context: ./docker/backup # Build context is the docker/backup directory
  file: ./docker/backup/Dockerfile
  # ...
```

This ensures the CI build behaves identically to local builds.

### Why This Matters

**Previous Regression (commit 9d297cc5)**:

- Workflow used `context: .` (repository root)
- Dockerfile had to use full paths: `COPY docker/backup/backup.sh /scripts/backup.sh`
- This was confusing and error-prone because paths looked like they were from the root

**Current Approach**:

- Workflow uses `context: ./docker/backup`
- Dockerfile uses natural relative paths: `COPY backup.sh /scripts/backup.sh`
- Both local builds and CI builds work identically
- Future developers won't accidentally change the context and break builds

## Testing

Tests run automatically during container build:

```bash
docker build -t torrust/backup:latest .
```

Build fails if any test fails. To run tests manually:

```bash
cd /path/to/docker/backup
bats backup_test.bats
```

## Retention Policy

The cleanup process:

1. Runs after each backup cycle
2. Finds backups older than `BACKUP_RETENTION_DAYS`
3. Deletes old backups from all backup directories (mysql, sqlite, config)
4. Logs count of deleted files

Example: With `BACKUP_RETENTION_DAYS=7`, backups older than 7 days are deleted.

## Troubleshooting

### Container exits immediately

Check configuration:

```bash
docker logs backup
```

Common issues:

- Config file not found
- Required variables missing (e.g., `DB_HOST` for MySQL)
- Database file not found (for SQLite)
- Paths file not found

### MySQL connection fails

Ensure:

- MySQL service is healthy before backup starts
- Container is on the same network as MySQL
- Credentials are correct
- Database name exists

### SQLite backup fails

Ensure:

- `DB_PATH` points to actual database file
- Path is accessible from container (check volume mount)
- Database file is not corrupted

### Backup files have wrong permissions

The container runs as UID 1000. Ensure:

- Host backup directory is writable by UID 1000
- Or adjust `BACKUP_UID` build arg when building image

## Development

### Build Arguments

```bash
docker build \
  --build-arg BACKUP_UID=1000 \
  --build-arg BACKUP_GID=1000 \
  -t torrust/backup:latest \
  .
```

### Local Testing

```bash
# Build image
docker build -t torrust/backup:test .

# Create test config
mkdir -p test-backup
cat > test-backup/backup.conf <<EOF
BACKUP_RETENTION_DAYS=7
DB_TYPE=none
EOF

# Run backup
docker run --rm \
  -v $(pwd)/test-backup/backup.conf:/etc/backup/backup.conf:ro \
  -v $(pwd)/test-backup:/backups \
  torrust/backup:test
```

## License

See [LICENSE](../../LICENSE) file in repository root.

## Related Documentation

- [Backup Strategy Research](../../docs/research/backup-strategies/)
- [Issue #315: Implement Backup Support](../../docs/issues/315-implement-backup-support.md)
- [Manual E2E Testing Guide](../../docs/e2e-testing/manual/)
