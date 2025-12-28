# Validation Constraints

Central configuration for validation rule limits across **TypeDialog Forms**, **Nickel Validators**, and **Config Files**.

## 📋 Overview

This directory contains constraint definitions that must stay **synchronized** with:
- `provisioning/fragments/tracker-section.toml` (TypeDialog form)
- `provisioning/validators/tracker.ncl` (Nickel validators)
- `provisioning/values/config.ncl` (Example configuration)

## 📁 Files

### constraints.toml (SINGLE Source of Truth)
**Purpose**: Central definition of validation limits used by ALL layers.

**Used by**:
1. **Form parser** - Dynamically loads via `${constraint.tracker.udp.max_items}` interpolation
2. **Nickel validators** - Import directly: `let constraints = import "../constraints.toml" in`
3. **Nickel configs** - Import directly in values files

**When changing limits**:
Edit **ONLY** `constraints.toml`:

```toml
[tracker.udp]
min_items = 2         # Changed from 1 to 2
max_items = 6         # Changed from 4 to 6
unique = true
```

Everything else updates automatically:
- ✅ Form parser sees new values next time it loads
- ✅ Nickel files import the new constraints directly
- ✅ No manual copying between files


## 🔧 Current Constraints

### Tracker Arrays

```nickel
{
  udp = {
    min_items = 1,            # At least 1 UDP listener required
    max_items = 4,            # Maximum 4 UDP listeners allowed
    unique_addresses = true,  # All bind_address must be unique
  },

  http = {
    min_items = 1,            # At least 1 HTTP listener required
    max_items = 4,            # Maximum 4 HTTP listeners allowed
    unique_addresses = true,  # All bind_address must be unique
  },
}
```

## ⚠️ Known Misalignments

### Form vs Nickel: min_items

**Form** (`config-form.toml`):
- Allows `min_items = 0` (optional trackers in UI)

**Nickel** (`constraints/tracker.ncl`):
- Enforces `min_items = 1` (at least 1 tracker required)

**Reason**: Form provides UI flexibility, Nickel enforces production safety.

**Resolution**: Document intentional difference or align both to `min_items = 1`.

## ✅ Changing Constraints

Simply edit `constraints.toml`. Everything else updates automatically:

### Single Step: Edit `constraints.toml`
```toml
[tracker.udp]
min_items = 2         # Changed from 1 to 2
max_items = 6         # Changed from 4 to 6
unique = true
```

Then verify:
```bash
nickel eval provisioning/values/config.ncl     # Imports new constraints
```

**That's it!** All layers use the same constraints automatically:
- Form parser loads new max_items from interpolation
- Nickel configs import new values from constraints.toml
- No manual copying or synchronization needed

## 🧪 Testing Constraints

Test with valid data:
```bash
nickel eval provisioning/values/config.ncl
```

Test with invalid data (empty array):
```bash
cd /Users/Akasha/Development/typedialog && cat > /tmp/test_zero.ncl <<'EOF'
let validators_tracker = import "provisioning/validators/tracker.ncl" in
let constraints = import "provisioning/constraints.toml" in

validators_tracker.ValidTrackerArrayFull
  []  # 0 items - should fail
  constraints.tracker.udp.min_items
  constraints.tracker.udp.max_items
EOF
nickel eval /tmp/test_zero.ncl
```

Test with invalid data (too many items):
```bash
cd /Users/Akasha/Development/typedialog && cat > /tmp/test_overflow.ncl <<'EOF'
let validators_tracker = import "provisioning/validators/tracker.ncl" in
let constraints = import "provisioning/constraints.toml" in

validators_tracker.ValidTrackerArrayFull
  [
    { bind_address = "0.0.0.0:6969" },
    { bind_address = "0.0.0.0:6970" },
    { bind_address = "0.0.0.0:6971" },
    { bind_address = "0.0.0.0:6972" },
    { bind_address = "0.0.0.0:6973" },  # 5 items - exceeds max of 4
  ]
  constraints.tracker.udp.min_items
  constraints.tracker.udp.max_items
EOF
nickel eval /tmp/test_overflow.ncl
```

## 🔗 Related Files

- **Source of Truth**: `provisioning/constraints.toml`
- **Form**: `provisioning/fragments/tracker-section.toml` (uses interpolation)
- **Validators**: `provisioning/validators/tracker.ncl` (imports from constraints.toml)
- **Example config**: `provisioning/values/config.ncl` (imports from constraints.toml)
- **Template**: `provisioning/templates/config-template.ncl.j2` (imports from constraints.toml)
