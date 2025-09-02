#!/bin/bash
# YAML linter wrapper with clean output

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
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

log_error() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6NZ")
    echo -e "${timestamp} ${RED}ERROR${NC} ${SCRIPT_NAME}: $1"
}

echo "🔍 YAML Linter"
echo "=============="

# Check if yamllint is installed
if ! command -v yamllint &> /dev/null; then
    log_info "Installing yamllint..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y yamllint
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y yamllint
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm yamllint
    elif command -v pip3 &> /dev/null; then
        pip3 install --user yamllint
    else
        log_error "Could not install yamllint. Please install it manually."
        exit 1
    fi
    log_success "yamllint installed"
fi

# Run the linter
log_info "Scanning YAML files..."
if yamllint -c .yamllint-ci.yml . 2>&1; then
    log_success "All YAML files passed linting!"
    exit 0
else
    echo ""
    log_error "YAML linting failed. Please fix the issues above."
    exit 1
fi
