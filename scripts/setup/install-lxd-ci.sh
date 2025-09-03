#!/bin/bash
# Install and configure LXD for CI environments
# 
# IMPORTANT: This script uses CI-specific approaches like 'sudo chmod 666' on the LXD socket
# and 'sudo' with LXD commands. These approaches are NOT recommended for local development.
# For local use, follow the proper group membership approach documented in config/tofu/lxd/README.md

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

# Check if LXD is already installed
if command -v lxd &> /dev/null; then
    CURRENT_VERSION=$(lxd version | head -n1 | awk '{print $1}')
    log_info "LXD is already installed: ${CURRENT_VERSION}"
    log_info "Proceeding with configuration..."
else
    log_info "Installing LXD via snap..."
    sudo snap install lxd
fi

# Wait for LXD to fully initialize
log_info "Waiting for LXD daemon to start..."
sleep 15

# Initialize LXD with default settings
log_info "Initializing LXD with default settings..."
sudo lxd init --auto

# Add runner to lxd group
log_info "Adding runner to lxd group..."
sudo usermod -a -G lxd runner

# IMPORTANT: This approach is ONLY for CI environments
# For local development, use proper group membership instead
# Fix socket permissions for CI environment (NOT recommended for local use)
log_warn "IMPORTANT: This approach is ONLY for CI environments"
log_warn "For local development, use proper group membership instead"
sudo chmod 666 /var/snap/lxd/common/lxd/unix.socket

# Test basic LXD functionality
log_info "Testing basic LXD functionality..."
sudo lxc list

LXD_VERSION=$(sudo lxc version)
log_success "✅ LXD successfully configured: ${LXD_VERSION}"
log_info "LXD is ready for use"
