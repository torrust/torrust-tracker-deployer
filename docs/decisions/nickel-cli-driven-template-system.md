# Decision: Nickel CLI-Driven Template System Architecture

## Status

Accepted

## Date

2025-12-22

## Context

We previously used Tera as our template engine for generating deployment configuration files (Ansible playbooks, OpenTofu variables, Docker Compose, etc.). Tera provided essential features like loops and conditionals but had significant limitations:

1. **Delimiter Conflicts**: Tera uses `{{ }}` and `{% %}` which conflict with Ansible/Jinja2, Kubernetes, Go templates, and frontend frameworks
2. **Tight Rust Coupling**: Template rendering required custom Rust code in the infrastructure layer
3. **Limited Type Safety**: Limited validation of configuration structure before rendering
4. **Escaping Complexity**: Extensive use of `{% raw %}` blocks reduced template readability
5. **Runtime Discovery**: No compile-time validation of template structure

The core requirement remained: **generate valid configuration files with loops, conditionals, and data validation**.

## Decision

We will **replace Tera templates with Nickel configuration language** using a **CLI-driven architecture with no Rust infrastructure layer**:

### Architecture: Three-Stage Pipeline

```
Nickel Template (.ncl)
    ↓ (nickel export --format json)
    ↓
JSON Output
    ↓ (Nushell/Bash scripts)
    ↓
Target Format (YAML, TOML, HCL, ENV)
```

### Key Principles

#### 1. CLI-First, Not Library-Dependent

Use `nickel export --format json` as the evaluation tool:

```bash
# Directly call Nickel CLI
nickel export --format json provisioning/templates/tracker/config.ncl
```

**Not** a Rust wrapper or custom evaluation layer. The CLI is the primary interface.

#### 2. Nickel for Type-Safe Configuration

Nickel provides:

- **Type contracts** via schemas (define required fields and types)
- **Validators** for runtime constraint checking
- **Import system** for composable configuration
- **No template delimiter conflicts** (uses plain Nickel syntax)
- **Compile-time error detection** (schema violations fail evaluation)

```nickel
# Import reusable modules
let schemas = import "../schemas/tracker.ncl" in
let validators = import "../validators/tracker.ncl" in
let values = import "../values/config.ncl" in

# Type-safe configuration with validation
{
  database = if values.provider == "mysql" then {
    driver = "mysql",
    host = values.mysql_host,
  } else {
    driver = "sqlite3",
    path = "/var/lib/tracker.db",
  },

  # Validators enforce constraints at evaluation time
  http_api = validators.ValidHttpApi values.http_api,
}
```

#### 3. Nushell for Orchestration

Use Nushell (modern shell with type system) for:

- **Format conversion**: JSON → YAML, TOML, HCL, ENV
- **Script composition**: Reusable functions for common operations
- **Error handling**: Consistent, informative error messages
- **Bash fallbacks**: Alternative implementations for portability

```nu
# Nushell script evaluates Nickel and converts format
export def nickel_to_yaml [template: path, output: path]: nothing {
    let json = (nickel export --format json $template | from json)
    $json | to yaml | save $output
}
```

#### 4. No Rust Abstraction Layer

**Rejected**: Custom Rust code to wrap Nickel evaluation

**Rationale**:
- Nickel CLI is proven and well-maintained
- Unnecessary Rust code adds maintenance burden
- Shell scripts are simpler and more transparent
- Follows Unix philosophy: tools do one thing well
- Reduces codebase complexity significantly

### Template Organization

```
provisioning/templates/
├── prometheus/config.ncl              # Prometheus YAML
├── tracker/config.ncl                 # Tracker TOML
├── docker-compose/
│   ├── compose.ncl                    # docker-compose.yml
│   └── env.ncl                        # .env file
├── ansible/
│   ├── inventory.ncl                  # inventory.yml
│   └── variables.ncl                  # variables.yml
└── tofu/
    ├── lxd/variables.ncl              # LXD tfvars
    ├── hetzner/variables.ncl          # Hetzner tfvars
    └── common/cloud-init.ncl          # cloud-init YAML
```

### Format Conversion Pipeline

Each format has specialized renderers:

**YAML Conversion** (via yq):
```bash
nickel export --format json template.ncl | yq -P . > output.yml
```

**HCL Conversion** (custom jq builder):
```bash
nickel export --format json template.ncl | jq -r 'to_entries[] |
  "\(.key) = \(.value | @json)"' > output.tfvars
```

**ENV Conversion** (custom jq builder):
```bash
nickel export --format json template.ncl | jq -r 'to_entries[] |
  "\(.key)=\(.value)"' > output.env
```

**TOML Conversion** (custom jq builder):
```bash
nickel export --format json template.ncl | jq -r 'to_entries[] |
  "\(.key) = \(.value | @json)"' > output.toml
```

### Validation Strategy

Three-layer validation ensures configuration correctness:

1. **Nickel Schema Validation** (at evaluation time):
   - Type contracts enforce structure
   - Validators check constraints
   - Missing fields cause evaluation failure

2. **Format Validation** (post-conversion):
   - `yq validate` for YAML
   - Custom validators for HCL, TOML, ENV

3. **Deployment Validation** (E2E tests):
   - Test actual deployment with generated configs
   - Acceptance criteria: successful infrastructure provisioning

## Consequences

### Positive

**Simplicity**:
- No custom Rust infrastructure needed
- Shell scripts are transparent and composable
- Fewer dependencies to maintain

**Type Safety**:
- Nickel schemas provide compile-time checks
- Validators enforce constraints at evaluation time
- Structured error messages on validation failure

**No Delimiter Conflicts**:
- Nickel syntax doesn't conflict with Ansible/Jinja2/Kubernetes
- Template readability improved (no `{% raw %}` blocks needed)
- Easier to embed in other formats

**Standards-Based**:
- Uses standard CLI tools (jq, yq, nickel)
- Follows Unix philosophy
- Scripts can be called from any language or tool

**Better Error Messages**:
- Nickel provides context-aware error reporting
- Validators give specific constraint violation messages
- JSON structure makes errors debuggable

### Negative

**Learning Curve**:
- Team must learn Nickel language (different from Jinja2)
- Different pattern for composing configurations

**Potential Duplication**:
- Some configuration repeated if not factored into shared modules
- Requires discipline to keep templates DRY

**Format Conversion Complexity**:
- Custom jq/Nu code needed for non-standard formats
- TOML conversion has limitations with nested structures
- Requires testing for each format

**Gradual Integration**:
- Cannot immediately remove all Tera templates
- Dual maintenance period during transition
- Existing Rust code expecting Tera must be adapted

### Mitigation Strategies

**Documentation**:
- Comprehensive template examples
- Guidelines for creating new templates
- Architecture decision record (this document)

**Validation**:
- Extensive E2E tests ensure generated configs work
- Format-specific validators catch conversion errors
- Pre-commit checks validate Nickel syntax

**Code Review**:
- Review template changes for proper structure
- Ensure validators are applied to constrained fields
- Check for DRY principle in schema/validator definitions

**Gradual Transition**:
- Keep existing Tera code until Nickel replaces all use cases
- Run both systems in parallel during transition
- Incremental migration per template type

## Alternatives Considered

### 1. Continue with Tera

**Rejected**: Doesn't solve core problems:
- Delimiter conflicts remain
- No type safety mechanism
- Tight Rust coupling continues
- Requires extensive escaping for complex formats

### 2. Rust Library Wrapper Around Nickel

Example (rejected):
```rust
struct NickelTemplateRenderer {
    template_path: PathBuf,
}

impl NickelTemplateRenderer {
    fn render_to_yaml(&self) -> Result<String> {
        let json = self.evaluate_nickel()?;
        Ok(self.json_to_yaml(&json)?)
    }
}
```

**Rejected Reasons**:
- Unnecessary abstraction layer
- Adds maintenance burden
- Hides CLI availability
- Duplicates work already done by nickel CLI
- Reduces transparency

**Rationale for CLI-first approach**:
- Nickel CLI is the proven tool
- Keep infrastructure simple
- Let shell scripts handle orchestration
- Tools integrate via standard interfaces

### 3. KCL (Kyverno Configuration Language)

**Rejected**:
- Primarily designed for Kubernetes validation
- Less suitable for multi-format configuration generation
- Smaller ecosystem than Nickel
- Would require learning another language

### 4. CUE Language

**Rejected**:
- Complex syntax, steeper learning curve
- Less familiar to team
- Overkill for our configuration needs

## Related Decisions

- [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md) - Previous approach using Tera, describes validation requirements
- [Environment Variable Injection in Docker Compose](./environment-variable-injection-in-docker-compose.md) - Specific configuration pattern
- [Database Configuration Structure in Templates](./database-configuration-structure-in-templates.md) - Configuration organization principles

## Implementation Status

**Completed**:
- 9 Nickel templates created and tested
- Nushell rendering scripts (5 variants: generic, yaml, toml, hcl, env)
- Bash fallback scripts for portability
- Cloud-init bootstrap template
- Comprehensive README with examples
- Validation at Nickel evaluation time

**Partially Complete**:
- TOML conversion works for simple structures (tracker template needs refinement for complex nested arrays)
- Rust integration pending (can call scripts via `Command::new("nu")` or `Command::new("bash")`)

**Future**:
- Incremental Rust integration for ProjectGenerator (minimal, script-calling only)
- Migration away from Tera templates as Nickel coverage expands
- Performance profiling and optimization if needed

## Success Criteria

✅ All 9 templates created and rendering correctly
✅ No Rust infrastructure layer needed
✅ Scripts work with both Nushell and Bash
✅ Output format compatibility verified
✅ E2E tests confirm generated configs work
✅ Documentation complete and examples provided

## References

- [Nickel Language Documentation](https://nickel-lang.org/)
- [Nickel GitHub Repository](https://github.com/tweag/nickel)
- [Nushell Documentation](https://www.nushell.sh/book/)
- [jq Manual](https://jqlang.github.io/jq/)
- [yq Documentation](https://mikefarah.gitbook.io/yq/)
- Template system documentation: `provisioning/templates/README.md`
- Nickel guidelines: `.claude/guidelines/nickel/NICKEL_GUIDELINES.md`

## Review Triggers

This decision should be revisited if:

- Template complexity grows beyond Nickel's capabilities
- Format conversion becomes unmaintainable
- Performance issues emerge from CLI-based approach
- Team feedback indicates Nickel syntax is too unfamiliar
- New configuration formats require custom converters
- Rust integration becomes necessary for other reasons

## Decision Log

- **2025-12-22**: Accepted - Nickel CLI-driven architecture implemented with 9 working templates and shell script orchestration
