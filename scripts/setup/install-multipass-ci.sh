#!/bin/bash
# Install and configure Multipass for CI environments
# 
# IMPORTANT: This script uses CI-specific approaches for Multipass setup.
# Multipass configuration may differ between local development and CI environments.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script name for structured logging
SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}" .sh)"

# Logging functions with tracing-style format
log_info() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6NZ")
    echo -e "${timestamp} ${BLUE} INFO${NC} ${SCRIPT_NAME}: $1"
}

log_success() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6NZ")
    echo -e "${timestamp} ${GREEN} INFO${NC} ${SCRIPT_NAME}: $1"
}

log_warn() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6NZ")
    echo -e "${timestamp} ${YELLOW} WARN${NC} ${SCRIPT_NAME}: $1"
}

log_error() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6NZ")
    echo -e "${timestamp} ${RED}ERROR${NC} ${SCRIPT_NAME}: $1"
}

# Check if we're in a CI environment
is_ci_environment() {
    [[ "${CI:-false}" == "true" ]] || [[ "${GITHUB_ACTIONS:-false}" == "true" ]] || [[ -n "${RUNNER_USER:-}" ]]
}

# Check if Multipass is already installed
if command -v multipass &> /dev/null; then
    CURRENT_VERSION=$(multipass version | head -n1 | awk '{print $2}')
    log_info "Multipass is already installed: ${CURRENT_VERSION}"
    log_info "Proceeding with configuration..."
else
    log_info "Installing Multipass via snap..."
    sudo snap install multipass
fi

# Wait for multipass to fully initialize
log_info "Waiting for multipass daemon to start..."
sleep 15

# Check if multipass daemon is running
log_info "Checking multipass daemon status..."
sudo systemctl status snap.multipass.multipassd.service || true

# Try to start the daemon explicitly if needed
log_info "Ensuring multipass daemon is started..."
sudo systemctl start snap.multipass.multipassd.service || true

# Wait a bit more for daemon to be ready
sleep 10

# Check multipass status
log_info "Checking multipass version..."
sudo multipass version

# Create multipass group if it doesn't exist
log_info "Setting up multipass group..."
sudo groupadd multipass || true

# Set up permissions - add current user to multipass group
CURRENT_USER="${USER:-$(whoami)}"
log_info "Adding user '${CURRENT_USER}' to multipass group..."
sudo usermod -a -G multipass "${CURRENT_USER}" || true

if is_ci_environment; then
    log_warn "CI environment detected - applying socket permission fix"
    # Fix socket permissions directly for CI
    sudo chmod 666 /var/snap/multipass/common/multipass_socket || true
else
    log_info "Non-CI environment detected"
    log_info "For group membership to take effect, you may need to log out and back in"
fi

# Try to configure multipass for virtualization
log_info "Configuring multipass driver..."
sudo multipass set local.driver=qemu || log_warn "Could not set driver, continuing..."

# Test basic multipass functionality with timeout
log_info "Testing basic multipass functionality..."
if timeout 30 sudo multipass list; then
    log_success "✅ Multipass list command successful"
else
    log_warn "Direct multipass list failed, trying with socket fix..."
    # Alternative: try to connect as root to test socket
    if sudo -u root multipass list; then
        log_success "✅ Root multipass access successful"
    else
        log_error "❌ Root access also failed, socket may need more time"
    fi
fi

# Get final version info
MULTIPASS_VERSION=$(sudo multipass version | head -n1)
log_success "✅ Multipass successfully configured: ${MULTIPASS_VERSION}"
log_info "Multipass is ready for use"
