# User Configuration Values

User-editable Nickel configuration files for environment setup.

## 📋 Overview

This directory contains user-provided environment configurations in **Nickel format**. Files here are:

- **User-created or wizard-generated** (not committed to version control)
- **Configuration values** combined with schemas and defaults
- **Validated** with Nickel validators before export
- **Exported to JSON** for use with the Torrust Tracker Deployer

**Directory purpose**: Store individual environment configurations before they're exported to JSON.

## 📝 File Organization

### `.gitignore`

Ignores all `.ncl` files except `config.ncl` and `README.md`.

This prevents committing sensitive configurations like:

- SSH key paths
- Database credentials
- API tokens
- Provider tokens

### `config.ncl`

Fully documented example configuration showing all possible fields and options.

This file is:

- ✅ Committed to version control (for reference)
- ✅ Heavily commented with explanations
- ✅ Shows both required and optional fields
- ✅ Demonstrates conditional sections (LXD vs Hetzner, SQLite vs MySQL)
- ✅ Template for users to copy and customize

## 🔄 Configuration Lifecycle

### Step 1: Generate Configuration

Choose one of these methods:

**Method A: Interactive Wizard** (Recommended)

```bash
./provisioning/scripts/config.sh
# or
./provisioning/scripts/config.nu

# Generated: provisioning/values/{env-name}.ncl
```

#### Method B: Copy Example

```bash
cp provisioning/values/config.ncl provisioning/values/my-env.ncl
vim provisioning/values/my-env.ncl
```

#### Method C: Manual Creation

```bash
cat > provisioning/values/my-env.ncl <<'EOF'
let schemas = import "../schemas/environment.ncl" in
let defaults = import "../defaults/environment.ncl" in
let validators = import "../validators/environment.ncl" in

let user_config = {
  environment = {
    name = validators.ValidEnvironmentName "my-env",
    # ... rest of configuration
  },
} in

defaults & user_config
EOF
```

### Step 2: Validate Configuration

```bash
./provisioning/scripts/validate-nickel.sh provisioning/values/my-env.ncl

# Or manually with Nickel CLI:
nickel eval provisioning/values/my-env.ncl
```

If validation fails, error messages will indicate what needs to be fixed.

### Step 3: Export to JSON

```bash
./provisioning/scripts/nickel-to-json.sh \
  provisioning/values/my-env.ncl \
  envs/my-env.json

# Generates: envs/my-env.json (ready for deployment)
```

### Step 4: Create Environment

```bash
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json
```

## 📄 Nickel Value File Format

A valid Nickel configuration file has this structure:

```nickel
# Imports schemas, defaults, and validators
let schemas = import "../schemas/environment.ncl" in
let defaults = import "../defaults/environment.ncl" in
let validators = import "../validators/environment.ncl" in
# ... more imports ...

# Define user configuration
let user_config = {
  environment = {
    name = validators.ValidEnvironmentName "dev",
    # instance_name omitted - will be auto-generated
  },

  provider = {
    provider = "lxd",
    profile_name = "torrust-profile-dev",
  },

  ssh_credentials = {
    private_key_path = "~/.ssh/id_rsa",
    public_key_path = "~/.ssh/id_rsa.pub",
    # username and port will use defaults
  },

  tracker = {
    core = {
      private = false,
      database = {
        driver = "sqlite3",
        database_name = "tracker.db",
      },
    },
    udp_trackers = [
      { bind_address = "0.0.0.0:6969" },
    ],
    http_trackers = [
      { bind_address = "0.0.0.0:7070" },
    ],
    http_api = {
      bind_address = "0.0.0.0:1212",
      admin_token = "MySecretToken",
    },
  },
} in

# Merge defaults with user config (defaults first)
defaults & user_config
```

## 🔄 Understanding Tracker Merge Strategy

### Why Selective Referencing?

Nickel cannot merge arrays with different lengths or values. The tracker configuration uses **selective referencing** to work around this:

1. **`core` - Records CAN merge**: Use `defaults_tracker.tracker.core & {...}` to inherit fields
2. **`udp_trackers` / `http_trackers` - Arrays CANNOT merge**: Either:
   - Reference entire default: `defaults_tracker.tracker.udp_trackers`
   - Replace completely with new values: `[{ bind_address = "0.0.0.0:7077" }]`
3. **`http_api` - NO DEFAULT exists**: Must always be provided by user

### Example: Gradual Customization

#### Scenario 1: Use all defaults except override HTTP port

```nickel
tracker = {
  core = defaults_tracker.tracker.core & { /* any overrides */ },
  udp_trackers = defaults_tracker.tracker.udp_trackers,        # Use default
  http_trackers = [{ bind_address = "0.0.0.0:8080" }],       # Override
  http_api = {
    bind_address = "0.0.0.0:1212",
    admin_token = "MyToken",
  },
}
```

#### Scenario 2: Use defaults for most, override UDP port too

```nickel
tracker = {
  core = defaults_tracker.tracker.core & { private = true },  # Merge to override private
  udp_trackers = [{ bind_address = "0.0.0.0:5555" }],        # Override UDP port
  http_trackers = defaults_tracker.tracker.http_trackers,     # Use default
  http_api = {
    bind_address = "0.0.0.0:1212",
    admin_token = "MyToken",
  },
}
```

#### Scenario 3: Explicit everything (no defaults reference)

```nickel
tracker = {
  core = {
    private = false,
    database = { driver = "sqlite3", database_name = "tracker.db" },
  },
  udp_trackers = [{ bind_address = "0.0.0.0:6969" }],
  http_trackers = [{ bind_address = "0.0.0.0:7070" }],
  http_api = {
    bind_address = "0.0.0.0:1212",
    admin_token = "MyToken",
  },
}
```

All three are valid. Choose based on your needs.

## 🎯 Configuration Sections

### Environment

```nickel
environment = {
  name = validators.ValidEnvironmentName "dev",
  # instance_name is optional - defaults to "torrust-tracker-vm-{env-name}"
},
```

### Provider (LXD)

```nickel
provider = {
  provider = "lxd",
  profile_name = "torrust-profile-dev",
},
```

### Provider (Hetzner)

```nickel
provider = {
  provider = "hetzner",
  api_token = "hetzner_token_here",
  server_type = "cx22",
  location = "nbg1",
  image = "ubuntu-24.04",
},
```

### SSH Credentials

```nickel
ssh_credentials = {
  private_key_path = "~/.ssh/id_rsa",
  public_key_path = "~/.ssh/id_rsa.pub",
  username = "torrust",        # optional, defaults to "torrust"
  port = 22,                   # optional, defaults to 22
},
```

### Database (SQLite)

```nickel
tracker = {
  core = {
    private = false,
    database = {
      driver = "sqlite3",
      database_name = "tracker.db",
    },
  },
  # ...
},
```

### Database (MySQL)

```nickel
tracker = {
  core = {
    private = false,
    database = {
      driver = "mysql",
      host = "db.example.com",
      port = 3306,
      database_name = "torrust_tracker",
      username = "tracker_user",
      password = "secure_password",
    },
  },
  # ...
},
```

### Tracker Services

**Merge Strategy**: Tracker configuration uses **selective referencing** with defaults:

| Field | Merge Strategy | Required? |
|-------|----------------|-----------|
| `core` | Merge with defaults | Required |
| `udp_trackers` | Reference defaults or override | Optional (has default) |
| `http_trackers` | Reference defaults or override | Optional (has default) |
| `http_api` | Must provide explicitly | **Required** (no default) |

**Key Points**:

- `core` can omit fields to inherit defaults (e.g., `private`, `database`)
- Arrays (`udp_trackers`, `http_trackers`) must be completely replaced if overriding (Nickel doesn't merge arrays)
- `http_api` has **no default** and must always be provided (security: admin token is sensitive)

**Example with defaults inheritance**:

```nickel
tracker = {
  # Merge with defaults - inherit private=false, database defaults
  core = defaults_tracker.tracker.core & {
    private = false,
    database = {
      driver = "sqlite3",
      database_name = "tracker.db",
    },
  },

  # Reference default UDP tracker (0.0.0.0:6969)
  udp_trackers = defaults_tracker.tracker.udp_trackers,

  # Override HTTP tracker with different port
  http_trackers = [
    { bind_address = "0.0.0.0:7077" },
    { bind_address = "0.0.0.0:7078" },
  ],

  # Required: must always provide
  http_api = {
    bind_address = "0.0.0.0:1212",
    admin_token = "SecretAdminToken",
  },
},
```

**Example without defaults (all fields explicit)**:

```nickel
tracker = {
  core = {
    private = false,
    database = {
      driver = "sqlite3",
      database_name = "tracker.db",
    },
  },
  udp_trackers = [
    { bind_address = "0.0.0.0:6969" },
  ],
  http_trackers = [
    { bind_address = "0.0.0.0:7070" },
  ],
  http_api = {
    bind_address = "0.0.0.0:1212",
    admin_token = "SecretAdminToken",
  },
},
```

### Features (Optional)

```nickel
features = {
  prometheus = {
    enabled = false,
    # bind_address and scrape_interval only if enabled = true
  },
  grafana = {
    enabled = false,
    # bind_address and admin_password only if enabled = true
  },
},
```

## ✅ Validation Checklist

Before exporting to JSON, verify:

### Environment & Infrastructure

- ✅ Environment name is lowercase, no spaces, no special chars
- ✅ Instance name (if provided) follows LXD naming rules
- ✅ Provider is either "lxd" or "hetzner"
- ✅ SSH key paths exist and are readable
- ✅ SSH port is between 1-65535

### Tracker Configuration

- ✅ **`tracker` section is always present** (required)
- ✅ **`http_api` is always provided** (required, no default):
  - `bind_address` - valid IP:PORT format
  - `admin_token` - non-empty string (security-sensitive)
- ✅ **All ports are between 1-65535** (not 0)
- ✅ Bind addresses are valid IP:PORT format
- ✅ Database driver is either "sqlite3" or "mysql"
- ✅ If MySQL: database credentials are correct

### Features & Integrations

- ✅ If Hetzner: API token is provided
- ✅ If Prometheus/Grafana enabled: configuration is complete
- ✅ All required fields are present (no missing imports)

### Common Issues

- ❌ **Missing `http_api`** - Schema requires this field; always provide `bind_address` and `admin_token`
- ❌ **Port 0** - Invalid; use port > 1024 (non-privileged)
- ❌ **Forgetting tracker merge** - If using defaults, ensure final merge includes: `defaults_env & defaults_ssh & defaults_provider & defaults_features & user_config`

## 🧪 Testing Your Configuration

Test before exporting:

```bash
# 1. Validate Nickel syntax
nickel eval provisioning/values/my-env.ncl

# 2. Check specific fields
nickel eval <<'EOF'
let config = import "provisioning/values/my-env.ncl" in
config.environment.name
EOF

# 3. Export to JSON and inspect
./provisioning/scripts/nickel-to-json.sh provisioning/values/my-env.ncl /tmp/test.json
cat /tmp/test.json | jq .

# 4. Validate JSON against schema
# (This would be done by Rust when creating environment)
```

## 🔒 Security Considerations

**NEVER commit** to version control:

- SSH private key paths that are actual paths
- Database passwords
- API tokens (Hetzner, etc)
- Tracker admin tokens
- Any sensitive credentials

**Protection**:

- `.gitignore` prevents accidental commits of `*.ncl`
- Only `config.ncl` and `README.md` are safe to commit
- Use environment variables or `.env` files for secrets in development
- In production, use proper secret management (Vault, etc)

## 📝 Creating a New Configuration

### Quick Start

```bash
# 1. Run wizard
./provisioning/scripts/config.sh

# 2. Answer questions interactively
# 3. Review generated configuration (optional)
cat provisioning/values/{env-name}.ncl

# 4. Export to JSON
./provisioning/scripts/nickel-to-json.sh \
  provisioning/values/{env-name}.ncl \
  envs/{env-name}.json

# 5. Create environment
cargo run --bin torrust-tracker-deployer -- \
  create environment --env-file envs/{env-name}.json
```

### Manual Edit

```bash
# 1. Copy example
cp provisioning/values/config.ncl provisioning/values/dev.ncl

# 2. Edit
vim provisioning/values/dev.ncl

# 3. Validate
./provisioning/scripts/validate-nickel.sh provisioning/values/dev.ncl

# 4. Export
./provisioning/scripts/nickel-to-json.sh \
  provisioning/values/dev.ncl envs/dev.json

# 5. Create
cargo run --bin torrust-tracker-deployer -- \
  create environment --env-file envs/dev.json
```

## 🔗 Related Documentation

- Schemas: `../schemas/README.md`
- Defaults: `../defaults/README.md`
- Validators: `../validators/README.md`
- Scripts: `../scripts/` (configuration generation and export)
- Example: `config.ncl` (fully documented example)
