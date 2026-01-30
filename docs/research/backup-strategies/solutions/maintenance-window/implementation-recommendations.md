# Lessons Learned and Implementation Concerns

**Document**: Practical insights from the Maintenance Window backup POC
**Related**: [README.md](README.md) | [Issue #310](../../../issues/310-research-database-backup-strategies.md)
**Status**: Research findings for future implementation

## Overview

This document captures practical concerns, edge cases, and lessons learned during
the POC implementation. These insights should inform future implementation
decisions without prescribing a specific approach.

## Template Complexity Concerns

### Database-Specific Environment Variables

The backup service in `docker-compose.yml` requires different environment
variables depending on the database type:

**MySQL Configuration:**

```yaml
backup:
  environment:
    - BACKUP_MYSQL_ENABLED=true
    - BACKUP_SQLITE_ENABLED=false
    - MYSQL_HOST=mysql
    - MYSQL_PORT=3306
    - MYSQL_DATABASE=${MYSQL_DATABASE}
    - MYSQL_USER=${MYSQL_USER}
    - MYSQL_PASSWORD=${MYSQL_PASSWORD}
```

**SQLite Configuration:**

```yaml
backup:
  environment:
    - BACKUP_MYSQL_ENABLED=false
    - BACKUP_SQLITE_ENABLED=true
    - SQLITE_DATABASE_PATH=/data/storage/tracker/lib/database/tracker.db
```

**Concern**: Templates will need conditionals to include/exclude these blocks.
This adds complexity to the Tera template and makes it harder to read.

**Possible approaches:**

1. Use Tera conditionals (adds template complexity)
2. Generate separate template files per database type
3. Use a template composition pattern (base + database-specific includes)

### Network Dependencies Differ

MySQL deployments need `database_network` in the backup service, SQLite does not:

```yaml
# MySQL: backup needs database_network to reach mysql container
backup:
  networks:
    - database_network
  depends_on:
    mysql:
      condition: service_healthy

# SQLite: no network needed, just depends on tracker
backup:
  depends_on:
    tracker:
      condition: service_healthy
```

**Concern**: The `networks` and `depends_on` sections change based on database.

## Edge Cases Discovered

### SSH Connection Failures

During testing, we encountered SSH "Too many authentication failures" errors.
The SSH agent had multiple keys loaded, and the server rejected the connection
before trying the correct key.

**Lesson**: When automating SSH connections, use `-o IdentitiesOnly=yes` to
prevent the SSH agent from trying all loaded keys.

**Deployer implication**: The SSH client implementation should consider this
for environments with multiple SSH keys.

### VM User vs Deploy User

The test environment uses `torrust` as the SSH user, but we initially tried
`ubuntu`. The deployment path is `/opt/torrust`, not `/home/ubuntu/...`.

**Lesson**: The deployer already stores this information in the environment
state. Always use `cargo run -- show <env-name>` to get correct connection
details rather than assuming.

**Documentation need**: Document that `/opt/torrust` is the standard deploy
directory (already in `templates/ansible/variables.yml.tera`).

### Backup Container Exits Immediately in Single Mode

When `BACKUP_MODE=single`, the container runs once and exits. This means:

1. `docker compose up -d backup` starts and stops almost immediately
2. `docker compose ps` shows no backup container running
3. `docker compose exec backup ...` fails because container isn't running

**Lesson**: For manual testing, use `docker compose run --rm backup` instead
of `docker compose up`. The crontab approach handles this correctly.

### Cron is Pre-installed on Ubuntu

We confirmed that `cron` package is pre-installed on Ubuntu cloud images.
No need to add installation steps.

**Lesson**: Can assume cron availability on Ubuntu VMs without explicit
installation playbook.

### SQLite Database Path Varies

The SQLite database path depends on tracker configuration:

- Default: `/var/lib/torrust/tracker/tracker.db` (inside container)
- Mounted as: `/opt/torrust/storage/tracker/lib/database/tracker.db` (host)
- Backup sees: `/data/storage/tracker/lib/database/tracker.db` (container mount)

**Concern**: Multiple path representations for the same file. The backup
container needs the path as seen from its mount point, not the host path.

**Lesson**: Document path translation clearly. Consider using a single
source of truth for paths.

## Configuration Validation Discoveries

### Missing `database_name` for SQLite

The environment config initially failed validation because `database_name`
was missing for SQLite. The schema requires it even though SQLite typically
uses a file path.

**Lesson**: The Rust config types in `src/application/command_handlers/create/config/`
have richer validation than the JSON schema. Always check Rust types.

### Empty Environment Variables Fall Back to Defaults

When we tested `SQLITE_DATABASE_PATH=` (empty), the container used the
Dockerfile default value instead of failing.

**Behavior**: This is correct Docker behavior - empty string in compose
doesn't override Dockerfile ENV.

**Concern**: This could mask configuration errors. Consider explicit
validation in the backup script.

## Backup Script Observations

### Compression Size is Minimal for Small Databases

All test backups were ~4KB compressed (639 bytes for SQLite, similar for MySQL).
This is because test databases have minimal data.

**Note for testing**: Don't use backup size as a success indicator. Check
actual content instead.

### The `.backup` SQLite Command Creates Locked Copy

We used `sqlite3 "$db" ".backup '$temp_file'"` which uses SQLite's backup API.
This is safe for concurrent access but may be slow for large databases.

**Trade-off**: Safe but potentially slow. The maintenance window approach
avoids this by stopping the tracker first.

### Retention Cleanup Uses Find with -mtime

```bash
find "$backup_dir" -name "*.gz" -type f -mtime +$retention_days -delete
```

**Concern**: This uses modification time, not creation time. If someone
touches a backup file, it won't be deleted on schedule.

**Lesson**: For production, consider storing backup metadata separately
or using filename-based date parsing.

## Docker Compose Observations

### Build Context for Backup Container

The backup service uses `build: context: ./backup` which requires the
backup directory structure:

```text
/opt/torrust/
├── backup/
│   ├── Dockerfile
│   └── backup.sh
├── docker-compose.yml
└── storage/backup/...
```

**Concern**: First `docker compose up` will build the image. Subsequent
runs use cache. If backup.sh changes, need `docker compose build backup`.

**Alternative**: Pre-build image and push to registry. Adds complexity
but avoids build-on-deploy issues.

### Volume Mounts Need Pre-existing Directories

Docker won't create host directories for bind mounts with proper ownership.
Ansible must create directories before first compose run.

**Lesson**: Storage directory creation is a prerequisite, not optional.

## Crontab Observations

### Root vs User Crontab

We installed crontab as root because:

1. Docker commands typically need root or docker group
2. Stopping/starting containers requires privileges

**Alternative**: Add deploy user to docker group. Security trade-off.

**Concern**: If deployer runs as non-root, crontab installation needs
privilege escalation (sudo or become in Ansible).

### Log Rotation Not Configured

The crontab appends to `/var/log/tracker-backup.log` indefinitely.

**Missing**: Log rotation configuration. Should add logrotate config
or use journald.

### Testing Crontab with Short Intervals

The `maintenance-backup-test.cron` uses 2-minute intervals for testing.
This worked well for verification but isn't suitable for production.

**Lesson**: Keep test configurations as artifacts but clearly mark them
as non-production.

## What Worked Well

### Single Mode Default

Changing `BACKUP_MODE` default from `continuous` to `single` was the right
choice. The container now behaves like certbot - run once, do the job, exit.

### Consistent Logging Format

Both MySQL and SQLite now show database details conditionally:

```text
[timestamp]   MySQL backup: true
[timestamp]   Database: torrust_tracker@mysql:3306
```

```text
[timestamp]   SQLite backup: true
[timestamp]   SQLite database: /data/storage/tracker/lib/database/tracker.db
```

This makes logs readable regardless of database type.

### Unit Tests in Bats

The 58 Bats unit tests caught several issues during development. The
testing approach (source script functions, test individually) works well.

### Dry-Run Mode

The `DRY_RUN=true` option in maintenance-backup.sh allows testing the
orchestration without affecting services. Valuable for debugging.

## Open Questions for Implementation

### Should Backup be Optional?

Current assumption: backup is always configured. Alternative: backup is
an optional feature that users enable explicitly.

**Trade-off**: Optional adds complexity but respects user choice.

### How to Handle Backup Failures?

Current behavior: Log error, continue, restart tracker anyway.

Questions:

- Should failures send notifications?
- Should consecutive failures trigger alerts?
- Should backup be retried on failure?

### How to Verify Backup Integrity?

Current POC doesn't verify backups are restorable. Production should:

- Test that MySQL dump can be imported
- Test that SQLite file is valid
- Consider periodic restore tests

### How to Handle Existing Environments?

Environments created before backup support won't have backup infrastructure.
Options:

- Detect and warn
- Auto-configure on next `configure` run
- Require explicit migration command

## Summary of Pain Points

| Pain Point                        | Severity | Notes                                 |
| --------------------------------- | -------- | ------------------------------------- |
| Template conditionals for DB type | Medium   | Affects compose and backup config     |
| Path translation (host/container) | Medium   | Multiple representations of same path |
| SSH agent key selection           | Low      | Solved with IdentitiesOnly            |
| Container exits in single mode    | Low      | Expected behavior, just surprising    |
| Log rotation missing              | Low      | Easy to add, often forgotten          |
| Backup verification missing       | Medium   | Important for production              |

## Artifacts Reference

All POC artifacts are in [`artifacts/`](artifacts/):

| File                                    | Purpose            | Production-Ready?  |
| --------------------------------------- | ------------------ | ------------------ |
| `backup-container/backup.sh`            | Backup logic       | ✅ Yes             |
| `backup-container/Dockerfile`           | Container image    | ✅ Yes             |
| `backup-container/backup_test.bats`     | Unit tests         | ✅ Yes             |
| `docker-compose-with-backup-mysql.yml`  | MySQL example      | ⚠️ Template needed |
| `docker-compose-with-backup-sqlite.yml` | SQLite example     | ⚠️ Template needed |
| `maintenance-backup.sh`                 | Host orchestration | ✅ Yes             |
| `maintenance-backup.cron`               | Production crontab | ✅ Yes             |
| `maintenance-backup-test.cron`          | Test crontab       | ❌ Testing only    |
