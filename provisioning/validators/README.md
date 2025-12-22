# Nickel Validators

Business logic validation rules that mirror Rust domain types exactly.

## 📋 Overview

Validators enforce **content rules** for configuration values. Each validator replicates Rust domain validation rules to ensure consistency across the system.

**Key principle**: Nickel validators must match Rust validators **exactly**.

**Validation hierarchy**:
1. **TypeDialog**: Syntax validation (field types, required fields)
2. **Nickel validators**: Business logic rules (format, ranges, constraints)
3. **Rust**: Final validation before environment creation

## 📁 Validator Files

### common.ncl
Common utilities and helpers used by all validators.

**Validators**:
- `ValidPort(port)`: Port range validation (1-65535, rejects 0)
- `NonEmptyString(s)`: Non-empty string check
- `ValidBindAddress(addr)`: Network address format validation (IP:PORT)

### environment.ncl
EnvironmentName validation (mirrors `src/domain/environment/name.rs`).

**Validation rules**:
- ✅ Non-empty
- ✅ Lowercase only (a-z, 0-9, -)
- ✅ Cannot start with digit
- ✅ Cannot start with dash
- ✅ Cannot end with dash
- ✅ No consecutive dashes
- ✅ No uppercase letters
- ✅ No underscores or special characters

**Rust reference**: `src/domain/environment/name.rs`

### instance.ncl
InstanceName validation (mirrors `src/domain/instance_name.rs`).

**Validation rules**:
- ✅ Non-empty
- ✅ 1-63 characters maximum
- ✅ ASCII letters, numbers, dashes only
- ✅ Cannot start with digit
- ✅ Cannot start with dash
- ✅ Cannot end with dash
- ✅ No uppercase required (allows mixed case)

**Rust reference**: `src/domain/instance_name.rs`

### username.ncl
Username validation (mirrors `src/shared/username.rs`).

**Validation rules**:
- ✅ Non-empty
- ✅ 1-32 characters maximum
- ✅ Must start with letter (a-z, A-Z) or underscore (_)
- ✅ Subsequent chars: letters, digits, underscores, hyphens
- ✅ Allows uppercase (case-sensitive)

**Rust reference**: `src/shared/username.rs`

### network.ncl
Network address validators.

**Validators**:
- `ValidBindAddress(addr)`: IP:PORT format
  - Pattern: `\d+\.\d+\.\d+\.\d+:\d+`
  - Validates port range via `ValidPort`
- `ValidPort(port)`: Port range 1-65535 (rejects 0)

### paths.ncl
Path validators for SSH keys and configuration files.

**Validators**:
- `ValidPath(p)`: Non-empty path string
- `ValidSshKeyPath(p)`: SSH key path format (no validation of file existence at validation time)

### tracker.ncl
Tracker array validators for UDP and HTTP listener configurations.

**Validators**:
- `ValidUniqueBindAddresses(listeners)`: Ensures all bind_address values in array are unique
  - Rejects arrays with duplicate addresses
  - Used for both UDP and HTTP tracker arrays
- `ValidTrackerArrayLength(listeners, min_count, max_count)`: Validates array size bounds
  - `min_count`: Minimum required items (typically 1)
  - `max_count`: Maximum allowed items (typically 4)
  - Rejects arrays with fewer or more items than allowed
- `ValidTrackerArrayFull(listeners, min_count, max_count)`: Combined validator
  - Validates both uniqueness AND length in one call
  - Recommended for production use

**Usage in config**:
```nickel
udp_trackers = validators_tracker.ValidTrackerArrayFull
  [
    { bind_address = "0.0.0.0:6969" },
    { bind_address = "0.0.0.0:6970" },
  ]
  1   # min_items
  4,  # max_items
```

**Validation rules**:
- ✅ All `bind_address` values must be unique (no duplicates)
- ✅ Array length between 1 and 4 (configurable)
- ✅ Each address validated with `ValidBindAddress` before validator
- ✅ Works with both inherited defaults and custom values

## 🎯 Validator Implementation Pattern

Each validator follows a consistent pattern:

```nickel
let ValidatorName = fun param =>
  if condition then param
  else std.record.fail "Error message"
in
ValidatorName
```

### Example: Port Validation

```nickel
let ValidPort = fun port =>
  if port >= 1 && port <= 65535 then
    port
  else
    std.record.fail "Port must be 1-65535, got %{std.string.from_number port}"
in
ValidPort
```

### Example: EnvironmentName Validation

```nickel
let ValidEnvironmentName = fun name =>
  let _ = if name == "" then
    std.record.fail "Environment name cannot be empty"
  else name in

  let _ = if std.string.is_match "^[0-9]" name then
    std.record.fail "Environment name cannot start with number"
  else name in

  let _ = if std.string.is_match "[A-Z]" name then
    std.record.fail "Environment name must be lowercase"
  else name in

  # ... more checks ...

  name
in
ValidEnvironmentName
```

## 🔄 Validation Workflow

```
User Input (JSON from TypeDialog)
    ↓
json-to-nickel.sh reads JSON
    ↓
Nickel values file created with validators applied
    ↓
Validators run during Nickel evaluation
    ↓
If validation fails → Error message + exit
If validation passes → Nickel record created
    ↓
nickel-to-json.sh exports validated Nickel
    ↓
Final JSON (all values validated)
    ↓
Rust EnvironmentCreationConfig (final validation)
```

## ✅ Cross-Validation: Rust ↔ Nickel

### Critical Alignment Points

**EnvironmentName** (must match exactly):
- Rust: `src/domain/environment/name.rs`
- Nickel: `provisioning/validators/environment.ncl`
- Validation: Lowercase, no numbers at start, no leading/trailing/consecutive dashes

**InstanceName** (must match exactly):
- Rust: `src/domain/instance_name.rs`
- Nickel: `provisioning/validators/instance.ncl`
- Validation: 1-63 chars, no leading digit/dash, no trailing dash

**Username** (must match exactly):
- Rust: `src/shared/username.rs`
- Nickel: `provisioning/validators/username.ncl`
- Validation: 1-32 chars, starts with letter or underscore

### Validation Test Matrix

| Type | Rust Validation | Nickel Validation | Alignment |
|------|-----------------|-------------------|-----------|
| EnvironmentName | `domain/environment/name.rs` | `environment.ncl` | Must match |
| InstanceName | `domain/instance_name.rs` | `instance.ncl` | Must match |
| Username | `shared/username.rs` | `username.ncl` | Must match |
| Port | Implicit in ranges | `common.ncl` | Must match |
| BindAddress | Implicit in network code | `network.ncl` | Must match |

## 🧪 Testing Validators

Test validators standalone with Nickel:

```bash
# Test environment name validator
nickel eval <<'EOF'
let validator = import "provisioning/validators/environment.ncl" in
validator.ValidEnvironmentName "dev"
EOF

# Test with invalid input (should error)
nickel eval <<'EOF'
let validator = import "provisioning/validators/environment.ncl" in
validator.ValidEnvironmentName "Dev"  # uppercase - should fail
EOF

# Test port validator
nickel eval <<'EOF'
let validator = import "provisioning/validators/common.ncl" in
validator.ValidPort 22
EOF

# Test invalid port (should error)
nickel eval <<'EOF'
let validator = import "provisioning/validators/common.ncl" in
validator.ValidPort 0  # port 0 not allowed - should fail
EOF

# Test tracker array validator (valid)
nickel eval <<'EOF'
let validator = import "provisioning/validators/tracker.ncl" in
let listeners = [
  { bind_address = "0.0.0.0:7070" },
  { bind_address = "0.0.0.0:7071" },
] in
validator.ValidTrackerArrayFull listeners 1 4
EOF

# Test tracker array validator with duplicates (should error)
nickel eval <<'EOF'
let validator = import "provisioning/validators/tracker.ncl" in
let listeners = [
  { bind_address = "0.0.0.0:7070" },
  { bind_address = "0.0.0.0:7070" },  # duplicate!
] in
validator.ValidTrackerArrayFull listeners 1 4
EOF

# Test tracker array validator with too many items (should error)
nickel eval <<'EOF'
let validator = import "provisioning/validators/tracker.ncl" in
let listeners = [
  { bind_address = "0.0.0.0:7070" },
  { bind_address = "0.0.0.0:7071" },
  { bind_address = "0.0.0.0:7072" },
  { bind_address = "0.0.0.0:7073" },
  { bind_address = "0.0.0.0:7074" },  # 5 items, max is 4!
] in
validator.ValidTrackerArrayFull listeners 1 4
EOF
```

## 🔧 Creating New Validators

To add a new validator:

1. **Identify the rule**: What constraint needs to be enforced?
2. **Find Rust reference**: Locate equivalent validation in Rust code
3. **Create validator function**: In appropriate `.ncl` file
4. **Test with both valid and invalid inputs**
5. **Document the rule**: Add comments explaining the constraint
6. **Update main config**: Reference validator in appropriate schema

## 📝 Validator Documentation Template

```nickel
# Validator: ValidName
# Purpose: Validate X according to Y rule
#
# Rule: Z
# Example valid: "example"
# Example invalid: "bad_example"
#
# Reference: src/path/to/rust/validation.rs
let ValidName = fun param =>
  if condition then param
  else std.record.fail "Error message"
in
ValidName
```

## 🎯 Error Message Standards

Error messages should:
- ✅ Be clear about what failed
- ✅ State the constraint that was violated
- ✅ Show what was attempted (for debugging)
- ✅ Suggest how to fix (when applicable)

**Good error message**:
```
"Environment name 'Dev-Prod' is invalid: contains uppercase letters.
Must be lowercase only. Examples: dev, staging, production, e2e-config"
```

**Bad error message**:
```
"Invalid input"
```

## 🔗 Related Documentation

- Schemas: `../schemas/README.md`
- Defaults: `../defaults/README.md`
- Values: `../values/README.md`
- Rust validation: `src/domain/`, `src/shared/`
- Nickel language: https://nickel-lang.org/
