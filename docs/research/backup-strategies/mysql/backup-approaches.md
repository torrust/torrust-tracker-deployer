# MySQL Backup Approaches

This document provides comprehensive information about MySQL backup strategies,
focusing on production-safe methods that work without interrupting database
operations.

## Key Question: Do We Need Locks?

**Short answer: No, for InnoDB tables.**

MySQL with InnoDB storage engine supports **lock-free backups** using the
`--single-transaction` option with `mysqldump`. This is comparable to SQLite's
`.backup` command.

### How `--single-transaction` Works

```bash
mysqldump --single-transaction --databases tracker_db > backup.sql
```

What happens:

1. Sets transaction isolation to `REPEATABLE READ`
2. Issues `START TRANSACTION` before dumping
3. Reads consistent snapshot of database at transaction start time
4. Does **NOT** lock tables (for InnoDB)
5. Other connections can read AND write during backup

**This is the MySQL equivalent of SQLite's Online Backup API!**

### Torrust Tracker Uses InnoDB by Default

The Torrust Tracker does not explicitly specify a storage engine in its table
creation SQL. However, since **MySQL 5.5.5+ uses InnoDB as the default storage
engine**, all tracker tables are automatically created as InnoDB tables.

You can verify this by running:

```bash
docker compose exec mysql mysql -u root -p -e "
SELECT TABLE_NAME, ENGINE
FROM information_schema.TABLES
WHERE TABLE_SCHEMA = 'torrust_tracker';"
```

Expected output:

```text
+----------------------------+--------+
| TABLE_NAME                 | ENGINE |
+----------------------------+--------+
| keys                       | InnoDB |
| torrents                   | InnoDB |
| torrent_aggregate_metrics  | InnoDB |
| whitelist                  | InnoDB |
+----------------------------+--------+
```

**Bottom line**: `--single-transaction` works out-of-the-box with Torrust Tracker.

## Backup Methods Comparison

### Overview Table

| Method                           | Type     | Locking        | Speed | Consistency          | Use Case         |
| -------------------------------- | -------- | -------------- | ----- | -------------------- | ---------------- |
| `mysqldump --single-transaction` | Logical  | ❌ No lock     | Slow  | ✅ ACID              | Small-medium DBs |
| `mysqldump` (default)            | Logical  | 🔒 Table locks | Slow  | ✅ ACID              | MyISAM tables    |
| Percona XtraBackup               | Physical | ❌ No lock\*   | Fast  | ✅ ACID              | Large DBs        |
| MySQL Enterprise Backup          | Physical | ❌ No lock\*   | Fast  | ✅ ACID              | Enterprise       |
| File copy                        | Physical | 🔒 Full lock   | Fast  | ⚠️ Requires shutdown | Testing only     |

\*Brief lock for non-InnoDB tables if present

## Method 1: mysqldump with --single-transaction (Recommended)

### When to Use

- All tables are InnoDB (default in MySQL 8.0+)
- Database size is manageable (under ~50GB for daily backups)
- Acceptable backup/restore time
- Simple deployment requirements

### Complete Command

```bash
mysqldump \
  --single-transaction \
  --routines \
  --triggers \
  --events \
  --quick \
  --opt \
  --databases tracker_db \
  > tracker_backup.sql
```

### Option Explanation

| Option                 | Purpose                                  |
| ---------------------- | ---------------------------------------- |
| `--single-transaction` | No locks for InnoDB, consistent snapshot |
| `--routines`           | Include stored procedures/functions      |
| `--triggers`           | Include triggers                         |
| `--events`             | Include scheduled events                 |
| `--quick`              | Don't buffer entire table in memory      |
| `--opt`                | Optimizations for faster restore         |

### Compressed Backup

```bash
mysqldump --single-transaction --databases tracker_db | gzip > backup.sql.gz
```

### Container Example

```bash
docker exec mysql-container mysqldump \
  --single-transaction \
  -u root -p"$MYSQL_ROOT_PASSWORD" \
  tracker_db > backup.sql
```

### Docker Compose Backup Script

```bash
#!/bin/bash
# backup-mysql.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/mysql"
DB_NAME="tracker_db"

# Create backup
docker compose exec -T mysql mysqldump \
  --single-transaction \
  --routines \
  --triggers \
  -u root -p"$MYSQL_ROOT_PASSWORD" \
  "$DB_NAME" | gzip > "$BACKUP_DIR/backup_$TIMESTAMP.sql.gz"

# Verify backup was created and has content
if [ -s "$BACKUP_DIR/backup_$TIMESTAMP.sql.gz" ]; then
    echo "Backup successful: backup_$TIMESTAMP.sql.gz"
else
    echo "ERROR: Backup failed or empty!"
    exit 1
fi
```

## Method 2: Percona XtraBackup (For Large Databases)

### When to Use

- Large databases (50GB+)
- Need faster backup/restore
- Need incremental backups
- Point-in-time recovery requirements
- Production systems with strict SLAs

### How It Works

1. Copies InnoDB data files directly (physical backup)
2. Records redo log entries during copy
3. No table locks for InnoDB (brief lock for MyISAM if present)
4. Crash recovery applied during "prepare" phase

### Key Advantage

- **Restore is much faster** than `mysqldump`
- Physical files can be copied back directly
- Incremental backups save space and time

### Installation

```bash
# Debian/Ubuntu
sudo apt-get install percona-xtrabackup-80

# Docker image available
docker pull percona/percona-xtrabackup:8.0
```

### Full Backup

```bash
xtrabackup --backup \
  --target-dir=/data/backups/full \
  --user=backup_user \
  --password=secret
```

### Prepare for Restore

```bash
xtrabackup --prepare --target-dir=/data/backups/full
```

### Docker Compose Example

```yaml
services:
  xtrabackup:
    image: percona/percona-xtrabackup:8.0
    volumes:
      - mysql_data:/var/lib/mysql:ro
      - ./backups:/backups
    command: >
      xtrabackup --backup
      --target-dir=/backups/full
      --host=mysql
      --user=root
      --password=$MYSQL_ROOT_PASSWORD
```

## Method 3: File-Level Backup (NOT Recommended for Production)

### ⚠️ Warning

Direct file copy while MySQL is running **can corrupt data**.

### When It's Safe

Only safe when:

1. MySQL is completely stopped, OR
2. Tables are locked with `FLUSH TABLES WITH READ LOCK`

### Why It's Problematic

- InnoDB has data in memory (buffer pool)
- Unflushed pages may not be on disk
- Redo log and data files may be inconsistent

### If You Must (Testing Only)

```bash
# Stop MySQL first!
docker compose stop mysql

# Copy data directory
cp -r ./mysql_data ./mysql_backup

# Restart MySQL
docker compose start mysql
```

## Comparison: MySQL vs SQLite Backup

| Aspect                              | SQLite            | MySQL (InnoDB)            |
| ----------------------------------- | ----------------- | ------------------------- |
| **Lock-free method**                | `.backup` command | `--single-transaction`    |
| **How it achieves consistency**     | Online Backup API | MVCC transaction snapshot |
| **Concurrent writes during backup** | ✅ Yes            | ✅ Yes                    |
| **Built-in tool**                   | `sqlite3 .backup` | `mysqldump`               |
| **External hot backup**             | Not needed        | Percona XtraBackup        |
| **Incremental backup**              | Not native        | Binary log + XtraBackup   |
| **Restore speed**                   | Fast (copy file)  | Slow (replay SQL)         |
| **Database size limit**             | ~1TB practical    | No practical limit        |

### Key Similarity

Both provide lock-free, consistent backups while the database is in use:

- SQLite: `.backup` uses the Online Backup API
- MySQL: `--single-transaction` uses MVCC snapshot

**Neither requires stopping writes or locking tables!**

## Backup Verification

### Verify Dump Integrity

```bash
# Check SQL syntax without executing
mysql --skip-column-names -e "SOURCE backup.sql" --force 2>&1 | grep ERROR

# Or use mysqlcheck on restored database
mysqlcheck -u root -p --check tracker_db
```

### Test Restore

```bash
# Create test database
mysql -u root -p -e "CREATE DATABASE test_restore;"

# Restore backup
mysql -u root -p test_restore < backup.sql

# Verify row counts match
mysql -u root -p -e "SELECT COUNT(*) FROM test_restore.torrents;"

# Clean up
mysql -u root -p -e "DROP DATABASE test_restore;"
```

## Restore Procedures

### From mysqldump

```bash
# Restore to same database
mysql -u root -p tracker_db < backup.sql

# Restore to different database
mysql -u root -p new_database < backup.sql
```

### From Compressed Backup

```bash
gunzip < backup.sql.gz | mysql -u root -p tracker_db
```

### From XtraBackup

```bash
# Stop MySQL
systemctl stop mysql

# Clear data directory (be careful!)
rm -rf /var/lib/mysql/*

# Copy backup files
xtrabackup --copy-back --target-dir=/data/backups/full

# Fix permissions
chown -R mysql:mysql /var/lib/mysql

# Start MySQL
systemctl start mysql
```

## Integration with Restic

Just like SQLite backups, MySQL dumps can be backed up with Restic:

```bash
#!/bin/bash
# backup-mysql-restic.sh

BACKUP_FILE="/tmp/mysql_backup.sql.gz"

# Create compressed dump
docker compose exec -T mysql mysqldump \
  --single-transaction \
  -u root -p"$MYSQL_ROOT_PASSWORD" \
  tracker_db | gzip > "$BACKUP_FILE"

# Backup to Restic repository
restic -r /backups/restic-repo backup "$BACKUP_FILE"

# Clean up temporary file
rm "$BACKUP_FILE"
```

Or pipe directly to Restic:

```bash
docker compose exec -T mysql mysqldump \
  --single-transaction \
  -u root -p"$MYSQL_ROOT_PASSWORD" \
  tracker_db | \
  restic -r /backups/restic-repo backup --stdin --stdin-filename mysql_backup.sql
```

## Performance Considerations

### mysqldump Performance

| Database Size | Backup Time\*       | Restore Time\*      |
| ------------- | ------------------- | ------------------- |
| 1 GB          | ~2-5 min            | ~10-20 min          |
| 10 GB         | ~20-50 min          | ~1-2 hours          |
| 50 GB         | ~2-4 hours          | ~6-12 hours         |
| 100 GB+       | Consider XtraBackup | Consider XtraBackup |

\*Approximate, varies by hardware and data complexity

### XtraBackup Performance

Generally 5-10x faster for both backup and restore compared to mysqldump
for large databases.

## Recommendations for Torrust Tracker

### Typical Deployment

1. **Use `mysqldump --single-transaction`** for most deployments
2. Database size is typically manageable
3. Simpler than XtraBackup
4. Consistent with SQLite approach

### Backup Script Template

```bash
#!/bin/bash
set -e

# Configuration
BACKUP_DIR="/backups/mysql"
RETENTION_DAYS=7
DB_NAME="tracker_db"

# Create timestamped backup
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/${DB_NAME}_${TIMESTAMP}.sql.gz"

echo "Starting MySQL backup..."

docker compose exec -T mysql mysqldump \
  --single-transaction \
  --routines \
  --triggers \
  -u root -p"$MYSQL_ROOT_PASSWORD" \
  "$DB_NAME" | gzip > "$BACKUP_FILE"

# Verify backup
if [ ! -s "$BACKUP_FILE" ]; then
    echo "ERROR: Backup file is empty!"
    exit 1
fi

echo "Backup created: $BACKUP_FILE"

# Clean old backups
find "$BACKUP_DIR" -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup complete!"
```

## Summary

### Key Takeaways

1. **No locking required** for InnoDB tables with `--single-transaction`
2. **mysqldump is sufficient** for most Torrust Tracker deployments
3. **Percona XtraBackup** for large databases or faster restore needs
4. **Same Restic workflow** applies as with SQLite

### Quick Reference

```bash
# Simple backup (recommended)
mysqldump --single-transaction tracker_db > backup.sql

# Compressed backup
mysqldump --single-transaction tracker_db | gzip > backup.sql.gz

# Full options backup
mysqldump --single-transaction --routines --triggers --events tracker_db > backup.sql
```

## References

- [MySQL Documentation: mysqldump](https://dev.mysql.com/doc/refman/8.0/en/mysqldump.html)
- [MySQL Documentation: Backup Methods](https://dev.mysql.com/doc/refman/8.0/en/backup-methods.html)
- [Percona XtraBackup Documentation](https://docs.percona.com/percona-xtrabackup/8.0/)
- [MySQL --single-transaction Explained](https://dev.mysql.com/doc/refman/8.0/en/mysqldump.html#option_mysqldump_single-transaction)
