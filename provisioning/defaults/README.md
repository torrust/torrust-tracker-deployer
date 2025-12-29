# Nickel Defaults

Default configuration values for Torrust Tracker environments.

## 📋 Overview

Default values provide sensible out-of-the-box configuration for deployment environments. Users can override any default by providing explicit values.

**How defaults work in Nickel**:

```nickel
# Defaults first, then user config
defaults & user_config
```

This record merge pattern:

1. Starts with defaults
2. User values override defaults where provided
3. Unspecified fields get defaults

## 📁 Default Files

### environment.ncl

Environment defaults (minimal).

**Defaults**:

- `instance_name`: Optional, auto-generated if omitted

### ssh.ncl

SSH connection defaults (standard conventions).

**Defaults**:

- `port`: 22 (standard SSH port)
- `username`: "torrust" (project convention)

### tracker.ncl

Tracker service defaults (standard ports and settings).

**Defaults**:

- UDP tracker: `0.0.0.0:6969`
- HTTP tracker: `0.0.0.0:7070`
- Private mode: false (public tracker)
- Database driver: sqlite3 (local)
- SQLite filename: "tracker.db"

**Required (No Default)**:

- HTTP API: `bind_address` and `admin_token` must be provided by user

**Merge Strategy for Tracker**:

The tracker configuration uses **selective referencing** rather than full merge:

| Field | Behavior | Reason |
|-------|----------|--------|
| `core` | Merges with defaults | Can inherit fields like `private`, `database` |
| `udp_trackers` | Reference or override | Array has default (0.0.0.0:6969); can use `defaults_tracker.tracker.udp_trackers` |
| `http_trackers` | Reference or override | Array has default (0.0.0.0:7070); must specify to change ports |
| `http_api` | **Required** | No default exists; user must always provide `bind_address` and `admin_token` |

**Why `http_api` has no default**:

- API admin token is security-sensitive (should never have a default)
- Each environment needs explicit configuration
- Schema enforces this by marking `http_api` as required (not optional)

**Example merge in user config**:

```nickel
tracker = {
  # Merge: inherit defaults for private mode and database
  core = defaults_tracker.tracker.core & {
    private = false,
    database = { driver = "sqlite3", database_name = "tracker.db" },
  },

  # Arrays: can reference defaults or completely replace
  udp_trackers = defaults_tracker.tracker.udp_trackers,
  http_trackers = [ { bind_address = "0.0.0.0:7077" } ],  # Override

  # Required: user must always provide
  http_api = {
    bind_address = "0.0.0.0:1212",
    admin_token = "MyAccessToken",
  },
}
```

### features.ncl

Optional features defaults (disabled by default).

**Defaults**:

- Prometheus: disabled
- Grafana: disabled

## 🎯 Default Configuration Strategy

### Principle: Secure Defaults

- Ports: Non-privileged (> 1024)
- Private mode: false (can be enabled)
- Features: Disabled (opt-in)
- Database: SQLite (simplest, file-based)
- Provider: LXD (local control)

### Principle: Convention Over Configuration

- SSH port: 22 (standard)
- SSH user: "torrust" (project standard)
- UDP port: 6969 (common bittorrent tracker convention)
- HTTP port: 7070 (arbitrary but consistent)
- API port: 1212 (arbitrary but consistent)

### Principle: Simplicity

- SQLite by default (no MySQL setup needed)
- No monitoring by default (Prometheus/Grafana optional)
- Local LXD by default (no cloud setup needed)

## 📝 Default Value Examples

### Minimal Configuration (All Defaults)

User provides only:

```nickel
{
  environment = {
    name = "dev",
  },
  ssh_credentials = {
    private_key_path = "~/.ssh/id_rsa",
    public_key_path = "~/.ssh/id_rsa.pub",
  },
  provider = {
    provider = "lxd",
    profile_name = "torrust-profile-dev",
  },
  tracker = {
    core = {
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
}

# After merge with defaults, results in:
# - SSH port: 22 (default)
# - SSH username: "torrust" (default)
# - Tracker private mode: false (default)
# - Prometheus: disabled (default)
# - Grafana: disabled (default)
```

### Custom Configuration (Overriding Defaults)

User provides overrides:

```nickel
{
  environment = {
    name = "staging",
  },
  ssh_credentials = {
    private_key_path = "~/.ssh/id_rsa",
    public_key_path = "~/.ssh/id_rsa.pub",
    username = "ubuntu",           # Override: custom username
    port = 2222,                   # Override: custom port
  },
  provider = {
    provider = "hetzner",          # Override: cloud provider
    api_token = "hetzner_token",
    server_type = "cx32",
    location = "nbg1",
    image = "ubuntu-24.04",
  },
  tracker = {
    core = {
      private = true,              # Override: private tracker
      database = {
        driver = "mysql",          # Override: use MySQL
        host = "db.example.com",
        port = 3306,
        database_name = "torrust_tracker",
        username = "tracker_user",
        password = "secure_password",
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
  features = {                     # Override: enable monitoring
    prometheus = {
      enabled = true,
      bind_address = "0.0.0.0:9090",
      scrape_interval = 15,
    },
    grafana = {
      enabled = true,
      bind_address = "0.0.0.0:3000",
      admin_password = "GrafanaPassword",
    },
  },
}
```

## 🔄 Defaults Workflow

```text
1. TypeDialog Form
   ↓
2. JSON Output (only user-provided fields)
   ↓
3. json-to-nickel.sh Converter
   ↓
4. Nickel Config (user values only)
   ↓
5. Merge with defaults: defaults & user_config
   ↓
6. Validators applied (all fields have values)
   ↓
7. nickel-to-json.sh Export
   ↓
8. Final JSON (with defaults filled in)
   ↓
9. Rust EnvironmentCreationConfig
```

## ✅ Defaults Philosophy

### What Should Have Defaults

- ✅ SSH port (standard is 22)
- ✅ SSH username (project convention)
- ✅ Tracker bind addresses (standard ports)
- ✅ Optional features (should be opt-in)
- ✅ Database choice (SQLite is simplest)

### What Should NOT Have Defaults

- ❌ Environment name (required user choice)
- ❌ SSH key paths (environment-specific)
- ❌ Provider configuration (must be explicit)
- ❌ Tracker HTTP API: `bind_address` and `admin_token` (security-sensitive, required)
- ❌ Database credentials (if using MySQL)

## 🧪 Testing Defaults

Test defaults merging with Nickel:

```bash
# Load defaults
nickel eval provisioning/defaults/ssh.ncl

# Merge with user values
nickel eval <<'EOF'
let defaults = import "provisioning/defaults/ssh.ncl" in
let user_config = {
  private_key_path = "~/.ssh/id_rsa",
  public_key_path = "~/.ssh/id_rsa.pub",
} in
defaults & user_config
EOF

# Result: port 22 and username "torrust" added automatically
```

## 🔗 Related Documentation

- Schemas: `../schemas/README.md`
- Validators: `../validators/README.md`
- Values: `../values/README.md`
- Nickel patterns: https://nickel-lang.org/user-manual/advanced/records
