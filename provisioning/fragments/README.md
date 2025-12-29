# TypeDialog Fragments

Modular, reusable TypeDialog form components for interactive environment configuration.

## 📋 Overview

Each fragment represents a logical section of the configuration form. The main form (`../config-form.toml`) loads these fragments conditionally based on user selections.

**Benefits of fragments**:

- ✅ **Modularity**: Each section is independent
- ✅ **Reusability**: Fragments can be used in multiple forms
- ✅ **Testability**: Each fragment can be validated independently
- ✅ **Maintainability**: Changes isolated to specific sections
- ✅ **Clarity**: Form structure is easy to understand

## 🔄 Fragment Loading Pattern

The main form uses TypeDialog's conditional `groups` + `includes` system:

```toml
[[items]]
name = "lxd_group"
type = "group"
when = "provider == lxd"
includes = ["fragments/provider-lxd-section.toml"]
```

**How it works**:

1. TypeDialog reads main form (`config-form.toml`)
2. When a `group` item has `includes`, the fragment file is loaded
3. The `when` condition determines if the fragment appears
4. All fragment fields are merged into the main form's namespace

## 📁 Fragment Files

### 1. **environment-section.toml**

Collects environment identification information.

**Fields**:

- `environment_name` (required): Lowercase with dashes allowed
- `instance_name` (optional): Auto-generated if omitted

### 2. **provider-lxd-section.toml**

LXD-specific provider configuration (conditional: appears when `provider == lxd`).

**Fields**:

- `lxd_profile_name` (required): Name of LXD profile

### 3. **provider-hetzner-section.toml**

Hetzner Cloud-specific provider configuration (conditional: appears when `provider == hetzner`).

**Fields**:

- `hetzner_api_token` (required): Hetzner API authentication token
- `hetzner_server_type` (required): Server instance type (cx22, cx32, etc)
- `hetzner_location` (required): Datacenter location (fsn1, nbg1, etc)
- `hetzner_image` (required): OS image (ubuntu-24.04, debian-12, etc)

### 4. **ssh-section.toml**

SSH credentials for remote access.

**Fields**:

- `ssh_private_key_path` (required): Path to SSH private key
- `ssh_public_key_path` (required): Path to SSH public key
- `ssh_username` (default: "torrust"): Linux username
- `ssh_port` (default: 22): SSH port

### 5. **database-sqlite-section.toml**

SQLite database configuration (conditional: appears when `database_driver == sqlite3`).

**Fields**:

- `sqlite_database_name` (default: "tracker.db"): Database filename

### 6. **database-mysql-section.toml**

MySQL database configuration (conditional: appears when `database_driver == mysql`).

**Fields**:

- `mysql_host` (default: "localhost"): MySQL server host
- `mysql_port` (default: 3306): MySQL server port
- `mysql_database_name` (default: "torrust_tracker"): Database name
- `mysql_username` (default: "tracker_user"): Database username
- `mysql_password` (required): Database password

### 7. **tracker-section.toml**

Tracker core configuration (always included).

**Fields**:

- `tracker_private_mode` (default: false): Enable private tracker mode
- `udp_trackers` (repeatinggroup): Array of UDP tracker listeners
- `http_trackers` (repeatinggroup): Array of HTTP tracker listeners
- `http_api_bind_address` (default: "0.0.0.0:1212"): HTTP API bind address
- `http_api_admin_token` (required): Admin API token

### 7a. **udp_trackers_item.toml**

Fragment defining structure for each UDP tracker array element.

**Fields**:

- `bind_address` (default: "0.0.0.0:6969"): UDP tracker bind address

### 7b. **http_trackers_item.toml**

Fragment defining structure for each HTTP tracker array element.

**Fields**:

- `bind_address` (default: "0.0.0.0:7070"): HTTP tracker bind address

### 8. **prometheus-section.toml**

Prometheus monitoring configuration (conditional: appears when `enable_prometheus == true`).

**Fields**:

- `prometheus_bind_address` (default: "0.0.0.0:9090"): Prometheus bind address
- `prometheus_scrape_interval` (default: 15): Scrape interval in seconds

### 9. **grafana-section.toml**

Grafana visualization configuration (conditional: appears when `enable_grafana == true`).

**Fields**:

- `grafana_bind_address` (default: "0.0.0.0:3000"): Grafana bind address
- `grafana_admin_password` (required): Grafana admin password

### 10. **confirmation-section.toml**

Review and confirm configuration (always included, shown last).

**Purpose**: Display summary and let user confirm before validation and export.

## 🎨 TypeDialog Fragment Syntax

Each fragment is a TOML file with optional section headers and fields:

```toml
name = "fragment_name"

# Optional: Section header for UI grouping
[[items]]
name = "header"
type = "section_header"
title = "📋 Section Title"
border_top = true
order = 1

# Fields: text input
[[fields]]
name = "field_name"
type = "text"
prompt = "Display prompt"
placeholder = "Example value"
required = true
help = "Help text"
default = "default_value"
order = 2

# Fields: select (dropdown)
[[fields]]
name = "select_field"
type = "select"
prompt = "Choose option"
options = ["option1", "option2", "option3"]
default = "option1"
required = true
order = 3

# Fields: password (hidden input)
[[fields]]
name = "password_field"
type = "password"
prompt = "Enter password"
required = true
help = "Your secret password"
order = 4

# Fields: confirm (boolean)
[[fields]]
name = "boolean_field"
type = "confirm"
prompt = "Enable feature?"
default = false
order = 5

# Fields: repeatinggroup (array of items)
[[fields]]
name = "udp_trackers"
type = "repeatinggroup"
prompt = "UDP Tracker Listeners"
fragment = "fragments/udp_trackers_item.toml"
min_items = 0
max_items = 10
default_items = 1
required = false
help = "Configure UDP tracker bind addresses"
order = 6
```

### RepeatingGroup Field Type

The `repeatinggroup` field type allows users to add/edit/delete multiple items of the same structure (array of records). This is useful for configuration like:

- Multiple listeners (UDP/HTTP trackers)
- Multiple database connections
- Multiple API endpoints
- Lists of servers, users, etc.

**Key attributes**:

- `fragment`: Path to fragment file defining the structure of each array element
- `min_items`: Minimum required items (0 = optional array)
- `max_items`: Maximum allowed items
- `default_items`: Initial number of items to display

**Fragment structure** (`udp_trackers_item.toml`):

```toml
name = "udp_trackers_item"
description = "UDP Tracker configuration"
display_mode = "complete"

[[elements]]
name = "bind_address"
type = "text"
prompt = "UDP Bind Address"
placeholder = "0.0.0.0:6969"
default = "0.0.0.0:6969"
required = true
order = 0
```

**Backend behavior**:

- **CLI**: Interactive menu with Add/Edit/Delete/Continue options
- **TUI**: Split-pane UI with item list and edit form
- **Web**: HTML cards with modal overlay for add/edit operations

**Nickel integration**:
Arrays automatically map to `Array(Record)` types in Nickel schemas:

```nickel
{
  TrackerUdp = { bind_address | String },
  Config = { udp_trackers | Array TrackerUdp | optional },
}
```

## 🔗 Fragment Integration

### Main Form Pattern

The main form (`../config-form.toml`) defines the overall structure and loads fragments:

```toml
# Always included: Environment
[[items]]
name = "environment_group"
type = "group"
includes = ["fragments/environment-section.toml"]

# Conditional: Provider selection then LXD or Hetzner
[[fields]]
name = "provider"
type = "select"
options = ["lxd", "hetzner"]

[[items]]
name = "lxd_group"
type = "group"
when = "provider == lxd"
includes = ["fragments/provider-lxd-section.toml"]

[[items]]
name = "hetzner_group"
type = "group"
when = "provider == hetzner"
includes = ["fragments/provider-hetzner-section.toml"]

# Conditional: Database selection then SQLite or MySQL
[[fields]]
name = "database_driver"
type = "select"
options = ["sqlite3", "mysql"]

[[items]]
name = "sqlite_group"
type = "group"
when = "database_driver == sqlite3"
includes = ["fragments/database-sqlite-section.toml"]

[[items]]
name = "mysql_group"
type = "group"
when = "database_driver == mysql"
includes = ["fragments/database-mysql-section.toml"]

# Conditional: Optional features
[[fields]]
name = "enable_prometheus"
type = "confirm"
default = false

[[items]]
name = "prometheus_group"
type = "group"
when = "enable_prometheus == true"
includes = ["fragments/prometheus-section.toml"]
```

## 📝 Creating New Fragments

To create a new fragment:

1. **Create the file**: `fragments/new-section.toml`
2. **Define name**: `name = "new_section"`
3. **Add section header** (optional): For UI organization
4. **Define fields**: text, select, password, confirm, etc
5. **Add to main form**: As a conditional or unconditional group
6. **Test independently**: `typedialog run fragments/new-section.toml`

## ✅ Validation

Fragments are automatically validated by:

1. **TypeDialog**: Field types, required fields, options
2. **Nickel**: Business logic rules (EnvironmentName, InstanceName, ports, etc)
3. **Rust**: Final validation before creating environment

Fragment validation is purely syntactic (TOML parsing). Business logic validation happens in Nickel validators.

## 🧪 Testing Fragments Independently

Test a single fragment with TypeDialog:

```bash
# Test environment section
typedialog run provisioning/fragments/environment-section.toml

# Test provider LXD section
typedialog run provisioning/fragments/provider-lxd-section.toml

# Test with preset values (advanced)
typedialog run provisioning/fragments/environment-section.toml --preset '{"environment_name":"dev"}'
```

## 🔄 Workflow: Fragment → Form Integration

```text
Fragment (.toml)
    ↓
TypeDialog reads & validates TOML syntax
    ↓
Main form loads fragment conditionally
    ↓
User fills fields in fragment
    ↓
TypeDialog outputs JSON with all fields
    ↓
json-to-nickel.sh converts JSON → Nickel
    ↓
Nickel validators apply business logic
    ↓
nickel-to-json.sh exports Nickel → final JSON
    ↓
Rust receives final JSON with all validations passed
```

## 📚 Related Documentation

### Examples

- **Fragments with Arrays**: [`examples/05-fragments/array-trackers.toml`](../../examples/05-fragments/array-trackers.toml)
- **Nickel Arrays**: [`examples/07-nickel-generation/arrays-schema.ncl`](../../examples/07-nickel-generation/arrays-schema.ncl)

### Provisioning

- Main form design: `../config-form.toml`
- Validation rules: `../validators/README.md`
- Form orchestration: `../scripts/config.sh`
- TypeDialog documentation: `../../README.md`
