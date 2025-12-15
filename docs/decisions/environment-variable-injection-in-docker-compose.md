# Environment Variable Injection in Docker Compose Templates

- **Status**: ✅ Accepted
- **Date**: 2025-12-13
- **Related Issues**: #232 (MySQL Slice - Release and Run Commands)
- **Related Decisions**:
  - [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md)
  - [Environment Variable Prefix](./environment-variable-prefix.md)

## Context

When implementing MySQL support for the tracker deployment (#232), we initially hardcoded MySQL credentials directly in the `docker-compose.yml.tera` template using Tera template variables:

```yaml
# ❌ INCORRECT: Hardcoded values at template generation time
environment:
  - MYSQL_ROOT_PASSWORD={{ database.mysql.root_password }}
  - MYSQL_DATABASE={{ database.mysql.database }}
  - MYSQL_USER={{ database.mysql.user }}
  - MYSQL_PASSWORD={{ database.mysql.password }}
```

This approach had a critical flaw: values are embedded into the `docker-compose.yml` file at template generation time (during the `release` command). System administrators managing a deployed system cannot modify these values without regenerating the entire template infrastructure.

The existing tracker service already uses the correct pattern - referencing environment variables from the `.env` file:

```yaml
# ✅ CORRECT: Reference to .env file variable
environment:
  - TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN}
```

This pattern allows system administrators to:

1. Edit the `.env` file with new values
2. Restart the Docker Compose stack: `docker-compose down && docker-compose up -d`
3. Have the new configuration take effect immediately

Without regenerating any templates or redeploying the entire application stack.

## Decision

**All configuration values that may need to be changed during system maintenance must be injected via environment variables from the `.env` file, not hardcoded in the `docker-compose.yml` template.**

### Implementation Pattern

**Template Generation (deploy-time)**:

- `EnvContext` contains actual credential values
- `.env.tera` template renders these values: `MYSQL_ROOT_PASSWORD='{{ mysql_root_password }}'`
- `docker-compose.yml.tera` references environment variables: `MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}`

**Runtime (maintenance)**:

- System administrator edits `.env` file with new credentials
- Restarts Docker Compose services
- New credentials take effect without template regeneration

### Template Documentation

Added comment block to `docker-compose.yml.tera` to prevent future violations:

```yaml
# IMPORTANT: Environment Variable Injection Pattern
# ================================================
# All configuration values that may need to be changed during maintenance
# should be injected via environment variables from the .env file, not
# hardcoded in this docker-compose template.
#
# Pattern to follow:
#   CORRECT:   - MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}
#   INCORRECT: - MYSQL_ROOT_PASSWORD=hardcoded_value
#
# Rationale:
# - System administrators can modify .env values and restart services without
#   regenerating templates
# - Supports runtime configuration changes without redeployment
# - Follows Docker Compose best practices for configuration management
# - Separates template generation (deploy-time) from configuration (runtime)
```

## Consequences

### Positive

1. **Runtime Configuration Changes**: System administrators can modify credentials and configuration without regenerating templates or redeploying
2. **Standard Docker Compose Practice**: Follows official Docker Compose recommendations for environment variable management
3. **Separation of Concerns**: Clear separation between:
   - Template structure (generated once during release)
   - Configuration values (modifiable during maintenance)
4. **Consistent Pattern**: All services follow the same environment variable injection pattern
5. **Security Benefits**: Credentials can be rotated without code changes or redeployment
6. **Reduced Coupling**: Template generation is independent of specific credential values

### Negative

1. **Template Complexity**: Requires maintaining both `.env.tera` and `docker-compose.yml.tera` templates
2. **Two-File Coordination**: Developers must ensure `.env.tera` defines all variables referenced in `docker-compose.yml.tera`
3. **Testing Considerations**: Tests must verify both template generation and environment variable injection work correctly

### Neutral

1. **Documentation Requirement**: Pattern must be clearly documented to prevent future hardcoding
2. **Code Review Focus**: PRs adding new services must verify environment variable injection pattern

## Implementation Notes

### Files Modified

1. **`src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`**:

   - Extended `EnvContext` to include optional MySQL credentials
   - Added `new_with_mysql()` constructor for MySQL mode
   - Added getters for MySQL fields

2. **`templates/docker-compose/.env.tera`**:

   - Added conditional MySQL environment variables section
   - Variables only rendered when MySQL is configured

3. **`templates/docker-compose/docker-compose.yml.tera`**:

   - Changed MySQL service to use `${MYSQL_ROOT_PASSWORD}` syntax
   - Added documentation comment explaining the pattern
   - Port hardcoded to 3306 (not configuration-dependent)

4. **`src/application/steps/rendering/docker_compose_templates.rs`**:
   - Updated to create `EnvContext` with MySQL credentials when configured
   - Passes MySQL values to both `EnvContext` and `DockerComposeContext`

### Testing Strategy

- Unit tests verify environment variable references appear in rendered `docker-compose.yml`
- Tests check for `${MYSQL_ROOT_PASSWORD}` instead of hardcoded values
- `.env` file generation tests verify MySQL variables are present when configured

## Alternatives Considered

### 1. Hardcode Values in docker-compose.yml

**Approach**: Keep using `{{ database.mysql.root_password }}` in `docker-compose.yml.tera`

**Rejected because**:

- Requires regenerating entire template stack for credential changes
- Violates Docker Compose best practices
- Inconsistent with existing tracker service pattern
- Forces redeployment for routine maintenance tasks

### 2. Use docker-compose.yml Variables with Defaults

**Approach**: Use `${MYSQL_ROOT_PASSWORD:-default_value}` syntax

**Rejected because**:

- Defaults in production deployments are security anti-pattern
- Still requires `.env` file for actual production values
- Adds unnecessary complexity
- No benefit over explicit `.env` requirement

### 3. External Configuration Management (Vault, AWS Secrets Manager)

**Approach**: Fetch credentials from external secrets management system

**Deferred because**:

- Out of scope for current milestone
- Adds infrastructure dependencies
- Current pattern is sufficient for target use case
- Can be added later without breaking changes

## References

- [Docker Compose Environment Variables Documentation](https://docs.docker.com/compose/environment-variables/)
- [Issue #232: MySQL Slice - Release and Run Commands](https://github.com/torrust/torrust-tracker-deployer/issues/232)
- [12-Factor App: Config](https://12factor.net/config)

## Notes

- This pattern applies to all Docker Compose services, not just MySQL
- Future services must follow the same environment variable injection pattern
- Template variable syntax (`{{ var }}`) should only be used for structural elements that never change at runtime
- Port mapping for MySQL hardcoded to 3306:3306 as it's not expected to vary per deployment
