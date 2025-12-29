# Nickel Schemas

Type contracts and data structures for environment configuration.

## 📋 Overview

Schemas define the **shape** and **structure** of configuration data. Each schema mirrors a section from the JSON configuration format.

**What schemas do**:

- ✅ Define expected types for all fields
- ✅ Enforce structure contracts
- ✅ Enable gradual typing in Nickel
- ✅ Document expected format

**Note**: Schemas define structure. Validators (in `../validators/`) define content rules.

## 📁 Schema Files

### environment.ncl

Environment identification schema.

**Type**: Record with fields

- `name`: String (EnvironmentName - validated by validators)
- `instance_name`: Optional String (InstanceName - validated by validators)

### provider.ncl

Provider configuration schema (discriminated union).

**Types**:

- `LxdConfig`: LXD provider
  - `provider`: Literal string "lxd"
  - `profile_name`: String (validated as InstanceName)

- `HetznerConfig`: Hetzner Cloud provider
  - `provider`: Literal string "hetzner"
  - `api_token`: String
  - `server_type`: String (e.g., "cx22", "cx32")
  - `location`: String (e.g., "fsn1", "nbg1")
  - `image`: String (e.g., "ubuntu-24.04")

### ssh.ncl

SSH credentials schema.

**Type**: Record with fields

- `private_key_path`: String (path to private key)
- `public_key_path`: String (path to public key)
- `username`: String (validated as Username)
- `port`: Number (validated as ValidPort: 1-65535)

### database.ncl

Database configuration schema (discriminated union).

**Types**:

- `SqliteConfig`: SQLite file-based
  - `driver`: Literal string "sqlite3"
  - `database_name`: String

- `MysqlConfig`: MySQL server-based
  - `driver`: Literal string "mysql"
  - `host`: String
  - `port`: Number (validated as ValidPort)
  - `database_name`: String
  - `username`: String
  - `password`: String

### tracker.ncl

Tracker configuration schema.

**Type**: Record with fields

- `core`: Record
  - `private`: Boolean
  - `database`: DatabaseConfig (SQLite | MySQL)
- `udp_trackers`: Array of Records
  - `bind_address`: String (validated as ValidBindAddress)
- `http_trackers`: Array of Records
  - `bind_address`: String (validated as ValidBindAddress)
- `http_api`: Record
  - `bind_address`: String (validated as ValidBindAddress)
  - `admin_token`: String

### features.ncl

Optional features schema.

**Type**: Record with fields

- `prometheus`: Record
  - `enabled`: Boolean
  - `bind_address`: Optional String (validated as ValidBindAddress)
  - `scrape_interval`: Optional Number (seconds)
- `grafana`: Record
  - `enabled`: Boolean
  - `bind_address`: Optional String (validated as ValidBindAddress)
  - `admin_password`: Optional String

## 🎯 Nickel Schema Patterns

### Simple Field

```nickel
{
  name = "field_name" : String,
  value = "default_value"
}
```

### Optional Field

```nickel
{
  optional_field | default = "default_value" : String,
}
```

### Discriminated Union (OneOf)

```nickel
let Provider = [
  | {
      provider = "lxd" : String,
      profile_name = "" : String
    }
  | {
      provider = "hetzner" : String,
      api_token = "" : String,
      server_type = "" : String,
      location = "" : String,
      image = "" : String
    }
] in

Provider
```

### Record with Metadata

```nickel
{
  | doc "User SSH credentials"
  private_key_path = "" : String,
  | doc "SSH port for remote connections"
  port = 22 : Number,
}
```

## 🔄 Schema Composition

The main configuration schema composes all sub-schemas:

```nickel
let Environment = import "environment.ncl" in
let Provider = import "provider.ncl" in
let SshCredentials = import "ssh.ncl" in
let Database = import "database.ncl" in
let Tracker = import "tracker.ncl" in
let Features = import "features.ncl" in

{
  environment : Environment,
  provider : Provider,
  ssh_credentials : SshCredentials,
  database : Database,
  tracker : Tracker,
  features : Features,
}
```

## ✅ Schema Validation Rules

Schemas define structure, validators define rules:

**Schema responsibility**: "Is this the right type?"

- Is `port` a Number?
- Is `provider` one of: "lxd" | "hetzner"?
- Are all required fields present?

**Validator responsibility**: "Is this value acceptable?"

- Is port between 1-65535?
- Is EnvironmentName lowercase with no leading numbers?
- Does SSH key file exist?

## 🧪 Testing Schemas

Test schema with Nickel CLI:

```bash
# Evaluate schema (checks syntax)
nickel eval provisioning/schemas/environment.ncl

# Check with sample data
nickel eval <<'EOF'
let env_schema = import "provisioning/schemas/environment.ncl" in
let env_data = {
  name = "dev",
  instance_name = "dev-vm",
} in
env_schema & env_data
EOF
```

## 📝 Creating New Schemas

To add a new schema:

1. **Create file**: `schemas/new-schema.ncl`
2. **Define top-level type**: Record or discriminated union
3. **Import in main schema**: In a composition file
4. **Document fields**: Use `| doc "description"` annotations
5. **Add validators**: Reference validators in `../validators/`
6. **Test**: `nickel eval provisioning/schemas/new-schema.ncl`

## 🔗 Related Documentation

- Validators: `../validators/README.md`
- Defaults: `../defaults/README.md`
- Values: `../values/README.md`
- Nickel language: https://nickel-lang.org/
