#!/bin/bash
# MySQL backup script using mariadb-dump with --single-transaction
# This provides a consistent backup without locking tables (InnoDB only)
#
# NOTE: We use mariadb-dump (from Debian's default-mysql-client package) to
# backup MySQL databases. MariaDB client tools are fully compatible with MySQL.
# The tracker uses MySQL as the database (see ADR: use-mysql-over-mariadb.md).

set -e

# Configuration from environment
MYSQL_HOST="${MYSQL_HOST:-mysql}"
MYSQL_PORT="${MYSQL_PORT:-3306}"
MYSQL_DATABASE="${MYSQL_DATABASE:-torrust_tracker}"
MYSQL_USER="${MYSQL_USER:-tracker_user}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-tracker_password}"
BACKUP_DIR="${BACKUP_DIR:-/backups}"

# Generate timestamp for filename
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')
BACKUP_FILE="${BACKUP_DIR}/mysql_${TIMESTAMP}.sql.gz"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting MySQL backup..."
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Database: ${MYSQL_DATABASE}@${MYSQL_HOST}:${MYSQL_PORT}"

# Run mariadb-dump with --single-transaction for consistent InnoDB backup
# Note: Using mariadb-dump (not mysqldump) to avoid deprecation warning
# --ssl=0 disables SSL to avoid self-signed certificate issues in Docker networks
# --no-tablespaces avoids needing PROCESS privilege (not needed for restore)
# Pipe through gzip for compression
mariadb-dump \
    --host="${MYSQL_HOST}" \
    --port="${MYSQL_PORT}" \
    --user="${MYSQL_USER}" \
    --password="${MYSQL_PASSWORD}" \
    --ssl=0 \
    --no-tablespaces \
    --single-transaction \
    --routines \
    --triggers \
    "${MYSQL_DATABASE}" | gzip > "${BACKUP_FILE}"

# Verify backup was created and has content
if [ -s "${BACKUP_FILE}" ]; then
    SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ✅ Backup complete: ${BACKUP_FILE} (${SIZE})"
else
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ❌ Backup failed: file is empty or missing"
    exit 1
fi
