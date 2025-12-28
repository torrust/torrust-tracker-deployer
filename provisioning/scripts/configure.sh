#!/usr/bin/env bash
# Torrust Tracker Environment Configuration Script
# Interactive configuration for tracker environments using TypeDialog + Nickel
#
# This script uses TypeDialog's nickel-roundtrip feature to:
# 1. Load existing Nickel configuration (if exists)
# 2. Launch interactive form with current values pre-populated
# 3. Save updated configuration back to Nickel format
# 4. Preserve validators and contracts in output
#
# Usage:
#   ./provisioning/scripts/configure.sh [cli|tui|web]
#
# Backend options:
#   cli - Command-line interface (simple prompts) [default]
#   tui - Terminal UI (interactive panels)
#   web - Web server (browser-based)

set -euo pipefail

# ============================================================================
# DIRECTORY STRUCTURE
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROVISIONING_DIR="$(dirname "$SCRIPT_DIR")"

# Configuration files
FORM_FILE="${PROVISIONING_DIR}/config-form.toml"
VALUES_DIR="${PROVISIONING_DIR}/values"
TEMPLATE_FILE="${PROVISIONING_DIR}/templates/config-template.ncl.j2"

# Default config file (can be overridden)
CONFIG_FILE="${VALUES_DIR}/config.ncl"

# ============================================================================
# ENVIRONMENT VARIABLES
# ============================================================================

# TypeDialog environment variables (can be overridden)
export TYPEDIALOG_PORT="${TYPEDIALOG_PORT:-9000}"
export TYPEDIALOG_HOST="${TYPEDIALOG_HOST:-localhost}"
export TYPEDIALOG_LANG="${TYPEDIALOG_LANG:-${LANG:-en_US.UTF-8}}"

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

print_header() {
    echo "═══════════════════════════════════════════════════════════"
    echo "🎯 $1"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
}

print_info() {
    echo "ℹ️  $1"
}

print_success() {
    echo "✅ $1"
}

print_error() {
    echo "❌ $1" >&2
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# ============================================================================
# DEPENDENCY VERIFICATION
# ============================================================================

verify_dependencies() {
    local missing_deps=()

    if ! command_exists "typedialog"; then
        missing_deps+=("typedialog (install with: cargo install typedialog)")
    fi

    if ! command_exists "nickel"; then
        missing_deps+=("nickel (install with: cargo install nickel-lang-cli)")
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        print_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "   - $dep" >&2
        done
        return 1
    fi

    return 0
}

# ============================================================================
# ARGUMENT PARSING
# ============================================================================

# Detect which TypeDialog backend to use (default: web)
BACKEND="${1:-web}"

# Validate backend
case "$BACKEND" in
    cli|tui|web)
        ;;
    *)
        echo "Usage: $0 [cli|tui|web]"
        echo ""
        echo "Launches TypeDialog for interactive tracker environment configuration."
        echo "Backend options:"
        echo "  cli - Command-line interface (simple prompts)"
        echo "  tui - Terminal UI (interactive panels)"
        echo "  web - Web server (browser-based) [default]"
        exit 1
        ;;
esac

# ============================================================================
# PRE-FLIGHT CHECKS
# ============================================================================

print_header "Torrust Tracker Configuration"

# Check dependencies
if ! verify_dependencies; then
    exit 1
fi
print_success "All dependencies available"
echo ""

# Check if form exists
if [[ ! -f "$FORM_FILE" ]]; then
    print_error "Form file not found: $FORM_FILE"
    exit 1
fi

# Check if template exists
if [[ ! -f "$TEMPLATE_FILE" ]]; then
    print_error "Template file not found: $TEMPLATE_FILE"
    exit 1
fi

# Create values directory if it doesn't exist
mkdir -p "$VALUES_DIR"

# Check if config exists, create minimal one if not
if [[ ! -f "$CONFIG_FILE" ]]; then
    print_info "No existing configuration found. Creating minimal config..."
    cat > "$CONFIG_FILE" <<'EOF'
# Torrust Tracker Environment Configuration
# This file will be populated by the configuration wizard

let schemas_env = import "../schemas/environment.ncl" in
let schemas_ssh = import "../schemas/ssh.ncl" in
let schemas_provider = import "../schemas/provider.ncl" in
let schemas_tracker = import "../schemas/tracker.ncl" in
let schemas_features = import "../schemas/features.ncl" in

let defaults_env = import "../defaults/environment.ncl" in
let defaults_ssh = import "../defaults/ssh.ncl" in
let defaults_provider = import "../defaults/provider.ncl" in
let defaults_tracker = import "../defaults/tracker.ncl" in
let defaults_features = import "../defaults/features.ncl" in

let constraints = import "../constraints.toml" in

let validators = import "../validators/environment.ncl" in
let validators_instance = import "../validators/instance.ncl" in
let validators_username = import "../validators/username.ncl" in
let validators_common = import "../validators/common.ncl" in
let validators_network = import "../validators/network.ncl" in
let validators_tracker = import "../validators/tracker.ncl" in

let user_config = {
} in

defaults_env & defaults_ssh & defaults_provider & defaults_tracker & defaults_features & user_config
EOF
    print_success "Created initial configuration"
    echo ""
fi

# Create backup if config exists and has content
if [[ -f "$CONFIG_FILE" ]] && [[ -s "$CONFIG_FILE" ]]; then
    BACKUP="${CONFIG_FILE}.$(date +%Y%m%d_%H%M%S).bak"
    cp "$CONFIG_FILE" "$BACKUP"
    print_info "Backed up existing config to: $(basename "$BACKUP")"
    echo ""
fi

# ============================================================================
# LAUNCH TYPEDIALOG
# ============================================================================

print_header "Launching TypeDialog ($BACKEND backend)"

# Show web server info if using web backend
if [[ "$BACKEND" == "web" ]]; then
    echo "🌐 Web server will start on: http://${TYPEDIALOG_HOST}:${TYPEDIALOG_PORT}"
    echo "   (Override with: TYPEDIALOG_PORT=8080 TYPEDIALOG_HOST=0.0.0.0 $0)"
    echo ""
fi

# Show template info
print_info "Using Nickel template: $(basename "$TEMPLATE_FILE")"
echo ""

# Build and execute nickel-roundtrip command based on backend
case "$BACKEND" in
    cli)
        print_info "Launching TypeDialog CLI..."
        typedialog nickel-roundtrip "$CONFIG_FILE" "$FORM_FILE" --output "$CONFIG_FILE" --ncl-template "$TEMPLATE_FILE"
        EXIT_CODE=$?
        ;;
    tui)
        if ! command_exists "typedialog-tui"; then
            print_error "typedialog-tui not found. Install with: cargo install typedialog --features tui"
            exit 1
        fi
        print_info "Launching TypeDialog TUI..."
        typedialog-tui nickel-roundtrip "$CONFIG_FILE" "$FORM_FILE" --output "$CONFIG_FILE" --ncl-template "$TEMPLATE_FILE"
        EXIT_CODE=$?
        ;;
    web)
        if ! command_exists "typedialog-web"; then
            print_error "typedialog-web not found. Install with: cargo install typedialog --features web"
            exit 1
        fi
        print_info "Launching TypeDialog Web..."
        typedialog-web nickel-roundtrip "$CONFIG_FILE" "$FORM_FILE" --output "$CONFIG_FILE" --ncl-template "$TEMPLATE_FILE"
        EXIT_CODE=$?
        ;;
esac

if [[ $EXIT_CODE -ne 0 ]]; then
    print_error "Configuration cancelled or failed (exit code: $EXIT_CODE)"
    if [[ -f "${BACKUP}" ]]; then
        print_info "Previous config preserved in backup"
    fi
    exit "$EXIT_CODE"
fi

# ============================================================================
# SUCCESS
# ============================================================================

print_success "Configuration saved to: $CONFIG_FILE"
echo ""

echo "Next steps:"
echo "  1. Review the configuration:"
echo "     cat $CONFIG_FILE"
echo ""
echo "  2. Export to JSON for deployment:"
echo "     nickel export --format json $CONFIG_FILE > envs/my-env.json"
echo ""
echo "  3. Create environment:"
echo "     cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json"
echo ""
echo "  4. Re-run this script anytime to update:"
echo "     $0 $BACKEND"
echo ""

# Clean up backup if everything succeeded
if [[ -f "${BACKUP}" ]]; then
    rm -f "$BACKUP"
fi
