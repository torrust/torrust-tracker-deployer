# Decision: Grafana Integration Pattern - Enabled by Default with Prometheus Dependency

## Status

Accepted

## Date

2025-12-20

## Context

Following the Prometheus integration (see [prometheus-integration-pattern.md](./prometheus-integration-pattern.md)), we needed to add Grafana as a metrics visualization service. The key design considerations were:

1. **Enablement Strategy**: Should Grafana be mandatory, opt-in, or enabled-by-default like Prometheus?
2. **Service Dependencies**: How should we enforce the Grafana-Prometheus dependency?
3. **Configuration Management**: Should Grafana have separate config files or use environment variables?
4. **Storage Pattern**: Should Grafana use bind mounts or named volumes for data persistence?
5. **Port Exposure**: How should Grafana UI be exposed for user access?

The decision impacts:

- User experience and deployment simplicity
- Validation logic and error messages
- System architecture consistency
- Security posture and network access

## Decision

### 1. Enabled-by-Default with Hard Prometheus Dependency

Grafana is **included by default** in generated environment templates but requires Prometheus to be enabled.

**Implementation**:

```rust
pub struct UserInputs {
    pub prometheus: Option<PrometheusConfig>, // Required if grafana is Some
    pub grafana: Option<GrafanaConfig>,       // Some by default, None to disable
}
```

**Configuration**:

```json
{
  "prometheus": {
    "scrape_interval": 15
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "SecurePassword123!"
  }
}
```

**Validation at Environment Creation**:

```rust
fn validate_grafana_prometheus_dependency(
    grafana: &Option<GrafanaConfig>,
    prometheus: &Option<PrometheusConfig>,
) -> Result<(), ConfigError> {
    match (grafana, prometheus) {
        (Some(_), None) => Err(ConfigError::GrafanaRequiresPrometheus { /* ... */ }),
        _ => Ok(()),
    }
}
```

**Disabling**: Remove the `grafana` section from the environment config. Prometheus can remain enabled independently.

**Rationale**:

- Grafana is useless without a data source - Prometheus is the natural choice
- Hard dependency at validation time prevents invalid configurations
- Users get complete monitoring stack (collection + visualization) by default
- Consistent with Prometheus enabled-by-default pattern
- Follows principle of least surprise (monitoring expected for production)

### 2. Environment Variable Configuration (No Separate Config Files)

Grafana is configured entirely through environment variables, not separate config files.

**Implementation**:

```yaml
# docker-compose.yml
services:
  grafana:
    environment:
      - GF_SECURITY_ADMIN_USER=${GF_SECURITY_ADMIN_USER}
      - GF_SECURITY_ADMIN_PASSWORD=${GF_SECURITY_ADMIN_PASSWORD}
```

```tera
# .env.tera
{% if grafana_config %}
GF_SECURITY_ADMIN_USER='{{ grafana_admin_user }}'
GF_SECURITY_ADMIN_PASSWORD='{{ grafana_admin_password }}'
{% endif %}
```

**Rationale**:

- Consistent with Docker Compose environment variable injection pattern (see [environment-variable-injection-in-docker-compose.md](./environment-variable-injection-in-docker-compose.md))
- Grafana supports comprehensive environment variable configuration
- Simplifies template structure (no separate grafana.ini file)
- Admin credentials are the only required configuration for MVP
- Future automation will handle datasource and dashboard provisioning

**Anti-Pattern Avoided**: Creating separate `grafana.ini` config file that duplicates what environment variables can handle.

### 3. Named Volume for Data Persistence

Grafana uses a named Docker volume, not a bind mount.

**Implementation**:

```yaml
services:
  grafana:
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  grafana_data: {}
```

**Rationale**:

- Standard Grafana practice (official Grafana Docker documentation uses named volumes)
- Named volumes are managed by Docker (automatic creation, cleanup)
- Simpler for users (no host directory permissions issues)
- Stores dashboards, datasources, user preferences persistently
- Different from Prometheus which uses bind mount for direct config access
- Grafana config is via environment variables, not files, so bind mount unnecessary

**Comparison with Prometheus**:

- **Prometheus**: Bind mount (`./prometheus.yml:/etc/prometheus/prometheus.yml`) - Direct access to config file for easy editing
- **Grafana**: Named volume (`grafana_data:/var/lib/grafana`) - Internal storage for user-created content

### 4. External Port Exposure for UI Access

Grafana UI is exposed on host port 3100 for external access.

**Implementation**:

```yaml
services:
  grafana:
    ports:
      - "3100:3000" # Host:Container
```

**Port Choice**: 3100 on host to avoid conflicts with common port 3000 usage (Node.js dev servers, other services).

**Security Considerations**:

- **Docker Bypasses UFW**: Published ports bypass firewall rules entirely (see [DRAFT-docker-ufw-firewall-security-strategy.md](../issues/DRAFT-docker-ufw-firewall-security-strategy.md))
- **Current Exposure**: Port 3100 accessible from any network that can reach the host
- **Acceptable for MVP**: Public exposure acceptable for development/testing environments
- **Future Security**: Reverse proxy with TLS termination (roadmap task 6)

**Rationale**:

- Users need web UI access from their local machines
- Simple port mapping for MVP (no reverse proxy complexity)
- Port 3100 avoids common conflicts
- Security tradeoffs documented and deferred to reverse proxy implementation

### 5. Service Dependencies in Docker Compose

Grafana service uses simple `depends_on` without health checks.

**Implementation**:

```yaml
services:
  grafana:
    depends_on:
      - prometheus
```

**Rationale**:

- Grafana UI remains functional even if Prometheus is temporarily unavailable
- Health check complexity not required for MVP
- Container startup order sufficient (Prometheus starts first)
- Users can access Grafana UI and configure it while Prometheus initializes

### 6. Manual Datasource and Dashboard Configuration (MVP)

Initial implementation does **not** auto-provision Prometheus datasource or import dashboards.

**User Experience**:

1. Grafana starts with default settings
2. User logs in with configured credentials
3. User manually adds Prometheus datasource (`http://prometheus:9090`)
4. User imports dashboards or creates custom ones

**Rationale**:

- Keep MVP scope minimal (prove service integration works)
- Manual setup well-documented in verification guide (see [grafana-verification.md](../e2e-testing/manual/grafana-verification.md))
- Future automation planned for better UX (see Future Work section)
- Sample dashboards available from torrust-demo for manual import

**Future Automation** (planned issue):

- Auto-provision Prometheus datasource during deployment
- Auto-import tracker dashboards (stats.json, metrics.json)
- Provide customizable dashboard templates

## Alternatives Considered

### Alternative 1: Opt-In Grafana (User Must Explicitly Enable)

**Approach**: Grafana not included in default templates, users add section to enable.

**Rejected Because**:

- Inconsistent with Prometheus enabled-by-default pattern
- More friction for users wanting visualization
- Monitoring is best practice - should be included by default
- Opt-out is simpler (just remove section)

### Alternative 2: Separate Config Files (grafana.ini)

**Approach**: Generate separate `grafana.ini` config file like `prometheus.yml`.

**Rejected Because**:

- Adds complexity without benefit for MVP requirements
- Environment variables sufficient for admin credentials
- Future automation will use Grafana provisioning directory, not grafana.ini
- Inconsistent with Docker Compose environment variable injection pattern

### Alternative 3: Mandatory Grafana (Always Included)

**Approach**: Grafana always deployed, no opt-out option.

**Rejected Because**:

- Users may only want Prometheus (programmatic access, custom visualization tools)
- Increases resource usage for minimal deployments
- Reduces deployment flexibility
- Inconsistent with optional service pattern

### Alternative 4: Separate Grafana Provisioning (Independent from Deployment)

**Approach**: Grafana deployed separately after tracker deployment completes.

**Rejected Because**:

- Fragments deployment workflow (multiple commands)
- Harder to ensure service compatibility
- Complicates docker-compose orchestration
- Better to include in single deployment workflow

### Alternative 5: Bind Mount for Grafana Data

**Approach**: Use bind mount like Prometheus instead of named volume.

**Rejected Because**:

- Named volume is Grafana standard practice
- No need for direct host access to Grafana database files
- Simplifies deployment (no host directory permissions issues)
- Grafana config via environment variables, not files

## Consequences

### Positive

1. **Complete Monitoring Stack Out-of-the-Box**:

   - Users get metrics collection (Prometheus) + visualization (Grafana) by default
   - Production-ready monitoring without manual setup
   - Consistent with infrastructure best practices

2. **Clear Dependency Management**:

   - Validation enforces Grafana-Prometheus dependency at creation time
   - Helpful error messages guide users to fix configuration
   - Prevents invalid configurations before deployment

3. **Consistent Configuration Pattern**:

   - All services use environment variable injection pattern
   - Predictable structure for users and maintainers
   - Easy to add future services (Alertmanager, Loki)

4. **Simple Storage Management**:

   - Named volume managed by Docker (no permission issues)
   - Persistent across container restarts
   - Standard Grafana practice

5. **Extensibility**:
   - Manual setup provides foundation for future automation
   - Verification guide documents complete workflow
   - Clear path to auto-provisioning (planned issue)

### Negative

1. **Manual Initial Setup Required**:

   - Users must add Prometheus datasource manually
   - Users must import/create dashboards manually
   - Extra steps before visualization works
   - **Mitigation**: Comprehensive verification guide provided
   - **Future**: Automation planned in follow-up issue

2. **Port Exposure Security Concerns**:

   - Port 3100 publicly accessible (Docker bypasses UFW)
   - No authentication beyond Grafana login (no TLS)
   - Potential security risk for production deployments
   - **Mitigation**: Documented security implications and limitations
   - **Future**: Reverse proxy with TLS (roadmap task 6)

3. **Hard Prometheus Dependency**:

   - Grafana cannot be enabled without Prometheus
   - Limits flexibility for users with alternative data sources
   - **Mitigation**: Prometheus is the natural choice for tracker metrics
   - **Acceptable**: Hard dependency makes sense for this use case

4. **Default Resource Overhead**:

   - Grafana container included by default increases memory/disk usage
   - Users who don't want visualization must manually remove section
   - **Mitigation**: Simple opt-out (remove config section)
   - **Acceptable**: Monitoring is best practice for production

5. **Named Volume Backup Complexity**:
   - Named volumes harder to backup than bind mounts
   - Requires Docker volume commands for backup/restore
   - **Mitigation**: Standard Docker volume management practices
   - **Acceptable**: Grafana dashboards can be exported/imported via UI

### Implementation Maintenance

1. **Template Consistency**:

   - Conditional Grafana service in docker-compose.yml.tera
   - Conditional environment variables in .env.tera
   - Conditional volume declaration
   - Must be kept in sync with environment state

2. **Validation Logic**:

   - Dependency validation called during environment creation
   - Error messages must remain clear and actionable
   - Unit tests cover all validation scenarios

3. **Testing**:
   - E2E tests validate Grafana deployment when enabled
   - Manual verification guide documents complete workflow
   - Unit tests cover GrafanaValidator logic (14 tests)

### Future Work

**Planned Automation** (separate issue):

1. **Auto-Provision Prometheus Datasource**:

   - Create `provisioning/datasources/prometheus.yml` during release
   - Grafana automatically connects to Prometheus on startup
   - Zero-config experience for users

2. **Auto-Import Tracker Dashboards**:

   - Copy `stats.json` and `metrics.json` from torrust-demo
   - Create `provisioning/dashboards/` directory during release
   - Dashboards available immediately after deployment

3. **Customizable Dashboard Templates**:
   - Allow users to provide custom dashboard JSON files
   - Support for dashboard provisioning configuration
   - Template-based dashboard generation

**Related Roadmap Items**:

- Task 6: Reverse proxy implementation with TLS termination
- Task 7: Automated backup and restore procedures
- Task 8: Multi-environment dashboard management

## Related Decisions

- [Prometheus Integration Pattern](./prometheus-integration-pattern.md) - Consistent enabled-by-default approach
- [Environment Variable Injection in Docker Compose](./environment-variable-injection-in-docker-compose.md) - Configuration pattern
- [DRAFT: Docker UFW Firewall Security Strategy](../issues/DRAFT-docker-ufw-firewall-security-strategy.md) - Port exposure security

## References

- [Grafana Docker Documentation](https://grafana.com/docs/grafana/latest/setup-grafana/installation/docker/)
- [Grafana Configuration Environment Variables](https://grafana.com/docs/grafana/latest/setup-grafana/configure-grafana/#override-configuration-with-environment-variables)
- [Grafana Provisioning](https://grafana.com/docs/grafana/latest/administration/provisioning/)
- [Torrust Demo Grafana Setup](https://github.com/torrust/torrust-demo/blob/main/compose.yaml)
- [Sample Dashboards](https://github.com/torrust/torrust-demo/tree/main/share/grafana/dashboards)
- [Manual Verification Guide](../e2e-testing/manual/grafana-verification.md)
