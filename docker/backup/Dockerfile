# ============================================================================
# Torrust Backup Container
# ============================================================================
# Production backup container for Torrust Tracker deployments.
# Configuration is provided via mounted config files - no environment variables.
#
# Configuration Files:
#   /etc/backup/backup.conf      - Main configuration (sourced by backup.sh)
#   /etc/backup/backup-paths.txt - List of files/directories to backup
#
# Mount Points:
#   /backups  - Output directory for all backups (read-write)
#   /data     - Source data directory (read-only, app storage mounted here)
#
# Output Structure:
#   /backups/mysql/mysql_YYYYMMDD_HHMMSS.sql.gz   - MySQL dumps (compressed)
#   /backups/sqlite/sqlite_YYYYMMDD_HHMMSS.db.gz  - SQLite backups (compressed)
#   /backups/config/config_YYYYMMDD_HHMMSS.tar.gz - Config archives (compressed)
#
# Security:
#   Container runs as uid 1000 (torrust user) to match app file ownership.
#   This ensures backup files have correct ownership on host.
#
# Testing:
#   Tests run during build using bats-core. Build fails if tests fail.
# ============================================================================

FROM debian:bookworm-slim AS base

# Install required utilities
# - bash: for scripting
# - default-mysql-client: MariaDB client (compatible with MySQL 8)
# - sqlite3: SQLite client for .backup command
# - gzip: for compression
# - tar: for config file archiving
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    default-mysql-client \
    sqlite3 \
    gzip \
    tar \
    && rm -rf /var/lib/apt/lists/*

# =============================================================================
# Test Stage - Run unit tests during build
# =============================================================================
FROM base AS test

# Install bats-core for testing
RUN apt-get update && apt-get install -y --no-install-recommends \
    bats \
    && rm -rf /var/lib/apt/lists/*

# Copy test files
COPY backup.sh /scripts/backup.sh
COPY backup_test.bats /scripts/backup_test.bats
RUN chmod +x /scripts/backup.sh

# Run tests - build fails if tests fail
# Create a marker file to prove tests passed
RUN cd /scripts && bats backup_test.bats && touch /scripts/.tests_passed

# =============================================================================
# Production Stage
# =============================================================================
FROM base AS production

# Require tests to have passed by copying marker from test stage
# This ensures test stage is always executed before production stage
COPY --from=test /scripts/.tests_passed /tmp/.tests_passed

# Create backup user with same UID as torrust app user
# This ensures backup files have correct ownership on host
# Using 'torrust' as the username to match the app user
ARG BACKUP_UID=1000
ARG BACKUP_GID=1000
RUN groupadd -g ${BACKUP_GID} torrust 2>/dev/null || true && \
    useradd -u ${BACKUP_UID} -g ${BACKUP_GID} -s /bin/bash torrust 2>/dev/null || true

# Create directories with correct ownership
RUN mkdir -p /scripts /backups/mysql /backups/sqlite /backups/config && \
    chown -R ${BACKUP_UID}:${BACKUP_GID} /backups

# Copy backup script (tests already passed in test stage)
COPY backup.sh /scripts/backup.sh
RUN chmod +x /scripts/backup.sh

# Run as non-root user (torrust, uid 1000)
USER torrust

ENTRYPOINT ["/scripts/backup.sh"]
