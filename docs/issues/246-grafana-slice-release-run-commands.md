# Grafana Slice - Release and Run Commands

**Issue**: [#246](https://github.com/torrust/torrust-tracker-deployer/issues/246)
**Parent Epic**: [#216](https://github.com/torrust/torrust-tracker-deployer/issues/216) (Implement ReleaseCommand and RunCommand with vertical slices)

## Overview

This task adds Grafana as a metrics visualization service for the Torrust Tracker deployment. It extends the docker-compose stack with a Grafana service that connects to Prometheus for displaying tracker metrics through dashboards. The service is **enabled by default** in generated environment templates, but can be disabled by removing the Grafana configuration section from the environment config file.

**Dependency**: Grafana requires Prometheus to be enabled. If Prometheus is not configured, attempting to enable Grafana will result in a validation error during environment creation.

## Goals

- [ ] Add Grafana service conditionally to docker-compose stack (only when present in environment config)
- [ ] Validate that Prometheus is enabled when Grafana is requested (dependency check)
- [ ] Create Grafana configuration section in environment schema
- [ ] Extend environment configuration schema to include Grafana monitoring section
- [ ] Configure service dependency - Grafana depends on Prometheus service
- [ ] Include Grafana in generated environment templates by default (enabled by default)
- [ ] Allow users to disable Grafana by removing its configuration section
- [ ] Deploy and verify Grafana connects to Prometheus and displays metrics

## Progress

_This section will be updated as implementation progresses._

- [ ] **Phase 1**: Environment Configuration & Validation
- [ ] **Phase 2**: Docker Compose Integration
- [ ] **Phase 3**: Testing & Verification
- [ ] **Phase 4**: Documentation

## 🏗️ Architecture Requirements

**DDD Layers**: Infrastructure + Domain + Application
**Module Paths**:

- `src/infrastructure/templating/docker_compose/` - Docker Compose template rendering with Grafana service
- `src/domain/grafana/` - Grafana configuration domain types (NEW)
- `src/application/command_handlers/create/config/validation/` - Grafana-Prometheus dependency validation (NEW)
- `src/application/steps/configure_grafana_firewall.rs` - Grafana firewall configuration step (NEW)
- `src/domain/environment/state.rs` - Add `ConfigureGrafanaFirewall` variant to `ConfigureStep` enum (NEW)

**Pattern**: Configuration-driven service selection with dependency validation

### Module Structure Requirements

- Follow template system architecture (see [docs/technical/template-system-architecture.md](../technical/template-system-architecture.md))
- Add Grafana configuration to environment domain model
- Implement validation logic to check Prometheus dependency
- Use `.tera` extension for docker-compose templates
- Environment config drives Grafana enablement
- No separate Grafana configuration files needed (service uses defaults + environment variables)

### Architectural Constraints

- ✅ Grafana service is included by default in generated environment templates
- ✅ Only included in docker-compose when Grafana section present in environment config
- ✅ Service can be disabled by removing the `grafana` section from config
- ✅ Grafana **requires** Prometheus to be enabled (hard dependency)
- ✅ Validation fails at environment creation if Grafana enabled but Prometheus disabled
- ✅ Grafana depends on Prometheus service in docker-compose (starts after Prometheus)
- ✅ Grafana credentials injected via environment variables from `.env` file
- ✅ Grafana uses persistent named volume `grafana_data` for dashboard state
- ✅ **No Separate Config Files**: Unlike Prometheus, Grafana doesn't require separate configuration file templates
  - Grafana is configured entirely through environment variables and docker-compose settings
  - Dashboards can be added later through the UI or mounted as optional files
  - **Rationale**: Grafana has sensible defaults and the Prometheus datasource can be configured through environment variables
- ✅ **Firewall Configuration**: Grafana UI port (3100) is exposed publicly through firewall during `configure` command
  - Firewall rules added conditionally (only when Grafana is enabled in environment config)
  - Port exposure is **temporary** until HTTPS/reverse proxy support is added (roadmap task 6)
  - When proxy is implemented, public port exposure will be removed
  - **Pattern**: Similar to tracker firewall configuration - opens port only if service enabled

### Anti-Patterns to Avoid

- ❌ Allowing Grafana to be enabled without Prometheus (breaks the service)
- ❌ Hardcoding Grafana credentials in docker-compose template (use environment variables)
- ❌ Starting Grafana before Prometheus is ready (use `depends_on`)
- ❌ Creating unnecessary Grafana configuration templates (service works with defaults)
- ❌ Making Grafana mandatory for all deployments (should be optional)
- ❌ Skipping dependency validation (could lead to runtime errors)

## Implementation Strategy

The implementation follows an **enabled-by-default, opt-out approach with dependency validation** where Grafana is included in generated templates but can be disabled by removing its configuration. **Critically, Grafana cannot be enabled unless Prometheus is also enabled.**

### Key Principles

1. **Enabled by Default**: Grafana included in generated environment templates
2. **Opt-Out Available**: Users can remove Grafana section to disable it
3. **Hard Dependency**: Grafana requires Prometheus - validation enforced at config creation
4. **Configuration-Driven**: Service presence controlled by config section existence
5. **Service Isolation**: Grafana as independent docker-compose service
6. **Service Dependencies**: Proper startup ordering with `depends_on`
7. **Environment Variable Injection**: Admin credentials via `.env` file (following docker-compose pattern)
8. **Progressive Enhancement**: Start with basic visualization, dashboards can be added later

### Dependency Validation Rules

**At Environment Creation** (`create environment --env-file`):

```text
If grafana section is present AND prometheus section is absent:
  → Fail with clear error message
  → Error should explain: "Grafana requires Prometheus to be enabled"
  → Suggest: "Add prometheus section to your environment config or remove grafana section"

If grafana section is present AND prometheus section is present:
  → Validation passes
  → Both services will be included in docker-compose

If grafana section is absent:
  → Validation passes
  → Grafana not included in docker-compose
  → Prometheus can be enabled independently
```

## Specifications

### Grafana Service Enablement

**Environment Configuration Addition** (top-level, alongside `tracker` and `prometheus`):

```json
{
  "environment": { "name": "my-deployment" },
  "provider": { ... },
  "ssh_credentials": { ... },
  "tracker": { ... },
  "prometheus": {
    "scrape_interval": 15
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin"
  }
}
```

**Default Behavior in Generated Templates**:

- The `grafana` section is **included by default** when generating environment templates
- If the section is **present** in the environment config AND Prometheus is enabled → Grafana service is included in docker-compose
- If the section is **removed/absent** from the environment config → Grafana service is NOT included
- If the section is **present** but Prometheus is disabled → Validation error at environment creation

**Service Detection**:

- Presence of `grafana` section + presence of `prometheus` section → Both services enabled
- Presence of `grafana` section + absence of `prometheus` section → Validation error
- Absence of `grafana` section → Grafana disabled (Prometheus can be enabled independently)

**Configuration Model**: Uses `Option<GrafanaConfig>` in Rust domain model:

- `Some(GrafanaConfig { admin_user, admin_password })` → Grafana requested (default in generated templates)
- `None` (section removed from config) → Grafana disabled

**Validation Logic**:

```rust
// Pseudo-code for validation
fn validate_grafana_dependency(
    grafana: Option<GrafanaConfig>,
    prometheus: Option<PrometheusConfig>
) -> Result<(), ConfigError> {
    match (grafana, prometheus) {
        (Some(_), None) => Err(ConfigError::GrafanaRequiresPrometheus {
            help: "Grafana requires Prometheus for metrics visualization. \
                   Either enable Prometheus by adding the 'prometheus' section, \
                   or disable Grafana by removing the 'grafana' section."
        }),
        _ => Ok(())
    }
}
```

### Grafana Service Configuration

**Docker Compose Service**: Add Grafana container conditionally to stack (only when `grafana_config` is present in template context)

**Template Location**: `templates/docker-compose/docker-compose.yml.tera` (uses Tera conditionals to include/exclude Grafana service)

**Configuration** (from torrust-demo reference):

```yaml
{% if grafana_config %}
  grafana:
    image: grafana/grafana:11.4.0
    container_name: grafana
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_USER=${GF_SECURITY_ADMIN_USER}
      - GF_SECURITY_ADMIN_PASSWORD=${GF_SECURITY_ADMIN_PASSWORD}
    networks:
      - backend_network
    ports:
      - "3100:3000"
    volumes:
      - grafana_data:/var/lib/grafana
    logging:
      options:
        max-size: "10m"
        max-file: "10"
    depends_on:
      - prometheus
{% endif %}
```

**Image**: `grafana/grafana:11.4.0` (stable version as of December 2025)

**Ports**:

- Internal: `3000` (Grafana UI and API)
- Exposed to host: `3100:3000` (allows access to Grafana UI from host)
- **Note**: This port should not be exposed to the internet in production (use reverse proxy)
- **Port Mapping**: Uses 3100 on host to avoid conflicts with other services that commonly use 3000

**Volumes**:

- `grafana_data:/var/lib/grafana` - Named volume for persistent storage
  - Stores dashboards, datasources, user preferences
  - Survives container restarts and updates
  - **Pattern**: Unlike Prometheus which uses bind mount, Grafana uses named volume (standard Grafana practice)

**Environment Variables** (injected from `.env` file):

- `GF_SECURITY_ADMIN_USER` - Admin username (from `grafana.admin_user`, default: "admin")
- `GF_SECURITY_ADMIN_PASSWORD` - Admin password (from `grafana.admin_password`, default: "admin")
- **Pattern**: Follows environment variable injection pattern (see [docs/decisions/environment-variable-injection-in-docker-compose.md](../decisions/environment-variable-injection-in-docker-compose.md))

**Network**: `backend_network` (shared with tracker and Prometheus services)

**Service Dependencies**:

- **Depends on**: `prometheus` service (simple dependency, no health check)
- **Rationale**: Grafana will start after Prometheus container starts. Grafana UI will be accessible even if Prometheus is temporarily unavailable.

### Firewall Configuration

**Grafana UI Port Exposure**: Port 3100 must be opened in the firewall to allow public access to the Grafana web interface.

**Ansible Playbook**: `templates/ansible/configure-grafana-firewall.yml` (NEW)

```yaml
---
# Configure Grafana-specific firewall rules
# Opens port 3100 for Grafana UI access (conditionally, only when Grafana is enabled)

- name: Configure Grafana Firewall Rules
  hosts: all
  become: true
  vars_files:
    - "{{ playbook_dir }}/group_vars/all/variables.yml"
  tasks:
    - name: Allow Grafana UI port through firewall (port 3100)
      community.general.ufw:
        rule: allow
        port: "3100"
        proto: tcp
        comment: "Grafana UI"
      when: grafana_enabled | default(false) | bool
      notify: Reload UFW

  handlers:
    - name: Reload UFW
      community.general.ufw:
        state: reloaded
```

**Variables in `group_vars/all/variables.yml`**:

```yaml
# Grafana Configuration (conditional)
grafana_enabled: {{ 'true' if grafana_config else 'false' }}
```

**Template Location**: `templates/ansible/configure-grafana-firewall.yml.tera` (uses Tera to inject variables)

**Execution**: During `configure` command, after `ConfigureTrackerFirewall` step

**Conditional Behavior**:

- If Grafana is **enabled** in environment config → Port 3100 opened in firewall
- If Grafana is **disabled** (section removed) → Playbook tasks skipped (no port opened)
- If `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true` → Entire firewall configuration skipped

**Security Note**: This public exposure is **temporary** until HTTPS support with reverse proxy is implemented (roadmap task 6). Once a reverse proxy (like nginx) is added with HTTPS, this public port exposure will be removed, and Grafana will only be accessible through the proxy.

**Firewall Configuration Pattern**:

1. First, UFW closes all ports except SSH (which may be a custom port)
2. Then, individual service ports are opened conditionally based on enabled services:
   - SSH port (always, custom or default)
   - Tracker ports (if tracker configured)
   - Prometheus port (if Prometheus enabled)
   - Grafana port (if Grafana enabled)
   - Future services...

### Environment Configuration Schema Extensions

**Add to Domain Layer** (`src/domain/grafana/`):

```rust
// New file: src/domain/grafana/mod.rs
pub mod config;
pub use config::GrafanaConfig;

// New file: src/domain/grafana/config.rs
use crate::shared::secrets::Password;

/// Grafana metrics visualization configuration
///
/// Configures Grafana service for displaying tracker metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrafanaConfig {
    /// Grafana admin username
    pub admin_user: String,

    /// Grafana admin password (should be changed in production)
    /// Uses Password wrapper from secrecy crate for secure handling
    pub admin_password: Password,
}

impl Default for GrafanaConfig {
    fn default() -> Self {
        use secrecy::Secret;
        Self {
            admin_user: "admin".to_string(),
            admin_password: Secret::new("admin".to_string()),
        }
    }
}
```

**Security Note**: The `admin_password` field uses the `Password` type (alias for `Secret<String>`) from the `secrecy` crate. This provides:

- Automatic redaction in debug output (shows `[REDACTED]` instead of actual password)
- Memory zeroing when the value is dropped
- Explicit `.expose_secret()` calls required to access the plaintext value

See [`docs/decisions/secrecy-crate-for-sensitive-data.md`](../decisions/secrecy-crate-for-sensitive-data.md) and [`docs/contributing/secret-handling.md`](../contributing/secret-handling.md) for complete security guidelines.

**Add to Environment User Inputs** (`src/domain/environment/user_inputs.rs` or similar):

The environment's user inputs struct should have a top-level optional `grafana` field:

```rust
pub struct UserInputs {
    pub provider: ProviderConfig,
    pub ssh_credentials: SshCredentials,
    pub tracker: TrackerConfig,
    /// Prometheus metrics collection (optional third-party service)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prometheus: Option<PrometheusConfig>,
    /// Grafana metrics visualization (optional third-party service)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana: Option<GrafanaConfig>,
}
```

**JSON Schema Addition** (`schemas/environment-config.json`):

```json
{
  "grafana": {
    "type": "object",
    "description": "Grafana metrics visualization service configuration. Remove this section to disable Grafana. Requires Prometheus to be enabled.",
    "properties": {
      "admin_user": {
        "type": "string",
        "description": "Grafana admin username",
        "default": "admin",
        "minLength": 1
      },
      "admin_password": {
        "type": "string",
        "description": "Grafana admin password (change this in production!)",
        "default": "admin",
        "minLength": 1
      }
    },
    "required": ["admin_user", "admin_password"]
  }
}
```

**Template Generation**: When generating environment templates with `create environment --template`, include the `grafana` section by default at the top level (alongside `tracker` and `prometheus`).

### Docker Compose Template Extensions

**Conditional Grafana Service** (add to `templates/docker-compose/docker-compose.yml.tera`):

```yaml
services:
  # ... existing services (tracker, mysql, prometheus) ...

{% if grafana_config %}
  grafana:
    image: grafana/grafana:11.4.0
    container_name: grafana
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_USER=${GF_SECURITY_ADMIN_USER}
      - GF_SECURITY_ADMIN_PASSWORD=${GF_SECURITY_ADMIN_PASSWORD}
    networks:
      - backend_network
    ports:
      - "3100:3000"
    volumes:
      - grafana_data:/var/lib/grafana
    logging:
      options:
        max-size: "10m"
        max-file: "10"
    depends_on:
      - prometheus
{% endif %}
```

**Conditional Volume Declaration** (extend existing volumes section):

```yaml
volumes:
  tracker_data: {}
{% if database_driver == "mysql" %}
  mysql_data: {}
{% endif %}
{% if grafana_config %}
  grafana_data: {}
{% endif %}
```

**Environment Variables** (add to `templates/docker-compose/.env.tera`):

```tera
{% if grafana_config %}
# Grafana Configuration
GF_SECURITY_ADMIN_USER='{{ grafana_admin_user }}'
GF_SECURITY_ADMIN_PASSWORD='{{ grafana_admin_password }}'
{% endif %}
```

### Validation Implementation

**Location**: `src/application/command_handlers/create/config/validation/` or similar

**Validation Function**:

```rust
pub fn validate_grafana_prometheus_dependency(
    grafana: &Option<GrafanaConfig>,
    prometheus: &Option<PrometheusConfig>,
) -> Result<(), ConfigError> {
    match (grafana, prometheus) {
        (Some(_), None) => {
            Err(ConfigError::GrafanaRequiresPrometheus {
                help: "Grafana requires Prometheus to be enabled for metrics visualization.\n\
                       \n\
                       To fix this issue, choose one of the following:\n\
                       \n\
                       1. Enable Prometheus: Add the 'prometheus' section to your environment config:\n\
                          \"prometheus\": {\n\
                            \"scrape_interval\": 15\n\
                          }\n\
                       \n\
                       2. Disable Grafana: Remove the 'grafana' section from your environment config\n\
                       \n\
                       See docs/user-guide/README.md for more information."
                    .to_string(),
            })
        }
        _ => Ok(()),
    }
}
```

**Error Type** (add to error module):

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Grafana requires Prometheus to be enabled")]
    GrafanaRequiresPrometheus { help: String },
    // ... other errors
}
```

**Integration Point**: Call validation in environment creation handler before saving environment:

```rust
// In create environment handler
fn create_environment_from_config(config: UserInputs) -> Result<Environment, ConfigError> {
    // Validate Grafana-Prometheus dependency
    validate_grafana_prometheus_dependency(&config.grafana, &config.prometheus)?;

    // Continue with environment creation...
}
```

## Implementation Plan

### Phase 1: Environment Configuration & Validation

**Duration**: 1-2 hours

**Tasks**:

1. **Domain Layer** (`src/domain/grafana/`):

   - [x] Create `src/domain/grafana/mod.rs` module
   - [x] Create `src/domain/grafana/config.rs` with `GrafanaConfig` struct
   - [x] Add `admin_user` and `admin_password` fields (both String)
   - [x] Implement `Default` trait with default values ("admin"/"admin")
   - [x] Add `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq` derives
   - [x] Add comprehensive unit tests (5+ tests covering defaults, serialization, deserialization)

2. **Environment User Inputs Extension**:

   - [x] Add `grafana: Option<GrafanaConfig>` field to `UserInputs` struct
   - [x] Add `#[serde(skip_serializing_if = "Option::is_none")]` attribute
   - [x] Update all constructors and test fixtures to include `grafana` field
   - [x] Update JSON schema (`schemas/environment-config.json`) with Grafana section

3. **Validation Logic** (`src/application/command_handlers/create/config/validation/`):

   - [x] Create validation module if it doesn't exist
   - [x] Implement `validate_grafana_prometheus_dependency()` function
   - [x] Add `ConfigError::GrafanaRequiresPrometheus` error variant
   - [x] Add comprehensive error help text with fix instructions
   - [x] Write unit tests for all validation scenarios:
     - [x] Both enabled (valid)
     - [x] Both disabled (valid)
     - [x] Only Prometheus enabled (valid)
     - [x] Only Grafana enabled (invalid - should error)
   - [x] Integrate validation call in environment creation handler
   - [x] Run linters and tests

4. **Testing**:
   - [x] Run `cargo test` - all tests should pass
   - [x] Run `cargo run --bin linter all` - all linters should pass

### Phase 2: Docker Compose Integration

**Duration**: 2-3 hours

**Tasks**:

1. **Docker Compose Context** (`src/infrastructure/templating/docker_compose/template/wrappers/compose/context.rs`):

   - [x] Add `grafana_config: Option<GrafanaConfig>` field to `DockerComposeContext`
   - [x] Implement `with_grafana()` method for context builder pattern
   - [x] Add unit tests for Grafana context inclusion

2. **Environment Variables Context** (`src/infrastructure/templating/docker_compose/template/wrappers/env/context.rs`):

   - [x] Add optional Grafana fields to `EnvContext` struct:
     - `grafana_admin_user: Option<String>`
     - `grafana_admin_password: Option<String>` (plain String for template rendering)
   - [x] Implement `new_with_grafana()` constructor method
   - [x] Constructor must call `.expose_secret()` on `Password` to extract plaintext for template
   - [x] Add getters for Grafana fields
   - [x] Add unit tests for environment variable generation

   **Security Note**: The `admin_password` is stored as plain `String` in the context because Tera templates need the plaintext value. The `Password` wrapper is only used in the domain model and configuration. Call `.expose_secret()` when constructing the context from `GrafanaConfig`.

3. **Docker Compose Template** (`templates/docker-compose/docker-compose.yml.tera`):

   - [x] Add conditional Grafana service block with `{% if grafana_config %}`
   - [x] Configure Grafana service:
     - Image: `grafana/grafana:11.4.0`
     - Container name: `grafana`
     - Restart policy: `unless-stopped`
     - Environment variables from `.env` (GF_SECURITY_ADMIN_USER, GF_SECURITY_ADMIN_PASSWORD)
     - Network: `backend_network`
     - Port mapping: `3100:3000`
     - Volume: `grafana_data:/var/lib/grafana`
     - Logging configuration (10m max-size, 10 max-file)
     - Depends on: `prometheus`
   - [x] Add conditional volume declaration for `grafana_data`

4. **Environment Template** (`templates/docker-compose/.env.tera`):

   - [x] Add conditional Grafana section with `{% if grafana_config %}`
   - [x] Add environment variables:
     - `GF_SECURITY_ADMIN_USER='{{ grafana_admin_user }}'`
     - `GF_SECURITY_ADMIN_PASSWORD='{{ grafana_admin_password }}'`

5. **Release Command Integration** (`src/application/command_handlers/release/`):

   - [x] Update docker-compose rendering step to include Grafana context
   - [x] Pass Grafana config to `DockerComposeContext::with_grafana()` when present
   - [x] Pass Grafana credentials to `EnvContext` when present

6. **Firewall Configuration** (NEW):

   - [x] Create Ansible playbook: `templates/ansible/configure-grafana-firewall.yml`
   - [x] ~~Add `grafana_enabled` variable to Ansible variables template~~ (NOT NEEDED - conditional at step level)
   - [x] Register playbook in `ProjectGenerator` (see `templates.md` for static template registration)
   - [x] Create `ConfigureGrafanaFirewallStep` in `src/application/steps/system/configure_grafana_firewall.rs`:
     - Implement `Step` trait with `execute()` method
     - Execute `configure-grafana-firewall.yml` playbook via Ansible client
     - Return appropriate error on failure
   - [x] Add `ConfigureGrafanaFirewall` variant to `ConfigureStep` enum in `src/domain/environment/state/configure_failed.rs`
   - [x] Integrate step in `ConfigureCommandHandler::execute_configuration_with_tracking()`:
     - Add after `ConfigureTrackerFirewall` step
     - Check `skip_firewall` flag (respect `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER`)
     - Skip with info log if firewall configuration is disabled
     - Execute `ConfigureGrafanaFirewallStep` only when Grafana is enabled
   - [x] Add unit tests for `ConfigureGrafanaFirewallStep`

7. **Testing**:
   - [x] Add unit tests for Grafana service rendering in docker-compose template
   - [x] Test conditional rendering (with/without Grafana)
   - [x] Test environment variable generation
   - [x] Test volume declaration
   - [x] Test firewall configuration playbook rendering
   - [x] Test `ConfigureGrafanaFirewallStep` execution
   - [x] Run `cargo test` - all tests should pass (1555 tests)
   - [x] Run `cargo run --bin linter all` - all linters should pass

### Phase 3: Testing & Verification

**Duration**: 2-3 hours

**Tasks**:

1. **E2E Test Configuration**:

   - [ ] Create test environment config with Grafana enabled: `envs/e2e-deployment-with-grafana.json`
   - [ ] Create test environment config without Grafana: `envs/e2e-deployment-no-grafana.json` (already exists)
   - [ ] Create test environment config to test validation error: `envs/e2e-deployment-grafana-no-prometheus.json`

2. **E2E Validation Extension** (`tests/e2e/validators/`):

   - [ ] Extend `ServiceValidation` struct with `grafana: bool` field
   - [ ] Create `GrafanaValidator` to verify Grafana deployment:
     - [ ] Check Grafana container is running (`docker ps`)
     - [ ] Verify Grafana UI is accessible (curl http://localhost:3100)
     - [ ] Verify admin credentials work (login test)
     - [ ] **Firewall Verification**: Check port 3100 is open in UFW (`sudo ufw status`)
     - [ ] **Firewall Verification**: Verify external access to Grafana UI (curl from test machine)
   - [ ] Update `run_release_validation()` to include Grafana checks when enabled

3. **E2E Test Updates**:

   - [ ] Update `e2e-deployment-workflow-tests` to test Grafana scenarios:
     - [ ] Test with Grafana enabled (full stack)
     - [ ] Test without Grafana (Prometheus only)
     - [ ] Test validation error (Grafana without Prometheus)
   - [ ] Run E2E tests: `cargo run --bin e2e-deployment-workflow-tests`
   - [ ] Verify all tests pass

4. **Manual E2E Testing**:

   - [ ] Create manual test environment: `envs/manual-test-grafana.json`
   - [ ] Run full deployment workflow:
     - [ ] `create environment --env-file envs/manual-test-grafana.json`
     - [ ] `provision`
     - [ ] `configure`
     - [ ] `release`
     - [ ] `run`
   - [ ] Verify Grafana deployment:
     - [ ] Check Grafana container running: `docker ps`
     - [ ] **Verify firewall configuration**:
       - [ ] SSH to VM: `ssh user@<vm-ip>`
       - [ ] Check UFW status: `sudo ufw status`
       - [ ] Verify port 3100 is allowed: Look for "3100/tcp" with "ALLOW" in UFW output
       - [ ] Exit SSH
     - [ ] **Verify external access**:
       - [ ] Access Grafana UI from local machine: `http://<vm-ip>:3100`
       - [ ] Verify UI loads successfully (Grafana login page appears)
     - [ ] Login with admin credentials
     - [ ] Add Prometheus datasource manually:
       - URL: `http://prometheus:9090`
       - Access: "Server (default)"
     - [ ] Verify Prometheus connection works ("Save & Test" button)
     - [ ] Import basic dashboard (optional)
   - [ ] Test dependency validation:
     - [ ] Create config with Grafana but without Prometheus
     - [ ] Verify environment creation fails with clear error message
     - [ ] Verify error suggests fixing by adding Prometheus or removing Grafana
   - [ ] Document manual testing steps in `docs/e2e-testing/manual/grafana-verification.md`

5. **Final Verification**:
   - [ ] Run all linters: `cargo run --bin linter all`
   - [ ] Run all unit tests: `cargo test`
   - [ ] Run E2E tests: `cargo run --bin e2e-deployment-workflow-tests`
   - [ ] Verify pre-commit checks pass: `./scripts/pre-commit.sh`

### Phase 4: Documentation

**Duration**: 1-2 hours

**Tasks**:

1. **Create ADR** (`docs/decisions/grafana-integration-pattern.md`):

   - [ ] Document enabled-by-default approach (consistent with Prometheus)
   - [ ] Explain Grafana-Prometheus dependency and validation
   - [ ] Document why no separate config files (uses defaults + env vars)
   - [ ] List alternatives considered (opt-in, mandatory, separate provisioning)
   - [ ] Document consequences for users and maintainers

2. **Update User Guide** (`docs/user-guide/README.md`):

   - [ ] Add Grafana configuration section
   - [ ] Document `grafana.admin_user` and `grafana.admin_password` parameters
   - [ ] Explain enabled-by-default behavior and opt-out pattern
   - [ ] Document Prometheus dependency requirement
   - [ ] Instructions for accessing Grafana UI (port 3100)
   - [ ] Instructions for adding Prometheus datasource
   - [ ] Link to manual verification guide
   - [ ] Add security warning about changing default password

3. **Create Manual Verification Guide** (`docs/e2e-testing/manual/grafana-verification.md`):

   - [ ] Document step-by-step Grafana verification process
   - [ ] Include exact commands and expected outputs
   - [ ] Add screenshots or ASCII diagrams for key steps (optional)
   - [ ] Document how to add Prometheus datasource
   - [ ] Document troubleshooting steps for common issues
   - [ ] Provide success criteria checklist

4. **Update Project Dictionary** (`project-words.txt`):

   - [ ] Add Grafana-related technical terms

5. **Final Documentation Review**:
   - [ ] Run markdown linter: `cargo run --bin linter markdown`
   - [ ] Verify all links work
   - [ ] Review for clarity and completeness

## Acceptance Criteria

### Functional Requirements

- [ ] Grafana service is included in docker-compose stack when `grafana` section is present in environment config
- [ ] Grafana service is excluded from docker-compose stack when `grafana` section is absent
- [ ] Environment creation fails with clear error if Grafana is enabled but Prometheus is disabled
- [ ] Grafana container starts successfully and UI is accessible on port 3100
- [ ] Grafana admin credentials from config work for login
- [ ] Grafana can connect to Prometheus service for metrics visualization
- [ ] Named volume `grafana_data` is created and persists across container restarts
- [ ] Service dependencies correctly enforced (Grafana starts after Prometheus)
- [ ] **Firewall**: Port 3100 is opened in UFW when Grafana is enabled during `configure` command
- [ ] **Firewall**: Port 3100 is NOT opened when Grafana is disabled (section removed from config)
- [ ] **Firewall**: Grafana UI is accessible externally from host machine (public internet access)
- [ ] **Firewall**: Firewall configuration skipped when `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true`

### Validation Requirements

- [ ] Validation logic correctly detects missing Prometheus when Grafana is enabled
- [ ] Error message clearly explains the problem and provides fix instructions
- [ ] Validation passes when both services enabled
- [ ] Validation passes when both services disabled
- [ ] Validation passes when only Prometheus enabled (Grafana is optional)

### Configuration Requirements

- [ ] Generated environment templates include `grafana` section by default
- [ ] Grafana credentials injected via `.env` file (not hardcoded in docker-compose)
- [ ] Default admin credentials are "admin"/"admin" (user should change in production)
- [ ] JSON schema includes Grafana configuration with proper validation rules

### Testing Requirements

- [ ] Unit tests cover:
  - [ ] GrafanaConfig domain model (defaults, serialization, deserialization)
  - [ ] Grafana-Prometheus dependency validation (all scenarios)
  - [ ] Docker Compose context with Grafana
  - [ ] Environment variable context with Grafana
  - [ ] Template rendering with/without Grafana
- [ ] E2E tests verify:
  - [ ] Full deployment with Grafana enabled
  - [ ] Deployment without Grafana
  - [ ] Validation error for Grafana without Prometheus
  - [ ] Grafana container running and accessible
- [ ] Manual testing confirms:
  - [ ] Grafana UI accessible and functional
  - [ ] Admin login works with configured credentials
  - [ ] Prometheus datasource can be added manually
  - [ ] Dashboards can be created/imported

### Documentation Requirements

- [ ] ADR documents Grafana integration pattern and design decisions
- [ ] User guide explains how to configure and access Grafana
- [ ] Manual verification guide provides step-by-step testing instructions
- [ ] Security warnings about changing default passwords
- [ ] Clear explanation of Prometheus dependency

### Quality Requirements

- [ ] All pre-commit checks pass: `./scripts/pre-commit.sh`
  - [ ] No unused dependencies (`cargo machete`)
  - [ ] All linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
  - [ ] All unit tests pass (`cargo test` - 1500+ tests)
  - [ ] All E2E tests pass (`cargo run --bin e2e-deployment-workflow-tests`)

**Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

## Related Documentation

### Architecture & Patterns

- [Codebase Architecture](../codebase-architecture.md) - DDD layers and module organization
- [Template System Architecture](../technical/template-system-architecture.md) - Project Generator pattern
- [DDD Layer Placement](../contributing/ddd-layer-placement.md) - Where to place code

### Configuration & Templates

- [Templates Guide](../contributing/templates.md) - Working with Tera templates
- [Environment Variable Injection](../decisions/environment-variable-injection-in-docker-compose.md) - Docker Compose environment variable pattern

### Related Features

- [Prometheus Slice](./238-prometheus-slice-release-run-commands.md) - Similar implementation pattern
- [Prometheus Integration Pattern ADR](../decisions/prometheus-integration-pattern.md) - Enabled-by-default approach

### Testing

- [E2E Testing Guide](../e2e-testing/README.md) - E2E test patterns
- [Manual Testing](../e2e-testing/manual-testing.md) - Step-by-step manual testing workflow

### Reference Implementation

- [torrust-demo compose.yaml](https://github.com/torrust/torrust-demo/blob/main/compose.yaml) - Reference Grafana configuration

## Sample Dashboards

The Torrust Live Demo uses two Grafana dashboards that visualize tracker metrics:

1. **Stats Dashboard** (`stats.json`)

   - Uses the `/api/v1/stats` tracker endpoint
   - Displays: Completed, Torrents, Seeders, Leechers
   - Shows UDP4 request rates (connections, announces, scrapes, errors)
   - Includes average processing times and banned request metrics
   - Source: [torrust-demo/share/grafana/dashboards/stats.json](https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/stats.json)

2. **Metrics Dashboard** (`metrics.json`)
   - Uses the `/api/v1/metrics` tracker endpoint (Prometheus format)
   - Displays: Completed, Torrents, Seeders, Leechers
   - Shows detailed UDP4 metrics per request type
   - Includes request/response rates and performance metrics
   - Source: [torrust-demo/share/grafana/dashboards/metrics.json](https://github.com/torrust/torrust-demo/blob/main/share/grafana/dashboards/metrics.json)

Both dashboards are used in the [Torrust Live Demo](https://index.torrust-demo.com/torrents).

### Integration Strategy

**NOT Auto-Provisioned**: These dashboards are provided as examples for end-users but are NOT automatically configured during deployment. This follows the principle of minimal setup - users can manually import them if desired.

**User Documentation**: Create `docs/user-guide/services/grafana/` directory with:

- `README.md` - Grafana overview, accessing the UI, adding Prometheus datasource
- `dashboards/stats.json` - Copy of the stats dashboard from torrust-demo
- `dashboards/metrics.json` - Copy of the metrics dashboard from torrust-demo
- `dashboard-import-guide.md` - Step-by-step instructions for importing dashboards into Grafana UI

**Implementation Tasks** (add to Phase 4: Documentation):

- [ ] Create `docs/user-guide/services/grafana/` directory structure
- [ ] Download and save both dashboard JSON files
- [ ] Create dashboard import guide with screenshots/steps
- [ ] Link to sample dashboards from main Grafana documentation

## Notes

### Implementation Considerations

1. **No Separate Configuration Files**: Unlike Prometheus (which requires `prometheus.yml`), Grafana doesn't need separate configuration file templates. It works with defaults and can be configured through:

   - Environment variables (admin credentials)
   - UI configuration (datasources, dashboards)
   - Optional mounted files (advanced use case, not part of this issue)

2. **Datasource Configuration**: This implementation does not auto-configure the Prometheus datasource. Users must manually add it through the Grafana UI:

   - This keeps the implementation simple
   - Provides flexibility for advanced users
   - Future enhancement could add auto-provisioning via config files

3. **Dashboard Provisioning**: No default dashboards are included in this slice. Users can:

   - Create dashboards through the UI
   - Import community dashboards
   - Future enhancement could add default tracker dashboard

4. **Security Considerations**:

   - Default credentials (admin/admin) are insecure - users should change them
   - Documentation must emphasize changing credentials in production
   - Port 3100 should not be exposed to internet (use reverse proxy)
   - Consider adding password strength validation in future enhancement

5. **Testing Strategy**:
   - Automated tests verify service is running and UI is accessible
   - Manual testing verifies actual functionality (login, datasource, dashboards)
   - Both test levels are important for comprehensive validation

### Future Enhancements

Potential improvements beyond this slice:

- Auto-provision Prometheus datasource via config file
- Include default tracker metrics dashboard
- Add support for custom dashboard provisioning
- Implement password strength validation
- Add support for Grafana plugins
- Configure SMTP for alerting notifications

These enhancements should be separate issues to keep this slice focused and deliverable.

## Related Issues

- Parent: [#216](https://github.com/torrust/torrust-tracker-deployer/issues/216) - Implement ReleaseCommand and RunCommand with vertical slices
- Depends on: [#238](https://github.com/torrust/torrust-tracker-deployer/issues/238) - Prometheus Slice (completed)
- Related: [#232](https://github.com/torrust/torrust-tracker-deployer/issues/232) - MySQL Slice (similar optional service pattern)
