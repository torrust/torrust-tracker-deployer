#!/usr/bin/env nu
# Render Nickel template to ENV format
#
# Evaluates a Nickel template and converts to ENV (KEY=VALUE) format
# Suitable for .env files and environment variable configuration
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
#
# Usage:
#   ./nickel-render-env.nu <template.ncl> <output.env>
#   ./nickel-render-env.nu provisioning/templates/docker-compose/env.ncl build/docker-compose/.env

def value_to_env [value]: any -> string {
    # Convert JSON value to ENV literal
    match ($value | type) {
        "null" => "",
        "bool" => ($value | into string),
        "int" => ($value | into string),
        "float" => ($value | into string),
        "string" => {
            # Quote if contains spaces, equals, or quotes
            if ($value | str contains ' ') or ($value | str contains '=') or ($value | str contains '"') {
                let escaped = ($value | str replace --all '"' '\"')
                $"\"($escaped)\""
            } else {
                $value
            }
        },
        "list" => {
            # Convert array to comma-separated string
            $value | each {|item|
                match ($item | type) {
                    "string" => $item,
                    _ => ($item | into string)
                }
            } | str join ","
        },
        "record" => {
            error make {msg: "Nested objects are not supported in ENV format"}
        },
        _ => ($value | into string)
    }
}

export def main [
    template_path: path,    # Path to Nickel template (.ncl)
    output_path: path,      # Where to write ENV output (.env)
]: nothing -> nothing {

    print $"📋 Rendering to ENV: ($template_path) → ($output_path)"

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
            error make {msg: "ENV requires a flat root-level object"}
        }

        # Step 3: Convert to ENV
        print "⏳ Converting to ENV..."
        let env_lines = ($parsed | items {|key, value|
            let env_value = (value_to_env $value)
            $"($key)=($env_value)"
        } | sort)
        let env_content = ($env_lines | str join "\n")

        # Step 4: Write output
        $env_content | save --force $output_path

        print "✅ ENV rendered successfully"

        let file_size = (ls $output_path | get size.0)
        print $"   File: ($output_path) (($file_size | into string)) bytes"
    } catch {|err|
        print -e $"❌ Rendering failed: ($err)"
        exit 1
    }
}

# Direct invocation:
# nu ./provisioning/scripts/nickel-render-env.nu provisioning/templates/docker-compose/env.ncl build/docker-compose/.env
