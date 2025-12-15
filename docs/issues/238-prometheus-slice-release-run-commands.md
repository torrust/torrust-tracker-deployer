# Prometheus Slice - Release and Run Commands

**Issue**: [#238](https://github.com/torrust/torrust-tracker-deployer/issues/238)
**Parent Epic**: [#216](https://github.com/torrust/torrust-tracker-deployer/issues/216) (Implement ReleaseCommand and RunCommand with vertical slices)

## Overview

This task adds Prometheus as a metrics collection service for the Torrust Tracker deployment. It extends the docker-compose stack with a Prometheus service that scrapes metrics and stats from the tracker's HTTP API. The service is **enabled by default** in generated environment templates, but can be disabled by removing the Prometheus configuration section from the environment config file.

## Goals

- [ ] Add Prometheus service conditionally to docker-compose stack (only when present in environment config)
- [ ] Create Prometheus configuration template with tracker metrics endpoints
- [ ] Extend environment configuration schema to include Prometheus monitoring section
- [ ] Configure service dependency - Prometheus depends on tracker service
- [ ] Include Prometheus in generated environment templates by default (enabled by default)
- [ ] Allow users to disable Prometheus by removing its configuration section
- [ ] Deploy and verify Prometheus collects metrics from tracker

## Progress

- ✅ **Phase 1**: Template Structure & Data Flow Design (commit: 2ca0fa9)

  - Created `PrometheusContext` struct with `scrape_interval`, `api_token`, `api_port` fields
  - Implemented module structure following existing patterns
  - Added comprehensive unit tests (5 tests)
  - Created `templates/prometheus/prometheus.yml.tera` template

- ✅ **Phase 2**: Environment Configuration (commit: 92aab59)

  - Created `PrometheusConfig` domain struct in `src/domain/prometheus/`
  - Added optional `prometheus` field to `UserInputs` (enabled by default)
  - Implemented comprehensive unit tests (5 tests)
  - Updated all constructors and test fixtures

- ✅ **Phase 3**: Prometheus Template Renderer (commit: 731eaf4)

  - Created `PrometheusConfigRenderer` to load and render `prometheus.yml.tera`
  - Implemented `PrometheusTemplate` wrapper for Tera integration
  - Created `PrometheusProjectGenerator` to orchestrate rendering workflow
  - Implemented context extraction from `PrometheusConfig` and `TrackerConfig`
  - Added 12 comprehensive unit tests with full coverage
  - All linters passing

- ✅ **Phase 4**: Docker Compose Integration (commit: 22790de)

  - Added `prometheus_config: Option<PrometheusConfig>` field to `DockerComposeContext`
  - Implemented `with_prometheus()` method for context builder pattern
  - Added conditional Prometheus service to `docker-compose.yml.tera` template
  - Prometheus service uses bind mount: `./storage/prometheus/etc:/etc/prometheus:Z`
  - Added 4 comprehensive unit tests for Prometheus service rendering
  - All linters passing

- ✅ **Phase 5**: Release Command Integration (commit: f20d45c)

  - **FIXED**: Moved Prometheus template rendering from docker-compose step to independent step in release handler
  - Created `RenderPrometheusTemplatesStep` to render Prometheus templates
  - Added `render_prometheus_templates()` method to `ReleaseCommandHandler`
  - Prometheus templates now rendered independently at Step 5 (after tracker templates, before docker-compose)
  - Docker Compose step only adds Prometheus config to context (no template rendering)
  - Added `RenderPrometheusTemplates` variant to `ReleaseStep` enum
  - Extended `EnvironmentTestBuilder` with `with_prometheus_config()` method
  - All linters passing, all tests passing (1507 tests)
  - **Architectural Principle**: Each service renders its templates independently in the release handler

- ✅ **Phase 6**: Ansible Deployment (commit: pending)

  - Created Ansible playbooks:
    - `templates/ansible/create-prometheus-storage.yml` - Creates `/opt/torrust/storage/prometheus/etc` directory
    - `templates/ansible/deploy-prometheus-config.yml` - Deploys `prometheus.yml` configuration file with verification
  - Created Rust application steps:
    - `CreatePrometheusStorageStep` - Executes create-prometheus-storage playbook
    - `DeployPrometheusConfigStep` - Executes deploy-prometheus-config playbook
  - Registered playbooks in `AnsibleProjectGenerator` (16 total playbooks)
  - Registered steps in `application/steps/application/mod.rs`
  - Updated release handler with new methods:
    - `create_prometheus_storage()` - Creates Prometheus storage directories (Step 5)
    - `deploy_prometheus_config_to_remote()` - Deploys Prometheus config (Step 7)
  - Added new `ReleaseStep` enum variants:
    - `CreatePrometheusStorage`
    - `DeployPrometheusConfigToRemote`
  - Added error handling:
    - `PrometheusStorageCreation` error variant with help text
    - Proper trace formatting and error classification
  - Updated workflow to 9 steps total:
    - Step 5: Create Prometheus storage (if enabled)
    - Step 6: Render Prometheus templates (if enabled)
    - Step 7: Deploy Prometheus config (if enabled)
    - Step 8: Render Docker Compose templates
    - Step 9: Deploy compose files
  - All linters passing, all tests passing (1507 tests)
  - **Pattern**: Independent Prometheus deployment following tracker pattern

- ✅ **Phase 6**: Ansible Deployment (commit: 9c1b91a)
- 🚧 **Phase 7**: Testing & Verification (in progress)
- ⏳ **Phase 8**: Documentation (pending)

## 🏗️ Architecture Requirements

**DDD Layers**: Infrastructure + Domain
**Module Paths**:

- `src/infrastructure/templating/prometheus/` - Prometheus configuration template system (NEW)
- `src/infrastructure/templating/docker_compose/` - Docker Compose template rendering with Prometheus service
- `src/domain/prometheus/` - Prometheus configuration domain types (NEW)
- `src/application/command_handlers/create/config/prometheus/` - Prometheus config creation handlers (NEW)

**Pattern**: Template System with Project Generator pattern + Configuration-driven service selection

### Module Structure Requirements

- [ ] Follow template system architecture (see [docs/technical/template-system-architecture.md](../technical/template-system-architecture.md))
- [ ] Create new Prometheus template module following existing patterns (tracker, docker-compose)
- [ ] Use Project Generator pattern for Prometheus templates
- [ ] Register Prometheus configuration template in renderer
- [ ] Use `.tera` extension for dynamic templates
- [ ] Environment config drives Prometheus enablement

### Architectural Constraints

- [ ] Prometheus service is included by default in generated environment templates
- [ ] Only included in docker-compose when Prometheus section present in environment config
- [ ] Service can be disabled by removing the monitoring.prometheus section from config
- [ ] Prometheus depends on tracker service (starts after tracker container starts, no health check)
- [ ] Metrics API token and port read from tracker HTTP API configuration (`tracker.http_api.admin_token` and `tracker.http_api.bind_address`)
- [ ] Prometheus configuration is dynamic (uses Tera templating)
- [x] **Independent Template Rendering**: Each service renders its templates independently in the release handler
  - Prometheus templates rendered by dedicated `RenderPrometheusTemplatesStep` in release handler
  - Tracker templates rendered by dedicated `RenderTrackerTemplatesStep` in release handler
  - Docker Compose templates rendered by dedicated `RenderDockerComposeTemplatesStep` in release handler
  - **Rationale**: Docker Compose templates are NOT the "master" templates - they only define service orchestration
  - **Source of Truth**: The environment configuration determines which services are enabled
  - **Example**: MySQL service has docker-compose configuration but no separate config files (service-specific)

### Anti-Patterns to Avoid

- ❌ Making Prometheus mandatory for all deployments
- ❌ Hardcoding API tokens in templates
- ❌ Starting Prometheus before tracker is ready
- ❌ Duplicating tracker endpoint configuration
- ❌ Mixing metrics collection logic with other services
- ❌ **Rendering service templates from within docker-compose template rendering** (CRITICAL)

  - Docker Compose step should ONLY render docker-compose files
  - Each service's templates should be rendered independently in the release handler
  - The handler orchestrates all template rendering steps based on environment config

- ❌ Making Prometheus mandatory for all deployments
- ❌ Hardcoding API tokens in templates
- ❌ Starting Prometheus before tracker is ready
- ❌ Duplicating tracker endpoint configuration
- ❌ Mixing metrics collection logic with other services

## Implementation Strategy

The implementation follows an **enabled-by-default, opt-out approach** where Prometheus is included in generated templates but can be disabled by removing its configuration.

### Key Principles

1. **Enabled by Default**: Prometheus included in generated environment templates
2. **Opt-Out Available**: Users can remove Prometheus section to disable it
3. **Configuration-Driven**: Service presence controlled by config section existence
4. **Service Isolation**: Prometheus as independent docker-compose service
5. **Service Dependencies**: Proper startup ordering with `depends_on`
6. **Progressive Configuration**: Start with basic metrics, then add advanced options
7. **Manual Verification**: Test both with and without Prometheus at each step

## Specifications

### Prometheus Service Enablement

**Environment Configuration Addition** (top-level, alongside `tracker`):

```json
{
  "environment": { "name": "my-deployment" },
  "provider": { ... },
  "ssh_credentials": { ... },
  "tracker": { ... },
  "prometheus": {
    "scrape_interval": 15
  }
}
```

**Default Behavior in Generated Templates**:

- The `prometheus` section is **included by default** when generating environment templates
- If the section is **present** in the environment config → Prometheus service is included in docker-compose
- If the section is **removed/absent** from the environment config → Prometheus service is NOT included

**Service Detection**:

- Presence of `prometheus` section (regardless of content) → Service enabled
- Absence of `prometheus` section → Service disabled

**Configuration Model**: Uses `Option<PrometheusConfig>` in Rust domain model:

- `Some(PrometheusConfig { scrape_interval: 15 })` → Prometheus enabled (default in generated templates)
- `None` (section removed from config) → Prometheus disabled

### Prometheus Service Configuration

**Docker Compose Service**: Add Prometheus container conditionally to stack (only when `prometheus_config` is present in template context)

**Template Location**: `templates/docker-compose/docker-compose.yml.tera` (uses Tera conditionals to include/exclude Prometheus service)

**Configuration** (from torrust-demo reference):

```yaml
prometheus:
  image: prom/prometheus:v3.0.1
  container_name: prometheus
  tty: true
  restart: unless-stopped
  networks:
    - backend_network
  ports:
    - "9090:9090"
  volumes:
    - ./storage/prometheus/etc:/etc/prometheus:Z
  logging:
    options:
      max-size: "10m"
      max-file: "10"
  depends_on:
    - tracker
```

**Image**: `prom/prometheus:v3.0.1` (latest stable version as of December 2025)

**Ports**:

- Internal: `9090` (Prometheus UI and API)
- Exposed to host: `9090:9090` (allows access to Prometheus UI from host)
- **Note**: This port should not be exposed to the internet in production

**Volumes**:

- `./storage/prometheus/etc:/etc/prometheus:Z` - Mounts entire Prometheus config directory
  - Contains `prometheus.yml` configuration file
  - `:Z` flag: SELinux relabeling for proper permissions (required on SELinux-enabled systems)

**Network**: `backend_network` (shared with tracker service)

**TTY**: `tty: true` - Allocates pseudo-TTY for container (useful for interactive debugging)

**Service Dependencies**:

- **Depends on**: `tracker` service (simple dependency, no health check)
- **Rationale**: Prometheus will start after tracker container starts, but doesn't wait for tracker to be fully healthy. Prometheus can handle temporary unavailability with scrape retries.

### Prometheus Configuration File

**Template Location**: `templates/prometheus/prometheus.yml.tera`

**Directory Structure**: Configuration will be rendered to `./storage/prometheus/etc/` directory which gets mounted into the container.

**Configuration Structure**:

```yaml
global:
  scrape_interval: {{ scrape_interval }}s # How often to scrape metrics

scrape_configs:
  - job_name: "tracker_stats"
    metrics_path: "/api/v1/stats"
    params:
      token: ["{{ api_token }}"]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:{{ metrics_port }}"]

  - job_name: "tracker_metrics"
    metrics_path: "/api/v1/metrics"
    params:
      token: ["{{ api_token }}"]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:{{ metrics_port }}"]
```

**Template Variables**:

- `{{ scrape_interval }}` - How often to scrape metrics (from `monitoring.prometheus.scrape_interval`, default: 15 seconds)
- `{{ api_token }}` - Tracker HTTP API admin token (from `tracker.http_api.admin_token`, required for metrics access)
- `{{ metrics_port }}` - Tracker HTTP API port (parsed from `tracker.http_api.bind_address`, e.g., 1212 from "0.0.0.0:1212")

**Job Descriptions**:

1. **tracker_stats**: Scrapes aggregate statistics from `/api/v1/stats` endpoint
   - Provides high-level tracker statistics
   - Includes torrents count, peers count, completed downloads
2. **tracker_metrics**: Scrapes detailed metrics from `/api/v1/metrics` endpoint
   - Provides detailed operational metrics
   - Includes request rates, response times, error rates

### Environment Configuration Schema Extensions

**Add to Domain Layer** (`src/domain/prometheus/`):

```rust
// New file: src/domain/prometheus/mod.rs
pub mod config;
pub use config::PrometheusConfig;

// New file: src/domain/prometheus/config.rs
/// Prometheus metrics collection configuration
///
/// Configures how Prometheus scrapes metrics from the tracker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrometheusConfig {
    /// Scrape interval in seconds
    pub scrape_interval: u32,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self { scrape_interval: 15 }
    }
}
```

**Add to Environment User Inputs** (`src/domain/environment/user_inputs.rs` or similar):

The environment's user inputs struct should have a top-level optional `prometheus` field:

```rust
pub struct UserInputs {
    pub provider: ProviderConfig,
    pub ssh_credentials: SshCredentials,
    pub tracker: TrackerConfig,
    /// Prometheus metrics collection (optional third-party service)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus: Option<PrometheusConfig>,
}
```

**JSON Schema Addition** (`schemas/environment-config.json`):

```json
{
  "prometheus": {
    "type": "object",
    "description": "Prometheus metrics collection service configuration. Remove this section to disable Prometheus.",
    "properties": {
      "scrape_interval": {
        "type": "integer",
        "description": "How often to scrape metrics from tracker (in seconds). Minimum 5s to avoid overwhelming the tracker.",
        "default": 15,
        "minimum": 5,
        "maximum": 300
      }
    }
  }
}
```

**Template Generation**: When generating environment templates with `create environment --template`, include the `prometheus` section by default at the top level (alongside `tracker`).

### Template File Organization

```text
templates/
├── prometheus/
│   └── prometheus.yml.tera    # Prometheus configuration template
├── docker-compose/
│   ├── docker-compose.yml.tera
│   └── .env.tera
└── tracker/
    └── tracker.toml.tera

# Build output structure:
build/
└── {environment-name}/
    └── storage/
        └── prometheus/
            └── etc/
                └── prometheus.yml   # Rendered from template
```

**Docker Compose Template** (conditional rendering):

```yaml
{% if prometheus_config %}
  prometheus:
    image: prom/prometheus:v3.0.1
    container_name: prometheus
    tty: true
    restart: unless-stopped
    networks:
      - backend_network
    ports:
      - "9090:9090" # This port should not be exposed to the internet
    volumes:
      - ./storage/prometheus/etc:/etc/prometheus:Z
    logging:
      options:
        max-size: "10m"
        max-file: "10"
    depends_on:
      - tracker
{% endif %}
```

**Conditional Volume Declaration**:

```yaml
volumes:
  tracker_data: {}
{% if database_driver == "mysql" %}
  mysql_data: {}
{% endif %}
```

**Note**: No additional named volumes needed for Prometheus. The `./storage/prometheus/etc` directory is a bind mount to the host filesystem, so Prometheus data persists in the storage directory alongside tracker and MySQL data.

### Ansible Deployment Configuration

**Operations**:

1. Create Prometheus configuration directory on VM: `/opt/torrust/storage/prometheus/etc/`
2. Copy rendered `prometheus.yml` to VM
3. Set appropriate file permissions

**Example Ansible Tasks**:

```yaml
- name: Create Prometheus configuration directory
  ansible.builtin.file:
    path: /opt/torrust/storage/prometheus/etc
    state: directory
    mode: "0755"
  when: prometheus_config is defined

- name: Copy Prometheus configuration
  ansible.builtin.copy:
    src: "{{ build_dir }}/storage/prometheus/etc/prometheus.yml"
    dest: /opt/torrust/storage/prometheus/etc/prometheus.yml
    mode: "0644"
  when: prometheus_config is defined
```

## Implementation Plan

> **Important Workflow**: After completing each phase:
>
> 1. Run `./scripts/pre-commit.sh` to verify all checks pass
> 2. Commit your changes with a descriptive message following the [commit conventions](../contributing/commit-process.md)
> 3. **STOP and wait for feedback/approval before proceeding to the next phase**
>
> This ensures incremental progress is saved, issues are caught early, and each phase is reviewed before moving forward.

### Phase 1: Template Structure & Data Flow Design (1 hour)

- [ ] Design `PrometheusContext` struct with fields: `scrape_interval`, `api_token`, `metrics_port`
- [ ] Document data flow: Environment Config → Application Layer → PrometheusContext
- [ ] Create `templates/prometheus/` directory structure
- [ ] Create initial `prometheus.yml.tera` template with placeholder variables
- [ ] Verify template variables match context struct fields

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit changes, and **WAIT FOR APPROVAL** before Phase 2.

### Phase 2: Environment Configuration (1-2 hours)

- [ ] Create new domain module `src/domain/prometheus/` with `PrometheusConfig` struct
- [ ] Add `prometheus: Option<PrometheusConfig>` to environment's `UserInputs` struct (top-level, alongside tracker)
- [ ] Update JSON schema with Prometheus configuration (top-level)
- [ ] Ensure generated templates include Prometheus section by default (at top level)
- [ ] Add unit tests for Prometheus configuration serialization/deserialization (with and without section)

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit changes, and **WAIT FOR APPROVAL** before Phase 3.

### Phase 3: Prometheus Template Renderer (2 hours)

**Why Phase 3**: Must create the Prometheus renderer BEFORE docker-compose integration can use it.

- [ ] Create `PrometheusProjectGenerator` following existing patterns (tracker, docker-compose)
- [ ] Implement `generate()` method to render `prometheus.yml` from template
- [ ] Extract `api_token` and `metrics_port` from tracker HTTP API config in context building
- [ ] Register Prometheus templates in project generator
- [ ] Add comprehensive unit tests for renderer (with different scrape intervals, tokens, ports)

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit changes, and **WAIT FOR APPROVAL** before Phase 4.

### Phase 4: Docker Compose Integration (2-3 hours)

**Why Phase 4**: Now we can integrate with the existing Prometheus renderer from Phase 3.

- [ ] Update `DockerComposeContext` with `prometheus_config: Option<PrometheusConfig>` field
- [ ] Add conditional Prometheus service to `docker-compose.yml.tera` (check for `prometheus_config` presence)
- [ ] Use bind mount volume: `./storage/prometheus/etc:/etc/prometheus:Z` (no named volume needed)
- [ ] Update docker-compose template renderer to handle Prometheus context
- [ ] Add unit tests for Prometheus service rendering (with and without Prometheus section)

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit changes, and **WAIT FOR APPROVAL** before Phase 5.

### Phase 5: Release Command Integration (1 hour)

**Why Phase 5**: Orchestrates both renderers (docker-compose + prometheus) created in previous phases.

- [ ] Update `RenderTemplatesStep` to call Prometheus renderer when config present
- [ ] Ensure Prometheus templates rendered to `build/{env}/storage/prometheus/etc/` directory
- [ ] Verify build directory structure includes Prometheus configuration
- [ ] Test release command with Prometheus enabled and disabled

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit changes, and **WAIT FOR APPROVAL** before Phase 6.

### Phase 6: Ansible Deployment (1 hour)

- [ ] Extend release playbook with Prometheus configuration tasks
- [ ] Add conditional directory creation for Prometheus
- [ ] Add conditional file copy for prometheus.yml
- [ ] Test Ansible playbook with Prometheus enabled/disabled

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit changes, and **WAIT FOR APPROVAL** before Phase 7.

### Phase 7: Testing & Verification (2-3 hours)

- [ ] Add E2E test for deployment with Prometheus enabled (default behavior)
- [ ] Add E2E test for deployment with Prometheus disabled (section removed from config)
- [ ] Manual verification: Deploy with Prometheus enabled and verify metrics collection
- [ ] Manual verification: Deploy without Prometheus section and verify no Prometheus service
- [ ] Verify Prometheus UI accessible at `http://<vm-ip>:9090`
- [ ] Verify Prometheus scrapes metrics from tracker endpoints
- [ ] Update manual testing documentation

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit changes, and **WAIT FOR APPROVAL** before Phase 8.

### Phase 8: Documentation (1 hour)

- [ ] Create ADR for Prometheus integration pattern
- [ ] Update user guide with Prometheus configuration examples
- [ ] Document Prometheus UI access and basic usage
- [ ] Add Prometheus to architecture diagrams
- [ ] Update AGENTS.md if needed

**Checkpoint**: Run `./scripts/pre-commit.sh`, commit final changes, and mark issue as complete.

**Total Estimated Time**: 13-17 hours

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Template System**:

- [ ] Prometheus configuration template created in `templates/prometheus/`
- [ ] Template uses `.tera` extension and renders with context variables
- [ ] PrometheusContext struct created with all required fields
- [ ] Template validation logic implemented

**Environment Configuration**:

- [ ] Environment schema extended with `monitoring.prometheus: Option<PrometheusConfig>` field
- [ ] JSON schema updated and validated
- [ ] Prometheus section included by default in generated environment templates
- [ ] Configuration with Prometheus section enables service
- [ ] Configuration without Prometheus section disables service
- [ ] Configuration serialization/deserialization tested (both cases)

**Docker Compose Integration**:

- [ ] Prometheus service conditionally rendered based on `prometheus_config` presence
- [ ] Service includes proper dependencies (`depends_on: - tracker`)
- [ ] Service includes `tty: true` flag
- [ ] Bind mount volume configured: `./storage/prometheus/etc:/etc/prometheus:Z`
- [ ] Network configuration correct (backend_network)
- [ ] Port 9090 exposed with security comment
- [ ] Template renders correctly when Prometheus section present
- [ ] Template renders correctly when Prometheus section absent

**Deployment**:

- [ ] Release command copies Prometheus configuration when enabled
- [ ] Ansible playbook deploys Prometheus configuration conditionally
- [ ] File permissions and ownership correct on VM

**Backward Compatibility**:

- [ ] Existing environment configs without Prometheus section continue to work (Prometheus disabled)
- [ ] New generated templates include Prometheus by default
- [ ] Users can remove Prometheus section from config to disable it
- [ ] No breaking changes to existing commands or configuration

**Testing**:

- [ ] Unit tests for Prometheus template rendering
- [ ] Unit tests for configuration parsing (with and without Prometheus section)
- [ ] E2E test with Prometheus section removed (disabled)
- [ ] E2E test with Prometheus section present (enabled - default)
- [ ] Manual verification: Generated templates include Prometheus by default
- [ ] Manual verification: Prometheus UI accessible when enabled
- [ ] Manual verification: Metrics scraped successfully when enabled
- [ ] Manual verification: No Prometheus service when section removed

**Documentation**:

- [ ] ADR created documenting Prometheus integration
- [ ] User guide updated with configuration examples
- [ ] Manual testing guide includes Prometheus verification steps

## Related Documentation

- [Template System Architecture](../technical/template-system-architecture.md)
- [DDD Layer Placement](../contributing/ddd-layer-placement.md)
- [Module Organization](../contributing/module-organization.md)
- [Error Handling](../contributing/error-handling.md)
- [Output Handling](../contributing/output-handling.md)
- [MySQL Slice Implementation](232-mysql-slice-release-run-commands.md) - Similar pattern
- [Tracker Slice Implementation](220-tracker-slice-release-run-commands.md) - Service integration pattern

## Reference Implementation

The [torrust-demo](https://github.com/torrust/torrust-demo) repository contains the reference Prometheus configuration:

- [compose.yaml](https://github.com/torrust/torrust-demo/blob/main/compose.yaml#L126-L147) - Prometheus service configuration
- [prometheus.yml](https://github.com/torrust/torrust-demo/blob/main/share/container/default/config/prometheus.yml) - Prometheus scrape configuration

These serve as the basis for the template implementation.

## Notes

### Prometheus Version

Using `prom/prometheus:v3.0.1` (December 2025 latest stable). This provides:

- Improved query performance
- Enhanced remote write capabilities
- Better resource efficiency

### Service Dependency Strategy

Prometheus uses simple `depends_on: - tracker` without health checks:

- **Rationale**: Prometheus can handle temporary unavailability and will retry scrapes automatically
- **Benefit**: Simpler configuration, faster stack startup (no health check complexity)
- **Consideration**: Initial scrapes may fail until tracker is ready (expected behavior - Prometheus will retry)

### Storage Strategy

Uses bind mount (`./storage/prometheus/etc:/etc/prometheus:Z`) instead of named volume:

- **Consistency**: Matches tracker and other services using `./storage/` directory structure
- **Accessibility**: Configuration files easily accessible on host filesystem
- **SELinux**: `:Z` flag handles SELinux relabeling automatically on enabled systems
- **No Named Volume**: Prometheus time-series data stored in default location inside container (ephemeral)
  - **Note**: Metrics data is lost on container restart - acceptable for development/testing
  - **Production**: Consider adding named volume `prometheus_data:/prometheus` for persistent metrics storage
- **Benefit**: Faster overall stack startup (no healthcheck wait)
- **Consideration**: Initial scrapes may fail until tracker is ready (expected behavior)

### Metrics Endpoints

The tracker provides two metrics endpoints:

1. **`/api/v1/stats`**: Aggregate statistics (torrents count, peers, etc.)
2. **`/api/v1/metrics`**: Detailed operational metrics (requests, errors, latency)

Both require authentication via `token` query parameter.

### Prometheus UI Access

Once deployed, Prometheus UI is accessible at:

- From host: `http://<vm-ip>:9090`
- UI provides: Query interface, target status, configuration viewer, alerts

### Future Enhancements

Consider for future iterations:

1. **Persistent Metrics Storage**: Add named volume for Prometheus data persistence in production
2. **Multiple Tracker Instances**: Support scraping from multiple trackers
3. **Alert Rules**: Add Prometheus alerting configuration
4. **Remote Write**: Support for remote Prometheus storage
5. **Service Discovery**: Dynamic tracker discovery instead of static targets
6. **Configurable Retention**: Expose data retention policy to environment config

### Known Limitations

- **Single Tracker**: Configuration assumes single tracker instance
- **Static Targets**: No service discovery, requires manual target updates
- **No Alerting**: Alert rules not included in this slice (future enhancement)
- **Basic Retention**: Default retention policy, not configurable yet
