#!/bin/bash
# Generic Nickel template renderer - evaluate and convert to target format
#
# Evaluates a Nickel (.ncl) template file and converts the JSON output to a target format.
#
# Usage:
#   ./nickel-render.sh <template.ncl> <output_format> <output_path>
#   ./nickel-render.sh provisioning/templates/tracker/config.ncl toml build/tracker.toml
#   ./nickel-render.sh provisioning/templates/prometheus/config.ncl yaml build/prometheus/prometheus.yml
#
# Supported formats: json, yaml, toml, hcl, env

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}✅${NC} $1"
}

log_error() {
    echo -e "${RED}❌${NC} $1" >&2
}

log_progress() {
    echo -e "${YELLOW}⏳${NC} $1"
}

log_header() {
    echo -e "\n${YELLOW}📋${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Validate dependencies
check_dependencies() {
    local format="$1"

    case "$format" in
        yaml)
            if ! command_exists yq; then
                log_error "yq is required for YAML conversion. Install with: brew install yq"
                exit 1
            fi
            ;;
        json)
            if ! command_exists jq; then
                log_error "jq is required for JSON validation. Install with: brew install jq"
                exit 1
            fi
            ;;
    esac
}

# Convert JSON value to TOML literal
value_to_toml_literal() {
    local value="$1"
    local type="$2"

    case "$type" in
        "null")
            echo "null"
            ;;
        "boolean")
            echo "$value"
            ;;
        "number")
            echo "$value"
            ;;
        "string")
            # Escape quotes and backslashes
            local escaped
            escaped=$(echo "$value" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | sed 's/$//')
            echo "\"$escaped\""
            ;;
        *)
            echo "\"$value\""
            ;;
    esac
}

# Convert JSON to TOML format
json_to_toml() {
    local json="$1"

    # Use jq to extract and format as TOML
    # This is a simplified approach - for complex structures use toml-cli
    echo "$json" | jq -r 'to_entries[] | "\(.key) = \(.value | @json)"'
}

# Convert JSON value to HCL literal
value_to_hcl_literal() {
    local value="$1"
    local type="$2"

    case "$type" in
        "null")
            echo "null"
            ;;
        "boolean")
            echo "$value"
            ;;
        "number")
            echo "$value"
            ;;
        "string")
            # Escape special characters for HCL
            local escaped
            escaped=$(echo "$value" | \
                sed 's/\\/\\\\/g' | \
                sed 's/"/\\"/g' | \
                sed 's/\$/\\$/g' | \
                sed 's/$//')
            echo "\"$escaped\""
            ;;
        *)
            echo "\"$value\""
            ;;
    esac
}

# Convert JSON to HCL format
json_to_hcl() {
    local json="$1"

    # Convert JSON object to HCL variable format
    echo "$json" | jq -r 'to_entries[] |
        if .value | type == "object" then
            "\(.key) = {\n" + (.value | to_entries[] | "  \(.key) = \(.value | @json)") + "\n}"
        else
            "\(.key) = \(.value | @json)"
        end'
}

# Convert JSON value to ENV literal
value_to_env_literal() {
    local value="$1"
    local type="$2"

    case "$type" in
        "null")
            echo ""
            ;;
        "boolean"|"number")
            echo "$value"
            ;;
        "string")
            # Quote if contains spaces or special characters
            if echo "$value" | grep -qE '[ ="]'; then
                local escaped
                escaped="${value//\"/\\\"}"
                echo "\"$escaped\""
            else
                echo "$value"
            fi
            ;;
        "array")
            # Convert array to comma-separated string
            echo "$value" | tr '\n' ','
            ;;
        *)
            echo "$value"
            ;;
    esac
}

# Convert JSON to ENV format
json_to_env() {
    local json="$1"

    # Flatten object and create KEY=VALUE pairs
    echo "$json" | jq -r 'to_entries[] |
        if .value | type == "array" then
            "\(.key)=" + (.value | join(","))
        elif .value | type == "null" then
            "\(.key)="
        else
            "\(.key)=\(.value)"
        end' | sort
}

# Main rendering function
render_nickel() {
    local template_path="$1"
    local output_format="$2"
    local output_path="$3"

    # Validate input
    if [[ ! -f "$template_path" ]]; then
        log_error "Template file not found: $template_path"
        exit 1
    fi

    log_header "Rendering Nickel template: $template_path"
    echo "Output format: $output_format"
    echo "Output path: $output_path"

    # Check dependencies
    check_dependencies "$output_format"

    # Step 1: Evaluate Nickel to JSON
    log_progress "Evaluating Nickel template..."
    local json
    if ! json=$(nickel export --format json "$template_path" 2>&1); then
        log_error "Failed to evaluate Nickel template"
        echo "$json" >&2
        exit 1
    fi
    log_info "Nickel template evaluated successfully"

    # Step 2: Convert to target format
    local formatted
    case "$output_format" in
        json)
            log_progress "Converting to JSON..."
            formatted="$json"
            ;;
        yaml)
            log_progress "Converting to YAML..."
            if ! formatted=$(echo "$json" | yq -P . 2>&1); then
                log_error "Failed to convert to YAML"
                exit 1
            fi
            ;;
        toml)
            log_progress "Converting to TOML..."
            formatted=$(json_to_toml "$json")
            ;;
        hcl)
            log_progress "Converting to HCL..."
            formatted=$(json_to_hcl "$json")
            ;;
        env)
            log_progress "Converting to ENV..."
            formatted=$(json_to_env "$json")
            ;;
        *)
            log_error "Unknown output format: $output_format"
            echo "Supported formats: json, yaml, toml, hcl, env"
            exit 1
            ;;
    esac

    # Step 3: Create output directory
    local output_dir
    output_dir=$(dirname "$output_path")
    if [[ ! -d "$output_dir" ]]; then
        log_progress "Creating output directory: $output_dir"
        mkdir -p "$output_dir"
    fi

    # Step 4: Write output file
    log_progress "Writing output file..."
    if ! echo "$formatted" > "$output_path"; then
        log_error "Failed to write output file: $output_path"
        exit 1
    fi
    log_info "Output file written successfully"

    # Step 5: Verify output
    if [[ ! -f "$output_path" ]]; then
        log_error "Output file was not created"
        exit 1
    fi

    local file_size
    file_size=$(wc -c < "$output_path")
    if [[ $file_size -eq 0 ]]; then
        log_error "Output file is empty"
        exit 1
    fi

    echo ""
    log_info "Success! Rendered template: $output_path ($file_size bytes)"
}

# Main
main() {
    if [[ $# -ne 3 ]]; then
        echo "Usage: $0 <template.ncl> <output_format> <output_path>"
        echo ""
        echo "Supported formats: json, yaml, toml, hcl, env"
        echo ""
        echo "Examples:"
        echo "  $0 provisioning/templates/tracker/config.ncl toml build/tracker.toml"
        echo "  $0 provisioning/templates/prometheus/config.ncl yaml build/prometheus/prometheus.yml"
        exit 1
    fi

    render_nickel "$1" "$2" "$3"
}

main "$@"
