# Phase 1.1b Manual Testing - Results

**Issue**: [#315 - Implement Backup Support](315-implement-backup-support.md)  
**Phase**: 1.1b (Manual Testing Checkpoint)  
**Status**: ✅ **PASSED**  
**Date**: February 2, 2026

## Test Summary

All manual E2E tests for the backup container passed successfully.

### Test 1: SQLite Database Backup ✅

**Environment**: `manual-sqlite-udp-only`  
**Database Path**: `/opt/torrust/storage/tracker/lib/database/tracker.db`

**Results**:

- ✅ Backup container ran successfully (exit code 0)
- ✅ SQLite backup created: `sqlite_20260202_141117.db.gz` (639 bytes)
- ✅ Config backup created: `config_20260202_141117.tar.gz` (1007 bytes)
- ✅ SQLite backup verified: Valid database format (contains "SQLite format 3" header)
- ✅ Config backup verified: Contains `/tracker/etc/tracker.toml`
- ✅ All services remained healthy (tracker, prometheus, grafana)

**Key Findings**:

- SQLite database is located at `/tracker/lib/database/tracker.db` (not `/tracker/lib/tracker.db`)
- Backup completed in < 1 second
- Volume mapping works correctly between container and host paths

### Test 2: MySQL Database Backup ✅

**Environment**: `manual-mysql-test`  
**Database**: `tracker@mysql:3306`

**Results**:

- ✅ Backup container ran successfully (exit code 0)
- ✅ MySQL backup created: `mysql_20260202_162652.sql.gz` (935 bytes)
- ✅ Config backup created: `config_20260202_162652.tar.gz` (1007 bytes)
- ✅ MySQL backup verified: Valid SQL dump (MariaDB dump format, contains CREATE TABLE statements)
- ✅ Config backup verified: Contains `/tracker/etc/tracker.toml`
- ✅ All services remained healthy (tracker, mysql)

**Key Findings**:

- mysqldump warning about PROCESS privilege is expected and does not affect backup quality
- Backup completed in < 1 second
- Database network isolation works correctly (backup container can access MySQL via docker network)

## Configuration Used

### SQLite Backup Configuration

```bash
DB_TYPE=sqlite
DB_PATH=/tracker/lib/database/tracker.db
BACKUP_RETENTION_DAYS=7
BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
```

### MySQL Backup Configuration

```bash
DB_TYPE=mysql
DB_HOST=mysql
DB_PORT=3306
DB_USER=tracker_user
DB_PASSWORD=tracker_password
DB_NAME=tracker
BACKUP_RETENTION_DAYS=7
BACKUP_PATHS_FILE=/etc/backup/backup-paths.txt
```

### Backup Paths File

```text
/tracker/etc/tracker.toml
```

## Docker Compose Integration

The backup service was manually added to the generated `docker-compose.yml`:

```yaml
backup:
  image: torrust/backup:test
  container_name: torrust-backup
  volumes:
    - ./backup/backup.conf:/etc/backup/backup.conf:ro
    - ./backup/backup-paths.txt:/etc/backup/backup-paths.txt:ro
    - ./storage/tracker:/tracker:ro
    - backup_mysql:/backups/mysql
    - backup_sqlite:/backups/sqlite
    - backup_config:/backups/config
  networks:
    - database_network # For MySQL access
  depends_on:
    - tracker
    - mysql # For MySQL test
  restart: "no"
```

## Backup Verification Commands

### SQLite Backup Verification

```bash
# List backups
docker run --rm -v torrust_backup_sqlite:/backups alpine ls -lah /backups

# Verify SQLite header
docker run --rm -v torrust_backup_sqlite:/backups alpine sh -c \
  'gunzip -c /backups/sqlite_*.db.gz | head -c 100'
# Expected output: "SQLite format 3..."
```

### MySQL Backup Verification

```bash
# List backups
docker run --rm -v torrust_backup_mysql:/backups alpine ls -lah /backups

# Verify SQL dump header
docker run --rm -v torrust_backup_mysql:/backups alpine sh -c \
  'gunzip -c /backups/mysql_*.sql.gz | head -20'
# Expected output: MySQL dump header with CREATE TABLE statements
```

### Config Backup Verification

```bash
# List backups
docker run --rm -v torrust_backup_config:/backups alpine ls -lah /backups

# Verify contents
docker run --rm -v torrust_backup_config:/backups alpine \
  tar -tzf /backups/config_*.tar.gz
# Expected output: tracker/etc/tracker.toml
```

## Issues Encountered and Resolved

### Issue 1: SQLite Database Path

**Problem**: Initial configuration used `/tracker/lib/tracker.db`, but the actual path was `/tracker/lib/database/tracker.db`.

**Solution**: Updated `backup.conf` with the correct path: `DB_PATH=/tracker/lib/database/tracker.db`.

**Impact**: None - discovered during testing before running backup.

### Issue 2: mysqldump PROCESS Privilege Warning

**Problem**: mysqldump shows warning: "Error: 'Access denied; you need (at least one of) the PROCESS privilege(s) for this operation' when trying to dump tablespaces".

**Analysis**: This is a known warning when the database user doesn't have the PROCESS privilege. It does not affect the backup quality - all tables and data are still backed up correctly.

**Resolution**: Warning is expected and safe to ignore. The backup contains valid SQL dump with all necessary CREATE TABLE and INSERT statements.

## Performance

- **SQLite Backup**: < 1 second (639 bytes compressed)
- **MySQL Backup**: < 1 second (935 bytes compressed)
- **Config Backup**: < 1 second (1007 bytes compressed)

All backups complete nearly instantly for test databases with minimal data.

## Conclusion

Phase 1.1b manual testing is **SUCCESSFUL**. The backup container works correctly for both SQLite and MySQL databases in real deployment scenarios.

**Key Achievements**:

1. ✅ Backup container builds and runs without errors
2. ✅ SQLite backups create valid compressed database files
3. ✅ MySQL backups create valid SQL dump files
4. ✅ Config file backups preserve absolute paths correctly
5. ✅ Backup files can be extracted and verified
6. ✅ Other services remain healthy during backup operations
7. ✅ Docker volume integration works correctly
8. ✅ Container exits cleanly after completing backup

**Ready for Next Phase**:

- Phase 1.2: GitHub workflow for publishing to Docker Hub
- Phase 1.2: Security scanning setup
- Phase 2: Rust domain/application layer integration
- Phase 3: Scheduled backups with crontab
- Phase 4: Final documentation and testing

## Test Execution Time

- SQLite test: ~15 minutes (including environment provisioning)
- MySQL test: ~2 hours (including environment provisioning and Prometheus/Grafana startup)
- Total testing time: ~2.25 hours

**Note**: Most time was spent provisioning infrastructure. Actual backup operations took < 1 second each.
