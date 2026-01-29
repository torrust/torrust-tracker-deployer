#!/bin/bash
# Backup sidecar entrypoint
# Phase 2: Minimal version - just logs messages
# Later phases will add actual backup logic

set -e

INTERVAL="${BACKUP_INTERVAL:-120}"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] Backup sidecar starting..."
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Backup interval: ${INTERVAL} seconds"

while true; do
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Backup service running..."
    sleep "${INTERVAL}"
done
