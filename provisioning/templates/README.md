# Nickel Template System

This directory contains Nickel configuration templates for generating deployment configuration files.

## Overview

The Nickel template system replaces the previous Tera-based templating approach with a **CLI-driven architecture**:

```text
Nickel Template (.ncl)
    ↓ (nickel export --format json)
    ↓
JSON Output
    ↓ (Nushell/Bash scripts)
    ↓
Target Format (YAML, TOML, HCL, ENV)
```

## Architecture

### Templates Directory Structure

```text
provisioning/templates/
├── README.md                    # This file
├── prometheus/
│   └── config.ncl              # Prometheus configuration template
├── tracker/
│   └── config.ncl              # Tracker configuration template
├── docker-compose/
│   ├── compose.ncl             # docker-compose.yml template
│   └── env.ncl                 # .env file template
├── ansible/
│   ├── inventory.ncl           # Ansible inventory template
│   └── variables.ncl           # Ansible variables template
└── tofu/
    ├── lxd/
    │   └── variables.ncl       # LXD tfvars template
    ├── hetzner/
    │   └── variables.ncl       # Hetzner tfvars template
    └── common/
        └── cloud-init.ncl      # cloud-init configuration template
```

### Rendering Scripts

Located in `provisioning/scripts/`, these scripts handle template evaluation and format conversion:

#### Nushell Scripts (.nu)

- **`nickel-render.nu`** - Generic Nickel renderer supporting all formats
- **`nickel-render-yaml.nu`** - Specialized YAML renderer (uses yq)
- **`nickel-render-toml.nu`** - Specialized TOML renderer
- **`nickel-render-hcl.nu`** - Specialized HCL renderer (for Terraform/OpenTofu)
- **`nickel-render-env.nu`** - Specialized ENV renderer (KEY=VALUE format)

#### Bash Scripts (.sh) - Alternative

- **`nickel-render.sh`** - Generic Nickel renderer (Bash version)
- **`nickel-render-yaml.sh`** - Specialized YAML renderer
- **`nickel-render-toml.sh`** - Specialized TOML renderer
- **`nickel-render-hcl.sh`** - Specialized HCL renderer
- **`nickel-render-env.sh`** - Specialized ENV renderer

Use Nushell scripts for consistency with the project. Use Bash scripts as alternative if Nushell is not available.

## Template Development

### Basic Nickel Template Structure

```nickel
# provisioning/templates/example/config.ncl

# Import reusable modules
let schemas = import "../schemas/tracker.ncl" in
let defaults = import "../defaults/tracker.ncl" in
let validators = import "../validators/tracker.ncl" in
let values = import "../values/config.ncl" in

# Define the configuration with imports
{
  metadata = {
    app = "example-app",
    schema_version = "1.0.0",
  },

  # Use validators for runtime checks
  setting1 = validators.ValidSetting values.setting1,

  # Support conditionals
  database = if values.provider == "mysql" then {
    driver = "mysql",
    host = values.mysql_host,
  } else {
    driver = "sqlite3",
    path = "/var/lib/example.db",
  },

  # Support arrays
  ports = std.array.map (fun port => {
    bind_address = port,
  }) values.ports,
}
```

## Using the Rendering Scripts

### Nushell - Generic Renderer

```bash
# Render to any format
nu ./provisioning/scripts/nickel-render.nu \
  provisioning/templates/tracker/config.ncl \
  toml \
  build/tracker/tracker.toml

# Supported formats: json, yaml, toml, hcl, env
```

### Bash - Alternative

```bash
# Generic renderer
bash ./provisioning/scripts/nickel-render.sh \
  provisioning/templates/tracker/config.ncl \
  toml \
  build/tracker/tracker.toml

# Format-specific
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/prometheus/config.ncl \
  build/prometheus/prometheus.yml
```

## Dependencies

### Required

- **nickel** - Nickel language CLI
- **yq** - YAML processor
- **jq** - JSON processor

### Installation

```bash
# macOS
brew install nickel yq jq

# Linux
apt-get install yq jq
# nickel from https://github.com/tweag/nickel/releases
```

## Template Status

| Template | Format | Status | Notes |
|----------|--------|--------|-------|
| **prometheus/config.ncl** | YAML | ✅ Working | Generates Prometheus scrape configuration |
| **tracker/config.ncl** | TOML | ✅ Evaluates | JSON export works; TOML conversion needs refinement |
| **docker-compose/compose.ncl** | YAML | ✅ Working | Generates docker-compose.yml |
| **docker-compose/env.ncl** | ENV | ✅ Working | Generates .env file (KEY=VALUE format) |
| **ansible/inventory.ncl** | YAML | ✅ Working | Generates Ansible inventory.yml |
| **ansible/variables.ncl** | YAML | ✅ Working | Generates Ansible variables.yml |
| **tofu/lxd/variables.ncl** | HCL | ✅ Working | Generates terraform.tfvars for LXD |
| **tofu/hetzner/variables.ncl** | HCL | ✅ Working | Generates terraform.tfvars for Hetzner |
| **tofu/common/cloud-init.ncl** | YAML | ✅ Working | Generates cloud-init user data for instance bootstrap |

## Example Usage

### Prometheus Configuration

```bash
# Render Prometheus config to YAML
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/prometheus/config.ncl \
  build/prometheus/prometheus.yml
```

### Docker Compose

```bash
# Render environment file
bash ./provisioning/scripts/nickel-render-env.sh \
  provisioning/templates/docker-compose/env.ncl \
  build/docker-compose/.env

# Render compose file
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/docker-compose/compose.ncl \
  build/docker-compose/docker-compose.yml
```

### Ansible

```bash
# Render inventory
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/ansible/inventory.ncl \
  build/ansible/inventory.yml

# Render variables
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/ansible/variables.ncl \
  build/ansible/variables.yml
```

### OpenTofu/Terraform

```bash
# LXD variables
bash ./provisioning/scripts/nickel-render-hcl.sh \
  provisioning/templates/tofu/lxd/variables.ncl \
  build/tofu/lxd/terraform.tfvars

# Hetzner variables
bash ./provisioning/scripts/nickel-render-hcl.sh \
  provisioning/templates/tofu/hetzner/variables.ncl \
  build/tofu/hetzner/terraform.tfvars

# Cloud-init bootstrap script
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/tofu/common/cloud-init.ncl \
  build/tofu/common/cloud-init.yml
```

## References

- [Nickel Language Documentation](https://nickel-lang.org/)
- Nickel guidelines: `.claude/guidelines/nickel/NICKEL_GUIDELINES.md`
- Nushell guidelines: `.claude/guidelines/nushell/`
