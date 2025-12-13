# MySQL Slice - Release and Run Commands

**Issue**: #232
**Parent Epic**: [#216](https://github.com/torrust/torrust-tracker-deployer/issues/216) (Implement ReleaseCommand and RunCommand with vertical slices)

## Overview

This task adds MySQL as a database option for the Torrust Tracker deployment. It extends the docker-compose stack with a MySQL service and allows users to choose between SQLite (default) and MySQL through environment configuration. This provides production-ready database scalability options while maintaining backward compatibility with SQLite for simple deployments.

## Goals

- [ ] Add MySQL service conditionally to docker-compose stack (only when enabled in environment)
- [ ] Extend environment configuration to support database selection (SQLite vs MySQL)
- [ ] Update tracker configuration template to support MySQL connection strings
- [ ] Create Ansible playbook to deploy MySQL configuration
- [ ] Maintain backward compatibility - SQLite remains default option
- [ ] Deploy and verify tracker works with both SQLite and MySQL

**Note**: MySQL database schema initialization is handled automatically by the Tracker application through built-in migrations - no manual SQL scripts required.

## 🏗️ Architecture Requirements

**DDD Layers**: Infrastructure + Domain
**Module Paths**:

- `src/infrastructure/templating/docker_compose/` - Docker Compose template rendering with MySQL service
- `src/infrastructure/templating/tracker/` - Tracker configuration template updates for MySQL
- `src/domain/config/environment/` - Environment configuration schema extensions

**Pattern**: Template System with Project Generator pattern + Configuration-driven service selection

### Module Structure Requirements

- [ ] Follow template system architecture (see [docs/technical/template-system-architecture.md](../technical/template-system-architecture.md))
- [ ] Use Project Generator pattern for MySQL-related templates
- [ ] Register static templates explicitly in renderer
- [ ] Use `.tera` extension for dynamic templates
- [ ] Environment config drives database driver selection

### Architectural Constraints

- [ ] MySQL service optional - enabled only when user selects MySQL driver
- [ ] SQLite remains default and requires no configuration changes
- [ ] Database selection controlled through environment configuration
- [ ] Tracker container configuration adapts to selected database driver
- [ ] MySQL credentials managed through environment variables

### Anti-Patterns to Avoid

- ❌ Breaking SQLite deployments when adding MySQL support
- ❌ Hardcoding MySQL credentials in templates
- ❌ Making MySQL mandatory for all deployments
- ❌ Duplicating tracker configuration for each database type
- ❌ Mixing database initialization logic across multiple templates

## Implementation Strategy

The implementation follows an **incremental, backward-compatible approach** where SQLite remains the default and MySQL is added as an opt-in feature.

### Key Principles

1. **Backward Compatibility**: SQLite deployments continue working unchanged
2. **Configuration-Driven**: Database selection through environment config
3. **Service Isolation**: MySQL as independent docker-compose service
4. **Progressive Configuration**: Start with hardcoded MySQL settings, then expose to environment config
5. **Manual Verification**: Test both SQLite and MySQL paths at each step

## Specifications

### Database Driver Selection

**Environment Configuration Addition**:

```json
{
  "deployment": {
    "tracker": {
      "database": {
        "driver": "sqlite3" // or "mysql"
      }
    }
  }
}
```

**Default Behavior**: If `driver` not specified, defaults to `sqlite3`

**Supported Values**:

- `sqlite3`: SQLite database (default, existing behavior)
- `mysql`: MySQL database (new option)

### MySQL Service Configuration

**Docker Compose Service**: Add MySQL 8.0 container conditionally to stack (only when MySQL driver selected in environment configuration)

**Template Location**: `templates/docker-compose/docker-compose.yml` (uses Tera conditionals to include/exclude MySQL service)

**Environment Variables**:

- `MYSQL_ROOT_PASSWORD`: Root user password
- `MYSQL_DATABASE`: Database name (e.g., `torrust_tracker`)
- `MYSQL_USER`: Application database user
- `MYSQL_PASSWORD`: Application user password
- `MYSQL_ROOT_HOST`: Allow root connections from any host (`%`)

**Initial Hardcoded Values** (to be progressively exposed to environment config):

```yaml
environment:
  MYSQL_ROOT_PASSWORD: "root_secret_password"
  MYSQL_DATABASE: "torrust_tracker"
  MYSQL_USER: "db_user"
  MYSQL_PASSWORD: "db_user_secret_password"
  MYSQL_ROOT_HOST: "%"
```

**Ports**:

- Internal: `3306` (MySQL default)
- Exposed: Not exposed to host (internal docker network only)

**Volumes**:

- `mysql_data:/var/lib/mysql` - Persistent database storage

**Network**: `backend_network` (shared with tracker service)

### MySQL Database Initialization

**Database Creation**: MySQL automatically creates the database on first start using `MYSQL_DATABASE` env var

**Schema Migrations**: Tables and schema are created automatically by the Tracker application on startup via built-in migrations (no manual SQL scripts required)

**Reference Schema** (from torrust-tracker codebase):

```sql
CREATE TABLE IF NOT EXISTS torrents (
    id integer PRIMARY KEY AUTO_INCREMENT,
    info_hash VARCHAR(40) NOT NULL UNIQUE,
    completed INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS torrent_aggregate_metrics (
    id integer PRIMARY KEY AUTO_INCREMENT,
    metric_name VARCHAR(50) NOT NULL UNIQUE,
    value INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS `keys` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `key` VARCHAR(32) NOT NULL,
    `valid_until` INT(10),
    PRIMARY KEY (`id`),
    UNIQUE (`key`)
);

CREATE TABLE IF NOT EXISTS whitelist (
    id integer PRIMARY KEY AUTO_INCREMENT,
    info_hash VARCHAR(40) NOT NULL UNIQUE
);
```

### Tracker Configuration for MySQL

**Configuration Path on Host**: `/opt/torrust/storage/tracker/etc/tracker.toml`

**Configuration Path in Container**: `/etc/torrust/tracker/tracker.toml` (mounted from host via docker volume)

**Database Section Update** (conditional based on environment config):

When `driver: mysql`:

```toml
[core.database]
driver = "mysql"
path = "mysql://db_user:db_user_secret_password@mysql:3306/torrust_tracker"
```

When `driver: sqlite3` (default):

```toml
[core.database]
driver = "sqlite3"
path = "./storage/tracker/lib/database/tracker.db"
```

**Template Logic**: Use Tera conditionals to select appropriate configuration block

### Docker Compose Environment Variables

**Tracker Container Updates** (when MySQL selected):

```yaml
environment:
  - TORRUST_TRACKER_DATABASE=mysql
  - TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=mysql
```

**Dependency Management**: Tracker service should depend on MySQL service when MySQL is selected

```yaml
services:
  tracker:
    depends_on:
      - mysql # Only when MySQL driver selected
```

### Storage Directory Structure

**No changes required** - MySQL uses docker volumes, not filesystem storage like SQLite

**SQLite continues using**:

```text
/opt/torrust/storage/tracker/lib/database/tracker.db
```

**MySQL stores data in**:

- Docker volume: `mysql_data`
- Container path: `/var/lib/mysql`

## Implementation Plan

### Phase 1: Add MySQL Service to Docker Compose (2-3 hours)

- [ ] 1.1: Create conditional MySQL service definition in `templates/docker-compose/docker-compose.yml`
  - Service name: `mysql`
  - Image: `mysql:8.0`
  - Hardcoded environment variables (credentials)
  - Volume configuration for data persistence
  - Network configuration (backend_network)
  - **Use Tera conditionals**: Only include service when MySQL driver selected in environment
- [ ] 1.2: Add volume definition for MySQL data (also conditional)
- [ ] 1.3: Test MySQL service starts successfully when driver is mysql
- [ ] 1.4: Test MySQL service is NOT included when driver is sqlite3
- [ ] 1.5: Verify MySQL service accessible from tracker container network

### Phase 2: Extend Environment Configuration (1-2 hours)

- [ ] 2.1: Add `database.driver` field to environment schema
  - Location: `src/domain/config/environment/`
  - Validation: Enum with `sqlite3` and `mysql` values
  - Default: `sqlite3` for backward compatibility
- [ ] 2.2: Update environment JSON schema file
- [ ] 2.3: Update environment template examples
- [ ] 2.4: Add validation tests for database driver selection

### Phase 3: Conditional Tracker Configuration (2-3 hours)

- [ ] 3.1: Update tracker.toml template with Tera conditionals
  - Check `deployment.tracker.database.driver` value
  - Render MySQL connection string when driver is mysql
  - Render SQLite path when driver is sqlite3 (default)
- [ ] 3.2: Update docker-compose template with conditional tracker environment variables
  - Add `TORRUST_TRACKER_DATABASE` based on driver selection
  - Add `TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER`
- [ ] 3.3: Test template rendering for both database drivers
- [ ] 3.4: Verify tracker.toml correctness for both paths

### Phase 4: Service Dependencies and Integration (1-2 hours)

- [ ] 4.1: Add conditional `depends_on` in docker-compose
  - Tracker depends on mysql service only when MySQL selected
  - Maintain existing dependencies for SQLite path
- [ ] 4.2: Update Ansible playbook to handle MySQL configuration
- [ ] 4.3: Update project generator to include MySQL templates

### Phase 5: Manual E2E Testing (2-3 hours)

- [ ] 5.1: Test SQLite path (backward compatibility)
  - Create environment with SQLite driver (or omit driver field)
  - Run full deployment workflow
  - Verify tracker starts and uses SQLite
  - Verify docker-compose stack does not include MySQL service
- [ ] 5.2: Test MySQL path (new feature)
  - Create environment with MySQL driver
  - Run full deployment workflow
  - Verify MySQL service starts
  - Verify tracker starts and connects to MySQL
  - Verify database tables created automatically
- [ ] 5.3: Test tracker functionality with MySQL
  - Test UDP announce endpoint
  - Test HTTP announce endpoint
  - Test API health check endpoint

### Phase 6: Progressive Configuration Exposure (2-3 hours)

- [ ] 6.1: Expose MySQL credentials to environment configuration
  - Add `database.mysql.root_password` field
  - Add `database.mysql.database_name` field
  - Add `database.mysql.user` field
  - Add `database.mysql.password` field
- [ ] 6.2: Update templates to use environment variables instead of hardcoded values
- [ ] 6.3: Update documentation with MySQL configuration examples
- [ ] 6.4: Test with custom MySQL credentials

### Phase 7: Documentation and Cleanup (1 hour)

- [ ] 7.1: Update user documentation with MySQL option
- [ ] 7.2: Add troubleshooting guide for MySQL connections
- [ ] 7.3: Document MySQL-specific environment variables
- [ ] 7.4: Add examples for both SQLite and MySQL deployments

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] MySQL service can be added to docker-compose stack via environment config
- [ ] SQLite remains default when database driver not specified
- [ ] Tracker successfully connects to MySQL when MySQL driver selected
- [ ] Tracker successfully uses SQLite when SQLite driver selected (backward compatibility)
- [ ] MySQL database tables created automatically on first tracker startup
- [ ] Docker volume persists MySQL data across container restarts

**Configuration Requirements**:

- [ ] Environment configuration supports `database.driver` field
- [ ] Environment configuration defaults to `sqlite3` if driver not specified
- [ ] Environment configuration validates driver values (only `sqlite3` or `mysql`)
- [ ] Tracker configuration correctly rendered for both database drivers
- [ ] Docker compose environment variables correctly set based on driver selection

**Testing Requirements**:

- [ ] Manual E2E test passes for SQLite path (backward compatibility verified)
- [ ] Manual E2E test passes for MySQL path (new feature verified)
- [ ] Tracker announce endpoints work with both database drivers
- [ ] Tracker API health check passes with both database drivers

**Documentation Requirements**:

- [ ] User guide updated with database driver selection documentation
- [ ] Environment configuration examples include MySQL option
- [ ] Troubleshooting guide includes MySQL connection issues

## Related Documentation

- **Roadmap**: [docs/roadmap.md](../roadmap.md) - Task 3.2.3
- **Epic**: [docs/issues/216-epic-release-and-run-commands.md](216-epic-release-and-run-commands.md)
- **Architecture**: [docs/technical/template-system-architecture.md](../technical/template-system-architecture.md)
- **Previous Slice**: [docs/issues/220-tracker-slice-release-run-commands.md](220-tracker-slice-release-run-commands.md)
- **Reference Implementation**: [torrust/torrust-demo](https://github.com/torrust/torrust-demo)
  - [compose.yaml](https://github.com/torrust/torrust-demo/blob/main/compose.yaml) - MySQL service configuration
  - [tracker config examples](https://github.com/torrust/torrust-tracker/tree/main/share/container/default/config)
- **Torrust Tracker MySQL Documentation**: [packages/tracker-core/README.md](https://github.com/torrust/torrust-tracker/blob/main/packages/tracker-core/README.md)
- **MySQL Driver Implementation**: [torrust-tracker mysql driver](https://github.com/torrust/torrust-tracker/blob/main/packages/tracker-core/src/databases/driver/mysql.rs)

## Notes

### MySQL Connection String Format

```text
mysql://<user>:<password>@<host>:<port>/<database>
```

Example: `mysql://db_user:db_user_secret_password@mysql:3306/torrust_tracker`

### MySQL Service Hostname

Within docker-compose network, the MySQL service is accessible at hostname `mysql` (the service name).

### Backward Compatibility Strategy

The implementation maintains complete backward compatibility:

1. **Default Behavior**: SQLite is default if no driver specified
2. **No Breaking Changes**: Existing SQLite deployments work unchanged
3. **Opt-In Feature**: MySQL only activated when explicitly configured
4. **Zero Impact**: SQLite path has no awareness of MySQL feature

### Progressive Configuration

The implementation follows the established pattern:

1. **Phase 1**: Hardcoded MySQL credentials in template
2. **Phase 2**: Expose credentials to environment configuration
3. **Phase 3**: Add advanced MySQL options (connection pool size, timeouts, etc.)

This ensures each step is testable and delivers value independently.

### Testing Strategy

Manual testing focuses on two critical paths:

1. **SQLite Path**: Verify no regressions, backward compatibility maintained
2. **MySQL Path**: Verify new feature works end-to-end

Both paths should be tested in the same session to ensure proper isolation.

### Future Enhancements

After this slice is complete, future enhancements could include:

- Exposing MySQL connection pool configuration
- Adding MySQL backup scripts to Ansible playbooks
- Supporting MySQL SSL/TLS connections
- Adding MySQL performance tuning options
- Supporting other database drivers (PostgreSQL)

These are explicitly out of scope for this task to maintain focus on the vertical slice delivering MySQL as a working option.
