# ============================================================================
# Torrust Backup Sidecar Container
# ============================================================================
# Configuration-driven backup container. All behavior is controlled via
# environment variables - no rebuild needed to change backup behavior.
#
# Environment Variables:
#   BACKUP_INTERVAL        - Seconds between backups (default: 120)
#   BACKUP_RETENTION_DAYS  - Days to keep backups before deletion (default: 7)
#   BACKUP_MYSQL_ENABLED   - Enable MySQL backup: true/false (default: false)
#   BACKUP_PATHS_FILE      - Path to file listing paths to backup (optional)
#   MYSQL_HOST, MYSQL_PORT, MYSQL_DATABASE, MYSQL_USER, MYSQL_PASSWORD
#
# Mount Points:
#   /backups  - Output directory for all backups
#   /data     - Source data directory (mount app storage here, read-only)
#   /config   - Backup configuration files (e.g., backup-paths.txt)
#
# Security:
#   Container runs as uid 1000 (torrust user) to match app file ownership.
#   This ensures backup files are accessible by the application user.
#
# Testing:
#   Tests run during build using bats-core. Build fails if tests fail.
# ============================================================================

FROM debian:bookworm-slim AS base

# Install utilities
# - bash: for scripting
# - default-mysql-client: MariaDB client (compatible with MySQL 8)
# - gzip: for compression
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    default-mysql-client \
    gzip \
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
RUN groupadd -g ${BACKUP_GID} torrust || true && \
    useradd -u ${BACKUP_UID} -g ${BACKUP_GID} -s /bin/bash torrust || true

# Create directories with correct ownership
RUN mkdir -p /scripts /backups/mysql /backups/config && \
    chown -R ${BACKUP_UID}:${BACKUP_GID} /backups

# Copy unified backup script (tests already passed in test stage)
COPY backup.sh /scripts/backup.sh
RUN chmod +x /scripts/backup.sh

# Default configuration (override via docker-compose environment)
ENV BACKUP_INTERVAL=120
ENV BACKUP_MYSQL_ENABLED=false

# MySQL connection defaults
ENV MYSQL_HOST=mysql
ENV MYSQL_PORT=3306
ENV MYSQL_DATABASE=torrust_tracker
ENV MYSQL_USER=tracker_user
# MYSQL_PASSWORD must be provided via docker-compose environment

# Run as non-root user (torrust, uid 1000)
USER torrust

ENTRYPOINT ["/scripts/backup.sh"]
