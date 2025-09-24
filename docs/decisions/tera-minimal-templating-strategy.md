# Decision: Tera Minimal Templating Strategy

## Status

Accepted

## Date

2025-09-09

## Context

We are implementing a deployment infrastructure using Rust with Tera as the template engine for generating configuration files (Ansible playbooks, OpenTofu configurations, etc.). A key requirement is generating Torrust tracker TOML configuration files that contain lists of multiple tracker instances with repeated configuration blocks per tracker.

During the design phase, we identified several challenges:

1. **Template Logic Requirements**: The Torrust tracker configuration requires:

   - **Loops**: To iterate over multiple tracker instances
   - **Conditionals**: To handle different configurations per environment or tracker type
   - **Complex data structures**: Lists of tracker objects with nested properties

2. **Delimiter Conflicts**: Tera uses `{{ }}` and `{% %}` delimiters, which are identical to:

   - Ansible/Jinja2 templates (`{{ variable }}`, `{% if condition %}`)
   - Kubernetes/Helm templates (`{{ .Values.name }}`)
   - Go templates (`{{ .Title }}`)
   - Frontend frameworks (Vue.js, Angular)

3. **Complexity Management**: Template systems can become complex with nested templates, multiple inheritance levels, and extensive variable passing.

4. **Maintainability**: More templates and variables mean more potential points of failure and harder debugging.

5. **Escaping Overhead**: Using `{% raw %}{% endraw %}` blocks extensively reduces template readability.

## Decision

We will **continue using Tera as our template engine** but adopt a **minimal templating strategy** with the following principles:

### 1. Minimize Number of Tera Variables

- Use only essential variables that truly need to be dynamic
- Prefer sensible defaults over configurability where possible
- Group related configuration into structured objects rather than many individual variables
- Avoid over-parameterization

### 2. Minimize Number of Tera Templates

- Create templates only for files that have significant variation between deployments
- Prefer static configuration files when customization is minimal
- Use composition over template inheritance
- Avoid deeply nested template hierarchies

### 3. Strategic Template Usage

Focus Tera templates on:

- **High-value customization points**: Infrastructure parameters that change between environments
- **Dynamic content generation**: Content that depends on runtime discovery or user input
- **Configuration consolidation**: Bringing together scattered configuration into unified files

Avoid Tera templates for:

- **Static boilerplate**: Files that rarely change
- **Complex nested structures**: Where escaping becomes the dominant pattern
- **Single-use configurations**: Files used only once with no variation

## Implementation Guidelines

### Variable Naming Strategy

```rust
#[derive(Serialize)]
struct DeploymentConfig {
    infrastructure: InfrastructureConfig,
    application: ApplicationConfig,
}
```

### Template File Strategy

```yaml
# ✅ Good: Essential dynamic content only
- name: Deploy {{ application.name }} to {{ infrastructure.environment }}
  hosts: {{ infrastructure.target_hosts }}
  vars:
    app_version: "{{ application.version }}"
  tasks:
    {% raw %}
    - name: Get system facts
      ansible.builtin.setup:
        gather_subset: "{{ ansible_gather_subset | default('all') }}"
    {% endraw %}
```

### Torrust Tracker Configuration Example

This shows the key use case that drove the choice of Tera - generating configuration files with loops and conditionals:

```rust
// Context structure for Torrust tracker configuration
#[derive(Serialize)]
struct TorrustDeploymentConfig {
    environment: String,
    trackers: Vec<TrackerConfig>,
}

#[derive(Serialize)]
struct TrackerConfig {
    name: String,
    bind_address: String,
    port: u16,
    ssl_enabled: bool,
    ssl_cert_path: Option<String>,
}
```

```toml
# Generated torrust-tracker.toml template
[logging]
level = "info"

[database]
connect_url = "sqlite://data.db?mode=rwc"

# Multiple HTTP tracker instances - requires loops
{% for tracker in trackers %}
[[http_trackers]]
name = "{{ tracker.name }}"
bind_address = "{{ tracker.bind_address }}:{{ tracker.port }}"

# Conditional SSL configuration - requires conditionals
{% if tracker.ssl_enabled %}
ssl_cert_path = "{{ tracker.ssl_cert_path }}"
ssl_key_path = "{{ tracker.ssl_cert_path | replace(from='.crt', to='.key') }}"
{% endif %}

# Environment-specific settings
{% if environment == "production" %}
access_tokens = { read = "production_read_token" }
{% else %}
access_tokens = { read = "dev_read_token" }
{% endif %}

{% endfor %}
```

### Escaping Strategy

- Use `{% raw %}{% endraw %}` blocks for sections with extensive Ansible/Jinja2 syntax
- Keep Tera variables minimal to reduce escaping needs
- Document which sections are Tera vs target template syntax

## Consequences

### Positive

- **Reduced complexity**: Fewer templates and variables to maintain
- **Better readability**: Less escaping noise in templates
- **Easier debugging**: Fewer moving parts when issues occur
- **Faster development**: Less time spent on template engineering
- **Lower learning curve**: Easier for team members to understand and modify

### Negative

- **Less flexibility**: Some customization might require code changes instead of configuration
- **Potential duplication**: Some configuration might be repeated instead of parameterized
- **Template escaping**: Still need `{% raw %}` blocks for complex target syntax

### Mitigation Strategies

- **Documentation**: Clearly document which parts are static vs dynamic
- **Code review focus**: Review template additions carefully for necessity
- **Regular refactoring**: Periodically review templates to identify over-parameterization
- **Alternative approaches**: Consider configuration files or environment variables for some customization

## Alternatives Considered

### 1. Custom Delimiter Configuration

**Rejected**: Tera doesn't support changing delimiters (`set_variable_marker` doesn't exist in current version)

### 2. Simple String Replacement

**Rejected**: Cannot handle the complexity of Torrust tracker configuration which requires:

- **Loops over tracker lists**: Need `{% for tracker in trackers %}` syntax
- **Conditional logic**: Different settings per environment or tracker type
- **Nested data access**: Complex TOML structure generation

Example of what we need to generate:

```toml
# Multiple tracker instances with repeated configuration
{% for tracker in trackers %}
[[http_trackers]]
name = "{{ tracker.name }}"
bind_address = "{{ tracker.bind_address }}"
{% if tracker.ssl_enabled %}
ssl_cert_path = "{{ tracker.ssl_cert_path }}"
{% endif %}
{% endfor %}
```

Simple string replacement like `template.replace("{{name}}", value)` cannot handle this logic.

### 3. Different Template Engine (Handlebars, Mustache, etc.)

**Rejected**: Would require:

- **Learning new syntax**: Team already familiar with Jinja2/Tera syntax
- **Migration effort**: Existing templates would need rewriting
- **Same delimiter conflicts**: Most template engines use `{{ }}` syntax

### 4. Multiple Template Engines

**Rejected**: Increases complexity and dependencies without solving the core requirement for loops and conditionals.

## Related Decisions

- [LXD over Multipass](./lxd-over-multipass.md) - Infrastructure choice affecting template needs
- [Docker Testing Evolution](./docker-testing-evolution.md) - Testing approach evolution that influences template complexity

## References

- [Tera Documentation](https://docs.rs/tera/latest/tera/)
- [Ansible Jinja2 Templates](https://docs.ansible.com/ansible/latest/user_guide/playbooks_templating.html)
- [GitHub Actions Network Issues](https://github.com/actions/runner-images/issues/2890) - Referenced in our templates

## Review

This decision should be reviewed if:

- Template complexity grows significantly
- Team feedback indicates templates are hard to maintain
- New requirements emerge that need extensive parameterization
- Performance issues arise from current approach
