# Decision: Database Configuration Structure in Templates

## Status

✅ Accepted

## Date

2025-12-14

## Context

When implementing MySQL support for the Torrust Tracker deployer (Issue #232), we needed to decide how to represent database connection information in the `tracker.toml.tera` template. There were two main approaches:

1. **Pre-resolved connection string approach**: Pass a single `database_path` variable to the template containing the complete connection string (e.g., `"mysql://user:pass@host:3306/db"` or `"/var/lib/torrust/tracker/sqlite3.db"`).

2. **Structured configuration approach**: Expose the database configuration structure in the template with separate variables for each database type (driver, host, port, user, password, database_name) and construct the connection string within the template.

The choice impacts:

- Template clarity and readability
- Alignment with future Torrust Tracker API design
- Maintainability and understanding of configuration
- Abstraction of ORM implementation details

## Decision

We will use the **structured configuration approach** - exposing database configuration fields in templates and constructing connection strings within the template using conditional logic based on `database_driver`.

Example implementation in `tracker.toml.tera`:

```toml
[database]
driver = "{{ database_driver }}"
{% if database_driver == "sqlite3" %}
path = "/var/lib/torrust/tracker/{{ database_name }}"
{% elif database_driver == "mysql" %}
path = "mysql://{{ database_user }}:{{ database_password }}@{{ database_host }}:{{ database_port }}/{{ database_name }}"
{% endif %}
```

## Consequences

### Positive

- **Alignment with future Tracker API design**: The Torrust Tracker project is considering exposing structured database configuration (host, port, user, password, database) rather than connection strings in its public API. This template structure aligns with that future direction.

- **Abstraction of implementation details**: Connection string format (using URLs) is an internal implementation detail of the SQLx ORM. By exposing structured fields, we hide this detail and make the configuration more intuitive.

- **Clear configuration structure**: Template readers can immediately see what database parameters are required for each database type without parsing connection strings.

- **Type-specific validation**: Each database type's required fields are explicit in the template, making validation and error messages clearer.

- **Easier to extend**: Adding new database types (e.g., PostgreSQL) requires adding a new conditional block with appropriate fields, rather than complex string manipulation logic.

### Negative

- **Template complexity**: The template contains conditional logic for constructing connection strings, making it slightly more complex than passing a pre-resolved string.

- **Duplication of connection logic**: If multiple templates need database configuration, the connection string construction logic may be duplicated (though this is currently only in `tracker.toml.tera`).

- **Temporary disconnect**: Until the Tracker implements structured configuration API, there's a mismatch between our exposed structure and the Tracker's actual API (connection string in TOML).

### Risks and Mitigations

- **Risk**: Connection string format changes in SQLx or Tracker dependencies.

  - **Mitigation**: All connection string construction is centralized in templates. Changes only require template updates, not Rust code changes.

- **Risk**: Template maintainers may not understand ORM connection string requirements.
  - **Mitigation**: Add comprehensive comments in the template explaining the format for each database type and linking to relevant documentation.

## Alternatives Considered

### Alternative 1: Pre-resolved Connection String

**Description**: Pass a single `database_path` variable containing the complete connection string resolved in Rust code before template rendering.

**Pros**:

- Simpler template with no conditional logic
- Connection string construction in type-safe Rust code
- Single source of truth for connection format

**Cons**:

- Exposes ORM implementation details in high-level configuration
- Doesn't align with planned Tracker API improvements
- Less clear what configuration parameters are required
- Connection strings are opaque to template readers

**Why rejected**: Prioritizing alignment with future Tracker design and configuration clarity over template simplicity.

### Alternative 2: Hybrid Approach

**Description**: Pass both structured fields and pre-resolved connection string, allowing templates to choose which to use.

**Pros**:

- Maximum flexibility
- Easy migration path

**Cons**:

- Redundant data in context
- Confusion about which to use
- Increased maintenance burden

**Why rejected**: Adds complexity without clear benefit. We should commit to one approach.

## Related Decisions

- [Environment Variable Injection in Docker Compose](./environment-variable-injection-in-docker-compose.md) - Similar principle of exposing configuration structure rather than pre-computed values
- Issue #232: MySQL slice release/run commands - The implementation that motivated this decision

## References

- [SQLx Database URLs](https://docs.rs/sqlx/latest/sqlx/trait.Database.html) - Connection string format documentation
- [Torrust Tracker Configuration](https://github.com/torrust/torrust-tracker/blob/develop/docs/config.md) - Current configuration format
- Future Tracker configuration API discussion (planned improvement, no reference yet)
- [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md) - Our template philosophy and variable usage approach
