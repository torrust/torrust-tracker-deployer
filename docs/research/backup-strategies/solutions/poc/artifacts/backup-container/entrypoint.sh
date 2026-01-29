#!/bin/bash
# Backup sidecar entrypoint
# Phase 2: Minimal version - just logs messages
# Phase 3: Runs MySQL backup on each interval

set -e

INTERVAL="${BACKUP_INTERVAL:-120}"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] Backup sidecar starting..."
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Backup interval: ${INTERVAL} seconds"

# Run first backup immediately
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Running initial backup..."
/scripts/backup-mysql.sh

while true; do
    sleep "${INTERVAL}"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Running scheduled backup..."
    /scripts/backup-mysql.sh
done
