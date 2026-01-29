# Minimal backup sidecar container for PoC
# Phase 2: Just logs a message every interval
# Later phases will add backup functionality

FROM alpine:3.21

# Install basic utilities (will add mysql-client in Phase 3)
RUN apk add --no-cache bash

# Create scripts directory
RUN mkdir -p /scripts

# Copy entrypoint
COPY entrypoint.sh /scripts/entrypoint.sh
RUN chmod +x /scripts/entrypoint.sh

# Default backup interval (in seconds) - 2 minutes for testing
ENV BACKUP_INTERVAL=120

ENTRYPOINT ["/scripts/entrypoint.sh"]
