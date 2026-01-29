# ============================================================================
# Torrust Backup Sidecar Container
# ============================================================================
# Configuration-driven backup container. All behavior is controlled via
# environment variables - no rebuild needed to change backup behavior.
#
# Environment Variables:
#   BACKUP_INTERVAL        - Seconds between backups (default: 120)
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
# ============================================================================

FROM debian:bookworm-slim

# Install utilities
# - bash: for scripting
# - default-mysql-client: MariaDB client (compatible with MySQL 8)
# - gzip: for compression
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    default-mysql-client \
    gzip \
    && rm -rf /var/lib/apt/lists/*

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

# Copy unified backup script
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
