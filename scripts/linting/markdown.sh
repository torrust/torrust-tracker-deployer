#!/bin/bash
# Markdown linter wrapper with clean output

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

echo "🔍 Markdown Linter"
echo "=================="

# Check if markdownlint is installed
if ! command -v markdownlint &> /dev/null; then
    log_info "Installing markdownlint-cli..."
    npm install -g markdownlint-cli
    log_success "markdownlint-cli installed"
fi

# Run the linter
log_info "Scanning markdown files..."
if find . -name "*.md" -type f -not -path "./.terraform/*" -exec markdownlint {} + 2>&1; then
    log_success "All markdown files passed linting!"
    exit 0
else
    echo ""
    log_error "Markdown linting failed. Please fix the issues above."
    exit 1
fi
