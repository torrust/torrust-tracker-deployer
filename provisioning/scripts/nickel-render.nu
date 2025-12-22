#!/usr/bin/env nu
# Generic Nickel template renderer - evaluate and convert to target format
#
# Evaluates a Nickel (.ncl) template file and converts the JSON output to a target format.
#
# Compliance: .claude/guidelines/nushell/NUSHELL_COMPLIANCE_CHECKLIST.md
# - Function signatures with explicit types
# - External commands prefixed with `^`
# - String interpolation: [$var] for variables
#
# Usage:
#   ./nickel-render.nu <template.ncl> <output_format> <output_path>
#   ./nickel-render.nu provisioning/templates/tracker/config.ncl toml build/tracker.toml
#   ./nickel-render.nu provisioning/templates/prometheus/config.ncl yaml build/prometheus/prometheus.yml
#
# Supported formats: json, yaml, toml, hcl, env

def render_nickel_to_json [template_path: path]: nothing -> string {
    # Evaluate Nickel template and export as JSON
    if not ($template_path | path exists) {
        error make {msg: $"Template file not found: ($template_path)"}
    }

    let json_output = ^nickel export --format json $template_path

    $json_output
}

def json_to_yaml [json: string]: string -> string {
    # Convert JSON to YAML using yq
    # Requires: yq (https://github.com/mikefarah/yq)

    $json | ^yq -P .
}

def json_to_toml [json: string]: string -> string {
    # Convert JSON to TOML using custom Nushell logic
    # Builds TOML key=value format from JSON object at root level

    let parsed = $json | from json

    if ($parsed | type) != "record" {
        error make {msg: "TOML format requires a root-level object"}
    }

    # Build TOML lines
    let lines = ($parsed | items {|key, value|
        let toml_value = (value_to_toml_literal $value)
        $"[$key] = ($toml_value)"
    })

    $lines | str join "\n"
}

def value_to_toml_literal [value]: any -> string {
    # Convert a JSON value to TOML literal representation
    match ($value | type) {
        "null" => "null",
        "bool" => ($value | into string),
        "int" => ($value | into string),
        "float" => ($value | into string),
        "string" => $"\"($value)\"",
        "list" => {
            let items = ($value | each {|item| value_to_toml_literal $item} | str join ", ")
            $"[($items)]"
        },
        "record" => {
            # Nested objects - format as inline table or multi-line table
            let pairs = ($value | items {|k, v|
                $"($k) = (value_to_toml_literal $v)"
            } | str join ", ")
            $"{ ($pairs) }"
        },
        _ => $"\"($value)\""
    }
}

def json_to_hcl [json: string]: string -> string {
    # Convert JSON to HCL/Terraform format
    # Supports variable definitions (tfvars files)

    let parsed = $json | from json

    if ($parsed | type) != "record" {
        error make {msg: "HCL format requires a root-level object"}
    }

    let lines = ($parsed | items {|key, value|
        let hcl_value = (value_to_hcl_literal $value)
        $"($key) = ($hcl_value)"
    })

    $lines | str join "\n"
}

def value_to_hcl_literal [value]: any -> string {
    # Convert a JSON value to HCL literal representation
    match ($value | type) {
        "null" => "null",
        "bool" => ($value | into string),
        "int" => ($value | into string),
        "float" => ($value | into string),
        "string" => {
            # Escape special characters in HCL strings
            let escaped = ($value
                | str replace --all '\\' '\\\\'
                | str replace --all '"' '\"'
                | str replace --all '\n' '\\n'
                | str replace --all '\r' '\\r'
                | str replace --all '\t' '\\t'
            )
            $"\"($escaped)\""
        },
        "list" => {
            let items = ($value | each {|item| value_to_hcl_literal $item} | str join ", ")
            $"[($items)]"
        },
        "record" => {
            # Nested objects - format as HCL object
            let pairs = ($value | items {|k, v|
                $"  ($k) = (value_to_hcl_literal $v)"
            } | str join "\n")
            $"{\n($pairs)\n}"
        },
        _ => $"\"($value)\""
    }
}

def json_to_env [json: string]: string -> string {
    # Convert JSON to ENV format (KEY=VALUE, one per line, sorted alphabetically)
    # Only supports flat objects at root level

    let parsed = $json | from json

    if ($parsed | type) != "record" {
        error make {msg: "ENV format requires a flat root-level object"}
    }

    let lines = ($parsed | items {|key, value|
        let env_value = (value_to_env_literal $value)
        $"($key)=($env_value)"
    } | sort)

    $lines | str join "\n"
}

def value_to_env_literal [value]: any -> string {
    # Convert a JSON value to ENV literal representation
    match ($value | type) {
        "null" => "",
        "bool" => ($value | into string),
        "int" => ($value | into string),
        "float" => ($value | into string),
        "string" => {
            # Quote if contains spaces or special characters
            if ($value | str contains ' ') or ($value | str contains '=') or ($value | str contains '"') {
                $"\"($value | str replace --all '"' '\"')\""
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
    template_path: path,      # Path to Nickel template file (.ncl)
    output_format: string,    # Output format: json, yaml, toml, hcl, env
    output_path: path,        # Where to write the output file
]: nothing -> nothing {

    print $"📋 Rendering Nickel template: ($template_path)"
    print $"📊 Output format: ($output_format)"
    print $"💾 Output path: ($output_path)"
    print ""

    # Step 1: Evaluate Nickel to JSON
    let json = try {
        print "⏳ Evaluating Nickel template..."
        let result = (render_nickel_to_json $template_path)
        print "✅ Nickel template evaluated successfully"
        $result
    } catch {|err|
        print -e $"❌ Failed to evaluate Nickel template: ($err)"
        exit 1
    }

    # Step 2: Convert to target format
    let formatted = try {
        match $output_format {
            "json" => {
                print "⏳ Converting to JSON..."
                $json
            },
            "yaml" => {
                print "⏳ Converting to YAML..."
                json_to_yaml $json
            },
            "toml" => {
                print "⏳ Converting to TOML..."
                json_to_toml $json
            },
            "hcl" => {
                print "⏳ Converting to HCL..."
                json_to_hcl $json
            },
            "env" => {
                print "⏳ Converting to ENV..."
                json_to_env $json
            },
            _ => {
                error make {msg: $"Unknown output format: ($output_format). Supported: json, yaml, toml, hcl, env"}
            }
        }
    } catch {|err|
        print -e $"❌ Failed to convert format: ($err)"
        exit 1
    }

    # Step 3: Create output directory if needed
    let output_dir = ($output_path | path dirname)
    if not ($output_dir | path exists) {
        try {
            mkdir $output_dir
        } catch {|err|
            print -e $"❌ Failed to create output directory: ($err)"
            exit 1
        }
    }

    # Step 4: Write output file
    try {
        print "⏳ Writing output file..."
        $formatted | save --force $output_path
        print "✅ Output file written successfully"
    } catch {|err|
        print -e $"❌ Failed to write output file: ($err)"
        exit 1
    }

    # Step 5: Verify output
    if not ($output_path | path exists) {
        print -e "❌ Output file was not created"
        exit 1
    }

    let file_size = (ls $output_path | get size.0)
    if ($file_size == 0) {
        print -e "❌ Output file is empty"
        exit 1
    }

    print ""
    print $"✅ Success! Rendered template: ($output_path) (($file_size | into string)) bytes"
}

# Script can be called directly:
# nu ./provisioning/scripts/nickel-render.nu templates/tracker/config.ncl toml build/tracker.toml
