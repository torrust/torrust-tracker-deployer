#!/bin/bash
# Render Nickel template to ENV format
#
# Evaluates a Nickel template and converts to ENV (KEY=VALUE) format
# Suitable for .env files and environment variable configuration
#
# Usage:
#   ./nickel-render-env.sh <template.ncl> <output.env>
#   ./nickel-render-env.sh provisioning/templates/docker-compose/env.ncl build/docker-compose/.env

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}✅${NC} $1"
}

log_error() {
    echo -e "${RED}❌${NC} $1" >&2
}

log_progress() {
    echo -e "${YELLOW}⏳${NC} $1"
}

# Convert JSON to ENV format
json_to_env() {
    local json="$1"

    # Convert to KEY=VALUE pairs, sorted alphabetically
    echo "$json" | jq -r '
        to_entries | map(
            if .value | type == "array" then
                .key + "=" + (.value | join(","))
            elif .value | type == "null" then
                .key + "="
            elif (.value | type) == "string" and (.value | contains(" ") or contains("=")) then
                .key + "=\"" + (.value | gsub("\""; "\\\"")) + "\""
            else
                .key + "=" + (.value | tostring)
            end
        ) | sort | join("\n")
    '
}

main() {
    if [[ $# -ne 2 ]]; then
        echo "Usage: $0 <template.ncl> <output.env>"
        exit 1
    fi

    local template_path="$1"
    local output_path="$2"

    echo -e "\n${YELLOW}📋${NC} Rendering to ENV: $template_path → $output_path"

    if [[ ! -f "$template_path" ]]; then
        log_error "Template not found: $template_path"
        exit 1
    fi

    # Create output directory
    local output_dir
    output_dir=$(dirname "$output_path")
    mkdir -p "$output_dir"

    # Evaluate Nickel to JSON
    log_progress "Evaluating Nickel..."
    if ! json=$(nickel export --format json "$template_path" 2>&1); then
        log_error "Nickel evaluation failed"
        echo "$json" >&2
        exit 1
    fi

    # Convert JSON to ENV
    log_progress "Converting to ENV..."
    if ! env_content=$(json_to_env "$json"); then
        log_error "ENV conversion failed"
        exit 1
    fi

    # Write output
    if ! echo "$env_content" > "$output_path"; then
        log_error "Failed to write output file"
        exit 1
    fi

    log_info "ENV rendered successfully"

    local file_size
    file_size=$(wc -c < "$output_path")
    echo "   File: $output_path ($file_size bytes)"
}

main "$@"
