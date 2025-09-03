#!/bin/bash
# Install OpenTofu (Terraform alternative)
# This script installs OpenTofu using the official installation script

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

# Check if OpenTofu is already installed
if command -v tofu &> /dev/null; then
    CURRENT_VERSION=$(tofu version | head -n1 | awk '{print $2}')
    log_info "OpenTofu is already installed: ${CURRENT_VERSION}"
    log_info "Skipping installation. Use --force to reinstall."
    exit 0
fi

log_info "Installing OpenTofu..."

# Download and run the official installation script
curl --proto '=https' -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
chmod +x install-opentofu.sh

# Install using deb method (works on Ubuntu/Debian)
./install-opentofu.sh --install-method deb

# Clean up the installation script
rm -f install-opentofu.sh

# Verify installation
if command -v tofu &> /dev/null; then
    INSTALLED_VERSION=$(tofu version | head -n1 | awk '{print $2}')
    log_success "✅ OpenTofu successfully installed: ${INSTALLED_VERSION}"
else
    log_error "❌ OpenTofu installation failed"
    exit 1
fi
