#!/bin/bash
# Install Ansible automation platform

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

# Check if Ansible is already installed
if command -v ansible &> /dev/null; then
    CURRENT_VERSION=$(ansible --version | head -n1 | awk '{print $3}')
    log_info "Ansible is already installed: ${CURRENT_VERSION}"
    log_info "Skipping installation. Use --force to reinstall."
    exit 0
fi

log_info "Updating package manager..."
sudo apt-get update

log_info "Installing Ansible..."
sudo apt-get install -y ansible

# Verify installation
if command -v ansible &> /dev/null; then
    INSTALLED_VERSION=$(ansible --version | head -n1 | awk '{print $3}')
    log_success "✅ Ansible successfully installed: ${INSTALLED_VERSION}"
    
    # Show additional info
    log_info "Ansible executable: $(which ansible)"
    log_info "Python version: $(ansible --version | grep "python version" | awk '{print $3}')"
else
    log_error "❌ Ansible installation failed"
    exit 1
fi
