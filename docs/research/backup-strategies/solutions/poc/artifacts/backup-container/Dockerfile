# Minimal backup sidecar container for PoC
# Phase 2: Just logs a message every interval
# Phase 3: Added mysql-client for database backups

# Use Debian slim for official MySQL client (Alpine's mysql-client is MariaDB)
FROM debian:bookworm-slim

# Install utilities
# - bash: for scripting
# - default-mysql-client: official MySQL client (supports caching_sha2_password)
# - gzip: for compression
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    default-mysql-client \
    gzip \
    && rm -rf /var/lib/apt/lists/*

# Create directories
RUN mkdir -p /scripts /backups

# Copy scripts
COPY entrypoint.sh /scripts/entrypoint.sh
COPY backup-mysql.sh /scripts/backup-mysql.sh
RUN chmod +x /scripts/*.sh

# Default backup interval (in seconds) - 2 minutes for testing
ENV BACKUP_INTERVAL=120

# MySQL connection defaults (override in docker-compose)
# Note: Credentials are passed via docker-compose environment, not hardcoded here
ENV MYSQL_HOST=mysql
ENV MYSQL_PORT=3306
ENV MYSQL_DATABASE=torrust_tracker
ENV MYSQL_USER=tracker_user
# MYSQL_PASSWORD must be provided via docker-compose environment

ENTRYPOINT ["/scripts/entrypoint.sh"]
