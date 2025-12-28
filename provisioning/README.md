# Torrust Tracker Environment Configuration Wizard

Interactive configuration system for deploying Torrust Tracker environments using **TypeDialog** (UI) + **Nickel** (validation) + **JSON** (execution).

## 🚀 Quick Start

### Option A: Interactive Configuration Wizard (Recommended)

**NEW: Nickel-based Configuration** (preserves validators, recommended):

**Bash variant**:

```bash
./provisioning/scripts/configure.sh [cli|tui|web]
```

**Nushell variant**:

```bash
nu ./provisioning/scripts/configure.nu [cli|tui|web]
```

Backend options:

- `cli` - Command-line interface (simple prompts)
- `tui` - Terminal UI (interactive panels, requires `cargo install typedialog --features tui`)
- `web` - Web server (browser-based, default, requires `cargo install typedialog --features web`)

These scripts will:

1. Launch interactive TypeDialog form
2. Render Nickel configuration with validators preserved
3. Validate with Nickel typecheck
4. Save to `provisioning/values/config.ncl`

**Legacy: JSON-based Workflow** (deprecated):

**Bash variant**:

```bash
./provisioning/scripts/config.sh
```

**Nushell variant**:

```bash
./provisioning/scripts/config.nu
```

Both scripts will:

1. Launch interactive TypeDialog form
2. Validate configuration with Nickel
3. Generate JSON in `envs/{env-name}.json`
4. Print next steps

### Option B: Manual Nickel Editing (Advanced)

```bash
# 1. Create/edit Nickel configuration
vim provisioning/values/my-env.ncl

# 2. Validate
./provisioning/scripts/validate-nickel.sh provisioning/values/my-env.ncl

# 3. Export to JSON
./provisioning/scripts/nickel-to-json.sh provisioning/values/my-env.ncl envs/my-env.json

# 4. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json
```

### Option C: Manual JSON (Traditional)

```bash
# Edit JSON directly
vim envs/my-env.json

# Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json
```

---

## 📁 Directory Organization

```text
provisioning/
├── README.md                          # This file
├── config-form.toml                   # Main TypeDialog form (modular)
│
├── fragments/                         # TypeDialog UI fragments (modular, reusable)
│   ├── environment-section.toml       # Environment identification
│   ├── provider-lxd-section.toml      # LXD provider config (conditional)
│   ├── provider-hetzner-section.toml  # Hetzner provider config (conditional)
│   ├── ssh-section.toml               # SSH credentials
│   ├── database-sqlite-section.toml   # SQLite database config (conditional)
│   ├── database-mysql-section.toml    # MySQL database config (conditional)
│   ├── tracker-section.toml           # Tracker core configuration
│   ├── prometheus-section.toml        # Prometheus monitoring (optional)
│   ├── grafana-section.toml           # Grafana visualization (optional)
│   └── confirmation-section.toml      # Review & confirm
│
├── constraints/                       # 🔧 VALIDATION LIMITS (Centralized)
│   ├── README.md                      # How to change validation limits
│   └── constraints.toml               # Array size + uniqueness constraints (single source of truth)
│
├── schemas/                           # 📋 TYPE CONTRACTS (Nickel)
│   ├── README.md                      # Schema documentation
│   ├── environment.ncl                # Environment type schema
│   ├── provider.ncl                   # Provider type schema
│   ├── ssh.ncl                        # SSH credentials schema
│   ├── tracker.ncl                    # Tracker config schema + http_api required
│   ├── database.ncl                   # Database schema (SQLite | MySQL)
│   └── features.ncl                   # Optional features schema
│
├── defaults/                          # 💾 DEFAULT VALUES (Nickel)
│   ├── README.md                      # Default strategy + merge patterns
│   ├── environment.ncl                # Default environment settings
│   ├── ssh.ncl                        # SSH defaults (port 22, user "torrust")
│   ├── provider.ncl                   # Provider defaults (LXD)
│   ├── tracker.ncl                    # Tracker defaults (ports, settings, NO http_api)
│   └── features.ncl                   # Features defaults (disabled)
│
├── validators/                        # ✅ VALIDATION LOGIC (Nickel)
│   ├── README.md                      # Validator patterns + testing
│   ├── tracker.ncl                    # Array uniqueness + length validation
│   ├── common.ncl                     # Port, string, address validators
│   ├── environment.ncl                # EnvironmentName validation
│   ├── instance.ncl                   # InstanceName validation (LXD rules)
│   ├── username.ncl                   # Username validation (Linux rules)
│   ├── network.ncl                    # Network address validators
│   └── paths.ncl                      # SSH key path validators
│
├── values/                            # User configurations (gitignored)
│   ├── .gitignore                     # Ignore *.ncl
│   ├── README.md                      # Values documentation
│   └── config.ncl                     # Documented example
│
├── templates/                         # Nickel-based configuration templates (CLI-driven)
│   ├── README.md                      # Template documentation
│   ├── prometheus/
│   │   └── config.ncl                 # Prometheus YAML configuration
│   ├── tracker/
│   │   └── config.ncl                 # Tracker TOML configuration
│   ├── docker-compose/
│   │   ├── compose.ncl                # docker-compose.yml template
│   │   └── env.ncl                    # .env file template
│   ├── ansible/
│   │   ├── inventory.ncl              # Ansible inventory.yml template
│   │   └── variables.ncl              # Ansible variables.yml template
│   └── tofu/
│       ├── lxd/
│       │   └── variables.ncl          # LXD terraform.tfvars template
│       ├── hetzner/
│       │   └── variables.ncl          # Hetzner terraform.tfvars template
│       └── common/
│           └── cloud-init.ncl         # cloud-init bootstrap template
│
└── scripts/                           # Orchestration & rendering scripts (bash + nushell)
    # Configuration Wizard Scripts
    ├── configure.sh                   # Bash: nickel-roundtrip wizard (recommended)
    ├── configure.nu                   # Nushell: nickel-roundtrip wizard (recommended)
    ├── config.sh                      # Bash: legacy JSON workflow wizard
    ├── config.nu                      # Nushell: legacy JSON workflow wizard
    ├── json-to-nickel.sh              # Bash: TypeDialog JSON → Nickel
    ├── json-to-nickel.nu              # Nushell: TypeDialog JSON → Nickel
    ├── nickel-to-json.sh              # Bash: Nickel → JSON export
    ├── nickel-to-json.nu              # Nushell: Nickel → JSON export
    ├── validate-nickel.sh             # Bash: Nickel validation
    ├── validate-nickel.nu             # Nushell: Nickel validation

    # Template Rendering Scripts (NEW)
    ├── nickel-render.sh               # Bash: Generic Nickel renderer (any format)
    ├── nickel-render.nu               # Nushell: Generic Nickel renderer
    ├── nickel-render-yaml.sh          # Bash: Nickel → YAML (via yq)
    ├── nickel-render-yaml.nu          # Nushell: Nickel → YAML
    ├── nickel-render-toml.sh          # Bash: Nickel → TOML
    ├── nickel-render-toml.nu          # Nushell: Nickel → TOML
    ├── nickel-render-hcl.sh           # Bash: Nickel → HCL (Terraform/OpenTofu)
    ├── nickel-render-hcl.nu           # Nushell: Nickel → HCL
    ├── nickel-render-env.sh           # Bash: Nickel → ENV (KEY=VALUE)
    └── nickel-render-env.nu           # Nushell: Nickel → ENV
```

---

## 🔧 Dependencies

### Required (Configuration Wizard)

- **TypeDialog**: Interactive form system
  - Install: `cargo install typedialog`
  - Or: Clone from `/Users/Akasha/Development/typedialog`

- **Nickel**: Configuration language with type safety
  - Install: `cargo install nickel-lang-cli`

### Required (Template Rendering)

- **yq**: YAML processor (for YAML conversion)
  - macOS: `brew install yq`
  - Linux: `apt-get install yq` or from source

- **jq**: JSON processor (for all rendering)
  - Usually pre-installed, or: `brew install jq` / `apt-get install jq`

### Optional

- **Nushell 0.109+**: For Nushell script variants (better JSON handling)
  - Install: `cargo install nu`
  - NOT required if using Bash variants only

**Verification**:

```bash
# Automated check via dependency installer
cargo run --bin dependency-installer -- check

# Or manually check required tools
which typedialog   # Required
which nickel       # Required
which yq           # Required (for YAML rendering)
which jq           # Required (for all rendering)
which nu           # Optional (for Nushell scripts)
```

---

## 📖 Usage Workflows

### Workflow 1: First-Time Setup (Interactive Wizard)

```bash
# 1. Run the wizard
./provisioning/scripts/config.sh

# 2. Answer questions interactively:
#    - Environment name (e.g., "dev", "staging", "production")
#    - Provider type (LXD or Hetzner)
#    - SSH credentials and port
#    - Database type (SQLite or MySQL)
#    - Tracker configuration (ports, privacy mode)
#    - Optional features (Prometheus, Grafana)

# 3. Wizard generates: envs/{env-name}.json

# 4. Review generated JSON (optional)
cat envs/my-env.json | jq .

# 5. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json

# 6. Provision environment
cargo run --bin torrust-tracker-deployer -- provision my-env
```

### Workflow 2: Advanced Configuration (Manual Nickel)

```bash
# 1. Create Nickel config from example
cp provisioning/values/config.ncl provisioning/values/my-env.ncl

# 2. Edit with your settings
vim provisioning/values/my-env.ncl

# 3. Validate configuration
./provisioning/scripts/validate-nickel.sh provisioning/values/my-env.ncl

# 4. Export to JSON
./provisioning/scripts/nickel-to-json.sh provisioning/values/my-env.ncl envs/my-env.json

# 5. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json
```

### Workflow 3: Reuse with Changes

```bash
# 1. Export existing JSON as Nickel for editing
# (Nushell script will support this in future versions)

# 2. Or manually copy and edit JSON
cp envs/prod.json envs/staging.json
vim envs/staging.json

# 3. Create environment with edited JSON
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/staging.json
```

### Workflow 4: Generate Deployment Configuration Files (NEW - Nickel Templates)

Use Nickel templates to generate deployment files for **Prometheus, Tracker, Docker Compose, Ansible, OpenTofu**:

```bash
# Prerequisite: Have a valid provisioning/values/config.ncl

# 1. Render Prometheus configuration (YAML)
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/prometheus/config.ncl \
  build/prometheus/prometheus.yml

# 2. Render Tracker configuration (TOML)
bash ./provisioning/scripts/nickel-render-toml.sh \
  provisioning/templates/tracker/config.ncl \
  build/tracker/tracker.toml

# 3. Render Docker Compose files
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/docker-compose/compose.ncl \
  build/docker-compose/docker-compose.yml

bash ./provisioning/scripts/nickel-render-env.sh \
  provisioning/templates/docker-compose/env.ncl \
  build/docker-compose/.env

# 4. Render Ansible inventory and variables (YAML)
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/ansible/inventory.ncl \
  build/ansible/inventory.yml

bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/ansible/variables.ncl \
  build/ansible/variables.yml

# 5. Render OpenTofu/Terraform variables (HCL)
bash ./provisioning/scripts/nickel-render-hcl.sh \
  provisioning/templates/tofu/lxd/variables.ncl \
  build/tofu/lxd/terraform.tfvars

bash ./provisioning/scripts/nickel-render-hcl.sh \
  provisioning/templates/tofu/hetzner/variables.ncl \
  build/tofu/hetzner/terraform.tfvars

# 6. Render cloud-init bootstrap script (YAML)
bash ./provisioning/scripts/nickel-render-yaml.sh \
  provisioning/templates/tofu/common/cloud-init.ncl \
  build/tofu/common/cloud-init.yml
```

**Alternative: Use Nushell scripts** (if available):

```bash
nu ./provisioning/scripts/nickel-render-yaml.nu provisioning/templates/prometheus/config.ncl build/prometheus/prometheus.yml
```

All templates use the same configuration from `provisioning/values/config.ncl`.

---

## 🎯 Configuration Sections

### Environment Identification

- **Name** (required): lowercase, no leading numbers, dashes allowed
- **Instance Name** (optional): auto-generated as `torrust-tracker-vm-{env-name}` if omitted

### Infrastructure Provider

- **LXD** (local/cloud): Profile name
- **Hetzner** (cloud): API token, server type, location, image

### SSH Credentials

- **Private Key Path**: Path to SSH private key
- **Public Key Path**: Path to SSH public key
- **Username** (default: "torrust"): Linux username
- **Port** (default: 22): SSH port

### Database Configuration

- **SQLite**: Database filename
- **MySQL**: Host, port, database name, username, password

### Tracker Configuration

- **Privacy Mode**: true (private tracker) | false (public)
- **UDP Tracker**: Bind address (e.g., "0.0.0.0:6969")
- **HTTP Tracker**: Bind address (e.g., "0.0.0.0:7070")
- **HTTP API**: Bind address (e.g., "0.0.0.0:1212"), admin token

### Optional Features

- **Prometheus**: Enable/disable, bind address, scrape interval
- **Grafana**: Enable/disable, bind address, admin password

---

## 🎨 Nickel Templates (NEW - Configuration Generation)

The new Nickel template system generates deployment configuration files from a single configuration source.

### Available Templates

| Template | Format | Purpose | Output |
|----------|--------|---------|--------|
| **prometheus/config.ncl** | YAML | Prometheus scrape configuration | `build/prometheus/prometheus.yml` |
| **tracker/config.ncl** | TOML | Torrust Tracker server config | `build/tracker/tracker.toml` |
| **docker-compose/compose.ncl** | YAML | Docker Compose orchestration | `build/docker-compose/docker-compose.yml` |
| **docker-compose/env.ncl** | ENV | Environment variables | `build/docker-compose/.env` |
| **ansible/inventory.ncl** | YAML | Ansible inventory | `build/ansible/inventory.yml` |
| **ansible/variables.ncl** | YAML | Ansible playbook variables | `build/ansible/variables.yml` |
| **tofu/lxd/variables.ncl** | HCL | LXD Terraform variables | `build/tofu/lxd/terraform.tfvars` |
| **tofu/hetzner/variables.ncl** | HCL | Hetzner Terraform variables | `build/tofu/hetzner/terraform.tfvars` |
| **tofu/common/cloud-init.ncl** | YAML | Cloud-init bootstrap script | `build/tofu/common/cloud-init.yml` |

### How Templates Work

```text
┌──────────────────────────────────────────────┐
│  provisioning/values/config.ncl              │
│  (Your configuration + imports)              │
└───────────────┬────────────────────────────┘
                │
    ┌───────────┴──────────┬────────────┐
    │                      │            │
    ▼                      ▼            ▼
┌──────────┐         ┌─────────┐  ┌──────────┐
│Prometheus│         │ Tracker │  │ Docker   │
│ config   │         │ config  │  │ Compose  │
└──────────┘         └─────────┘  └──────────┘
    │                      │            │
    ├──nickel export       ├─nickel     ├─nickel
    │  --format json       │ export     │ export
    └──▶ JSON              └▶ JSON      └▶ JSON
        │                     │           │
        ├──yq -P             ├─custom    ├─custom
        │  (convert YAML)    │ (TOML)    │ (ENV)
        │                     │           │
        ▼                      ▼           ▼
    prometheus.yml        tracker.toml  .env
```

### Rendering Commands

All rendering scripts are in `provisioning/scripts/`:

```bash
# YAML format (via yq)
bash nickel-render-yaml.sh <template.ncl> <output.yml>

# TOML format (custom converter)
bash nickel-render-toml.sh <template.ncl> <output.toml>

# HCL format (custom converter)
bash nickel-render-hcl.sh <template.ncl> <output.tfvars>

# ENV format (custom converter)
bash nickel-render-env.sh <template.ncl> <output.env>

# Generic (any format)
bash nickel-render.sh <template.ncl> <format> <output>
```

### Configuration Source

All templates import from `provisioning/values/config.ncl`:

- User-specific configuration
- Environment name, provider, SSH credentials
- Tracker ports, database settings
- Feature flags (Prometheus, Grafana)

**Single source of truth**: Change config.ncl once, all templates reflect the change.

See `templates/README.md` for detailed template documentation.

---

## ⚠️ Synchronization: Single Source of Truth

For **tracker arrays** (UDP/HTTP), constraints are defined in ONE place and automatically used everywhere:

### 1. Constraint Definition (Single Source of Truth)

**File**: `constraints/constraints.toml`

```toml
[tracker.udp]
min_items = 1
max_items = 4
unique = true
```

### 2. Form Uses Constraint Interpolation

**File**: `fragments/tracker-section.toml`

```toml
[[elements]]
name = "udp_trackers"
type = "repeatinggroup"
min_items = 0  # ⚠️  Form allows 0 (UI flexibility)
max_items = "${constraint.tracker.udp.max_items}"  # ✅ Dynamically loaded from constraints.toml
unique = true
```

Form parser automatically resolves `${constraint.tracker.udp.max_items}` to the value in constraints.toml.

### 3. Nickel Files Import Constraints Directly

**File**: `values/config.ncl`

```nickel
let constraints = import "../constraints.toml" in

udp_trackers = validators_tracker.ValidTrackerArrayFull
  [...]
  constraints.tracker.udp.min_items      # ✅ Direct import from constraints.toml
  constraints.tracker.udp.max_items,     # ✅ Direct import from constraints.toml
```

### Known Mismatch: min_items

| Place | Value | Note |
|-------|-------|------|
| Form | `min_items = 0` | Allows optional trackers in UI |
| Constraints | `min_items = 1` | Enforces at least 1 in production |
| Nickel | `min_items = 1` | Validates during config evaluation |

**Why the difference?**

- Form: Provides UI flexibility for testing/exploration
- Nickel: Enforces production safety (at least 1 tracker required)

**Resolution**: Document as intentional or align all to `min_items = 1`

## ✅ Validation Rules (All Layers)

All validations are coordinated across:

1. **Form layer** (TypeDialog) - User input restrictions
2. **Constraint layer** (Nickel) - Centralized limits
3. **Validator layer** (Nickel) - Business logic checks
4. **Schema layer** (Nickel) - Type contracts
5. **Rust layer** - Final validation before execution

### Tracker Arrays

- **Min items**: 1 (at least one UDP/HTTP tracker required)
- **Max items**: 4 (maximum 4 listeners of each type)
- **Uniqueness**: All `bind_address` values must be unique
- **Format**: Each address validated as `IP:PORT`

### EnvironmentName (mirrors `src/domain/environment/name.rs`)

- Lowercase only (a-z, 0-9, -)
- Cannot start with number or dash
- Cannot end with dash
- No consecutive dashes

### InstanceName (mirrors `src/domain/instance_name.rs`, LXD naming)

- 1-63 characters
- ASCII letters, numbers, dashes only
- Cannot start with digit or dash
- Cannot end with dash

### Username (mirrors `src/shared/username.rs`, Linux system)

- 1-32 characters
- Must start with letter or underscore
- Can contain letters, digits, underscores, hyphens

### Network Addresses

- Format: `IP:PORT`
- Port range: 1-65535 (port 0 not allowed per project ADR)

---

## 🐛 Troubleshooting

### Error: "TypeDialog not found"

```bash
# Install TypeDialog
cargo install typedialog

# Or use local checkout
export PATH="/Users/Akasha/Development/typedialog/target/release:$PATH"
```

### Error: "Nickel validation failed"

- Review error message for specific rule violation
- Common issues:
  - Environment name has uppercase letters
  - Instance name starts with digit
  - Port number out of range (0 or > 65535)
  - Invalid characters in fields

### Error: "JSON export failed"

- Verify Nickel file syntax with: `nickel eval provisioning/values/{env}.ncl`
- Check that all required fields are present
- Validate against schema

### Scripts not executable

```bash
chmod +x provisioning/scripts/*.sh
chmod +x provisioning/scripts/*.nu
```

### Nushell version mismatch

- Minimum required: Nushell 0.109+
- Check version: `nu --version`
- Update: `cargo install nu --locked`

---

## 🏗️ Configuration System Architecture

The provisioning system validates and merges configuration in **7 layers**:

```text
Form (TypeDialog)
    ↓ Constraint Interpolation: ${constraint.tracker.udp.max_items}
    ↓
Constraints (constraints.toml)
    ↓ Single Source of Truth: min=1, max=4, unique=true
    ↓ Imported by: Forms (interpolation), Nickel files (direct import)
    ↓
Values (values/config.ncl)
    ↓ User config + imports constraints + applies validators
    ↓
Validators (validators/tracker.ncl)
    ↓ Checks: uniqueness, array length, formats
    ↓ Uses constraint values from constraints.toml
    ↓
Schemas (schemas/tracker.ncl)
    ↓ Type contracts: required fields, types
    ↓
Defaults (defaults/tracker.ncl)
    ↓ Merge: inherit values not specified by user
    ↓
JSON Export
    ↓
Rust Validation (Final layer)
```

### Key Layer: Constraints (Single Source of Truth!)

**File**: `constraints/constraints.toml`

Centralizes validation limits used by:

- **Form** (`fragments/tracker-section.toml`) - via constraint interpolation `${constraint.tracker.udp.max_items}`
- **Nickel** (`validators/tracker.ncl`, `values/config.ncl`) - via direct import

#### Usage Example: Changing Max Trackers from 4 to 6

##### Step 1: Edit constraints.toml (ONLY place you need to change)

```toml
[tracker.udp]
min_items = 1
max_items = 6         # ← Changed from 4 to 6
unique = true
```

##### Step 2: Form automatically gets new limit

**File**: `fragments/tracker-section.toml` (NO CHANGES NEEDED!)

```toml
[[elements]]
name = "udp_trackers"
type = "repeatinggroup"
prompt = "UDP Tracker Listeners"
min_items = 0
max_items = "${constraint.tracker.udp.max_items}"  # ← Automatically resolves to 6 now!
unique = true
nickel_path = ["tracker", "udp_trackers"]
```

When form loads, form parser sees `"${constraint.tracker.udp.max_items}"` and replaces it with `6` from constraints.toml

##### Step 3: Nickel validators automatically use new limit

**File**: `values/config.ncl` (NO CHANGES NEEDED!)

```nickel
# Line 31: Import constraints (already there)
let constraints = import "../constraints.toml" in

# Lines 116-126: Use the constraints in validators
udp_trackers = validators_tracker.ValidTrackerArrayFull
  [
    {
      bind_address = validators_network.ValidBindAddress "0.0.0.0:6969",
    },
    {
      bind_address = validators_network.ValidBindAddress "0.0.0.0:6970",
    },
  ]
  constraints.tracker.udp.min_items       # ← Now reads min=1
  constraints.tracker.udp.max_items,      # ← Now reads max=6
```

No changes needed - Nickel automatically imports and uses the new values!

##### Data Flow Diagram

```text
┌─────────────────────────────────────────────┐
│   constraints/constraints.toml              │
│   [tracker.udp] max_items = 6    ← Edit here
└──────────────┬──────────────────────────────┘
               │ Single Source of Truth
    ┌──────────┴──────────┐
    │                     │
    ▼                     ▼
┌──────────────────┐  ┌──────────────────┐
│ Form             │  │ Nickel files     │
│ (Interpolation)  │  │ (Direct import)  │
│                  │  │                  │
│ max_items =      │  │ let constraints  │
│ "${constraint    │  │ = import         │
│ .tracker.udp     │  │ "constraints     │
│ .max_items}"     │  │ .toml" in        │
│                  │  │ ...              │
│ Parser replaces  │  │ constraints      │
│ with value: 6 ✓  │  │ .tracker.udp     │
│                  │  │ .max_items ✓     │
└──────────────────┘  └──────────────────┘
```

##### Result

✅ Form allows up to 6 UDP trackers
✅ Nickel validators enforce max 6
✅ Only ONE file edited: `constraints/constraints.toml`
✅ All three layers auto-sync automatically

**Verify the change works**:

```bash
# 1. Form parser loads and resolves interpolation
just build::default && cargo test -p typedialog-core test_constraint_interpolation

# 2. Nickel evaluates with new constraints
nickel eval provisioning/values/config.ncl

# 3. Full roundtrip with constraint auto-sync
typedialog nickel-roundtrip \
  --form provisioning/config-form.toml \
  --input provisioning/values/config.ncl \
  --output /tmp/test.ncl
```

See `constraints/README.md` for detailed constraint testing and troubleshooting.

### Key Layer: Validators

**File**: `validators/tracker.ncl`

Enforces constraints using functions:

- `ValidTrackerArrayFull(array, min, max)` - Combined check
- `ValidUniqueBindAddresses(array)` - Duplicate detection
- `ValidTrackerArrayLength(array, min, max)` - Size bounds

Used in `values/config.ncl`:

```nickel
http_trackers = validators_tracker.ValidTrackerArrayFull
  [{ bind_address = "0.0.0.0:7070" }]
  constraints.http.min_items
  constraints.http.max_items,
```

See `validators/README.md` for details.

### Key Layer: Defaults

**File**: `defaults/tracker.ncl`

Provides fallback values when user doesn't specify.

**Merge Strategy for Tracker**:

- `core` - Merges with defaults (inherit fields)
- `udp_trackers` - Can reference OR replace
- `http_trackers` - Can reference OR replace
- `http_api` - **NO default** (always user-provided for security)

Example:

```nickel
tracker = {
  core = defaults_tracker.tracker.core & { private = false },
  udp_trackers = defaults_tracker.tracker.udp_trackers,  # Inherit default
  http_trackers = [{ bind_address = "0.0.0.0:8080" }],  # Override
  http_api = { bind_address = "0.0.0.0:1212", admin_token = "..." },
}
```

See `defaults/README.md` for details.

### Key Decision: http_api is Required

In `schemas/tracker.ncl`, `http_api` is **NOT optional**:

```nickel
http_api | TrackerApi,  # Required, not optional
```

**Reason**: Security

- Admin token must never have a default
- Every environment must explicitly set credentials
- No risk of accidental defaults in production

See `schemas/README.md` for details.

## 📚 Documentation Map

See individual subdirectories for detailed documentation:

| Directory | Purpose | What to Read |
|-----------|---------|--------------|
| `constraints/` | **Validation Limits** | `README.md` - How to change min/max items |
| `schemas/` | **Type Contracts** | `README.md` - Type definitions |
| `defaults/` | **Default Values** | `README.md` - Defaults + merge strategy |
| `validators/` | **Validation Logic** | `README.md` - Validator patterns |
| `values/` | **User Configs** | `README.md` - Config patterns + examples |
| `fragments/` | **Form Design** | `README.md` - TypeDialog components |
| `templates/` | **Nickel Templates** | `README.md` - 9 templates for deployment configs |
| `scripts/` | **Automation** | Headers in each script |

### Related Project Documentation

- **`docs/decisions/nickel-cli-driven-template-system.md`** - Architecture decision for Nickel templates (replaces Tera)
- **`docs/technical/nickel-projectgenerator-integration.md`** - How to integrate Nickel templates into ProjectGenerator code
- **`.claude/guidelines/nickel/`** - Nickel coding standards and patterns

---

## 🔄 Workflow Diagram

```text
┌─────────────────────────────────────────┐
│  User Input Decision                    │
└────────────┬──────────────┬─────────────┘
             │              │
    ┌────────▼──────┐   ┌───▼─────────┐
    │   TypeDialog  │   │ Manual JSON │
    │    Wizard     │   │ or Nickel   │
    └────────┬──────┘   └───┬─────────┘
             │              │
    ┌────────▼──────────────▼──────┐
    │  JSON Configuration           │
    │  (envs/{env-name}.json)       │
    └────────┬─────────────────────┘
             │
    ┌────────▼─────────────────────┐
    │  Rust Domain Validation       │
    │  (existing EnvironmentConfig) │
    └────────┬─────────────────────┘
             │
    ┌────────▼─────────────────────┐
    │  Create Environment Command   │
    │  (Application State Machine)  │
    └─────────────────────────────┘
```

---

## 📋 Next Steps

### Quick Start (Configuration & Deployment)

1. **Familiarize yourself** with configuration structure (read sections above)
2. **Run wizard**: `./provisioning/scripts/config.sh`
3. **Review generated JSON**: `cat envs/{env-name}.json | jq .`
4. **Create environment**: Follow printed instructions
5. **Provision**: Run deployment commands

### Advanced (Using Nickel Templates)

1. **Create/edit Nickel config**: `cp provisioning/values/config.ncl provisioning/values/my-env.ncl`
2. **Validate configuration**: `./provisioning/scripts/validate-nickel.sh provisioning/values/my-env.ncl`
3. **Generate deployment files**: Use Workflow 4 (see above) to render all templates
4. **Review generated configs**: Check `build/` directory for generated files
5. **Deploy**: Use generated configs with OpenTofu, Ansible, Docker Compose, etc.

---

## 🤝 Contributing

### Configuration System

When modifying the configuration:

- Update relevant TypeDialog fragments in `fragments/`
- Keep Nickel validators synchronized with Rust domain types
- Update defaults for new fields in `defaults/`
- Add tests for new validators in `validators/`
- Update `constraints.toml` if adding new validation limits
- Update this README if adding new configuration sections

### Nickel Templates

When creating or modifying templates:

- Follow `.claude/guidelines/nickel/NICKEL_GUIDELINES.md`
- Templates must import from `provisioning/values/config.ncl`
- Each template must handle its output format correctly
- Test rendering with appropriate bash script (yaml/toml/hcl/env)
- Document new template in `provisioning/templates/README.md`
- See `docs/technical/nickel-projectgenerator-integration.md` for ProjectGenerator integration

---

## 📞 Support

For issues or questions:

### Configuration Issues

1. Check troubleshooting section above
2. Review generated JSON structure: `cat envs/{env-name}.json | jq .`
3. Validate Nickel manually: `nickel eval provisioning/values/{env}.ncl`
4. Check constraint synchronization: `cat constraints/constraints.toml | grep -A 3 tracker`

### Template Issues

1. Verify Nickel evaluation: `nickel export --format json provisioning/templates/{type}/config.ncl`
2. Check rendering script exists: `ls provisioning/scripts/nickel-render-*.sh`
3. Test rendering manually: `bash provisioning/scripts/nickel-render-yaml.sh <template> <output>`
4. Review output file: `cat <output>`

### Further Help

- Project documentation: `docs/user-guide/`
- ADR on Nickel architecture: `docs/decisions/nickel-cli-driven-template-system.md`
- ProjectGenerator integration: `docs/technical/nickel-projectgenerator-integration.md`
- Nickel language docs: https://nickel-lang.org/
