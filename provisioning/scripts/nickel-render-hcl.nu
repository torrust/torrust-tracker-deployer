#!/usr/bin/env nu
# Render Nickel template to HCL format
#
# Evaluates a Nickel template and converts to HCL (Terraform/OpenTofu) format
# Primarily for generating tfvars files
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
#
# Usage:
#   ./nickel-render-hcl.nu <template.ncl> <output.tfvars>
#   ./nickel-render-hcl.nu provisioning/templates/tofu/lxd/variables.ncl build/tofu/lxd/terraform.tfvars

def value_to_hcl [value]: any -> string {
    # Convert JSON value to HCL literal
    match ($value | type) {
        "null" => "null",
        "bool" => ($value | into string),
        "int" => ($value | into string),
        "float" => ($value | into string),
        "string" => {
            let escaped = ($value
                | str replace --all '\\' '\\\\'
                | str replace --all '"' '\"'
                | str replace --all '$' '\$'
                | str replace --all '\n' '\\n'
                | str replace --all '\r' '\\r'
                | str replace --all '\t' '\\t'
            )
            $"\"($escaped)\""
        },
        "list" => {
            if ($value | length) == 0 {
                "[]"
            } else {
                let items = ($value | each {|item| value_to_hcl $item} | str join ", ")
                $"[($items)]"
            }
        },
        "record" => {
            # Nested object - HCL object format
            let pairs = ($value | items {|k, v|
                let hcl_val = (value_to_hcl $v)
                $"  ($k) = ($hcl_val)"
            } | str join "\n")
            $"{\n($pairs)\n}"
        },
        _ => $"\"($value)\""
    }
}

export def main [
    template_path: path,    # Path to Nickel template (.ncl)
    output_path: path,      # Where to write HCL output (usually .tfvars)
]: nothing -> nothing {

    print $"📋 Rendering to HCL: ($template_path) → ($output_path)"

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
        let parsed = ($json | from json)

        if ($parsed | type) != "record" {
            error make {msg: "HCL requires a root-level object"}
        }

        # Step 3: Convert to HCL
        print "⏳ Converting to HCL..."
        let hcl_lines = ($parsed | items {|key, value|
            let hcl_value = (value_to_hcl $value)
            $"($key) = ($hcl_value)"
        })
        let hcl_content = ($hcl_lines | str join "\n")

        # Step 4: Write output
        $hcl_content | save --force $output_path

        print "✅ HCL rendered successfully"

        let file_size = (ls $output_path | get size.0)
        print $"   File: ($output_path) (($file_size | into string)) bytes"
    } catch {|err|
        print -e $"❌ Rendering failed: ($err)"
        exit 1
    }
}

# Direct invocation:
# nu ./provisioning/scripts/nickel-render-hcl.nu provisioning/templates/tofu/lxd/variables.ncl build/tofu/lxd/terraform.tfvars
