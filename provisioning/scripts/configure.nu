#!/usr/bin/env nu
# Torrust Tracker Environment Configuration Script (Nushell variant)
# Interactive configuration for tracker environments using TypeDialog + Nickel
#
# Usage:
#   nu ./provisioning/scripts/configure.nu [backend]
#
# Backend options:
#   cli - Command-line interface (simple prompts)
#   tui - Terminal UI (interactive panels)
#   web - Web server (browser-based) [default]

def main [
    backend: string = "web"  # Backend to use: cli, tui, or web (default: web)
] {
    # =========================================================================
    # DIRECTORY STRUCTURE
    # =========================================================================

    let script_dir = ($env.FILE_PWD)
    let provisioning_dir = ($script_dir | path dirname)
    let project_root = ($provisioning_dir | path dirname)

    # Configuration files
    let form_file = ($provisioning_dir | path join "config-form.toml")
    let values_dir = ($provisioning_dir | path join "values")
    let template_file = ($provisioning_dir | path join "templates" "config-template.ncl.j2")
    let config_file = ($values_dir | path join "config.ncl")

    # =========================================================================
    # ENVIRONMENT VARIABLES
    # =========================================================================

    $env.TYPEDIALOG_PORT = ($env.TYPEDIALOG_PORT? | default "9000")
    $env.TYPEDIALOG_HOST = ($env.TYPEDIALOG_HOST? | default "localhost")
    $env.TYPEDIALOG_LANG = ($env.TYPEDIALOG_LANG? | default ($env.LANG? | default "en_US.UTF-8"))

    # =========================================================================
    # VALIDATE BACKEND
    # =========================================================================

    if $backend not-in ["cli" "tui" "web"] {
        print "Error: Invalid backend. Must be one of: cli, tui, web"
        print ""
        print "Usage: nu configure.nu [cli|tui|web]"
        print ""
        print "Backend options:"
        print "  cli - Command-line interface (simple prompts)"
        print "  tui - Terminal UI (interactive panels)"
        print "  web - Web server (browser-based) [default]"
        return
    }

    # =========================================================================
    # PRE-FLIGHT CHECKS
    # =========================================================================

    print "═══════════════════════════════════════════════════════════"
    print "🎯 Torrust Tracker Configuration"
    print "═══════════════════════════════════════════════════════════"
    print ""

    # Check dependencies
    mut missing_deps = []

    if (which typedialog | is-empty) {
        $missing_deps = ($missing_deps | append "typedialog (install with: cargo install typedialog)")
    }

    if (which nickel | is-empty) {
        $missing_deps = ($missing_deps | append "nickel (install with: cargo install nickel-lang-cli)")
    }

    if not ($missing_deps | is-empty) {
        print -e "❌ Missing dependencies:"
        for dep in $missing_deps {
            print -e $"   - ($dep)"
        }
        return
    }

    print "✅ All dependencies available"
    print ""

    # Check if form exists
    if not ($form_file | path exists) {
        print -e $"❌ Form file not found: ($form_file)"
        return
    }

    # Check if template exists
    if not ($template_file | path exists) {
        print -e $"❌ Template file not found: ($template_file)"
        return
    }

    # Create values directory if it doesn't exist
    mkdir $values_dir

    # Check if config exists, create minimal one if not
    if not ($config_file | path exists) {
        print $"ℹ️  No existing configuration found. Creating minimal config..."

        let minimal_config = "# Torrust Tracker Environment Configuration
# This file will be populated by the configuration wizard

let schemas_env = import \"../schemas/environment.ncl\" in
let schemas_ssh = import \"../schemas/ssh.ncl\" in
let schemas_provider = import \"../schemas/provider.ncl\" in
let schemas_tracker = import \"../schemas/tracker.ncl\" in
let schemas_features = import \"../schemas/features.ncl\" in

let defaults_env = import \"../defaults/environment.ncl\" in
let defaults_ssh = import \"../defaults/ssh.ncl\" in
let defaults_provider = import \"../defaults/provider.ncl\" in
let defaults_tracker = import \"../defaults/tracker.ncl\" in
let defaults_features = import \"../defaults/features.ncl\" in

let constraints = import \"../constraints.toml\" in

let validators = import \"../validators/environment.ncl\" in
let validators_instance = import \"../validators/instance.ncl\" in
let validators_username = import \"../validators/username.ncl\" in
let validators_common = import \"../validators/common.ncl\" in
let validators_network = import \"../validators/network.ncl\" in
let validators_tracker = import \"../validators/tracker.ncl\" in

let user_config = {
} in

defaults_env & defaults_ssh & defaults_provider & defaults_tracker & defaults_features & user_config
"

        $minimal_config | save -f $config_file
        print "✅ Created initial configuration"
        print ""
    }

    # Create backup if config exists and has content
    if ($config_file | path exists) and ((open $config_file | str length) > 0) {
        let timestamp = (date now | format date "%Y%m%d_%H%M%S")
        let backup = $"($config_file).($timestamp).bak"
        cp $config_file $backup
        print $"ℹ️  Backed up existing config to: ($backup | path basename)"
        print ""
    }

    # =========================================================================
    # LAUNCH TYPEDIALOG
    # =========================================================================

    print "═══════════════════════════════════════════════════════════"
    print $"🎯 Launching TypeDialog - ($backend) backend"
    print "═══════════════════════════════════════════════════════════"
    print ""

    # Show web server info if using web backend
    if $backend == "web" {
        print $"🌐 Web server will start on: http://($env.TYPEDIALOG_HOST):($env.TYPEDIALOG_PORT)"
        print "   (Override with: TYPEDIALOG_PORT=8080 TYPEDIALOG_HOST=0.0.0.0 nu configure.nu)"
        print ""
    }

    # Show template info
    print $"ℹ️  Using Nickel template: ($template_file | path basename)"
    print ""

    # Build and execute nickel-roundtrip command based on backend
    let result = try {
        match $backend {
            "cli" => {
                print $"ℹ️  Launching TypeDialog CLI..."
                ^typedialog nickel-roundtrip $config_file $form_file --output $config_file --ncl-template $template_file
            },
            "tui" => {
                if (which typedialog-tui | is-empty) {
                    print -e "❌ typedialog-tui not found. Install with: cargo install typedialog --features tui"
                    return
                }
                print $"ℹ️  Launching TypeDialog TUI..."
                ^typedialog-tui nickel-roundtrip $config_file $form_file --output $config_file --ncl-template $template_file
            },
            "web" => {
                if (which typedialog-web | is-empty) {
                    print -e "❌ typedialog-web not found. Install with: cargo install typedialog --features web"
                    return
                }
                print $"ℹ️  Launching TypeDialog Web..."
                ^typedialog-web nickel-roundtrip $config_file $form_file --output $config_file --ncl-template $template_file
            }
        }
        {success: true}
    } catch {
        |err| {success: false, error: $err}
    }

    if $result.success {
        # Success
        print ""
        print $"✅ Configuration saved to: ($config_file)"
        print ""

        print "Next steps:"
        print "  1. Review the configuration:"
        print $"     cat ($config_file)"
        print ""
        print "  2. Export to JSON for deployment:"
        print $"     nickel export --format json ($config_file) > envs/my-env.json"
        print ""
        print "  3. Create environment:"
        print "     cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json"
        print ""
        print "  4. Re-run this script anytime to update:"
        print $"     nu configure.nu ($backend)"
        print ""
    } else {
        print -e $"❌ Configuration failed"
    }
}
