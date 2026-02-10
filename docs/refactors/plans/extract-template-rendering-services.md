# Extract Template Rendering Application Services

## 📋 Overview

Extract template rendering logic from the render command handler and the release step wrappers into shared application-layer rendering services. This eliminates significant code duplication between two independent code paths that perform the same rendering operations.

**Target Files:**

- `src/application/services/mod.rs` - New `rendering` submodule
- `src/application/services/rendering/mod.rs` - Module index (new)
- `src/application/services/rendering/ansible.rs` - Moved from `ansible_template_service.rs` (renamed)
- `src/application/services/rendering/opentofu.rs` - New service
- `src/application/services/rendering/tracker.rs` - New service
- `src/application/services/rendering/prometheus.rs` - New service
- `src/application/services/rendering/grafana.rs` - New service
- `src/application/services/rendering/docker_compose.rs` - New service
- `src/application/services/rendering/caddy.rs` - New service
- `src/application/services/rendering/backup.rs` - New service
- `src/application/command_handlers/render/handler.rs` - Simplified (consumer)
- `src/application/steps/rendering/*.rs` - Simplified (consumers)
- `src/application/command_handlers/release/steps/*.rs` - Simplified (consumers)

**Scope:**

- Create application-level rendering services for all 8 template types
- Move the existing `AnsibleTemplateService` into the new `rendering` module (renamed to `AnsibleTemplateRenderingService`)
- Refactor the render command handler to delegate to services instead of duplicating rendering logic
- Refactor the rendering Steps to delegate core logic to services while retaining deployment tracing
- Remove duplicated context-building logic (especially Docker Compose, Caddy, Backup)

## 📊 Progress Tracking

**Total Active Proposals**: 5
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 5

### Phase Summary

- **Phase 0 - Create rendering module and move Ansible (Medium Impact, Low Effort)**: ⏳ 0/1 completed (0%)
- **Phase 1 - Simple rendering services (High Impact, Low Effort)**: ⏳ 0/1 completed (0%)
- **Phase 2 - Complex rendering services (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)
- **Phase 3 - Refactor render handler to use services (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)
- **Phase 4 - Refactor Steps to use services (Medium Impact, Medium Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

None

### Postponed Proposals

None

## 🎯 Key Problems Identified

### 1. Duplicated Rendering Logic Across Two Code Paths

The render command handler (`src/application/command_handlers/render/handler.rs`) and the rendering Steps (`src/application/steps/rendering/*.rs`) both contain the same core rendering logic: create template manager, create infrastructure generator, build context from domain types, call render. Each template type is duplicated.

For example, there are two independent implementations for:

- Docker Compose context building (~130 lines each): `has_caddy_enabled()`, `build_tracker_config()`, SQLite/MySQL context creation, optional service configuration
- Caddy context building (~40 lines each): `build_caddy_context()` with TLS service assembly
- Backup database config conversion (~20 lines each): `convert_database_config_to_backup()`

### 2. Render Handler Is Overly Large

The render command handler is ~936 lines. Approximately 600 lines are template rendering methods that should not live in a command handler. The handler's responsibility should be input validation, workflow orchestration, and error mapping — not template context assembly.

### 3. Steps Conflate Deployment Tracing with Rendering Logic

The rendering Steps (e.g., `RenderDockerComposeTemplatesStep`) mix two concerns:

- **Core rendering logic**: context building, generator invocation (reusable)
- **Deployment pipeline concerns**: `#[instrument]` spans, step-level logging, `Arc<Environment<S>>` ownership (deployment-specific)

The render command doesn't need the deployment pipeline concerns but currently duplicates the rendering logic to avoid them.

### 4. Inconsistent Service Location

`AnsibleTemplateService` already solves this problem for Ansible — it lives in `src/application/services/` and is shared by Provision, Register, and Render handlers. But the other 7 template types lack equivalent services, forcing each consumer to duplicate the logic.

## 🚀 Refactoring Phases

---

## Phase 0: Create Rendering Module and Move Ansible (Foundation)

Establish the `src/application/services/rendering/` module structure and relocate the existing `AnsibleTemplateService` as the first resident, setting the pattern for all subsequent services.

### Proposal 0: Establish rendering module and relocate Ansible service

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None

#### Problem

`AnsibleTemplateService` currently lives at `src/application/services/ansible_template_service.rs` as a standalone module. As we add 7 more rendering services, they need a common home.

#### Proposed Solution

1. Create `src/application/services/rendering/mod.rs`
2. Move `ansible_template_service.rs` to `rendering/ansible.rs`, renaming the type to `AnsibleTemplateRenderingService` for consistency
3. Update `src/application/services/mod.rs` to expose the `rendering` submodule and add backward-compatible type aliases during the transition
4. Update all consumers to use the new import path

**New module structure:**

```text
src/application/services/
├── mod.rs                  # Exposes rendering module + backward-compat aliases
└── rendering/
    ├── mod.rs              # Module index with re-exports
    └── ansible.rs          # AnsibleTemplateRenderingService (moved + renamed)
```

**`services/mod.rs` with backward-compatible aliases:**

```rust
pub mod rendering;

// Backward-compatible aliases (remove once all consumers are migrated)
pub use rendering::AnsibleTemplateRenderingService as AnsibleTemplateService;
pub use rendering::AnsibleTemplateRenderingServiceError as AnsibleTemplateServiceError;
```

#### Rationale

Moving the existing service first validates the module structure with zero risk — the type already exists and works. Backward-compatible aliases ensure no breaking changes during migration.

#### Benefits

- ✅ Establishes module structure for all subsequent services
- ✅ Zero functional change — only moves and renames
- ✅ Backward-compatible during transition

#### Implementation Checklist

- [ ] Create `src/application/services/rendering/mod.rs`
- [ ] Create `src/application/services/rendering/ansible.rs` with renamed types
- [ ] Delete `src/application/services/ansible_template_service.rs`
- [ ] Update `src/application/services/mod.rs` with backward-compatible aliases
- [ ] Update all import paths (provision, register, render handlers + provision errors)
- [ ] Remove backward-compatible aliases once all imports are updated
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Compile-time verification — all existing tests and the pre-commit script should pass without changes.

---

## Phase 1: Simple Rendering Services (Quick Wins)

Create rendering services for template types with straightforward context requirements: OpenTofu, Tracker, Prometheus, and Grafana. Each takes explicit domain config arguments and delegates to the corresponding infrastructure generator.

### Proposal 1: Create OpenTofu, Tracker, Prometheus, and Grafana rendering services

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P1
**Depends On**: Proposal 0

#### Problem

The render handler contains 4 simple rendering methods that duplicate logic from their corresponding Steps:

- `render_opentofu_templates()` — duplicates `RenderOpenTofuTemplatesStep`
- `render_tracker_templates()` — duplicates `RenderTrackerTemplatesStep`
- `render_prometheus_templates()` — duplicates `RenderPrometheusTemplatesStep`
- `render_grafana_templates()` — duplicates `RenderGrafanaTemplatesStep`

Each follows the same pattern: create template manager, create generator, extract config, call render.

#### Proposed Solution

Create 4 services following the `AnsibleTemplateRenderingService` precedent. Each service:

- Takes `templates_dir`, `build_dir`, and `clock` via a `from_paths()` factory
- Has a `render()` method accepting explicit domain config types (not `Environment<S>`)
- Returns a thin error type wrapping the infrastructure generator error

**Example — `TrackerTemplateRenderingService`:**

```rust
pub struct TrackerTemplateRenderingService {
    templates_dir: PathBuf,
    build_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

impl TrackerTemplateRenderingService {
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self { ... }

    pub fn render(
        &self,
        tracker_config: &TrackerConfig,
    ) -> Result<PathBuf, TemplateRenderingError> { ... }
}
```

**Example — `PrometheusTemplateRenderingService`:**

```rust
pub fn render(
    &self,
    prometheus_config: &PrometheusConfig,
    tracker_config: &TrackerConfig,
) -> Result<PathBuf, TemplateRenderingError> { ... }
```

**Example — `GrafanaTemplateRenderingService`:**

```rust
pub fn render(
    &self,
    prometheus_config: &PrometheusConfig,
) -> Result<PathBuf, TemplateRenderingError> { ... }
```

**Example — `OpenTofuTemplateRenderingService`:**

```rust
pub fn from_params(
    templates_dir: PathBuf,
    build_dir: PathBuf,
    ssh_credentials: SshCredentials,
    ssh_port: u16,
    instance_name: InstanceName,
    provider_config: ProviderConfig,
    clock: Arc<dyn Clock>,
) -> Self { ... }

pub async fn render(&self) -> Result<(), TemplateRenderingError> { ... }
```

Note: OpenTofu has more constructor parameters because the generator itself requires provider config, SSH credentials, etc.

#### Rationale

These 4 services are straightforward — their `render()` methods have small, well-defined signatures with 1-2 domain config arguments. Starting here validates the service pattern before tackling the more complex cases.

#### Benefits

- ✅ Removes ~100 lines of duplication from render handler
- ✅ Establishes clear service pattern for the team
- ✅ Each service is independently testable with explicit inputs

#### Implementation Checklist

- [ ] Create `src/application/services/rendering/opentofu.rs`
- [ ] Create `src/application/services/rendering/tracker.rs`
- [ ] Create `src/application/services/rendering/prometheus.rs`
- [ ] Create `src/application/services/rendering/grafana.rs`
- [ ] Update `rendering/mod.rs` with re-exports
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Unit tests for each service using test fixtures (TempDir for templates/build dirs, MockClock). Existing integration tests provide coverage through the render command.

---

## Phase 2: Complex Rendering Services (Highest Code Savings)

Create rendering services for Docker Compose, Caddy, and Backup — the template types with complex context-building logic that is currently duplicated most severely.

### Proposal 2: Create Docker Compose, Caddy, and Backup rendering services

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P1
**Depends On**: Proposal 0

#### Problem

Docker Compose context building is ~130 lines of pure logic: database-variant selection, topology computation, optional service configuration (Prometheus, Grafana, Backup, Caddy), Grafana env context, MySQL setup config. This logic exists in both:

- `RenderDockerComposeTemplatesStep::execute()` (steps/rendering/docker_compose_templates.rs)
- `RenderCommandHandler::render_docker_compose_templates()` (render/handler.rs)

Similarly:

- `build_caddy_context()` with TLS service assembly (~40 lines) is duplicated
- `convert_database_config_to_backup()` and backup context building (~30 lines) is duplicated

#### Proposed Solution

**`DockerComposeTemplateRenderingService`:**

This service takes `&UserInputs` plus `admin_token` since Docker Compose needs almost all configuration:

```rust
pub struct DockerComposeTemplateRenderingService {
    templates_dir: PathBuf,
    build_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

impl DockerComposeTemplateRenderingService {
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self { ... }

    pub async fn render(
        &self,
        user_inputs: &UserInputs,
        admin_token: &str,
    ) -> Result<PathBuf, TemplateRenderingError> {
        // All context-building logic moves here:
        // - build_tracker_config (topology + TrackerServiceContext)
        // - has_caddy_enabled check
        // - SQLite vs MySQL context creation
        // - apply_prometheus_config, apply_grafana_config, etc.
        // - apply_grafana_env_context
    }
}
```

**`CaddyTemplateRenderingService`:**

```rust
pub fn render(
    &self,
    user_inputs: &UserInputs,
) -> Result<Option<PathBuf>, TemplateRenderingError> {
    // Returns None if HTTPS not configured or no TLS services
    // Builds CaddyContext from user_inputs (admin_email, TLS domains, ports)
}
```

**`BackupTemplateRenderingService`:**

```rust
pub async fn render(
    &self,
    backup_config: &BackupConfig,
    database_config: &DatabaseConfig,
    created_at: DateTime<Utc>,
) -> Result<Option<PathBuf>, TemplateRenderingError> {
    // Converts DatabaseConfig to BackupDatabaseConfig
    // Builds BackupContext
    // Calls generator.render(&context, schedule)
}
```

#### Rationale

Docker Compose is the single largest source of duplication (~130 lines duplicated). Moving it into a service provides the highest code savings of any single change. Caddy and Backup are included because they share the same "complex context building" characteristic.

#### Benefits

- ✅ Removes ~250 lines of duplicated context-building logic
- ✅ Single source of truth for Docker Compose topology computation
- ✅ `has_caddy_enabled()`, `build_caddy_context()`, `convert_database_config_to_backup()` exist once

#### Implementation Checklist

- [ ] Create `src/application/services/rendering/docker_compose.rs`
- [ ] Create `src/application/services/rendering/caddy.rs`
- [ ] Create `src/application/services/rendering/backup.rs`
- [ ] Move context-building logic from Steps into services
- [ ] Move context-building logic from render handler into services
- [ ] Update `rendering/mod.rs` with re-exports
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Unit tests for Docker Compose service with both SQLite and MySQL configurations. Unit tests for Caddy with/without HTTPS and with/without TLS services. Unit tests for Backup with SQLite and MySQL database configs.

---

## Phase 3: Refactor Render Handler to Use Services (Simplification)

Replace the render handler's inline rendering methods with service calls, drastically reducing the handler.

### Proposal 3: Simplify render command handler

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P2
**Depends On**: Proposals 1 and 2

#### Problem

The render handler is ~936 lines. Approximately 600 lines are rendering methods that duplicate logic now available in the services. The handler should focus on:

- Input mode selection (env-name vs env-file)
- State validation (Created state only)
- IP address parsing
- Orchestrating the rendering sequence
- Error mapping to presentation-layer errors

#### Proposed Solution

Each rendering method in the handler becomes a thin delegation:

```rust
async fn render_all_templates(
    &self,
    environment: &Environment<Created>,
    target_ip: IpAddr,
) -> Result<(), RenderCommandHandlerError> {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let templates_dir = environment.templates_dir();
    let build_dir = environment.build_dir();

    // 1. OpenTofu
    OpenTofuTemplateRenderingService::from_params(...)
        .render().await
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed { reason: e.to_string() })?;

    // 2. Ansible
    AnsibleTemplateRenderingService::from_paths(templates_dir.clone(), build_dir.clone(), clock.clone())
        .render_templates(&environment.context().user_inputs, target_ip, None).await
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed { reason: e.to_string() })?;

    // 3. Docker Compose
    DockerComposeTemplateRenderingService::from_paths(templates_dir.clone(), build_dir.clone(), clock.clone())
        .render(&environment.context().user_inputs, &environment.admin_token().to_string()).await
        .map_err(|e| RenderCommandHandlerError::TemplateRenderingFailed { reason: e.to_string() })?;

    // 4-8. Tracker, Prometheus, Grafana, Caddy, Backup (similar pattern)
    // ...

    Ok(())
}
```

This reduces `render_all_templates` + its 8 helper methods from ~600 lines to ~80 lines.

#### Rationale

The handler becomes focused on its actual responsibility (orchestration and error mapping) instead of containing template rendering implementation details.

#### Benefits

- ✅ Handler reduced from ~936 lines to ~350 lines
- ✅ Clear separation: handler orchestrates, services render
- ✅ Removing `has_caddy_enabled()`, `build_caddy_context()`, etc. from handler
- ✅ Easier to test handler logic in isolation (mock services)

#### Implementation Checklist

- [ ] Replace `render_opentofu_templates()` with service call
- [ ] Replace `render_ansible_templates()` — already uses service, just update import
- [ ] Replace `render_docker_compose_templates()` with service call
- [ ] Replace `render_tracker_templates()` with service call
- [ ] Replace `render_prometheus_templates()` with service call
- [ ] Replace `render_grafana_templates()` with service call
- [ ] Replace `render_caddy_templates()` with service call
- [ ] Replace `render_backup_templates()` with service call
- [ ] Remove `has_caddy_enabled()` and `build_caddy_context()` from handler
- [ ] Remove unused infrastructure imports from handler
- [ ] Verify render command still works with manual test
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

Run the existing render command manual test: `cargo run -- render --env-name lxd-local-example --instance-ip 10.0.0.1`. Verify output directory contains all expected template files. Existing unit tests for IP parsing and input modes remain unchanged.

---

## Phase 4: Refactor Steps to Use Services (Optional, Clean Architecture)

Simplify the rendering Steps to delegate their core logic to the new services while retaining their deployment-specific concerns (tracing spans, step-level logging).

### Proposal 4: Simplify rendering Steps

**Status**: ⏳ Not Started
**Impact**: 🟢🟢 Medium
**Effort**: 🔵🔵 Medium
**Priority**: P3
**Depends On**: Proposals 1 and 2

#### Problem

After services exist, the Steps contain duplicated rendering logic. Steps should become thin wrappers that add deployment pipeline concerns on top of service calls:

- `#[instrument]` tracing spans
- Step-level `info!()` logging ("Rendering X templates", "X templates rendered successfully")
- `Option<PathBuf>` return values with skip logging
- `Arc<Environment<S>>` consumption pattern

#### Proposed Solution

Steps delegate core rendering to services while keeping deployment tracing:

```rust
impl<S> RenderPrometheusTemplatesStep<S> {
    pub fn execute(&self) -> Result<Option<PathBuf>, PrometheusProjectGeneratorError> {
        let Some(prometheus_config) = self.environment.context().user_inputs.prometheus() else {
            info!(step = "render_prometheus_templates", status = "skipped", ...);
            return Ok(None);
        };

        info!(step = "render_prometheus_templates", "Rendering Prometheus configuration templates");

        let service = PrometheusTemplateRenderingService::from_paths(
            self.environment.templates_dir(),
            self.build_dir.clone(),
            self.clock.clone(),
        );

        let tracker_config = self.environment.context().user_inputs.tracker();
        let prometheus_build_dir = service.render(prometheus_config, tracker_config)?;

        info!(step = "render_prometheus_templates", status = "success", ...);

        Ok(Some(prometheus_build_dir))
    }
}
```

#### Rationale

This phase is lower priority because the Steps already work correctly. The benefit is architectural consistency — Steps become pure deployment pipeline wrappers with no rendering logic of their own. It also ensures that any future bug fix to rendering logic only needs to happen in one place (the service).

#### Benefits

- ✅ Steps become thin deployment pipeline wrappers (~20 lines each)
- ✅ Rendering logic exists in only one place per template type
- ✅ Easier to maintain — bug fixes propagate automatically
- ✅ Steps still own their deployment-specific concerns (tracing, logging)

#### Implementation Checklist

- [ ] Refactor `RenderTrackerTemplatesStep` to use `TrackerTemplateRenderingService`
- [ ] Refactor `RenderPrometheusTemplatesStep` to use `PrometheusTemplateRenderingService`
- [ ] Refactor `RenderGrafanaTemplatesStep` to use `GrafanaTemplateRenderingService`
- [ ] Refactor `RenderDockerComposeTemplatesStep` to use `DockerComposeTemplateRenderingService`
- [ ] Refactor `RenderCaddyTemplatesStep` to use `CaddyTemplateRenderingService`
- [ ] Refactor `RenderBackupTemplatesStep` to use `BackupTemplateRenderingService`
- [ ] Remove duplicated helper methods from Steps
- [ ] Verify all tests pass (step-level unit tests + E2E)
- [ ] Run linter and fix any issues

#### Testing Strategy

Existing step-level unit tests should continue to pass unchanged. E2E tests (both split variants) verify the full release pipeline still works. Run `./scripts/pre-commit.sh` to validate everything.

---

## 📈 Timeline

- **Start Date**: TBD
- **Estimated Duration**: 2-3 sessions
  - Phase 0: ~30 minutes (move + rename)
  - Phase 1: ~1-2 hours (4 simple services)
  - Phase 2: ~2-3 hours (3 complex services, especially Docker Compose)
  - Phase 3: ~1-2 hours (refactor render handler)
  - Phase 4: ~2 hours (refactor 6 Steps)

## 🔍 Review Process

### Approval Criteria

- [ ] Plan reviewed by maintainer
- [ ] Technical feasibility validated
- [ ] Aligns with DDD layer placement guide
- [ ] Services take explicit domain types, not `Environment<S>`

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing
- [ ] All linters passing (`./scripts/pre-commit.sh`)
- [ ] Render command manual test passes
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../../development-principles.md) - Core principles guiding refactoring
- [DDD Layer Placement](../../contributing/ddd-layer-placement.md) - Where rendering services belong
- [Module Organization](../../contributing/module-organization.md) - Code organization conventions
- [Codebase Architecture](../../codebase-architecture.md) - Three-level command/step/action pattern

## 💡 Notes

### Why Application Layer, Not Infrastructure

Template rendering services belong in the application layer because they:

- **Orchestrate**: bridge multiple domain types into infrastructure generator calls
- **Make decisions**: "if Prometheus is configured, include it in Docker Compose"
- **Transform**: map domain types to template-specific context types (e.g., `convert_database_config_to_backup()`)

The infrastructure layer (generators/ProjectGenerator types) knows how to render a template given a context struct. The application services know _what_ to render and _how to assemble_ the context from domain data.

### Existing Precedent

`AnsibleTemplateService` already implements this exact pattern — it lives in `src/application/services/`, takes paths + clock, and provides a `render_templates()` method that bridges domain types to infrastructure. This refactoring extends the same pattern to all 8 template types.

### Service Input Design Principle

Services take **explicit domain config types** (e.g., `&PrometheusConfig`, `&TrackerConfig`), not `Environment<S>`. This:

- Removes the generic state parameter complexity
- Makes the API explicit about what data is actually needed
- Allows both the render handler (with `Environment<Created>`) and the release steps (with `Environment<Releasing>`) to call them by extracting config from their respective environment types

---

**Created**: February 10, 2026
**Last Updated**: February 10, 2026
**Status**: 📋 Planning
