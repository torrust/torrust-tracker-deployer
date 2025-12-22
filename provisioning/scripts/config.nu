#!/usr/bin/env nu
# Torrust Tracker Environment Configuration Wizard (Nushell variant)
# Main orchestration script for the configuration wizard workflow
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
# - No try-catch: Uses `do { } | complete` pattern
# - No let mut: Pure immutable transformations
# - Function signatures with explicit types
# - External commands prefixed with `^`
# - String interpolation: [$var] for variables, ($expr) for expressions
#
# This script:
# 1. Verifies TypeDialog and Nickel are installed
# 2. Launches interactive TypeDialog form
# 3. Converts JSON output to Nickel configuration
# 4. Validates with Nickel validators
# 5. Exports final JSON to envs/ directory
#
# Usage:
#   ./provisioning/scripts/config.nu

# Check if a command exists
def check-command [cmd: string]: nothing -> bool {
    (do { ^which $cmd } | complete).exit_code == 0
}

# Print section header
def print-header [msg: string]: nothing -> nothing {
    print "═══════════════════════════════════════════════════════════"
    print $"🎯 ($msg)"
    print "═══════════════════════════════════════════════════════════"
    print ""
}

# Print step message with progress
def print-step [step: string, total: string, msg: string]: nothing -> nothing {
    print $"📝 Step ($step)/($total): ($msg)..."
}

# Print success message
def print-success [msg: string]: nothing -> nothing {
    print $"✅ ($msg)"
}

# Print info message
def print-info [msg: string]: nothing -> nothing {
    print $"ℹ️  ($msg)"
}

# Print error message to stderr
def print-error [msg: string]: nothing -> nothing {
    print -e $"❌ ($msg)"
}

# Verify dependencies are installed
def verify-dependencies []: nothing -> bool {
    print "Checking dependencies..."
    print ""

    let deps = [
        {name: "typedialog", install: "cargo install typedialog"}
        {name: "nickel", install: "cargo install nickel-lang-cli"}
        {name: "jq", install: "brew install jq (or apt-get install jq)"}
    ]

    let missing = ($deps | where {|dep| not (check-command $dep.name)})

    if ($missing | is-not-empty) {
        print-error "Missing dependencies:"
        $missing | each {|dep|
            print -e $"   - ($dep.name): ($dep.install)"
        }
        return false
    }

    print-success "All dependencies available"
    print ""
    return true
}

# Main wizard function
def main []: nothing -> nothing {
    print-header "Torrust Tracker - Environment Configuration Wizard"

    # Verify dependencies
    if not (verify-dependencies) {
        exit 1
    }

    # Get directory paths - script is expected to be in provisioning/scripts/
    # Default to known location, can be overridden with TORRUST_SCRIPT_DIR env var
    let script_dir = (
        $env | get --optional TORRUST_SCRIPT_DIR // "/Users/Akasha/Development/torrust-tracker-deployer/provisioning/scripts"
    )
    let provisioning_dir = ($script_dir | path dirname)
    let project_root = ($provisioning_dir | path dirname)
    let envs_dir = ($project_root | path join "envs")
    let values_dir = ($provisioning_dir | path join "values")
    let form_path = ($provisioning_dir | path join "config-form.toml")

    # Create directories if they don't exist
    ^mkdir -p $envs_dir
    ^mkdir -p $values_dir

    # Step 1: Run TypeDialog form
    print-step "1" "4" "Collecting configuration via interactive form"

    let temp_output = $"/tmp/typedialog-output-(date now | format date '%s').json"

    # TypeDialog outputs to stdout, redirect to file
    ^typedialog $form_path | save --force $temp_output

    # Check if output file has content
    if not ($temp_output | path exists) or (open $temp_output | is-empty) {
        print-error "TypeDialog output is empty. Wizard cancelled."
        exit 1
    }

    print-success "Configuration collected"
    print ""

    # Step 2: Extract environment name
    print-step "2" "4" "Processing configuration"

    let config = (open $temp_output)
    let env_name = $config.environment_name

    if ($env_name | is-empty) {
        print-error "Could not extract environment name from form output"
        exit 1
    }

    print-info $"Environment name: ($env_name)"

    let values_file = ($values_dir | path join $"($env_name).ncl")
    let json_file = ($envs_dir | path join $"($env_name).json")

    # Step 3: Convert JSON to Nickel
    print-step "3" "4" "Converting to Nickel configuration"

    let converter_script = ($script_dir | path join "json-to-nickel.nu")
    ^nu -c $"source '($converter_script)'; main '($temp_output)' '($values_file)'"

    if not ($values_file | path exists) {
        print-error "Nickel file generation failed"
        exit 1
    }

    print-success $"Nickel configuration generated: ($values_file)"
    print ""

    # Step 4: Validate Nickel
    print-info "Validating Nickel configuration..."

    let validate_script = ($script_dir | path join "validate-nickel.nu")
    ^nu -c $"source '($validate_script)'; main '($values_file)'"

    print-success "Nickel validation passed"
    print ""

    # Step 5: Export Nickel to JSON
    print-step "4" "4" "Exporting to JSON format"

    let exporter_script = ($script_dir | path join "nickel-to-json.nu")
    ^nu -c $"source '($exporter_script)'; main '($values_file)' '($json_file)'"

    if not ($json_file | path exists) {
        print-error "JSON export failed"
        exit 1
    }

    print-success $"JSON configuration exported: ($json_file)"
    print ""

    # Cleanup temporary file
    ^rm -f $temp_output

    # Success summary
    print-header "Configuration Generation Complete!"
    print ""

    print-info "Generated files:"
    print $"   - Nickel: ($values_file)"
    print $"   - JSON:   ($json_file)"
    print ""

    print-info "Next steps:"
    print $"   1. Review configuration: cat ($json_file) | jq ."
    print $"   2. Create environment:   cargo run --bin torrust-tracker-deployer -- create environment --env-file ($json_file)"
    print $"   3. Provision:            cargo run --bin torrust-tracker-deployer -- provision ($env_name)"
    print ""
}

# Execute main
main
