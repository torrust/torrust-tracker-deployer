# Grafana Slice - Extension Tasks

**Parent Issue**: [#246](https://github.com/torrust/torrust-tracker-deployer/issues/246) - Grafana Slice - Release and Run Commands
**Branch**: `246-grafana-slice`
**Pull Request**: [#247](https://github.com/torrust/torrust-tracker-deployer/pull/247)
**Status**: Active - Implementation in Progress

## Overview

This document tracks extension tasks for the Grafana service that were identified after the initial implementation but were not included in the original issue scope. These enhancements will be implemented immediately on the current branch and included in PR #247 before merging. They improve the robustness, usability, and automation of the Grafana deployment.

**Current State**: Grafana is deployed with basic configuration - admin credentials are configured via environment variables, but the Prometheus datasource and dashboards require manual setup through the Grafana UI.

**Enhancement Goals**: Make Grafana deployment fully automated and production-ready by adding health checks and automatic configuration provisioning.

## Extension Tasks

### Task 1: Add Prometheus Docker Health Check

**Status**: ✅ Completed
**Priority**: Medium (improves deployment reliability)

#### Problem Statement

The Prometheus container has no health check configured, which means:

- Docker Compose doesn't know when Prometheus is actually ready to serve requests
- The `depends_on` directive only waits for container start, not service readiness
- Services depending on Prometheus can't wait for actual service readiness

#### Proposed Solution

Add Docker Compose health check using Prometheus's built-in health API endpoint:

```yaml
prometheus:
  image: prom/prometheus:v3.0.1
  # ... existing configuration ...
  healthcheck:
    test: ["CMD", "wget", "--spider", "-q", "http://localhost:9090/-/healthy"]
    interval: 10s
    timeout: 5s
    retries: 5
    start_period: 10s
```

**Health Check Configuration**:

- `test`: Uses Prometheus's `/-/healthy` endpoint
  - `http://localhost:9090`: Internal container port
- `interval: 10s`: Check health every 10 seconds after initial start period
- `timeout: 5s`: Each health check must complete within 5 seconds
- `retries: 5`: Container marked unhealthy after 5 consecutive failures
- `start_period: 10s`: Grace period for Prometheus initialization

**Expected Benefits**:

1. **Docker Awareness**: Docker Compose knows when Prometheus is truly ready
2. **Better Monitoring**: Health status visible in `docker ps` and `docker-compose ps`
3. **Service Dependencies**: Grafana can wait for `service_healthy` condition
4. **Automatic Restart**: Can configure restart policies based on health status

**Implementation Impact**:

- **Files to Modify**: `templates/docker-compose/docker-compose.yml.tera` - Add healthcheck block to Prometheus service
- **Testing**: Verify health check works correctly during E2E tests

---

### Task 2: Add Grafana Docker Health Check

**Status**: ✅ Completed
**Priority**: High (improves deployment reliability and reduces E2E retry logic)

**Status**: ✅ Completed
**Priority**: High (improves deployment reliability and reduces E2E retry logic)

#### Problem Statement

The Grafana container has no health check configured, which means:

- Docker Compose doesn't know when Grafana is actually ready to serve requests
- The `depends_on` directive only waits for container start, not service readiness
- E2E tests need retry logic to handle Grafana startup delay (30 attempts × 2 seconds)
- Manual deployments may access Grafana before it's fully initialized

#### Proposed Solution

Add Docker Compose health check using Grafana's built-in health API endpoint:

```yaml
grafana:
  image: grafana/grafana:11.4.0
  # ... existing configuration ...
  healthcheck:
    test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/api/health"]
    interval: 10s
    timeout: 5s
    retries: 5
    start_period: 30s
```

**Health Check Configuration**:

- `test`: Uses `wget --spider` to check Grafana's `/api/health` endpoint
  - `--spider`: Don't download, just check if the page exists
  - `-q`: Quiet mode (no output)
  - `http://localhost:3000`: Internal container port (not the mapped external port 3100)
- `interval: 10s`: Check health every 10 seconds after initial start period
- `timeout: 5s`: Each health check must complete within 5 seconds
- `retries: 5`: Container marked unhealthy after 5 consecutive failures
- `start_period: 30s`: Grace period for Grafana initialization (longer than Prometheus due to heavier startup)

**Expected Benefits**:

1. **Docker Awareness**: Docker Compose knows when Grafana is truly ready
2. **Simplified Validators**: E2E tests can potentially remove retry logic (wait for healthy status instead)
3. **Better Monitoring**: Health status visible in `docker ps` and `docker-compose ps`
4. **Automatic Restart**: Can configure restart policies based on health status
5. **Service Dependencies**: Can depend on Prometheus `service_healthy` condition

**Implementation Impact**:

- **Files to Modify**: `templates/docker-compose/docker-compose.yml.tera` - Add healthcheck block to Grafana service
- **Validation Changes**: `src/infrastructure/remote_actions/validators/grafana.rs` - Consider simplifying retry logic
- **Testing**: Verify health check works correctly during E2E tests

---

### Task 3: Automatically Configure Prometheus Datasource in Grafana

**Status**: ✅ Completed
**Priority**: High (eliminates manual configuration, core automation goal)

```yaml
prometheus:
  image: prom/prometheus:v3.0.1
  # ... existing configuration ...
  healthcheck:
    test: ["CMD", "wget", "--spider", "-q", "http://localhost:9090/-/healthy"]
    interval: 10s
    timeout: 5s
    retries: 5
    start_period: 10s
```

**Health Check Configuration**:

**Grafana**:

- `test`: Uses `wget --spider` to check Grafana's `/api/health` endpoint
  - `--spider`: Don't download, just check if the page exists
  - `-q`: Quiet mode (no output)
  - `http://localhost:3000`: Internal container port (not the mapped external port 3100)
- `interval: 10s`: Check health every 10 seconds after initial start period
- `timeout: 5s`: Each health check must complete within 5 seconds
- `retries: 5`: Container marked unhealthy after 5 consecutive failures
- `start_period: 30s`: Grace period for Grafana initialization (health check failures ignored)

**Prometheus**:

- `test`: Uses Prometheus's `/-/healthy` endpoint
  - `http://localhost:9090`: Internal container port (not the mapped external port)
- `interval: 10s`: Check health every 10 seconds after initial start period
- `timeout: 5s`: Each health check must complete within 5 seconds
- `retries: 5`: Container marked unhealthy after 5 consecutive failures
- `start_period: 10s`: Shorter grace period (Prometheus starts faster than Grafana)

**Expected Benefits**:

1. **Docker Awareness**: Docker Compose and orchestrators know when services are truly ready
2. **Simplified Validators**: E2E tests can potentially remove retry logic (wait for healthy status instead)
3. **Better Monitoring**: Health status visible in `docker ps` and `docker-compose ps`
4. **Automatic Restart**: Can configure restart policies based on health status
5. **Service Dependencies**: Other services can wait for `service_healthy` condition
6. **Grafana Dependency**: Grafana can wait for Prometheus to be healthy before starting (using `depends_on: prometheus: condition: service_healthy`)

**Implementation Impact**:

- **Files to Modify**:
  - `templates/docker-compose/docker-compose.yml.tera` - Add healthcheck blocks to both Prometheus and Grafana services
- **Validation Changes**:
  - `src/infrastructure/remote_actions/validators/grafana.rs` - Consider simplifying retry logic or checking health status
  - `src/infrastructure/remote_actions/validators/prometheus.rs` - Consider adding health status check
- **Testing**:
  - Verify health checks work correctly during E2E tests
  - Test that containers report healthy after successful startup
  - Test that unhealthy containers trigger restart (if restart policy configured)
  - Verify Grafana waits for Prometheus to be healthy before starting

**Alternative Approaches Considered**:

1. **Use `curl` instead of `wget`**: Images may not have curl pre-installed
2. **Check published ports (3100/9090)**: Would check the published port, not the internal service
3. **No health check**: Current approach (requires retry logic in consumers)
4. **Different endpoints**: Grafana and Prometheus both offer multiple health endpoints, chose the most standard ones

**Risks and Considerations**:

- Health checks add slight CPU/network overhead (negligible for 10s interval)
- `wget` must be available in both Grafana and Prometheus containers (verify in images)
- Health endpoints might return 200 before all features are ready (acceptable - basic service health)
- Prometheus starts faster than Grafana (10s vs 30s start periods reflect this difference)

---

### Task 2: Automatically Configure Grafana (Datasource + Dashboards)

**Status**: ✅ Completed
**Priority**: High (eliminates manual configuration, core automation goal)

#### Problem Statement

Currently, users must manually configure the Prometheus datasource in Grafana after deployment:

1. Login to Grafana UI
2. Navigate to Data Sources settings
3. Add Prometheus datasource
4. Configure connection URL (`http://prometheus:9090`)
5. Test and save connection

This manual process is:

- Time-consuming (2-3 minutes per deployment)
- Error-prone (users may misconfigure the URL)
- Inconsistent (different deployments may have different settings)
- Against "Infrastructure as Software" vision

#### Proposed Solution

Implement Grafana provisioning to automatically configure the Prometheus datasource on container startup using Grafana's [datasource provisioning feature](https://grafana.com/docs/grafana/latest/administration/provisioning/#data-sources).

**Create Template**: `templates/grafana/provisioning/datasources/prometheus.yml.tera`

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "{{ prometheus_scrape_interval_in_secs }}s"
      httpMethod: POST
```

**Template Variables**:

- `prometheus_scrape_interval_in_secs`: From `PrometheusConfig::scrape_interval_in_secs`

**Datasource Configuration**:

- `url: http://prometheus:9090`: Uses Docker internal network name
- `isDefault: true`: Makes this the default datasource for all dashboards
- `editable: false`: Prevents accidental modification through UI (infrastructure-as-code principle)

**Docker Compose Integration**: Update `templates/docker-compose/docker-compose.yml.tera`

````yaml
grafana:
  # ... existing configuration ...
  volumes:
    - grafana_data:/var/lib/grafana
    - ./storage/grafana/provisioning:/etc/grafana/provisioning:ro # NEW
```yaml

**File Structure After Deployment**:

```text
/opt/torrust/storage/grafana/provisioning/
└── datasources/
    └── prometheus.yml
````

**Expected Benefits**:

1. **Zero Manual Configuration**: Prometheus datasource automatically configured
2. **Consistent Deployments**: Every deployment has identical datasource setup
3. **Error Prevention**: No misconfigured datasource URLs
4. **Infrastructure as Code**: Configuration is version-controlled and reproducible

**Implementation Impact**:

- **Files to Create**:
  - `templates/grafana/provisioning/datasources/prometheus.yml.tera` (Tera template)
  - `templates/ansible/deploy-grafana-provisioning.yml` (new Ansible playbook - static file)
  - `src/infrastructure/templating/grafana/` (new module structure)
- **Files to Modify**:
  - `templates/docker-compose/docker-compose.yml.tera` - Add provisioning volume mount
  - `src/application/command_handlers/configure/handler.rs` - Add Grafana provisioning step
  - `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs` - Register new playbook in `copy_static_templates()`
- **Testing**: Verify datasource appears automatically in Grafana UI after deployment

---

### Task 4: Preload Grafana Dashboards

**Status**: ✅ Completed
**Priority**: High (completes full automation, provides immediate value)

**Status**: ✅ Completed
**Priority**: High (completes full automation, provides immediate value)

#### Problem Statement

Currently, users must manually import dashboards after deployment:

1. Search for suitable Grafana dashboards online or create custom ones
2. Export/import dashboard JSON files
3. Configure dashboard queries and variables
4. Save dashboards

This manual process is:

- Time-consuming (5-10 minutes per dashboard)
- Requires Grafana expertise (query syntax, panel configuration)
- Results may vary (different users create different dashboards)
- Users don't immediately see the value of the monitoring stack

#### Proposed Solution

Implement Grafana provisioning to automatically load two pre-configured dashboards on container startup using Grafana's [dashboard provisioning feature](https://grafana.com/docs/grafana/latest/administration/provisioning/#dashboards).

**Create Dashboard Provider Config**: `templates/grafana/provisioning/dashboards/torrust.yml`

```yaml
apiVersion: 1

providers:
  - name: "Torrust Dashboards"
    orgId: 1
    folder: "Torrust Tracker"
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards/torrust
      foldersFromFilesStructure: false
```

**Dashboard Files to Create**:

1. **Torrust Tracker Stats**: `templates/grafana/dashboards/stats.json`

   - Source: https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/stats.json
   - Prometheus Job: `tracker_stats` (scrapes `/api/v1/stats` endpoint)
   - Displays tracker aggregate statistics and state metrics
   - Pre-configured dashboard from Torrust Tracker Live Demo

2. **Torrust Tracker Metrics**: `templates/grafana/dashboards/metrics.json`
   - Source: https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/metrics.json
   - Prometheus Job: `tracker_metrics` (scrapes `/api/v1/metrics` endpoint)
   - Displays detailed operational metrics and performance data
   - Pre-configured dashboard from Torrust Tracker Live Demo

**Prometheus Job Mapping** (from `templates/prometheus/prometheus.yml.tera`):

```yaml
scrape_configs:
  # Stats dashboard queries this job
  - job_name: "tracker_stats"
    metrics_path: "/api/v1/stats"

  # Metrics dashboard queries this job
  - job_name: "tracker_metrics"
    metrics_path: "/api/v1/metrics"
```

**Docker Compose Integration**: Already handled by Task 3 volume mount

**File Structure After Deployment**:

```text
/opt/torrust/storage/grafana/provisioning/
├── datasources/
│   └── prometheus.yml
└── dashboards/
    ├── torrust.yml
    └── torrust/
        ├── stats.json
        └── metrics.json
```

**Expected Benefits**:

1. **Immediate Value**: Users see metrics visualization immediately after deployment
2. **Consistent Experience**: All deployments have the same dashboards as the live demo
3. **Proven Dashboards**: Uses battle-tested dashboards from Torrust Tracker Live Demo
4. **Faster Time to Value**: No dashboard creation/import required
5. **Customizable**: Users can modify dashboards through UI (allowUiUpdates: true)

**Implementation Impact**:

- **Files to Create**:
  - `templates/grafana/provisioning/dashboards/torrust.yml` (static YAML)
  - `templates/grafana/dashboards/stats.json` (copied from torrust-demo)
  - `templates/grafana/dashboards/metrics.json` (copied from torrust-demo)
- **Files to Modify**:
  - Grafana template rendering module (copy dashboard files to build directory)
- **Ansible Integration**: Uses `deploy-grafana-provisioning.yml` created in Task 3 (single playbook handles both datasource and dashboards)
- **Testing**: Verify dashboards appear automatically in Grafana UI and display metrics correctly

**Dashboard Sources**:

- Stats Dashboard: https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/stats.json (uses `tracker_stats` Prometheus job)
- Metrics Dashboard: https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/metrics.json (uses `tracker_metrics` Prometheus job)
- Dashboard Documentation: https://github.com/torrust/torrust-demo/tree/main/share/grafana/dashboards

---

## Implementation Sequence

**Recommended Order**:

1. **Task 1** (Prometheus Health Check) - Simple, no dependencies
2. **Task 2** (Grafana Health Check) - Simple, can depend on Task 1
3. **Task 3** (Prometheus Datasource) - More complex, enables Task 4
4. **Task 4** (Preload Dashboards) - Depends on Task 3 for datasource

**Dependencies**:

- Task 2 can optionally use `depends_on: prometheus: condition: service_healthy` (requires Task 1)
- Task 4 requires Task 3 (dashboards need datasource to display metrics)
- Tasks 1 and 2 are independent and can be done in parallel
- Tasks 3 and 4 could be combined but separated for better tracking

**Estimated Effort**:

- Task 1: 1-2 hours (simple healthcheck addition)
- Task 2: 1-2 hours (simple healthcheck addition + optional retry logic simplification)
- Task 3: 4-6 hours (template creation + module structure + rendering integration)
- Task 4: 4-6 hours (dashboard JSON creation + testing metrics display)
- **Total**: 10-16 hours

---

## Success Criteria

### Task 1 (Prometheus Health Check)

- [ ] Health check added to Prometheus service in docker-compose template
- [ ] `docker-compose ps` shows `healthy` status for Prometheus after startup
- [ ] Health check fails appropriately if Prometheus service crashes
- [ ] E2E tests pass with Prometheus health check enabled

### Task 2 (Grafana Health Check)

- [ ] Health check added to Grafana service in docker-compose template
- [ ] `docker-compose ps` shows `healthy` status for Grafana after startup
- [ ] Health check fails appropriately if Grafana service crashes
- [ ] Grafana optionally depends on Prometheus being healthy (using `condition: service_healthy`)
- [ ] E2E tests pass with Grafana health check enabled
- [ ] Consider simplifying Grafana validator retry logic

### Task 3 (Prometheus Datasource)

- [ ] Prometheus datasource template created (`prometheus.yml.tera`)
- [ ] Grafana templating module structure created (`src/infrastructure/templating/grafana/`)
- [ ] Ansible playbook created (`deploy-grafana-provisioning.yml`)
- [ ] Playbook registered in `copy_static_templates()` method
- [ ] Datasource provisioning integrated into `configure` command
- [ ] Provisioning directory mounted in docker-compose
- [ ] Datasource appears automatically in Grafana UI after deployment
- [ ] Datasource connection to Prometheus works (test query succeeds)
- [ ] E2E tests verify datasource is configured

### Task 4 (Preload Dashboards)

- [ ] Dashboard provider config created (`torrust.yml`)
- [ ] Stats dashboard JSON copied from torrust-demo repository
- [ ] Metrics dashboard JSON copied from torrust-demo repository
- [ ] Dashboard files copied to build directory during template rendering
- [ ] Both dashboards appear automatically in Grafana UI after deployment
- [ ] Dashboards display metrics correctly (panels show data, no errors)
- [ ] Dashboards organized in "Torrust Tracker" folder
- [ ] Users can modify dashboards through UI
- [ ] E2E tests verify dashboards are accessible

---

## Implementation Details (Consolidated from Tasks 2-4)

### Grafana Provisioning Configuration

#### 2.1. Datasource Provisioning

**Create Template**: `templates/grafana/provisioning/datasources/prometheus.yml.tera`

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "{{ prometheus_scrape_interval_in_secs }}s"
      httpMethod: POST
```

**Template Variables**:

- `prometheus_scrape_interval_in_secs`: From `PrometheusConfig::scrape_interval_in_secs`
- Datasource URL uses Docker internal network name: `http://prometheus:9090`
- `editable: false`: Prevents accidental modification through UI (following infrastructure-as-code principle)
- `isDefault: true`: Makes this the default datasource for all dashboards

**File Placement After Rendering**:

- Build directory: `build/{env-name}/grafana/provisioning/datasources/prometheus.yml`
- Remote host: `/opt/torrust/storage/grafana/provisioning/datasources/prometheus.yml`

#### 2.2. Dashboard Provisioning

**Create Dashboard Provider Config**: `templates/grafana/provisioning/dashboards/torrust.yml`

```yaml
apiVersion: 1

providers:
  - name: "Torrust Dashboards"
    orgId: 1
    folder: "Torrust Tracker"
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards/torrust
      foldersFromFilesStructure: false
```

**Dashboard Provider Settings**:

- `folder: 'Torrust Tracker'`: Dashboards organized in dedicated folder
- `disableDeletion: false`: Users can delete/modify dashboards through UI (not enforced like datasource)
- `allowUiUpdates: true`: Users can customize dashboards and save changes
- `path`: Directory containing dashboard JSON files

**Dashboard JSON Files**: Copy two pre-configured dashboards from torrust-demo

1. **Torrust Tracker Stats**: `templates/grafana/dashboards/stats.json`

   - Source: https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/stats.json
   - Prometheus Job: `tracker_stats` (scrapes `/api/v1/stats` endpoint)
   - Displays tracker aggregate statistics and state metrics

2. **Torrust Tracker Metrics**: `templates/grafana/dashboards/metrics.json`
   - Source: https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/metrics.json
   - Prometheus Job: `tracker_metrics` (scrapes `/api/v1/metrics` endpoint)
   - Displays detailed operational metrics and performance data

**File Placement After Rendering**:

- Build directory: `build/{env-name}/grafana/provisioning/dashboards/`
- Remote host: `/opt/torrust/storage/grafana/provisioning/dashboards/`

#### 2.3. Docker Compose Integration

**Update**: `templates/docker-compose/docker-compose.yml.tera`

Add bind mount for provisioning directory:

```yaml
grafana:
  image: grafana/grafana:11.4.0
  container_name: torrust-grafana
  restart: unless-stopped
  depends_on:
    - prometheus
  ports:
    - "{{ grafana_external_port }}:3000"
  environment:
    - GF_SECURITY_ADMIN_USER=${GRAFANA_ADMIN_USER}
    - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
    - GF_INSTALL_PLUGINS= # Space for future plugin additions
  volumes:
    - grafana_data:/var/lib/grafana
    - ./storage/grafana/provisioning:/etc/grafana/provisioning:ro # NEW: Provisioning configs
  networks:
    - torrust-network

volumes:
  grafana_data:
```

**Mount Configuration**:

- `./storage/grafana/provisioning:/etc/grafana/provisioning:ro`
- Bind mount (not named volume) - allows editing files on host
- Read-only (`:ro`) - Grafana doesn't need write access to provisioning configs
- Path relative to docker-compose file: `storage/grafana/provisioning/`

#### 2.4. Ansible Deployment Integration

**Create New Playbook**: `templates/ansible/deploy-grafana-provisioning.yml`

Single playbook handles both datasource and dashboard provisioning (follows one-operation-per-playbook pattern):

```yaml
---
- name: Deploy Grafana provisioning configuration
  hosts: all
  vars_files:
    - variables.yml

  tasks:
    - name: Create Grafana provisioning directories
      ansible.builtin.file:
        path: "{{ item }}"
        state: directory
        mode: "0755"
      loop:
        - "{{ deploy_dir }}/storage/grafana/provisioning/datasources"
        - "{{ deploy_dir }}/storage/grafana/provisioning/dashboards"
        - "{{ deploy_dir }}/storage/grafana/provisioning/dashboards/torrust"
      when: grafana_enabled | default(false)

    - name: Deploy Grafana provisioning files
      ansible.builtin.copy:
        src: "{{ build_dir }}/grafana/provisioning/"
        dest: "{{ deploy_dir }}/storage/grafana/provisioning/"
        mode: "0644"
      when: grafana_enabled | default(false)
```

**Design Decision**: Single playbook for all Grafana provisioning

- **Rationale**: Grafana provisioning is one atomic operation (datasource + dashboards together)
- **User Flexibility**: Users can delete unwanted dashboard files after deployment: `rm -rf /opt/torrust/storage/grafana/provisioning/dashboards/torrust/*.json`
- **Consistent with Project Pattern**: Follows `templates/ansible/README.md` philosophy (one logical operation per playbook)
- **Registration Required**: Must add to `copy_static_templates()` in `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs`

#### 2.5. Template Rendering Integration

**Update**: `src/infrastructure/templating/grafana/` (NEW module structure)

Create module structure similar to Prometheus:

```text
src/infrastructure/templating/grafana/
├── mod.rs
└── template/
    ├── mod.rs
    └── renderer/
        ├── mod.rs
        ├── project_generator.rs   # NEW: Grafana provisioning generator
        └── datasource.rs           # NEW: Datasource YAML context
        └── dashboards.rs           # NEW: Dashboard provider YAML context
```

**Project Generator**: Similar to `PrometheusProjectGenerator`

```rust
pub struct GrafanaProjectGenerator {
    source_templates_dir: PathBuf,
    build_output_dir: PathBuf,
}

impl GrafanaProjectGenerator {
    pub fn generate(&self, context: &GrafanaContext) -> Result<(), TemplateError> {
        // 1. Render datasource YAML from template
        // 2. Copy dashboard provider YAML (static file)
        // 3. Copy dashboard JSON files (static files)
        // 4. Create directory structure in build output
    }
}
```

**Context Structs**:

```rust
pub struct GrafanaContext {
    pub prometheus_scrape_interval_in_secs: u32,
}

pub struct DatasourceContext {
    pub prometheus_scrape_interval_in_secs: u32,
}
```

**Template Registration**:

- Datasource template: `templates/grafana/provisioning/datasources/prometheus.yml.tera` (dynamic)
- Dashboard provider: `templates/grafana/provisioning/dashboards/torrust.yml` (static - copy directly)
- Dashboard JSONs: `templates/grafana/dashboards/*.json` (static - copy directly)

#### 2.6. Configure Command Integration

**Update**: `src/application/command_handlers/configure/handler.rs`

Add Grafana provisioning step to configure workflow:

```rust
// After deploying tracker config, before or alongside Prometheus config
if let Some(grafana_config) = &user_inputs.grafana {
    // Generate Grafana provisioning files
    let grafana_generator = GrafanaProjectGenerator::new(
        template_paths.grafana_templates_dir(),
        build_paths.grafana_build_dir(),
    );

    let grafana_context = GrafanaContext {
        prometheus_scrape_interval_in_secs: user_inputs
            .prometheus
            .as_ref()
            .map(|p| p.scrape_interval_in_secs)
            .unwrap_or(15), // Default if not specified
    };

    grafana_generator.generate(&grafana_context)?;
}
```

**Error Handling**:

- If Grafana enabled but template generation fails → `ConfigureStep::ConfigureGrafana` variant in `ConfigureFailed` state
- Include clear error messages about which provisioning file failed to generate

#### Expected Benefits

1. **Zero Manual Configuration**: Users deploy and immediately access fully configured Grafana
2. **Consistent Deployments**: Every deployment has identical datasource and dashboard setup
3. **Faster Time to Value**: Users see metrics immediately without setup delays
4. **Reduced Documentation**: User guide shows dashboards, not setup instructions
5. **Infrastructure as Code**: Grafana configuration is version-controlled and reproducible
6. **Error Prevention**: No misconfigured datasource URLs or authentication issues

#### Implementation Impact

**Files to Create**:

- `templates/grafana/provisioning/datasources/prometheus.yml.tera` (Tera template)
- `templates/grafana/provisioning/dashboards/torrust.yml` (static YAML)
- `templates/grafana/dashboards/stats.json` (copied from torrust-demo)
- `templates/grafana/dashboards/metrics.json` (copied from torrust-demo)
- `src/infrastructure/templating/grafana/` (new module tree)

**Files to Modify**:

- `templates/docker-compose/docker-compose.yml.tera` - Add provisioning volume mount
- `templates/ansible/deploy-docker-compose-files.yml` - Create provisioning directories
- `src/application/command_handlers/configure/handler.rs` - Add Grafana provisioning step
- `src/domain/environment/state/configure_failed.rs` - Add `ConfigureStep::ConfigureGrafana` variant (if needed)

**Testing Requirements**:

1. **Unit Tests**: Grafana context serialization, template rendering
2. **Integration Tests**: Provisioning file generation, directory structure creation
3. **E2E Tests**: Full deployment verification, dashboard accessibility
4. **Manual Testing**: Dashboard functionality, metric queries, Prometheus connection

#### Alternative Approaches Considered

1. **Manual Configuration (Current)**: Simple but poor user experience
2. **UI Automation (Selenium/API)**: Complex, brittle, requires Grafana to be running
3. **Pre-configured Container Image**: Less flexible, harder to customize
4. **Init Container Script**: More complex than provisioning, non-standard approach

#### Risks and Considerations

- **Dashboard Maintenance**: Dashboard JSON files need updates when metrics change
- **Grafana Version Compatibility**: Provisioning format may change between Grafana versions
- **Dashboard Customization**: Users may want different dashboards (document how to add custom dashboards)
- **Prometheus Metrics**: Dashboards assume specific metric names from Torrust Tracker
- **Testing Complexity**: Need to verify dashboard queries return valid data

---

## Implementation Sequence

**Recommended Order**:

1. **Task 1 (Health Check)** - Simpler, provides immediate value, no breaking changes
2. **Task 2 (Provisioning)** - More complex, builds on Task 1's health check for reliability

**Estimated Effort**:

- Task 1: 2-4 hours (implementation + testing)
- Task 2: 8-12 hours (implementation + dashboard creation + testing + documentation)

**Testing Strategy**:

- Both tasks should be tested together in E2E workflow
- Verify health check reports healthy status after provisioning completes
- Verify dashboards load automatically and display metrics correctly

---

## Success Criteria

### Task 1 (Health Check)

- [ ] Health checks added to docker-compose template (both Prometheus and Grafana)
- [ ] `docker-compose ps` shows `healthy` status after services start
- [ ] Health checks fail appropriately if services crash
- [ ] Grafana can optionally depend on Prometheus being healthy (using `condition: service_healthy`)
- [ ] E2E tests pass with health checks enabled
- [ ] Documentation updated to mention health check feature

### Task 2 (Provisioning)

- [ ] Prometheus datasource automatically configured on deployment
- [ ] Two dashboards (Tracker Overview + System Metrics) automatically loaded
- [ ] Dashboards display metrics correctly (no empty/broken panels)
- [ ] Users can access dashboards immediately after deployment without manual setup
- [ ] Provisioning files generated during `configure` command
- [ ] Provisioning directories created by Ansible playbooks
- [ ] User guide updated to show pre-configured dashboards instead of manual setup instructions
- [ ] E2E tests verify dashboard accessibility

---

## Manual Testing Guide

### Task 1: Verify Prometheus Health Check

**After Implementation**:

1. **Deploy environment with Prometheus enabled**:

   ```bash
   cargo run -- create environment --env-file envs/manual-test-prometheus.json
   cargo run -- provision <env-name>
   cargo run -- configure <env-name>
   cargo run -- release <env-name>
   cargo run -- run <env-name>
   ```

2. **Check health status**:

   ```bash
   # SSH into the VM
   ssh -i ~/.ssh/your-key user@vm-ip

   # Check container health status
   cd /opt/torrust
   docker-compose ps

   # Should show 'healthy' in STATUS column for prometheus container
   # Example: torrust-prometheus ... Up 2 minutes (healthy)
   ```

3. **Verify health check endpoint**:

   ```bash
   # From inside VM
   docker exec torrust-prometheus wget --spider -q http://localhost:9090/-/healthy
   echo $?  # Should return 0 (success)

   # Test failure scenario (stop Prometheus)
   docker-compose stop prometheus
   docker-compose ps  # Should show 'unhealthy' or 'exited'
   ```

4. **Check health check configuration**:

   ```bash
   # Inspect healthcheck settings
   docker inspect torrust-prometheus | jq '.[0].State.Health'

   # Should show:
   # - Status: "healthy"
   # - FailingStreak: 0
   # - Log entries with exit code 0
   ```

**Expected Results**:

- ✅ Container shows `(healthy)` status within 20 seconds of startup
- ✅ Health endpoint returns success (exit code 0)
- ✅ Container becomes unhealthy when Prometheus stops

---

### Task 2: Verify Grafana Health Check

**After Implementation**:

1. **Deploy environment with Grafana enabled**:

   ```bash
   cargo run -- create environment --env-file envs/manual-test-grafana.json
   cargo run -- provision <env-name>
   cargo run -- configure <env-name>
   cargo run -- release <env-name>
   cargo run -- run <env-name>
   ```

2. **Check health status**:

   ```bash
   # SSH into the VM
   ssh -i ~/.ssh/your-key user@vm-ip
   cd /opt/torrust

   # Check both Prometheus and Grafana health
   docker-compose ps

   # Should show 'healthy' for both containers:
   # torrust-prometheus ... Up 2 minutes (healthy)
   # torrust-grafana    ... Up 2 minutes (healthy)
   ```

3. **Verify Grafana health check endpoint**:

   ```bash
   # From inside VM
   docker exec torrust-grafana wget --spider -q http://localhost:3000/api/health
   echo $?  # Should return 0 (success)

   # Check health status details
   docker exec torrust-grafana wget -qO- http://localhost:3000/api/health
   # Should return: {"commit":"...","database":"ok","version":"11.4.0"}
   ```

4. **Verify dependency on Prometheus** (if implemented):

   ```bash
   # Check if Grafana waits for Prometheus to be healthy
   docker-compose down
   docker-compose up -d

   # Watch container startup order
   docker-compose ps --format "table {{.Name}}\t{{.Status}}"

   # Prometheus should reach 'healthy' before Grafana starts
   ```

5. **Test E2E validator simplification**:

   ```bash
   # Run E2E tests - should no longer need 30-retry logic
   cargo run --bin e2e-deployment-workflow-tests

   # Check validator code - retry logic should be simplified or removed
   ```

**Expected Results**:

- ✅ Grafana shows `(healthy)` status within 40 seconds of startup
- ✅ Health endpoint returns success with database status
- ✅ Grafana waits for Prometheus to be healthy (if dependency configured)
- ✅ E2E tests pass without long retry delays

---

### Task 3: Verify Prometheus Datasource Auto-Configuration

**After Implementation**:

1. **Deploy with Grafana provisioning**:

   ```bash
   cargo run -- create environment --env-file envs/manual-test-grafana.json
   cargo run -- provision <env-name>
   cargo run -- configure <env-name>  # Should generate provisioning files
   cargo run -- release <env-name>
   cargo run -- run <env-name>
   ```

2. **Verify provisioning files were generated**:

   ```bash
   # Check build directory
   ls -la build/manual-test-grafana/grafana/provisioning/datasources/
   # Should contain: prometheus.yml

   cat build/manual-test-grafana/grafana/provisioning/datasources/prometheus.yml
   # Verify: url: http://prometheus:9090, isDefault: true, editable: false
   ```

3. **Verify files deployed to remote host**:

   ```bash
   # SSH into VM
   ssh -i ~/.ssh/your-key user@vm-ip

   # Check provisioning directory structure
   tree /opt/torrust/storage/grafana/provisioning/

   # Should show:
   # /opt/torrust/storage/grafana/provisioning/
   # └── datasources/
   #     └── prometheus.yml

   cat /opt/torrust/storage/grafana/provisioning/datasources/prometheus.yml
   # Verify content matches build directory
   ```

4. **Verify datasource in Grafana UI**:

   ```bash
   # Get VM IP
   VM_IP=$(terraform -chdir=build/manual-test-grafana/tofu output -raw instance_ip)

   # Access Grafana (credentials from environment config)
   # Open browser: http://$VM_IP:3100
   # Login with admin credentials
   ```

   In Grafana UI:

   - Navigate to **Configuration** → **Data Sources**
   - Should see **Prometheus** datasource (with star icon indicating default)
   - Click on it to view settings:
     - URL: `http://prometheus:9090`
     - Access: `Server (default)`
     - Editable: No (grayed out fields)
   - Click **Test** button → Should show "Data source is working"

5. **Verify Prometheus queries work**:

   ```bash
   # In Grafana UI
   # Navigate to Explore (compass icon)
   # Select Prometheus datasource
   # Run query: up
   # Should show metrics for tracker_stats and tracker_metrics jobs
   ```

6. **Test datasource was created at startup** (not manually):

   ```bash
   # Check Grafana logs for provisioning messages
   docker logs torrust-grafana | grep -i provisioning

   # Should see:
   # "Provisioning datasources"
   # "Provisioned datasources: Prometheus"
   ```

**Expected Results**:

- ✅ Provisioning files generated in build directory during `configure`
- ✅ Files deployed to `/opt/torrust/storage/grafana/provisioning/datasources/`
- ✅ Prometheus datasource appears in Grafana UI automatically
- ✅ Datasource is marked as default (star icon)
- ✅ Datasource fields are not editable through UI
- ✅ Test connection succeeds
- ✅ Can query Prometheus metrics in Explore view

---

### Task 4: Verify Preloaded Grafana Dashboards

**After Implementation**:

1. **Deploy with dashboard provisioning**:

   ```bash
   cargo run -- create environment --env-file envs/manual-test-grafana.json
   cargo run -- provision <env-name>
   cargo run -- configure <env-name>
   cargo run -- release <env-name>
   cargo run -- run <env-name>
   ```

2. **Verify dashboard files were generated**:

   ```bash
   # Check build directory for dashboard files
   ls -la build/manual-test-grafana/grafana/provisioning/dashboards/
   # Should contain: torrust.yml

   ls -la build/manual-test-grafana/grafana/dashboards/
   # Should contain: stats.json, metrics.json

   # Verify dashboard provider config
   cat build/manual-test-grafana/grafana/provisioning/dashboards/torrust.yml
   # Should specify: path: /etc/grafana/provisioning/dashboards/torrust
   ```

3. **Verify files deployed to remote host**:

   ```bash
   # SSH into VM
   ssh -i ~/.ssh/your-key user@vm-ip

   # Check complete provisioning directory structure
   tree /opt/torrust/storage/grafana/provisioning/

   # Should show:
   # /opt/torrust/storage/grafana/provisioning/
   # ├── datasources/
   # │   └── prometheus.yml
   # └── dashboards/
   #     ├── torrust.yml
   #     └── torrust/
   #         ├── stats.json
   #         └── metrics.json
   ```

4. **Verify dashboards in Grafana UI**:

   ```bash
   # Access Grafana: http://$VM_IP:3100
   ```

   In Grafana UI:

   - Navigate to **Dashboards** (four squares icon)
   - Should see folder **"Torrust Tracker"** with 2 dashboards
   - Click on folder to expand:
     - **Torrust Tracker Stats**
     - **Torrust Tracker Metrics**

5. **Verify Stats Dashboard**:

   - Open **Torrust Tracker Stats** dashboard
   - Should see panels with data (not empty):
     - Tracker statistics and state metrics
     - Data from `tracker_stats` Prometheus job
   - Check time range selector (top right) - adjust if needed
   - All panels should display metrics (no "No data" messages)
   - Check datasource (top right) - should be "Prometheus"

6. **Verify Metrics Dashboard**:

   - Open **Torrust Tracker Metrics** dashboard
   - Should see panels with data:
     - Operational metrics and performance data
     - Data from `tracker_metrics` Prometheus job
   - All panels should display metrics

7. **Verify dashboards are editable**:

   - In any dashboard, click **Dashboard settings** (gear icon)
   - Try editing a panel (click panel title → Edit)
   - Make a change (e.g., modify title)
   - Click **Save dashboard** (disk icon)
   - Should save successfully (allowUiUpdates: true)

8. **Verify dashboard provisioning logs**:

   ```bash
   # Check Grafana logs for dashboard provisioning
   docker logs torrust-grafana | grep -i dashboard

   # Should see:
   # "Provisioning dashboards"
   # "Dashboard provisioned: Torrust Tracker Stats"
   # "Dashboard provisioned: Torrust Tracker Metrics"
   ```

9. **Verify Prometheus job mapping**:

   ```bash
   # Check Prometheus targets to ensure jobs are configured
   # Open browser: http://$VM_IP:9090/targets

   # Should see two targets:
   # - tracker_stats (endpoint: http://tracker:1212/api/v1/stats)
   # - tracker_metrics (endpoint: http://tracker:1212/api/v1/metrics)
   # Both should be in "UP" state
   ```

10. **Test dashboard persistence**:

    ```bash
    # Restart Grafana container
    docker-compose restart grafana

    # Wait for healthy status
    docker-compose ps

    # Dashboards should still be present and unchanged
    ```

**Expected Results**:

- ✅ Dashboard provider config and JSON files generated in build directory
- ✅ Files deployed to `/opt/torrust/storage/grafana/provisioning/dashboards/`
- ✅ "Torrust Tracker" folder appears in Grafana UI
- ✅ Both dashboards (Stats and Metrics) are visible in the folder
- ✅ Stats dashboard displays metrics from tracker_stats job
- ✅ Metrics dashboard displays metrics from tracker_metrics job
- ✅ All panels show data (no empty panels)
- ✅ Dashboards can be edited and saved through UI
- ✅ Dashboards persist after container restart
- ✅ Prometheus targets show both jobs in UP state

---

### Complete Integration Test

**After All Tasks Implemented**:

1. **Full deployment workflow**:

   ```bash
   # Clean slate
   cargo run -- destroy <env-name> --force
   rm -rf build/manual-test-grafana data/manual-test-grafana

   # Complete workflow
   cargo run -- create environment --env-file envs/manual-test-grafana.json
   cargo run -- provision <env-name>
   cargo run -- configure <env-name>
   cargo run -- release <env-name>
   cargo run -- run <env-name>
   ```

2. **Verify complete stack**:

   ```bash
   # Check all containers are healthy
   ssh -i ~/.ssh/your-key user@vm-ip
   cd /opt/torrust
   docker-compose ps

   # Should show all healthy:
   # torrust-tracker    ... Up (healthy)
   # torrust-prometheus ... Up (healthy)
   # torrust-grafana    ... Up (healthy)
   ```

3. **Verify end-to-end metrics flow**:

   - Tracker generates metrics
   - Prometheus scrapes both endpoints
   - Grafana displays metrics in both dashboards
   - No manual configuration required

4. **Run E2E tests**:

   ```bash
   cargo run --bin e2e-deployment-workflow-tests
   # Should pass with Prometheus and Grafana validation
   ```

**Expected Results**:

- ✅ Complete deployment works without manual intervention
- ✅ All containers healthy within expected timeframes
- ✅ Grafana accessible with datasource and dashboards pre-configured
- ✅ Metrics flow from tracker → Prometheus → Grafana
- ✅ E2E tests pass
- ✅ User can immediately view metrics without any setup

---

## Future Enhancements (Out of Scope for This PR)

These are potential future improvements not included in the current implementation (may be addressed in separate issues):

- **Alert Configuration**: Provision Grafana alert rules for tracker health monitoring
- **Additional Dashboards**: More specialized dashboards (database metrics, cache metrics)
- **Multi-Datasource Support**: Support for additional datasources beyond Prometheus
- **Custom Plugin Installation**: Allow users to specify Grafana plugins in environment config
- **LDAP/OAuth Integration**: Enterprise authentication instead of admin credentials
- **Dashboard Versioning**: Track dashboard changes in git, allow rollback
- **Grafana as Code**: Use Terraform provider or Grafonnet for dashboard definition

---

## Related Documentation

- **Parent Issue**: [Issue #246](https://github.com/torrust/torrust-tracker-deployer/issues/246)
- **Parent Issue Tracking**: [docs/issues/246-grafana-slice-release-run-commands.md](./246-grafana-slice-release-run-commands.md)
- **Grafana Integration ADR**: [docs/decisions/grafana-integration-pattern.md](../decisions/grafana-integration-pattern.md)
- **Grafana User Guide**: [docs/user-guide/services/grafana.md](../user-guide/services/grafana.md)
- **Template System Architecture**: [docs/technical/template-system-architecture.md](../technical/template-system-architecture.md)
- **Prometheus Implementation**: See `src/infrastructure/templating/prometheus/` for similar pattern

---

## Notes

- These tasks will be implemented on branch `246-grafana-slice` and included in PR #247
- Implementation to begin immediately after document review
- Dashboard JSON content will be defined during implementation (requires knowledge of actual Prometheus metrics exposed by Torrust Tracker)
- Both tasks should be completed before merging PR #247 to main
