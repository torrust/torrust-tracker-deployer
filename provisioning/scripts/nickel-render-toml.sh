#!/bin/bash
# Render Nickel template to TOML format
#
# Evaluates a Nickel template and converts to TOML format
#
# Usage:
#   ./nickel-render-toml.sh <template.ncl> <output.toml>
#   ./nickel-render-toml.sh provisioning/templates/tracker/config.ncl build/tracker.toml

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

# Convert JSON to TOML-like format
# Note: This is a simplified converter. For complex TOML structures,
# consider using toml-cli or similar tool
json_to_toml() {
    local json="$1"

    # Use jq to extract key-value pairs
    # Handle nested objects and arrays
    echo "$json" | jq -r '
        def to_toml:
            if type == "object" then
                to_entries | map(
                    if .value | type == "object" then
                        "[" + .key + "]\n" + (.value | to_toml)
                    elif .value | type == "array" then
                        if (.value[0] | type) == "object" then
                            "[[" + .key + "]]\n" +
                            (.value | map(to_toml) | join("\n[[" + .key + "]]\n"))
                        else
                            .key + " = " + (.value | @json)
                        end
                    else
                        .key + " = " + (.value | @json)
                    end
                ) | join("\n")
            else
                @json
            end;

        to_toml
    '
}

main() {
    if [[ $# -ne 2 ]]; then
        echo "Usage: $0 <template.ncl> <output.toml>"
        exit 1
    fi

    local template_path="$1"
    local output_path="$2"

    echo -e "\n${YELLOW}📋${NC} Rendering to TOML: $template_path → $output_path"

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
    local json
    if ! json=$(nickel export --format json "$template_path" 2>&1); then
        log_error "Nickel evaluation failed"
        echo "$json" >&2
        exit 1
    fi

    # Convert JSON to TOML
    log_progress "Converting to TOML..."
    if ! toml_content=$(json_to_toml "$json"); then
        log_error "TOML conversion failed"
        exit 1
    fi

    # Write output
    if ! echo "$toml_content" > "$output_path"; then
        log_error "Failed to write output file"
        exit 1
    fi

    log_info "TOML rendered successfully"

    local file_size
    file_size=$(wc -c < "$output_path")
    echo "   File: $output_path ($file_size bytes)"
}

main "$@"
