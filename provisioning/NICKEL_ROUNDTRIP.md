# Nickel Roundtrip Integration

How the provisioning system integrates with TypeDialog's nickel-roundtrip feature for multi-backend form workflows.

## 📋 Overview

The nickel-roundtrip workflow allows:
1. **Form-driven configuration** - TypeDialog form collects user input
2. **Nickel generation** - Converts JSON → Nickel with validators and constraints
3. **Validation** - Nickel validators enforce business rules
4. **Roundtrip editing** - Edit Nickel, then back to form for UI review

## 🔄 Workflow: TypeDialog → Nickel → Roundtrip

```
┌─────────────────────────────────────────┐
│  1. TypeDialog Form (config-form.toml)  │
│     - Collects: env, provider, DB, etc  │
│     - Restricts: min/max, unique, fmt   │
└─────────────────┬───────────────────────┘
                  │ JSON output
                  ↓
┌─────────────────────────────────────────┐
│  2. json-to-nickel Converter            │
│     - Uses: values-template.ncl.j2      │
│     - Injects: user values into template│
│     - Adds: imports, validators, merges │
└─────────────────┬───────────────────────┘
                  │ Generated .ncl file
                  ↓
┌─────────────────────────────────────────┐
│  3. Nickel Validation                   │
│     - Imports: validators/tracker.ncl   │
│     - Imports: constraints.toml         │
│     - Validates: arrays, types, bounds  │
│     - Applies: ValidTrackerArrayFull    │
└─────────────────┬───────────────────────┘
                  │ Validated .ncl
                  ↓
┌─────────────────────────────────────────┐
│  4. Export to JSON                      │
│     - nickel-to-json converter          │
│     - Final validated configuration     │
└─────────────────┬───────────────────────┘
                  │ JSON ready for Rust
                  ↓
┌─────────────────────────────────────────┐
│  5. Create Environment (Rust)           │
│     - Final validation before creation  │
└─────────────────────────────────────────┘
```

## 📄 Template Files

### Main Template: `provisioning/templates/values-template.ncl.j2`

**Used by**: json-to-nickel converter during roundtrip

**What it does**:
1. Imports schemas, defaults, validators, constraints
2. Fills in user-provided values from form
3. Applies validators with constraints
4. Generates complete .ncl file

**Key sections**:
```jinja2
# Imports validators and constraints
let validators_tracker = import "../validators/tracker.ncl" in
let constraints = import "../constraints.toml" in

# Generates tracker arrays with validation
udp_trackers = validators_tracker.ValidTrackerArrayFull
  [
    { bind_address = "0.0.0.0:6969" },
  ]
  constraints.udp.min_items
  constraints.udp.max_items,
```

## 🔧 How Validators & Constraints Are Applied

When the template is rendered with user input:

### 1. Form Validation (TypeDialog)
```toml
# config-form.toml
[[elements]]
name = "udp_trackers"
type = "repeatinggroup"
min_items = 0  # Form allows flexibility
max_items = 4  # Restricts to 4
unique = true  # Enforces uniqueness
```

### 2. Template Rendering
**Input**: JSON from form
```json
{
  "udp_tracker_bind_address": "0.0.0.0:6969",
  "http_tracker_bind_address": "0.0.0.0:7070"
}
```

**Output**: Nickel code with validators
```nickel
let validators_tracker = import "../validators/tracker.ncl" in
let constraints = import "../constraints.toml" in

udp_trackers = validators_tracker.ValidTrackerArrayFull
  [
    {
      bind_address = validators_network.ValidBindAddress "0.0.0.0:6969",
    },
  ]
  constraints.udp.min_items
  constraints.udp.max_items,
```

### 3. Nickel Validation
During `nickel eval`, the validators run:
- `ValidTrackerArrayFull` checks:
  - ✅ Uniqueness: No duplicate `bind_address`
  - ✅ Array length: Between `min_items` (1) and `max_items` (4)
  - ✅ Each address: Validated with `ValidBindAddress`

### 4. Merging with Defaults
```nickel
defaults_env & defaults_ssh & defaults_provider & defaults_features & user_config
```

## 💾 Template Variables

The template expects these form field outputs:

| Variable | Source | Example |
|----------|--------|---------|
| `environment_name` | Form | `"dev"` |
| `provider` | Form | `"lxd"` or `"hetzner"` |
| `ssh_username` | Form | `"torrust"` |
| `database_driver` | Form | `"sqlite3"` or `"mysql"` |
| `udp_tracker_bind_address` | Form | `"0.0.0.0:6969"` |
| `http_tracker_bind_address` | Form | `"0.0.0.0:7070"` |
| `http_api_bind_address` | Form | `"0.0.0.0:1212"` |
| `http_api_admin_token` | Form | `"MyToken"` |
| `enable_prometheus` | Form | `true` or `false` |

## 🎯 Key Features in Templates

### Feature 1: Conditional Fields
```jinja2
{%- if udp_tracker_bind_address %}
# Only rendered if user provided value
udp_trackers = validators_tracker.ValidTrackerArrayFull [...]
{%- endif %}
```

### Feature 2: Validator Application
```jinja2
bind_address = validators_network.ValidBindAddress "{{ udp_tracker_bind_address }}",
```

### Feature 3: Constraint References
```jinja2
validators_tracker.ValidTrackerArrayFull
  [...]
  constraints.udp.min_items      {# Uses centralized constraints #}
  constraints.udp.max_items,
```

## 🚀 Workflow Examples

### Example 1: Simple Roundtrip
```bash
# 1. Run form through TypeDialog
typedialog form provisioning/config-form.toml > form-output.json

# 2. Convert JSON to Nickel using template
# (Internally uses provisioning/templates/values-template.ncl.j2)
json-to-nickel form-output.json > values/my-env.ncl

# 3. Nickel evaluates and validates
nickel eval values/my-env.ncl
# Validators run, constraints enforced

# 4. Export to JSON
nickel-to-json values/my-env.ncl > envs/my-env.json

# 5. Create environment
cargo run --bin torrust-tracker-deployer -- create environment --env-file envs/my-env.json
```

### Example 2: Edit and Roundtrip Again
```bash
# 1. User manually edits Nickel
vim values/my-env.ncl
# Changes: udp_trackers ports

# 2. Validate changes
nickel eval values/my-env.ncl
# Validators check new values

# 3. Use in roundtrip-enabled form editor
typedialog roundtrip \
  --form provisioning/config-form.toml \
  --input values/my-env.ncl \
  --template provisioning/templates/values-template.ncl.j2
# Form shows current values, user can edit in UI

# 4. Continue workflow...
```

## 📝 Synchronization Rules

For template to work correctly with validators and constraints:

### Rule 1: Import All Validators
```jinja2
let validators_tracker = import "../validators/tracker.ncl" in  ✅
let constraints = import "../constraints.toml" in        ✅
```

### Rule 2: Apply Validators to Arrays
```jinja2
# ✅ Correct: Uses ValidTrackerArrayFull
udp_trackers = validators_tracker.ValidTrackerArrayFull
  [...]
  constraints.udp.min_items
  constraints.udp.max_items,

# ❌ Wrong: Missing validator
udp_trackers = [
  { bind_address = "0.0.0.0:6969" },
],
```

### Rule 3: Use Constraints in Templates
```jinja2
# ✅ Correct: Pulls from centralized constraints
constraints.udp.min_items
constraints.udp.max_items

# ❌ Wrong: Hardcoded values (diverges from constraints)
1  # What if constraints change to min=2?
4
```

## 🔗 Related Files

- **Template**: `provisioning/templates/values-template.ncl.j2`
- **Validators**: `provisioning/validators/tracker.ncl`
- **Constraints**: `provisioning/constraints.toml`
- **Form**: `provisioning/config-form.toml`
- **Documentation**: `provisioning/README.md` (Architecture overview)

## 🐛 Troubleshooting

### Problem: Form generates Nickel without validators
**Cause**: Template not updated, or old template still in use

**Solution**:
1. Verify `values-template.ncl.j2` has:
   - ✅ `let validators_tracker = import ...`
   - ✅ `let constraints = import ...`
   - ✅ `ValidTrackerArrayFull` on tracker arrays
2. Ensure converter uses correct template path
3. Test template rendering: `nickel eval generated.ncl`

### Problem: Validators not found during evaluation
**Cause**: Template imports incorrect path or file missing

**Solution**:
1. Check import path: `import "../validators/tracker.ncl"`
2. Verify file exists: `ls provisioning/validators/tracker.ncl`
3. Test import standalone: `nickel eval provisioning/validators/tracker.ncl`

### Problem: Constraints reference incorrect values
**Cause**: Constraints changed but template not updated

**Solution**:
1. Check template uses `constraints.udp.min_items` (dynamic)
2. NOT hardcoded `1` or `4` (static)
3. Update constraints in ONE place: `constraints.toml`
4. Template will automatically use new values

## ✅ Validation Checklist

After updating templates:

- [ ] `values-template.ncl.j2` has `validators_tracker` import
- [ ] `values-template.ncl.j2` has `constraints` import
- [ ] Tracker arrays use `ValidTrackerArrayFull`
- [ ] Template uses `constraints.udp.*` not hardcoded values
- [ ] Generated .ncl evaluates without errors
- [ ] Validators check uniqueness and array bounds
- [ ] Multiple tracker items work (if supported by form)

## 🎯 Next Steps

When form is updated to support multiple trackers per type:

1. Update form `config-form.toml` to capture multiple items
2. Update template to loop over array items:
   ```jinja2
   {%- for tracker in udp_tracker_addresses %}
   { bind_address = validators_network.ValidBindAddress "{{ tracker }}" },
   {%- endfor %}
   ```
3. Template will automatically validate with `ValidTrackerArrayFull`
4. Constraints will enforce min/max/uniqueness

## 📚 See Also

- `provisioning/README.md` - Complete provisioning system architecture
- `provisioning/values/config.ncl` - Example manual config (reference)
- `provisioning/constraints/README.md` - How to change validation limits
- `provisioning/validators/README.md` - Validator implementation details
