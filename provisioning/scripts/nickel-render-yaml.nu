#!/usr/bin/env nu
# Render Nickel template to YAML format
#
# Evaluates a Nickel template and converts to YAML using yq
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
#
# Usage:
#   ./nickel-render-yaml.nu <template.ncl> <output.yml>
#   ./nickel-render-yaml.nu provisioning/templates/prometheus/config.ncl build/prometheus/prometheus.yml

export def main [
    template_path: path,    # Path to Nickel template (.ncl)
    output_path: path,      # Where to write YAML output
]: nothing -> nothing {

    print $"📋 Rendering to YAML: ($template_path) → ($output_path)"

    if not ($template_path | path exists) {
        print -e $"❌ Template not found: ($template_path)"
        exit 1
    }

    # Step 1: Create output directory
    let output_dir = ($output_path | path dirname)
    if not ($output_dir | path exists) {
        mkdir $output_dir
    }

    # Step 2: Evaluate Nickel to JSON
    try {
        print "⏳ Evaluating Nickel..."
        let json = (^nickel export --format json $template_path)

        # Step 3: Convert JSON to YAML using yq
        print "⏳ Converting to YAML..."
        $json | ^yq -P . | save --force $output_path

        print "✅ YAML rendered successfully"

        let file_size = (ls $output_path | get size.0)
        print $"   File: ($output_path) (($file_size | into string)) bytes"
    } catch {|err|
        print -e $"❌ Rendering failed: ($err)"
        exit 1
    }
}

# Direct invocation:
# nu ./provisioning/scripts/nickel-render-yaml.nu provisioning/templates/prometheus/config.ncl build/prometheus/prometheus.yml
