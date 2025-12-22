#!/bin/bash
# Render Nickel template to YAML format
#
# Evaluates a Nickel template and converts to YAML using yq
#
# Usage:
#   ./nickel-render-yaml.sh <template.ncl> <output.yml>
#   ./nickel-render-yaml.sh provisioning/templates/prometheus/config.ncl build/prometheus/prometheus.yml

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

main() {
    if [[ $# -ne 2 ]]; then
        echo "Usage: $0 <template.ncl> <output.yml>"
        exit 1
    fi

    local template_path="$1"
    local output_path="$2"

    echo -e "\n${YELLOW}📋${NC} Rendering to YAML: $template_path → $output_path"

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

    # Convert JSON to YAML using yq
    log_progress "Converting to YAML..."
    if ! echo "$json" | yq -P . > "$output_path" 2>&1; then
        log_error "YAML conversion failed"
        exit 1
    fi

    log_info "YAML rendered successfully"

    local file_size
    file_size=$(wc -c < "$output_path")
    echo "   File: $output_path ($file_size bytes)"
}

main "$@"
