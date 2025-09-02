#!/bin/bash
# Clippy linter with comprehensive checks

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

log_info "Running Rust Clippy linter..."

# Run clippy with comprehensive checks (same as meson target)
CARGO_INCREMENTAL=0 cargo clippy \
    --quiet \
    --no-deps \
    --tests \
    --benches \
    --examples \
    --workspace \
    --all-targets \
    --all-features \
    -- \
    -D clippy::correctness \
    -D clippy::suspicious \
    -D clippy::complexity \
    -D clippy::perf \
    -D clippy::style \
    -D clippy::pedantic

log_success "Clippy linting completed successfully!"
