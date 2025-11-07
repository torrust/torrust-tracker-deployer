# Environment Variable Naming: Condition vs Action

This guide explains when to use condition-based vs action-based naming for environment variables, with practical examples and decision-making frameworks.

## The Fundamental Question

When naming environment variables, you face a choice:

- **Condition-based** (describes context/state): `RUNNING_IN_AGENT_ENV`, `IS_PRODUCTION`, `IN_CI`
- **Action-based** (describes behavior/effect): `SKIP_SLOW_TESTS`, `ENABLE_DEBUG`, `USE_CACHE`

**There is no universal "always use X" rule** - the best choice depends on the **scope of impact** and **purpose** of the variable.

## Two Schools of Thought

### Condition-Based Naming

**Philosophy**: Describe _what is happening_ or _where you are_

**Examples:**

```bash
NODE_ENV=production
RAILS_ENV=development
CI=true
RUNNING_IN_DOCKER=true
IS_PREVIEW_ENVIRONMENT=true
```

**Characteristics:**

- Declarative style: "I am X"
- Describes the system's state or context
- Often triggers multiple behavioral changes
- Common for platform/infrastructure concerns
- Application logic decides what to do with the condition

**Code pattern:**

```rust
if env::var("RUNNING_IN_AGENT_ENV").unwrap_or_default() == "true" {
    // Agent has time constraints
    skip_slow_tests = true;
    reduce_logging = true;
    disable_interactive_prompts = true;
}
```

### Action-Based Naming

**Philosophy**: Describe _what effect you want_ or _what behavior to change_

**Examples:**

```bash
SKIP_SLOW_TESTS=true
ENABLE_DEBUG_LOGGING=true
USE_PRODUCTION_DATABASE=true
DISABLE_TELEMETRY=true
FORCE_COLOR_OUTPUT=true
```

**Characteristics:**

- Imperative style: "Do X"
- Describes the intended behavior directly
- Usually controls one specific behavior
- Common for feature toggles
- Clear, single-purpose intent

**Code pattern:**

```rust
if env::var("SKIP_SLOW_TESTS").unwrap_or_default() == "true" {
    // Very clear: skip slow tests
    skip_slow_tests = true;
}
```

## Decision Framework

Use this decision tree to choose the right approach:

```text
Does this variable affect multiple subsystems?
├─ YES → Consider Condition-Based
│   └─ Example: ENVIRONMENT=staging affects logging, DB, cache, API endpoints
│
└─ NO → Prefer Action-Based
    └─ Example: SKIP_VALIDATION=true only affects validation logic

Is this a platform/infrastructure concern?
├─ YES → Condition-Based
│   └─ Example: CI=true, IS_DOCKER=true, KUBERNETES_SERVICE_HOST
│
└─ NO → Action-Based
    └─ Example: ENABLE_FEATURE_X=true, USE_CACHE=true

Do multiple developers/systems need to interpret this differently?
├─ YES → Condition-Based (let each decide behavior)
│   └─ Example: NODE_ENV=production → bundler optimizes, logger reduces verbosity
│
└─ NO → Action-Based (explicit intent)
    └─ Example: COMPRESS_RESPONSES=true → clear, unambiguous
```

## Scope-Based Guidelines

### 1. Platform/Infrastructure Level → Condition-Based

**When the environment itself is the concern:**

```bash
# Good: Describes where the application is running
ENVIRONMENT=production
NODE_ENV=development
RAILS_ENV=test
KUBERNETES_SERVICE_HOST=10.0.0.1
CI=true
```

**Why condition-based?**

- Multiple subsystems need to know the context
- Different components may react differently
- Standard conventions (NODE_ENV, RAILS_ENV) aid ecosystem compatibility

**Example in practice:**

```javascript
// package.json - Multiple tools use NODE_ENV
{
  "scripts": {
    "start": "NODE_ENV=production node server.js",
    "dev": "NODE_ENV=development nodemon server.js"
  }
}

// Webpack, Babel, Express, etc. all check NODE_ENV and adjust behavior
```

### 2. Feature/Behavior Level → Action-Based

**When toggling specific functionality:**

```bash
# Good: Describes what behavior to change
SKIP_SLOW_TESTS=true
ENABLE_DEBUG_LOGGING=true
USE_REDIS_CACHE=true
DISABLE_RATE_LIMITING=true
FORCE_SSL=true
```

**Why action-based?**

- Single responsibility - one variable, one behavior
- Self-documenting - immediately clear what changes
- No ambiguity about intent
- Easier to test - toggle one thing at a time

**Example in practice:**

```rust
// Clear, focused behavior control
if env::var("SKIP_SLOW_TESTS").unwrap_or_default() == "true" {
    steps.retain(|step| !step.is_slow());
}

if env::var("ENABLE_DEBUG_LOGGING").unwrap_or_default() == "true" {
    logger.set_level(Level::Debug);
}
```

### 3. Hybrid Approach (Real-World Pattern)

**Many systems use both strategies together:**

```bash
# Condition (broad context)
NODE_ENV=production

# Actions (specific overrides)
ENABLE_DEBUG=true              # Override: debug even in production
SKIP_MINIFICATION=true         # Override: skip minification in production
USE_LOCAL_DATABASE=true        # Override: use local DB in production
```

**Pattern:**

- Condition sets defaults for an environment
- Actions provide fine-grained control to override defaults

## Real-World Examples

### Example 1: GitHub Actions CI

**Condition-based:**

```yaml
env:
  CI: true # Condition: "we are in CI"
  GITHUB_ACTIONS: true # Condition: "we are in GitHub Actions"
```

**Why?** Many tools check `CI=true` and adjust behavior (disable colors, reduce interactivity, etc.)

### Example 2: Feature Flags

**Action-based:**

```bash
ENABLE_NEW_PAYMENT_FLOW=true
ENABLE_EXPERIMENTAL_API=true
ENABLE_BETA_FEATURES=true
```

**Why?** Each variable controls one specific feature - clear, testable, manageable.

### Example 3: Docker Detection (Anti-pattern)

**❌ Poor (action-based for context):**

```bash
SKIP_HOST_NETWORK_CHECK=true   # Why are we skipping? Not clear!
```

**✅ Better (condition-based):**

```bash
RUNNING_IN_DOCKER=true          # Context is clear
# Code decides: "if in Docker, skip host network check"
```

### Example 4: Torrust Tracker Deployer (This Project)

**Our choice: Action-based (`TORRUST_TD_SKIP_SLOW_TESTS`)**

**Rationale:**

- ✅ Specific behavior toggle (not platform context)
- ✅ Single responsibility (skip slow tests only)
- ✅ Clear intent (no guessing what happens)
- ✅ Reusable in multiple contexts (agent, local dev, CI)
- ✅ Testable (easy to verify behavior)

**Alternative we rejected: `TORRUST_TD_RUNNING_IN_AGENT_ENV`**

**Why rejected:**

- ❌ Describes context, not intent
- ❌ Code must infer: "if agent env, then what?"
- ❌ Tied to one specific context (agent)
- ❌ Less reusable (can't use for other scenarios)

## Naming Conventions

### General Best Practices

1. **Use SCREAMING_SNAKE_CASE**

   - `ENVIRONMENT`, not `environment` or `Environment`
   - Industry standard across all languages

2. **Be explicit and descriptive**

   - ✅ `ENABLE_REQUEST_LOGGING`
   - ❌ `LOG` (too vague)

3. **Use prefixes for namespacing**

   - ✅ `TORRUST_TD_SKIP_SLOW_TESTS`
   - Prevents conflicts with system/library variables

4. **Boolean-like variables should be obvious**

   - ✅ `ENABLE_X=true/false`
   - ✅ `SKIP_X=true/false`
   - ✅ `IS_X=true/false`
   - ❌ `X=1` (unclear what 1 means)

5. **Avoid negative logic when possible**
   - ✅ `ENABLE_CACHE=false` (clear)
   - ❌ `DISABLE_CACHE=false` (double negative)
   - Exception: When the default is "enabled" and you want to disable

### Action-Based Naming Patterns

Common prefixes for action-based variables:

- `ENABLE_*` - Turn on a feature
- `DISABLE_*` - Turn off a feature
- `SKIP_*` - Skip an operation
- `USE_*` - Use a specific implementation/resource
- `FORCE_*` - Override normal behavior
- `ALLOW_*` - Permission-related
- `REQUIRE_*` - Requirement-related

**Examples:**

```bash
ENABLE_DEBUG_MODE=true
DISABLE_TELEMETRY=true
SKIP_MIGRATIONS=true
USE_MOCK_DATA=true
FORCE_HTTPS=true
ALLOW_ANONYMOUS_ACCESS=true
REQUIRE_EMAIL_VERIFICATION=true
```

### Condition-Based Naming Patterns

Common patterns for condition-based variables:

- `*_ENV` - Environment type
- `IS_*` - Boolean state
- `IN_*` - Location/context
- `RUNNING_IN_*` - Execution context
- `HAS_*` - Capability presence

**Examples:**

```bash
NODE_ENV=production
IS_PRODUCTION=true
IN_KUBERNETES=true
RUNNING_IN_CONTAINER=true
HAS_GPU=true
```

## Common Anti-Patterns

### ❌ Ambiguous Names

```bash
# Bad: What does this do?
MODE=fast
OPTIMIZATION=1
CONFIG_TYPE=special
```

**Fix:** Be explicit about behavior:

```bash
# Good: Clear what happens
SKIP_SLOW_TESTS=true
ENABLE_OPTIMIZATIONS=true
USE_PRODUCTION_CONFIG=true
```

### ❌ Inconsistent Naming

```bash
# Bad: Mixing styles without reason
IS_PRODUCTION=true
SKIP_TESTS=true
debugMode=enabled
```

**Fix:** Choose a consistent pattern:

```bash
# Good: Consistent style
ENVIRONMENT=production
SKIP_SLOW_TESTS=true
ENABLE_DEBUG_MODE=true
```

### ❌ Overly Generic Platform Variables for Specific Behaviors

```bash
# Bad: Too broad for specific behavior
AGENT_MODE=true  # What does this actually do?
```

**Fix:** Use action-based for specific behaviors:

```bash
# Good: Clear, specific behaviors
SKIP_SLOW_TESTS=true
REDUCE_LOG_VERBOSITY=true
DISABLE_INTERACTIVE_PROMPTS=true
```

### ❌ Coupling Too Many Behaviors to One Condition

```bash
# Bad: One variable does too much
if env::var("RUNNING_IN_AGENT").is_ok() {
    skip_tests();
    disable_logging();
    skip_validation();
    compress_output();
    use_fast_mode();
    disable_colors();
    // ... and 10 more things
}
```

**Fix:** Separate concerns or use explicit action variables:

```bash
# Good: Each behavior is controllable
SKIP_SLOW_TESTS=true
LOG_LEVEL=warn
SKIP_VALIDATION=false
COMPRESS_OUTPUT=true
```

## Testing Considerations

**Action-based variables are easier to test:**

```rust
#[test]
fn it_should_skip_slow_tests_when_env_var_set() {
    std::env::set_var("SKIP_SLOW_TESTS", "true");

    let steps = get_verification_steps();

    assert!(!steps.iter().any(|s| s.is_slow()));
}
```

**Condition-based requires testing all derived behaviors:**

```rust
#[test]
fn it_should_adapt_to_agent_environment() {
    std::env::set_var("RUNNING_IN_AGENT_ENV", "true");

    // Must test all behaviors that change
    assert!(tests_are_skipped());
    assert!(logging_is_reduced());
    assert!(prompts_are_disabled());
    // ... many more assertions
}
```

## Migration Strategy

If you need to migrate from condition-based to action-based (or vice versa):

### Step 1: Support Both (Transitional Period)

```rust
// Support old and new variable names
let skip_slow_tests = env::var("SKIP_SLOW_TESTS").is_ok()
    || env::var("RUNNING_IN_AGENT_ENV").is_ok();
```

### Step 2: Deprecation Warning

```rust
if env::var("RUNNING_IN_AGENT_ENV").is_ok() {
    eprintln!("Warning: RUNNING_IN_AGENT_ENV is deprecated. Use SKIP_SLOW_TESTS=true instead.");
}
```

### Step 3: Update Documentation

Document the new variable and migration path.

### Step 4: Remove Old Variable (After Grace Period)

After sufficient time (version bumps, etc.), remove support for old variable.

## Summary and Recommendations

### Use Condition-Based When:

- ✅ Describing platform/infrastructure context
- ✅ Multiple subsystems need to know the context
- ✅ Following ecosystem conventions (NODE_ENV, RAILS_ENV)
- ✅ The "environment" itself is the concern

### Use Action-Based When:

- ✅ Controlling specific behaviors or features
- ✅ Single-purpose toggles
- ✅ Clear, testable behavior changes
- ✅ Could be reused in multiple contexts
- ✅ User/developer needs explicit control

### For Torrust Tracker Deployer:

We chose **action-based** (`TORRUST_TD_SKIP_SLOW_TESTS`) because:

- It's a specific behavior control (not platform context)
- Single responsibility (one variable, one purpose)
- Reusable across contexts (agent, dev, CI)
- Clear, testable, self-documenting

### General Principle

> **"Start action-based (specific), move to condition-based only when multiple behaviors need to coordinate."**

If you find yourself checking one condition to control many unrelated behaviors, that's a sign you might need multiple action-based variables instead.

## References

- [The Twelve-Factor App - Config](https://12factor.net/config)
- [Environment Variables Naming Convention](https://en.wikipedia.org/wiki/Environment_variable#Naming_conventions)
- [Docker Environment Variables Best Practices](https://docs.docker.com/compose/environment-variables/best-practices/)
- [GitHub Actions Environment Variables](https://docs.github.com/en/actions/learn-github-actions/environment-variables)

## Related Documentation

- [Environment Variable Prefix ADR](../decisions/environment-variable-prefix.md) - Project naming convention
- [Copilot Agent Pre-commit Config](./copilot-agent/pre-commit-config.md) - Practical example of this decision
