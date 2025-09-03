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

# Add current user to lxd group
if is_ci_environment; then
    # In CI environments (like GitHub Actions), the user is typically 'runner'
    CURRENT_USER="${RUNNER_USER:-runner}"
    log_info "Adding CI user '${CURRENT_USER}' to lxd group..."
else
    # For local development
    CURRENT_USER="${USER:-$(whoami)}"
    log_info "Adding user '${CURRENT_USER}' to lxd group..."
fi
sudo usermod -a -G lxd "${CURRENT_USER}"

# CI-specific socket permission fix
if is_ci_environment; then
    log_warn "CI environment detected - applying socket permission fix"
    log_warn "IMPORTANT: This approach is ONLY for CI environments"
    log_warn "For local development, use proper group membership instead"
    
    # Fix socket permissions for CI environment (NOT recommended for local use)
    sudo chmod 666 /var/snap/lxd/common/lxd/unix.socket
else
    log_info "Non-CI environment detected"
    log_info "For group membership to take effect, you may need to:"
    log_info "1. Run 'newgrp lxd' for immediate effect in current shell"
    log_info "2. Or log out and log back in"
    log_info "3. Or restart your terminal"
    log_info "See config/tofu/lxd/README.md for detailed instructions"
fi

# Test basic LXD functionality
log_info "Testing basic LXD functionality..."
if is_ci_environment; then
    sudo lxc list
    LXD_VERSION=$(sudo lxc version)
else
    # For local development, try without sudo first
    if lxc list &> /dev/null; then
        lxc list
        LXD_VERSION=$(lxc version)
    else
        log_warn "Direct lxc access failed, you may need to activate group membership"
        log_warn "Try: newgrp lxd"
        sudo lxc list
        LXD_VERSION=$(sudo lxc version)
    fi
fi

log_success "✅ LXD successfully configured: ${LXD_VERSION}"
log_info "LXD is ready for use"
