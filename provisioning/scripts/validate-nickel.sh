#!/bin/bash
# Validate Nickel configuration file
#
# This script evaluates a Nickel configuration file to check for syntax errors
# and validation failures. If validation succeeds, all values are validated
# according to the defined validators.
#
# Usage:
#   ./validate-nickel.sh <config.ncl>
#
# Arguments:
#   config.ncl - Nickel configuration file to validate (required)

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <config.ncl>" >&2
    exit 1
fi

readonly INPUT_NICKEL="$1"

if [[ ! -f "$INPUT_NICKEL" ]]; then
    echo "❌ Error: File not found: $INPUT_NICKEL" >&2
    exit 1
fi

echo "Validating Nickel configuration: $INPUT_NICKEL"
echo ""

if nickel eval "$INPUT_NICKEL" > /dev/null 2>&1; then
    echo "✅ Nickel configuration is valid"
    exit 0
else
    echo "❌ Nickel validation failed. Errors:"
    echo ""
    nickel eval "$INPUT_NICKEL"
    exit 1
fi
