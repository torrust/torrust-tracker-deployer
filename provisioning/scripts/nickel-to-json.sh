#!/bin/bash
# Export Nickel configuration to JSON format
#
# This script evaluates a Nickel configuration file and exports it as JSON.
# The resulting JSON is suitable for use with the Torrust Tracker Deployer.
#
# Usage:
#   ./nickel-to-json.sh <input.ncl> <output.json>
#
# Arguments:
#   input.ncl   - Nickel configuration file (required)
#   output.json - JSON output file (required)

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <input.ncl> <output.json>" >&2
    exit 1
fi

readonly INPUT_NICKEL="$1"
readonly OUTPUT_JSON="$2"

if [[ ! -f "$INPUT_NICKEL" ]]; then
    echo "Error: Input Nickel file not found: $INPUT_NICKEL" >&2
    exit 1
fi

# Export Nickel to JSON using nickel CLI
if ! nickel export --format json "$INPUT_NICKEL" > "$OUTPUT_JSON" 2>&1; then
    echo "❌ Nickel export failed" >&2
    cat "$OUTPUT_JSON" >&2
    rm -f "$OUTPUT_JSON"
    exit 1
fi

if [[ ! -s "$OUTPUT_JSON" ]]; then
    echo "❌ Nickel export produced empty output" >&2
    rm -f "$OUTPUT_JSON"
    exit 1
fi

echo "✅ JSON exported: $OUTPUT_JSON"
