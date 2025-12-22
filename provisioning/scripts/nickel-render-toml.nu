#!/usr/bin/env nu
# Render Nickel template to TOML format
#
# Evaluates a Nickel template and converts to TOML format
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
#
# Usage:
#   ./nickel-render-toml.nu <template.ncl> <output.toml>
#   ./nickel-render-toml.nu provisioning/templates/tracker/config.ncl build/tracker.toml

def value_to_toml [value]: any -> string {
    # Convert JSON value to TOML literal
    match ($value | type) {
        "null" => "null",
        "bool" => ($value | into string),
        "int" => ($value | into string),
        "float" => ($value | into string),
        "string" => {
            let escaped = ($value
                | str replace --all '\\' '\\\\'
                | str replace --all '"' '\"'
                | str replace --all '\n' '\\n'
                | str replace --all '\r' '\\r'
            )
            $"\"($escaped)\""
        },
        "list" => {
            if ($value | length) == 0 {
                "[]"
            } else {
                let is_simple_list = ($value | all {|item| ($item | type) in ["null", "bool", "int", "float", "string"]})

                if $is_simple_list {
                    # Simple inline array
                    let items = ($value | each {|item| value_to_toml $item} | str join ", ")
                    $"[($items)]"
                } else {
                    # Array of tables - will be handled specially
                    "[[array]]"
                }
            }
        },
        "record" => {
            # Inline table
            let pairs = ($value | items {|k, v|
                $"($k) = (value_to_toml $v)"
            } | str join ", ")
            $"{ ($pairs) }"
        },
        _ => $"\"($value)\""
    }
}

def flatten_toml_object [obj: record, prefix: string = ""]: record -> list {
    # Flatten nested TOML object into lines
    let lines = ($obj | items {|key, value|
        let full_key = if ($prefix | is-empty) { $key } else { $"($prefix).($key)" }

        match ($value | type) {
            "record" => {
                # Recursively flatten nested objects
                flatten_toml_object $value $full_key
            },
            "list" => {
                # Check if list contains objects (array of tables)
                if ($value | length) > 0 and ($value.0 | type) == "record" {
                    # Array of tables: [[section]]
                    $value | enumerate | each {|item|
                        let idx = $item.index
                        let table_obj = $item.item
                        let table_items = ($table_obj | items {|k, v|
                            $"($k) = (value_to_toml $v)"
                        } | str join "\n")
                        $"[[$full_key]]\n($table_items)"
                    }
                } else {
                    # Simple array
                    let array_value = value_to_toml $value
                    $"($full_key) = ($array_value)"
                }
            },
            _ => {
                let toml_value = value_to_toml $value
                $"($full_key) = ($toml_value)"
            }
        }
    } | flatten)

    $lines
}

export def main [
    template_path: path,    # Path to Nickel template (.ncl)
    output_path: path,      # Where to write TOML output
]: nothing -> nothing {

    print $"📋 Rendering to TOML: ($template_path) → ($output_path)"

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
            error make {msg: "TOML requires a root-level object"}
        }

        # Step 3: Convert to TOML
        print "⏳ Converting to TOML..."
        let toml_lines = (flatten_toml_object $parsed)
        let toml_content = ($toml_lines | str join "\n")

        # Step 4: Write output
        $toml_content | save --force $output_path

        print "✅ TOML rendered successfully"

        let file_size = (ls $output_path | get size.0)
        print $"   File: ($output_path) (($file_size | into string)) bytes"
    } catch {|err|
        print -e $"❌ Rendering failed: ($err)"
        exit 1
    }
}

# Direct invocation:
# nu ./provisioning/scripts/nickel-render-toml.nu provisioning/templates/tracker/config.ncl build/tracker.toml
