#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script name and function for structured logging
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

log_warning() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6NZ")
    echo -e "${timestamp} ${YELLOW} WARN${NC} ${SCRIPT_NAME}: $1"
}

log_error() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%S.%6NZ")
    echo -e "${timestamp} ${RED}ERROR${NC} ${SCRIPT_NAME}: $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install shellcheck
install_shellcheck() {
    log_info "Installing ShellCheck..."
    
    if command_exists apt-get; then
        sudo apt-get update && sudo apt-get install -y shellcheck
    elif command_exists dnf; then
        sudo dnf install -y ShellCheck
    elif command_exists pacman; then
        sudo pacman -S --noconfirm shellcheck
    elif command_exists brew; then
        brew install shellcheck
    else
        log_error "Could not install shellcheck: unsupported package manager"
        log_info "Please install shellcheck manually: https://github.com/koalaman/shellcheck#installing"
        return 1
    fi
}

# Main function
main() {
    log_info "Running ShellCheck on shell scripts..."

    if ! command_exists shellcheck; then
        log_warning "shellcheck not found. Attempting to install..."
        if ! install_shellcheck; then
            return 1
        fi
    fi

    # Use glob pattern to find shell scripts, excluding .git and .terraform directories
    # Enable globstar for ** patterns
    shopt -s globstar nullglob

    # Find shell scripts with common extensions
    shell_files=()
    for pattern in "**/*.sh" "**/*.bash"; do
        for file in $pattern; do
            # Skip files in .git and .terraform directories
            if [[ "$file" != *".git"* && "$file" != *".terraform"* ]]; then
                shell_files+=("$file")
            fi
        done
    done

    if [ ${#shell_files[@]} -eq 0 ]; then
        log_warning "No shell scripts found"
        return 0
    fi

    log_info "Found ${#shell_files[@]} shell script(s) to check"

    # Add source-path to help shellcheck find sourced files
    # Exclude SC1091 (not following sourced files) as it's informational only
    if shellcheck --source-path=SCRIPTDIR --exclude=SC1091 "${shell_files[@]}"; then
        log_success "shellcheck passed"
        return 0
    else
        log_error "shellcheck failed"
        return 1
    fi
}

main "$@"
