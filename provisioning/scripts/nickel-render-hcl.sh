#!/bin/bash
# Render Nickel template to HCL format
#
# Evaluates a Nickel template and converts to HCL (Terraform/OpenTofu) format
# Primarily for generating tfvars files
#
# Usage:
#   ./nickel-render-hcl.sh <template.ncl> <output.tfvars>
#   ./nickel-render-hcl.sh provisioning/templates/tofu/lxd/variables.ncl build/tofu/lxd/terraform.tfvars

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

# Convert JSON to HCL format
json_to_hcl() {
    local json="$1"

    # Use jq to generate HCL variable definitions
    echo "$json" | jq -r '
        def to_hcl:
            if type == "object" then
                to_entries | map(
                    if .value | type == "object" then
                        .key + " = {\n" +
                        (.value | to_entries | map("  \(.key) = \(.value | @json)") | join("\n")) +
                        "\n}"
                    elif .value | type == "array" then
                        .key + " = [" +
                        (.value | map(@json) | join(", ")) +
                        "]"
                    else
                        .key + " = \(.value | @json)"
                    end
                ) | join("\n")
            else
                @json
            end;

        to_hcl
    '
}

main() {
    if [[ $# -ne 2 ]]; then
        echo "Usage: $0 <template.ncl> <output.tfvars>"
        exit 1
    fi

    local template_path="$1"
    local output_path="$2"

    echo -e "\n${YELLOW}📋${NC} Rendering to HCL: $template_path → $output_path"

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

    # Convert JSON to HCL
    log_progress "Converting to HCL..."
    if ! hcl_content=$(json_to_hcl "$json"); then
        log_error "HCL conversion failed"
        exit 1
    fi

    # Write output
    if ! echo "$hcl_content" > "$output_path"; then
        log_error "Failed to write output file"
        exit 1
    fi

    log_info "HCL rendered successfully"

    local file_size
    file_size=$(wc -c < "$output_path")
    echo "   File: $output_path ($file_size bytes)"
}

main "$@"
