#!/bin/bash
# Entrypoint script for E2E configuration testing container
# Initializes SSH server via supervisor for Ansible connectivity

set -e

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >&2
}

log "Starting container initialization..."

# Create supervisor log directory
mkdir -p /var/log/supervisor

# Create SSH host keys if they don't exist
if [ ! -f /etc/ssh/ssh_host_rsa_key ]; then
    log "Generating SSH host keys..."
    ssh-keygen -A
fi

# Ensure SSH directory has proper permissions
chown torrust:torrust /home/torrust/.ssh
chmod 700 /home/torrust/.ssh
chmod 600 /home/torrust/.ssh/authorized_keys

# Signal that container is ready
log "Container initialization complete. Starting supervisor..."

# Execute the provided command (supervisor by default)
exec "$@"