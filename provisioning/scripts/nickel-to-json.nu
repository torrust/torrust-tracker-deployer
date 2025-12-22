#!/usr/bin/env nu
# Export Nickel configuration to JSON format (Nushell variant)
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
# - Function signatures with explicit types
# - External commands prefixed with `^`
# - String interpolation: [$var] for variables
#
# Usage:
#   ./nickel-to-json.nu <input.ncl> <output.json>

def main [input_nickel: string, output_json: string]: nothing -> nothing {
    if not ($input_nickel | path exists) {
        print -e $"Error: Input Nickel file not found: ($input_nickel)"
        exit 1
    }

    # Export Nickel to JSON
    ^nickel export --format json $input_nickel | save --force $output_json

    # Verify output was generated
    if not ($output_json | path exists) {
        print -e "❌ Nickel export failed"
        exit 1
    }

    let file_size = (ls $output_json | get size.0)
    if ($file_size == 0) {
        print -e "❌ Nickel export produced empty output"
        ^rm -f $output_json
        exit 1
    }

    print $"✅ JSON exported: ($output_json)"
}

# Script is a library - call main directly:
# nu -c 'source ./nickel-to-json.nu; main "input.ncl" "output.json"'
