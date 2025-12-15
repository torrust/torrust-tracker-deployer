# Dynamic Tracker Ports in Docker Compose Template

**Issue**: #236
**Parent Epic**: [#216](https://github.com/torrust/torrust-tracker-deployer/issues/216) (Implement ReleaseCommand and RunCommand with vertical slices)
**Related**:

- [#232](https://github.com/torrust/torrust-tracker-deployer/issues/232) - MySQL Slice (establishes docker-compose template foundation)
- [ADR: Environment Variable Injection in Docker Compose](../decisions/environment-variable-injection-in-docker-compose.md)

## Overview

The tracker service ports in the Docker Compose template are currently hardcoded instead of being dynamically configured from the tracker configuration in the environment file. This prevents users from customizing tracker ports during deployment without manually editing the generated `docker-compose.yml` file.

This task makes tracker port configuration dynamic by extracting ports from the tracker configuration (UDP tracker ports, HTTP tracker ports, and HTTP API port) and rendering them into the `docker-compose.yml.tera` template using Tera variables, following the same pattern already used in:

- `templates/tracker/tracker.toml.tera` - Tracker configuration template with dynamic port bindings
- `templates/ansible/variables.yml.tera` - Ansible variables with tracker port lists
- `templates/ansible/configure-firewall.yml` - Firewall configuration using those port lists

## Goals

- [ ] Extract tracker port configuration from environment config (UDP, HTTP, API ports)
- [ ] Add port configuration to `DockerComposeContext` for template rendering
- [ ] Update `docker-compose.yml.tera` template to use Tera variables for tracker ports
- [ ] Maintain port mapping pattern: `host_port:container_port` where both are the same
- [ ] Handle multiple UDP and HTTP tracker instances dynamically
- [ ] Support single HTTP API port configuration
- [ ] Ensure backward compatibility with existing deployments

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure
**Module Path**: `src/infrastructure/templating/docker_compose/template/`
**Pattern**: Template System with Project Generator pattern - extend existing `DockerComposeContext` and renderer

### Module Structure Requirements

- [ ] Follow template system architecture (see [docs/technical/template-system-architecture.md](../technical/template-system-architecture.md))
- [ ] Extend existing `DockerComposeContext` in `wrappers/docker_compose/context.rs`
- [ ] Use existing `DockerComposeRenderer` - no new renderer needed
- [ ] Update `.tera` template with dynamic port variables
- [ ] Follow port extraction pattern from `AnsibleVariablesContext`

### Architectural Constraints

- [ ] Port configuration must be extracted from `TrackerConfig` domain model
- [ ] Port lists must support multiple tracker instances (UDP/HTTP)
- [ ] API port is single instance (one HTTP API per tracker)
- [ ] Port mapping follows Docker convention: `host:container` (typically same value)
- [ ] Must follow existing Tera templating patterns in the codebase

### Anti-Patterns to Avoid

- ❌ Hardcoding default ports in the template (defeats the purpose)
- ❌ Creating duplicate port extraction logic (reuse domain model accessors)
- ❌ Breaking existing template rendering for database configuration
- ❌ Making ports optional when they're required for tracker operation

## Specifications

### Current Hardcoded Implementation

**File**: `templates/docker-compose/docker-compose.yml.tera`

```yaml
services:
  tracker:
    # ... other configuration ...
    ports:
      - 6868:6868/udp
      - 6969:6969/udp
      - 7070:7070
      - 1212:1212
```

**Problem**: These ports are fixed and don't reflect the user's tracker configuration from the environment file.

### Target Dynamic Implementation

**Expected Tera Template Syntax**:

```yaml
services:
  tracker:
    # ... other configuration ...
    ports:
{%- for port in udp_tracker_ports %}
      - {{ port }}:{{ port }}/udp
{%- endfor %}
{%- for port in http_tracker_ports %}
      - {{ port }}:{{ port }}
{%- endfor %}
      - {{ http_api_port }}:{{ http_api_port }}
```

**Rationale**:

- Loops iterate over UDP and HTTP tracker port lists
- Each port uses same value for host and container (`port:port`)
- UDP ports explicitly marked with `/udp` protocol
- HTTP ports and API port use TCP (default, no explicit protocol)
- Whitespace control with `{%-` to prevent empty lines

### Port Extraction from Configuration

**Source**: `TrackerConfig` domain model in `src/domain/tracker/config.rs`

**Port Sources**:

1. **UDP Tracker Ports**: Extract from `tracker_config.udp_trackers[].bind_address`
2. **HTTP Tracker Ports**: Extract from `tracker_config.http_trackers[].bind_address`
3. **HTTP API Port**: Extract from `tracker_config.http_api.bind_address`

**Example Configuration**:

```json
{
  "deployment": {
    "tracker": {
      "udp_trackers": [
        { "bind_address": "0.0.0.0:6868" },
        { "bind_address": "0.0.0.0:6969" }
      ],
      "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
      "http_api": {
        "bind_address": "0.0.0.0:1212"
      }
    }
  }
}
```

**Extracted Ports**:

- UDP: `[6868, 6969]`
- HTTP: `[7070]`
- API: `1212`

### Context Extension

**File**: `src/infrastructure/templating/docker_compose/template/wrappers/docker_compose/context.rs`

**Add to `DockerComposeContext`**:

```rust
/// Context for rendering the docker-compose.yml template
#[derive(Serialize, Debug, Clone)]
pub struct DockerComposeContext {
    /// Database configuration
    pub database: DatabaseConfig,

    /// UDP tracker ports (NEW)
    pub udp_tracker_ports: Vec<u16>,

    /// HTTP tracker ports (NEW)
    pub http_tracker_ports: Vec<u16>,

    /// HTTP API port (NEW)
    pub http_api_port: u16,
}
```

**Constructor Updates**:

Update both `new_sqlite()` and `new_mysql()` constructors to accept port parameters:

```rust
impl DockerComposeContext {
    pub fn new_sqlite(
        udp_tracker_ports: Vec<u16>,
        http_tracker_ports: Vec<u16>,
        http_api_port: u16,
    ) -> Self {
        Self {
            database: DatabaseConfig {
                driver: "sqlite3".to_string(),
                mysql: None,
            },
            udp_tracker_ports,
            http_tracker_ports,
            http_api_port,
        }
    }

    pub fn new_mysql(
        root_password: String,
        database: String,
        user: String,
        password: String,
        port: u16,
        udp_tracker_ports: Vec<u16>,
        http_tracker_ports: Vec<u16>,
        http_api_port: u16,
    ) -> Self {
        Self {
            database: DatabaseConfig {
                driver: "mysql".to_string(),
                mysql: Some(MysqlConfig {
                    root_password,
                    database,
                    user,
                    password,
                    port,
                }),
            },
            udp_tracker_ports,
            http_tracker_ports,
            http_api_port,
        }
    }
}
```

### Port Extraction Helper

**Pattern Reference**: Follow the same pattern as `AnsibleVariablesContext::extract_tracker_ports()`

**Location**: Can be a private helper method in `DockerComposeContext` or reuse existing extraction logic

**Implementation Example**:

```rust
/// Extract port numbers from tracker configuration
fn extract_tracker_ports(tracker_config: &TrackerConfig) -> (Vec<u16>, Vec<u16>, u16) {
    // Extract UDP tracker ports
    let udp_ports: Vec<u16> = tracker_config
        .udp_trackers
        .iter()
        .map(|tracker| tracker.bind_address.port())
        .collect();

    // Extract HTTP tracker ports
    let http_ports: Vec<u16> = tracker_config
        .http_trackers
        .iter()
        .map(|tracker| tracker.bind_address.port())
        .collect();

    // Extract HTTP API port
    let api_port = tracker_config.http_api.bind_address.port();

    (udp_ports, http_ports, api_port)
}
```

### Integration Point

**File**: `src/application/steps/rendering/docker_compose_templates.rs`

**Current Context Creation** (excerpt):

```rust
let (env_context, docker_compose_context) = match database_config {
    DatabaseConfig::Sqlite { .. } => {
        let env_context = EnvContext::new(admin_token);
        let docker_compose_context = DockerComposeContext::new_sqlite();
        (env_context, docker_compose_context)
    }
    DatabaseConfig::Mysql { ... } => {
        // MySQL variant...
    }
};
```

**Updated Context Creation** (needs port extraction):

```rust
// Extract ports from tracker configuration
let tracker_config = &self.environment.context().user_inputs.tracker;
let (udp_ports, http_ports, api_port) = extract_ports_from_tracker_config(tracker_config);

let (env_context, docker_compose_context) = match database_config {
    DatabaseConfig::Sqlite { .. } => {
        let env_context = EnvContext::new(admin_token);
        let docker_compose_context = DockerComposeContext::new_sqlite(
            udp_ports,
            http_ports,
            api_port,
        );
        (env_context, docker_compose_context)
    }
    DatabaseConfig::Mysql { ... } => {
        // Similar pattern with ports
    }
};
```

## Implementation Plan

### Phase 1: Extend DockerComposeContext (2 hours)

- [ ] Add port fields to `DockerComposeContext` struct
  - `udp_tracker_ports: Vec<u16>`
  - `http_tracker_ports: Vec<u16>`
  - `http_api_port: u16`
- [ ] Update `new_sqlite()` constructor to accept port parameters
- [ ] Update `new_mysql()` constructor to accept port parameters
- [ ] Add getter methods for port fields (if needed for tests)
- [ ] Update existing unit tests in `context.rs` to provide port values
- [ ] Verify tests pass with new constructor signatures

### Phase 2: Port Extraction Logic (1.5 hours)

- [ ] Create helper function to extract ports from `TrackerConfig`
  - Follow pattern from `AnsibleVariablesContext::extract_tracker_ports()`
  - Extract UDP tracker ports from `udp_trackers[].bind_address`
  - Extract HTTP tracker ports from `http_trackers[].bind_address`
  - Extract HTTP API port from `http_api.bind_address`
- [ ] Write unit tests for port extraction helper
  - Test with single UDP/HTTP tracker
  - Test with multiple UDP/HTTP trackers
  - Test with default ports (6868, 6969, 7070, 1212)
  - Test with custom ports

### Phase 3: Update Template Integration (1.5 hours)

- [ ] Update `RenderDockerComposeTemplatesStep` in `docker_compose_templates.rs`
  - Extract ports from environment's tracker config
  - Pass ports to `DockerComposeContext::new_sqlite()`
  - Pass ports to `DockerComposeContext::new_mysql()`
- [ ] Update all call sites creating `DockerComposeContext`
  - Check for any test code creating context instances
  - Update with appropriate port values
- [ ] Verify step tests still pass

### Phase 4: Update Tera Template (1 hour)

- [ ] Update `templates/docker-compose/docker-compose.yml.tera`
  - Replace hardcoded UDP ports with Tera loop over `udp_tracker_ports`
  - Replace hardcoded HTTP ports with Tera loop over `http_tracker_ports`
  - Replace hardcoded API port with `http_api_port` variable
  - Use `{%-` for whitespace control to prevent empty lines
  - Maintain `/udp` protocol suffix for UDP ports
- [ ] Add template comment explaining the dynamic port configuration

### Phase 5: Testing and Verification (2 hours)

- [ ] Run unit tests: `cargo test`
- [ ] Test template rendering with default ports (6868, 6969, 7070, 1212)
- [ ] Test template rendering with custom ports
- [ ] Test with multiple UDP tracker instances
- [ ] Test with multiple HTTP tracker instances
- [ ] Verify rendered `docker-compose.yml` syntax is valid
- [ ] Manual E2E test: Deploy tracker with custom ports (see [docs/e2e-testing/manual-testing.md](../e2e-testing/manual-testing.md))
  - Create environment config with non-default ports for all services:
    - UDP tracker ports (e.g., 7868, 7969 instead of 6868, 6969)
    - HTTP tracker port (e.g., 8070 instead of 7070)
    - HTTP API port (e.g., 2212 instead of 1212)
  - Run through full deployment workflow
  - Verify ports in rendered docker-compose.yml match config
  - Verify tracker starts successfully with custom ports
  - Verify all services are accessible on the custom ports

### Phase 6: Documentation (30 minutes)

- [ ] Add comment in `docker-compose.yml.tera` explaining port configuration
- [ ] Update any relevant documentation about port customization
- [ ] Note that ports can be customized via environment configuration

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Implementation Criteria**:

- [ ] `DockerComposeContext` includes port configuration fields
- [ ] Port extraction logic correctly parses `TrackerConfig` domain model
- [ ] Both SQLite and MySQL context constructors accept port parameters
- [ ] Template uses Tera loops to render UDP and HTTP tracker ports dynamically
- [ ] Template uses Tera variable for HTTP API port
- [ ] Rendered docker-compose.yml has valid YAML syntax
- [ ] Port mappings follow `host:container` pattern with matching values
- [ ] UDP ports include `/udp` protocol suffix

**Testing Criteria**:

- [ ] All existing unit tests pass with updated constructors
- [ ] New unit tests cover port extraction from tracker config
- [ ] Template rendering tests verify correct port output
- [ ] Manual E2E test confirms deployment works with custom ports
- [ ] Rendered docker-compose.yml matches expected format

**Backward Compatibility**:

- [ ] Default tracker configuration (6868, 6969, 7070, 1212) still works
- [ ] Existing deployments with standard ports are unaffected
- [ ] Template continues to work with both SQLite and MySQL drivers

## Related Documentation

- [Template System Architecture](../technical/template-system-architecture.md) - Understanding the Project Generator pattern
- [Tera Template Variable Syntax](../contributing/templates.md) - Correct Tera syntax and common pitfalls
- [ADR: Environment Variable Injection in Docker Compose](../decisions/environment-variable-injection-in-docker-compose.md) - Why we use .env for runtime config
- [Codebase Architecture](../codebase-architecture.md) - DDD layer responsibilities
- [Ansible Variables Context](../../src/infrastructure/templating/ansible/template/wrappers/variables/context.rs) - Similar port extraction pattern

## Notes

### Why This Was Left Incomplete

This issue documents work that was not completed during the initial docker-compose template implementation (#232). At that time, the focus was on establishing the template system foundation and MySQL support. Port configuration was hardcoded as a known limitation to be addressed later.

### Design Decisions

**Port Mapping Pattern**: We use `host:container` with the same port value because:

- Tracker bind addresses in config already specify the desired ports
- Simpler configuration - no port translation needed
- Follows principle of least surprise for system administrators
- Consistent with how Ansible firewall rules are configured

**No Port Validation**: We don't validate port ranges or conflicts at the context level because:

- `TrackerConfig` domain model already validates `SocketAddr` on deserialization
- Docker Compose will fail with clear error if ports are invalid or in use
- Keeps context layer focused on data transformation, not validation

**API Port as Single Value**: Unlike UDP/HTTP trackers which support multiple instances, the HTTP API is always a single instance in the tracker architecture, so we use `u16` instead of `Vec<u16>`.

### Analysis of Other Hardcoded Values

During specification, we reviewed the entire `docker-compose.yml.tera` template to identify what else should be made dynamic. Current hardcoded values:

**Tracker Service:**

- Image tag: `torrust/tracker:develop` - Could be configurable but stable tags/versions are typically chosen during deployment and rarely changed
- Container name: `tracker` - Naming convention, no user customization needed
- USER_ID: `1000` - Standard non-root user ID, operational default
- Network name: `backend_network` - Internal convention, no need to expose
- Volume paths: `./storage/tracker/*` - Operational concern, sysadmin can modify post-deployment
- Logging config: `max-size`, `max-file` - Operational tuning, not deployment config
- Restart policy: `unless-stopped` - Operational default

**MySQL Service:**

- Image tag: `mysql:8.0` - Version selection, stable across deployment lifecycle
- MySQL port: `3306:3306` - Standard MySQL port, rarely needs customization
- Healthcheck intervals/retries - Operational tuning parameters

**Decision**: Only tracker ports warrant deployment-time configuration because:

1. They're part of the tracker's core application configuration
2. Users explicitly configure them in environment.json
3. They must match the tracker.toml bind addresses
4. They're already dynamically configured in Ansible firewall rules
5. Different deployments commonly use different ports (dev/staging/prod)

Other values follow the principle from the Environment Variable Injection ADR: they're either stable deployment-time choices (image versions) or operational parameters that sysadmins manage post-deployment by editing files and restarting services.

### Future Enhancements

- Consider adding port validation to fail fast before template rendering
- Consider supporting different host vs container port mappings if use cases emerge
- Could extract port lists to a shared helper if more templates need this logic
- Image tag configuration could be added if users request the ability to deploy specific tracker versions
