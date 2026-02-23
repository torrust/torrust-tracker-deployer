---
name: create-environment-variables
description: Guide for creating and naming environment variables in this project. All project environment variables must use the TORRUST_TD_ prefix. Covers the condition-based vs action-based naming decision framework, when to use each approach, and practical examples. Use when adding new environment variables, changing existing env var names, or deciding how to name a toggle or flag. Triggers on "environment variable", "env var", "TORRUST_TD_", "variable naming", "feature flag", "feature toggle", "configuration flag", or "add env var".
metadata:
  author: torrust
  version: "1.0"
---

# Creating Environment Variables

## Mandatory: TORRUST*TD* Prefix

**All project environment variables must use the `TORRUST_TD_` prefix.**

```bash
# ✅ Good
TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true
TORRUST_TD_LOG_LEVEL=debug

# ❌ Bad: missing prefix
SKIP_FIREWALL=true
LOG_LEVEL=debug
```

The prefix prevents naming conflicts with system, OS, or tool environment variables.

## Condition-Based vs Action-Based Naming

### Condition-Based: "I am X" / "Running in X"

Describes **context or state**. Use when the variable affects multiple behaviors.

```bash
TORRUST_TD_RUNNING_IN_AGENT_ENV=true   # affects logging, timeouts, interactivity
```

```rust
if env::var("TORRUST_TD_RUNNING_IN_AGENT_ENV").unwrap_or_default() == "true" {
    skip_slow_tests = true;
    reduce_logging = true;
    disable_interactive_prompts = true;
}
```

### Action-Based: "Do X" / "Skip X"

Describes **a specific behavior**. Use when the variable controls one thing.

```bash
TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true  # controls one specific behavior
```

```rust
if env::var("TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER").unwrap_or_default() == "true" {
    // only affects firewall step
}
```

## Decision Framework

```text
Does this variable affect multiple subsystems?
├─ YES → Condition-Based ("TORRUST_TD_IN_CI", "TORRUST_TD_IS_CONTAINER")
└─ NO → Action-Based ("TORRUST_TD_SKIP_VALIDATION", "TORRUST_TD_ENABLE_X")

Is it a platform/infrastructure concern?
├─ YES → Condition-Based
└─ NO → Action-Based
```

## Boolean Values Convention

Use `"true"` or `"false"` (lowercase strings):

```bash
TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true   # ✅
TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=True   # ❌ (inconsistent)
TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=1      # ❌ (use strings)
```

## Examples from the Codebase

| Variable                                | Type   | Controls                      |
| --------------------------------------- | ------ | ----------------------------- |
| `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER` | Action | Skip UFW config in containers |

## Reference

Naming guide: [`docs/contributing/environment-variables-naming.md`](../../docs/contributing/environment-variables-naming.md)
Prefix ADR: [`docs/decisions/environment-variable-prefix.md`](../../docs/decisions/environment-variable-prefix.md)
