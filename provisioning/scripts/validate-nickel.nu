#!/usr/bin/env nu
# Validate Nickel configuration file (Nushell variant)
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
# - Function signatures with explicit types
# - External commands prefixed with `^`
# - String interpolation: [$var] for variables
#
# Usage:
#   ./validate-nickel.nu <config.ncl>

def main [input_nickel: string]: nothing -> nothing {
    if not ($input_nickel | path exists) {
        print -e $"❌ Error: File not found: ($input_nickel)"
        exit 1
    }

    print $"Validating Nickel configuration: ($input_nickel)"
    print ""

    let validate_result = (do { ^nickel eval $input_nickel } | complete)

    if $validate_result.exit_code == 0 {
        print "✅ Nickel configuration is valid"
        exit 0
    } else {
        print "❌ Nickel validation failed. Errors:"
        print ""
        print $validate_result.stdout
        exit 1
    }
}

# Script is a library - call main directly:
# nu -c 'source ./validate-nickel.nu; main "config.ncl"'
