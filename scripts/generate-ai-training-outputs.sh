#!/bin/bash
# Generate rendered template outputs for AI training example configurations
# This script processes all example configs in docs/ai-training/examples/ and
# generates the corresponding deployment artifacts using the 'render' command.
#
# Purpose: Complete the AI training dataset with input/output pairs:
#   - Input: Environment configuration JSON files
#   - Output: Rendered deployment templates (OpenTofu, Ansible, Docker, etc.)
#
# The script:
#   1. Replaces generic SSH paths with actual fixture paths
#   2. Calls 'render' command with placeholder IP address
#   3. Outputs artifacts to docs/ai-training/outputs/<example-name>/
#   4. Reports success/failure for each example

set -euo pipefail

# Configuration
EXAMPLES_DIR="docs/ai-training/examples"
OUTPUTS_DIR="docs/ai-training/outputs"
PLACEHOLDER_IP="203.0.113.1"  # TEST-NET-1 (RFC 5737) - documentation IP range
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_PRIVATE_KEY="${PROJECT_ROOT}/fixtures/testing_rsa"
FIXTURE_PUBLIC_KEY="${PROJECT_ROOT}/fixtures/testing_rsa.pub"

# Counters
SUCCESS_COUNT=0
FAILURE_COUNT=0
FAILED_EXAMPLES=()

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Generate rendered deployment artifacts for AI training examples.

Options:
    -h, --help          Show this help message
    -c, --clean         Remove existing outputs directory before generating

Examples:
    $0                  Generate outputs for all examples
    $0 --clean          Clean and regenerate all outputs

Output:
    Generated artifacts are written to: ${OUTPUTS_DIR}/
    Each example gets its own subdirectory with rendered templates.
EOF
}

# Function to log messages
log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*"
}

log_error() {
    echo -e "${RED}✗${NC} $*"
}

# Function to replace SSH paths in JSON config
replace_ssh_paths() {
    local input_file=$1
    local output_file=$2
    
    # Use jq to replace SSH key paths
    jq --arg private_key "$FIXTURE_PRIVATE_KEY" \
       --arg public_key "$FIXTURE_PUBLIC_KEY" \
       '.ssh_credentials.private_key_path = $private_key | 
        .ssh_credentials.public_key_path = $public_key' \
       "$input_file" > "$output_file"
}

# Function to extract example name from filename
get_example_name() {
    local filename=$1
    basename "$filename" .json
}

# Function to process a single example
process_example() {
    local example_file=$1
    local example_name
    local temp_config
    local output_dir
    
    example_name=$(get_example_name "$example_file")
    output_dir="${OUTPUTS_DIR}/${example_name}"
    temp_config=$(mktemp)
    
    log_info "Processing: ${example_name}"
    
    # Replace SSH paths in config
    if ! replace_ssh_paths "$example_file" "$temp_config"; then
        log_error "Failed to process config: ${example_name}"
        rm -f "$temp_config"
        return 1
    fi
    
    # Run render command (capture output for error reporting)
    # Use --force to allow overwriting existing output directories
    local render_output
    if render_output=$(cargo run -q -- render \
        --env-file "$temp_config" \
        --instance-ip "$PLACEHOLDER_IP" \
        --output-dir "$output_dir" \
        --force 2>&1); then
        log_success "Generated: ${example_name}"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        log_error "Failed to render: ${example_name}"
        echo "$render_output" | grep -E "(Error|Failed|error)" || echo "$render_output"
        FAILED_EXAMPLES+=("$example_name")
        FAILURE_COUNT=$((FAILURE_COUNT + 1))
    fi
    
    # Cleanup temp file
    rm -f "$temp_config"
}

# Function to clean outputs directory
clean_outputs() {
    if [[ -d "$OUTPUTS_DIR" ]]; then
        log_info "Cleaning existing outputs directory..."
        rm -rf "$OUTPUTS_DIR"
        log_success "Outputs directory cleaned"
    fi
}

# Main execution
main() {
    local clean_mode=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -c|--clean)
                clean_mode=true
                shift
                ;;
            *)
                echo "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    echo "🚀 AI Training Outputs Generator"
    echo "================================="
    echo
    log_info "Project root: ${PROJECT_ROOT}"
    log_info "Examples directory: ${EXAMPLES_DIR}"
    log_info "Outputs directory: ${OUTPUTS_DIR}"
    log_info "Placeholder IP: ${PLACEHOLDER_IP}"
    echo
    
    # Clean if requested
    if [[ "$clean_mode" == true ]]; then
        clean_outputs
        echo
    fi
    
    # Check if examples directory exists
    if [[ ! -d "$EXAMPLES_DIR" ]]; then
        log_error "Examples directory not found: ${EXAMPLES_DIR}"
        exit 1
    fi
    
    # Check if fixture SSH keys exist
    if [[ ! -f "$FIXTURE_PRIVATE_KEY" ]] || [[ ! -f "$FIXTURE_PUBLIC_KEY" ]]; then
        log_error "Fixture SSH keys not found:"
        log_error "  Expected: ${FIXTURE_PRIVATE_KEY}"
        log_error "  Expected: ${FIXTURE_PUBLIC_KEY}"
        exit 1
    fi
    
    # Check if jq is installed
    if ! command -v jq &> /dev/null; then
        log_error "jq is required but not installed"
        log_error "Install with: sudo apt-get install jq"
        exit 1
    fi
    
    # Create outputs directory
    mkdir -p "$OUTPUTS_DIR"
    
    # Find all example JSON files
    local examples
    mapfile -t examples < <(find "$EXAMPLES_DIR" -maxdepth 1 -name "*.json" | sort)
    
    if [[ ${#examples[@]} -eq 0 ]]; then
        log_error "No example files found in ${EXAMPLES_DIR}"
        exit 1
    fi
    
    log_info "Found ${#examples[@]} example configuration(s)"
    echo
    
    # Process each example
    for example_file in "${examples[@]}"; do
        process_example "$example_file"
    done
    
    # Summary
    echo
    echo "📊 Summary"
    echo "=========="
    log_success "Successfully generated: ${SUCCESS_COUNT}"
    
    if [[ $FAILURE_COUNT -gt 0 ]]; then
        log_error "Failed to generate: ${FAILURE_COUNT}"
        echo
        log_error "Failed examples:"
        for failed in "${FAILED_EXAMPLES[@]}"; do
            echo "  - ${failed}"
        done
        exit 1
    else
        echo
        log_success "All examples processed successfully!"
        log_info "Outputs available in: ${OUTPUTS_DIR}/"
    fi
}

# Run main function
main "$@"
