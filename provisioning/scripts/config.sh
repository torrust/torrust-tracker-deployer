#!/bin/bash
# Torrust Tracker Environment Configuration Wizard (Bash variant)
# Main orchestration script for the configuration wizard workflow
#
# This script:
# 1. Verifies TypeDialog and Nickel are installed
# 2. Launches interactive TypeDialog form
# 3. Converts JSON output to Nickel configuration
# 4. Validates with Nickel validators
# 5. Exports final JSON to envs/ directory
#
# Usage:
#   ./provisioning/scripts/config.sh

set -euo pipefail

# ============================================================================
# CONFIGURATION
# ============================================================================

readonly SCRIPT_DIR
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROVISIONING_DIR
PROVISIONING_DIR="$(dirname "$SCRIPT_DIR")"
readonly PROJECT_ROOT
PROJECT_ROOT="$(dirname "$PROVISIONING_DIR")"
readonly ENVS_DIR="${PROJECT_ROOT}/envs"
readonly VALUES_DIR="${PROVISIONING_DIR}/values"
readonly FORM_PATH="${PROVISIONING_DIR}/config-form.toml"
readonly SCRIPTS_DIR="$SCRIPT_DIR"

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

print_header() {
    echo "═══════════════════════════════════════════════════════════"
    echo "🎯 $1"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
}

print_step() {
    echo "📝 Step $1/$2: $3..."
}

print_success() {
    echo "✅ $1"
}

print_error() {
    echo "❌ $1" >&2
}

print_info() {
    echo "ℹ️  $1"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# ============================================================================
# DEPENDENCY VERIFICATION
# ============================================================================

verify_dependencies() {
    print_header "Checking Dependencies"

    local missing_deps=()

    if ! command_exists "typedialog"; then
        missing_deps+=("typedialog (install with: cargo install typedialog)")
    fi

    if ! command_exists "nickel"; then
        missing_deps+=("nickel (install with: cargo install nickel-lang-cli)")
    fi

    if ! command_exists "jq"; then
        missing_deps+=("jq (install with: brew install jq or apt-get install jq)")
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        print_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "   - $dep" >&2
        done
        return 1
    fi

    print_success "All dependencies available"
    echo ""
}

# ============================================================================
# MAIN WORKFLOW
# ============================================================================

main() {
    print_header "Torrust Tracker - Environment Configuration Wizard"

    # Step 0: Verify dependencies
    if ! verify_dependencies; then
        exit 1
    fi

    # Ensure directories exist
    mkdir -p "$ENVS_DIR"
    mkdir -p "$VALUES_DIR"

    # Step 1: Run TypeDialog form
    print_step "1" "4" "Collecting configuration via interactive form"

    local temp_output
    temp_output=$(mktemp)
    trap 'rm -f "$temp_output"' EXIT

    if ! typedialog run "$FORM_PATH" > "$temp_output" 2>&1; then
        print_error "TypeDialog form failed"
        cat "$temp_output" >&2
        exit 1
    fi

    if [[ ! -s "$temp_output" ]]; then
        print_error "TypeDialog output is empty. Wizard cancelled."
        exit 1
    fi

    print_success "Configuration collected"
    echo ""

    # Step 2: Extract environment name
    print_step "2" "4" "Processing configuration"

    local env_name
    env_name=$(jq -r '.environment_name' "$temp_output")

    if [[ -z "$env_name" ]]; then
        print_error "Could not extract environment name from form output"
        exit 1
    fi

    print_info "Environment name: $env_name"

    local values_file="${VALUES_DIR}/${env_name}.ncl"
    local json_file="${ENVS_DIR}/${env_name}.json"

    # Step 3: Convert JSON to Nickel
    print_step "3" "4" "Converting to Nickel configuration"

    if ! bash "$SCRIPTS_DIR/json-to-nickel.sh" "$temp_output" "$values_file"; then
        print_error "Nickel file generation failed"
        exit 1
    fi

    print_success "Nickel configuration generated: $values_file"
    echo ""

    # Step 4: Validate Nickel
    print_info "Validating Nickel configuration..."

    if ! nickel eval "$values_file" > /dev/null 2>&1; then
        print_error "Nickel validation failed"
        nickel eval "$values_file" >&2
        exit 1
    fi

    print_success "Nickel validation passed"
    echo ""

    # Step 5: Export Nickel to JSON
    print_step "4" "4" "Exporting to JSON format"

    if ! bash "$SCRIPTS_DIR/nickel-to-json.sh" "$values_file" "$json_file"; then
        print_error "JSON export failed"
        exit 1
    fi

    print_success "JSON configuration exported: $json_file"
    echo ""

    # Success summary
    print_header "Configuration Generation Complete!"
    echo ""

    print_info "Generated files:"
    echo "   - Nickel: $values_file"
    echo "   - JSON:   $json_file"
    echo ""

    print_info "Next steps:"
    echo "   1. Review configuration: cat '$json_file' | jq ."
    echo "   2. Create environment:   cargo run --bin torrust-tracker-deployer -- create environment --env-file '$json_file'"
    echo "   3. Provision:            cargo run --bin torrust-tracker-deployer -- provision '$env_name'"
    echo ""
}

# Run main function
main "$@"
