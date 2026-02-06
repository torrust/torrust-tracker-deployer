# Decision: Use MySQL Over MariaDB for Database Backend

## Status

Accepted

## Date

2025-07-08

## Context

As part of the migration from SQLite to a production-ready database backend for the Torrust Tracker, we needed to choose between MySQL and MariaDB. Both are popular open-source relational database management systems, with MariaDB being a fork of MySQL that aims to maintain compatibility while offering additional features and performance improvements.

### Key Considerations

1. **Official Torrust Tracker Support**: The Torrust Tracker project officially documents and tests specific database drivers
2. **Compatibility and Reliability**: Ensuring maximum compatibility with the tracker application and minimal integration issues
3. **Community and Documentation**: Availability of support, documentation, and troubleshooting resources
4. **Future Maintenance**: Long-term maintainability and alignment with upstream project decisions
5. **Performance**: Database performance for BitTorrent tracker workloads (primarily CRUD operations)

### Technical Requirements

The Torrust Tracker requires:

- Support for connection pooling via `r2d2_mysql` Rust crate
- MySQL-specific connection string format: `mysql://user:password@host:port/database`
- Compatibility with tracker's database schema and SQL syntax
- Auto-increment primary keys and foreign key constraints
- UTF-8 character encoding support

## Decision

We will use **MySQL 8.0** as the default database backend for the Torrust Tracker deployments instead of MariaDB.

### Rationale

#### 1. Official Torrust Tracker Support

- **Documented Support**: Torrust Tracker documentation explicitly mentions "SQLite and MySQL" support, with no mention of MariaDB
- **Default Configuration**: All official examples, Docker configurations, and test setups use MySQL 8.0
- **Codebase Implementation**: The tracker uses the `r2d2_mysql` crate specifically designed for MySQL, not a generic MariaDB-compatible driver

#### 2. Testing and Validation

- **Integration Tests**: Torrust Tracker's test suite specifically uses MySQL 8.0 containers for database driver testing
- **CI/CD Pipeline**: The official project's continuous integration validates against MySQL, ensuring compatibility
- **Environment Variable**: Tests are controlled via `TORRUST_TRACKER_CORE_RUN_MYSQL_DRIVER_TEST=true`, indicating MySQL-specific testing

#### 3. Community and Ecosystem Alignment

- **BitTorrent Community Standard**: The broader BitTorrent tracker community predominantly uses MySQL
- **Documentation Consistency**: All troubleshooting guides, configuration examples, and community solutions reference MySQL
- **Upstream Alignment**: Following the same database choice as the upstream Torrust Tracker project ensures consistent behavior

#### 4. Technical Implementation Benefits

- **Connection String Compatibility**: MySQL connection strings are guaranteed to work with the tracker's database driver
- **SQL Syntax Alignment**: The tracker's SQL queries are written and tested against MySQL 8.0 specifically
- **Migration Path**: Future updates to Torrust Tracker will be tested against MySQL, ensuring smooth upgrades

#### 5. Reduced Risk Profile

- **Known Configuration**: Using MySQL eliminates uncertainty about MariaDB compatibility edge cases
- **Proven Compatibility**: MySQL 8.0 is the tested and validated database backend for Torrust Tracker
- **Support Availability**: Issues and solutions are more readily available for MySQL configurations

### Implementation

The MySQL 8.0 configuration is implemented as follows:

**Docker Compose Configuration**:

```yaml
mysql:
  image: mysql:8.0
  environment:
    - MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD}
    - MYSQL_DATABASE=${MYSQL_DATABASE}
    - MYSQL_USER=${MYSQL_USER}
    - MYSQL_PASSWORD=${MYSQL_PASSWORD}
  command: --character-set-server=utf8mb4 --collation-server=utf8mb4_unicode_ci
```

**Tracker Configuration**:

```toml
[core.database]
driver = "mysql"
path = "mysql://torrust:password@mysql:3306/torrust_tracker"
```

**Environment Variables**:

```bash
TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER=mysql
TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__PATH=mysql://torrust:password@mysql:3306/torrust_tracker
```

## Consequences

### Positive

- **Maximum Compatibility**: Guaranteed compatibility with current and future Torrust Tracker versions
- **Better Documentation**: All configuration examples and troubleshooting guides apply directly
- **Simplified Maintenance**: No need to test or validate MariaDB-specific compatibility issues
- **Community Support**: Easier to get help from the Torrust community when using the standard database
- **Future-Proof**: Alignment with upstream project decisions reduces migration risks

### Neutral

- **Performance**: Both MySQL and MariaDB would provide similar performance for tracker workloads
- **Resource Usage**: Memory and CPU usage are comparable between the two databases
- **Feature Set**: The tracker uses basic SQL features available in both databases

### Negative

- **Vendor Lock-in**: Choosing MySQL over MariaDB means following Oracle's MySQL development path
- **License Considerations**: MySQL has Oracle's commercial licensing model (though GPL version is used)
- **Missing MariaDB Features**: We won't benefit from MariaDB-specific performance improvements or features

## Alternatives Considered

### MariaDB 10.x

**Pros**:

- Generally faster than MySQL in some benchmarks
- More storage engines and advanced features
- Fully open-source governance model
- Active development and regular releases

**Cons**:

- Not officially supported or tested by Torrust Tracker
- Potential compatibility issues with MySQL-specific driver code
- Less documentation and community support for tracker use cases
- Risk of subtle differences causing issues in production
- May require additional validation and testing

**Conclusion**: While MariaDB offers technical advantages, the benefits don't outweigh the risks and compatibility concerns for this specific use case.

## Related Decisions

- [Database Configuration Structure in Templates](./database-configuration-structure-in-templates.md) - How database configuration is exposed in templates
- [Environment Variable Injection in Docker Compose](./environment-variable-injection-in-docker-compose.md) - How database credentials are injected
- [Bind Mount Standardization](./bind-mount-standardization.md) - Storage strategy for MySQL data persistence

## References

- [Torrust Tracker Documentation](https://docs.rs/torrust-tracker/)
- [Torrust Tracker Database Configuration](https://docs.rs/torrust-tracker-configuration/latest/torrust_tracker_configuration/v2_0_0/database/struct.Database.html)
- [Torrust Tracker MySQL Driver Implementation](https://github.com/torrust/torrust-tracker/blob/main/packages/tracker-core/src/databases/driver/mysql.rs)
- [Torrust Tracker Docker Compose Example](https://github.com/torrust/torrust-tracker/blob/develop/docs/containers.md)
- [Original PoC ADR-003](https://github.com/torrust/torrust-tracker-deploy-bash-poc/blob/main/docs/adr/003-use-mysql-over-mariadb.md) - Source of this decision from proof-of-concept repository
