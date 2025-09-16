# Structured Logging Implementation Plan

**Status**: 🟡 **Planning Phase**  
**Created**: September 16, 2025  
**Last Updated**: September 16, 2025

## 📋 Overview

This document outlines the implementation plan for introducing hierarchical structured logging using tracing spans to align with our three-level architecture (Commands → Steps → Remote Actions).

### 🎯 Goals

- ✅ Create hierarchical log structure that mirrors the three-level architecture
- ✅ Improve debugging and monitoring capabilities
- ✅ Maintain format independence (console, JSON, OpenTelemetry compatibility)
- ✅ Preserve existing structured logging functionality
- ✅ Enable better correlation and filtering of related log events

### 🏗️ Architecture Alignment

```text
Level 1: Commands (Top-level orchestration)
├── Level 2: Steps (Mid-level execution units)
│   └── Level 3: Remote Actions (Leaf-level operations)
```

## 🚀 Implementation Phases

### Phase 1: Commands (Level 1) - Foundation

**Status**: 🔴 **Not Started**  
**Priority**: High  
**Estimated Effort**: 2-3 days

#### Tasks

- [ ] **1.1** Add `#[instrument]` to `ProvisionCommand::execute()`

  - File: `src/commands/provision.rs`
  - Span name: `provision_command`
  - Fields: `command_type="provision"`

- [ ] **1.2** Add `#[instrument]` to `ConfigureCommand::execute()`

  - File: `src/commands/configure.rs`
  - Span name: `configure_command`
  - Fields: `command_type="configure"`

- [ ] **1.3** Add `#[instrument]` to `TestCommand::execute()`

  - File: `src/commands/test.rs`
  - Span name: `test_command`
  - Fields: `command_type="test"`

- [ ] **1.4** Verify dependencies and features
  - Ensure `tracing = { features = ["attributes"] }` in `Cargo.toml`
  - Test span creation and nesting

#### Acceptance Criteria

- All command-level operations are wrapped in spans
- Spans include relevant contextual fields
- Log output shows hierarchical structure at command level
- E2E tests pass with new spans

#### Files to Modify

- `src/commands/provision.rs`
- `src/commands/configure.rs`
- `src/commands/test.rs`
- `Cargo.toml` (if needed)

---

### Phase 2: Steps (Level 2) - Core Operations

**Status**: 🔴 **Not Started**  
**Priority**: High  
**Estimated Effort**: 3-4 days

#### Infrastructure Steps

- [ ] **2.1** `InitializeInfrastructureStep::execute()`

  - File: `src/steps/infrastructure/initialize.rs`
  - Span: `initialize_infrastructure`
  - Fields: `step_type="infrastructure"`, `operation="init"`

- [ ] **2.2** `PlanInfrastructureStep::execute()`

  - File: `src/steps/infrastructure/plan.rs`
  - Span: `plan_infrastructure`
  - Fields: `step_type="infrastructure"`, `operation="plan"`

- [ ] **2.3** `ApplyInfrastructureStep::execute()`

  - File: `src/steps/infrastructure/apply.rs`
  - Span: `apply_infrastructure`
  - Fields: `step_type="infrastructure"`, `operation="apply"`, `auto_approve`

- [ ] **2.4** `GetInstanceInfoStep::execute()`
  - File: `src/steps/infrastructure/get_instance_info.rs`
  - Span: `get_instance_info`
  - Fields: `step_type="infrastructure"`, `operation="info"`

#### Rendering Steps

- [ ] **2.5** `RenderOpenTofuTemplatesStep::execute()`

  - File: `src/steps/rendering/opentofu_templates.rs`
  - Span: `render_opentofu_templates`
  - Fields: `step_type="rendering"`, `template_type="opentofu"`

- [ ] **2.6** `RenderAnsibleTemplatesStep::execute()`
  - File: `src/steps/rendering/ansible_templates.rs`
  - Span: `render_ansible_templates`
  - Fields: `step_type="rendering"`, `template_type="ansible"`

#### System Steps

- [ ] **2.7** `WaitForCloudInitStep::execute()`
  - File: `src/steps/system/wait_cloud_init.rs`
  - Span: `wait_cloud_init`
  - Fields: `step_type="system"`, `component="cloud_init"`

#### Connectivity Steps

- [ ] **2.8** `WaitForSSHConnectivityStep::execute()`
  - File: `src/steps/connectivity/wait_ssh_connectivity.rs`
  - Span: `wait_ssh_connectivity`
  - Fields: `step_type="connectivity"`, `protocol="ssh"`

#### Software Steps

- [ ] **2.9** `InstallDockerStep::execute()`

  - File: `src/steps/software/docker.rs`
  - Span: `install_docker`
  - Fields: `step_type="software"`, `component="docker"`, `method="ansible"`

- [ ] **2.10** `InstallDockerComposeStep::execute()`
  - File: `src/steps/software/docker_compose.rs`
  - Span: `install_docker_compose`
  - Fields: `step_type="software"`, `component="docker_compose"`, `method="ansible"`

#### Validation Steps

- [ ] **2.11** All validation steps in `src/steps/validation/`
  - Consistent naming: `validate_{component}`
  - Fields: `step_type="validation"`, `component="{name}"`

#### Acceptance Criteria

- All step-level operations are wrapped in spans
- Steps nest properly under command spans
- Spans include operation types and relevant contextual information
- Log output shows two-level hierarchy (Command → Step)

#### Files to Modify

- `src/steps/infrastructure/*.rs` (4 files)
- `src/steps/rendering/*.rs` (2 files)
- `src/steps/system/*.rs` (1 file)
- `src/steps/connectivity/*.rs` (1 file)
- `src/steps/software/*.rs` (2 files)
- `src/steps/validation/*.rs` (3 files)

---

### Phase 3: Remote Actions (Level 3) - Leaf Operations

**Status**: 🔴 **Not Started**  
**Priority**: Medium  
**Estimated Effort**: 2-3 days

#### Remote Action Trait

- [ ] **3.1** Update `RemoteAction` trait definition
  - File: `src/remote_actions/mod.rs`
  - Add `#[instrument]` guidance to trait documentation
  - Consider adding span fields to trait methods

#### Validation Actions

- [ ] **3.2** `CloudInitValidator::execute()`

  - File: `src/remote_actions/cloud_init.rs`
  - Span: `cloud_init_validation`
  - Fields: `action_type="validation"`, `component="cloud_init"`, `server_ip`

- [ ] **3.3** `DockerValidator::execute()`

  - File: `src/remote_actions/docker.rs`
  - Span: `docker_validation`
  - Fields: `action_type="validation"`, `component="docker"`, `server_ip`

- [ ] **3.4** `DockerComposeValidator::execute()`
  - File: `src/remote_actions/docker_compose.rs`
  - Span: `docker_compose_validation`
  - Fields: `action_type="validation"`, `component="docker_compose"`, `server_ip`

#### Acceptance Criteria

- All remote actions are wrapped in spans
- Remote actions nest properly under step spans
- Complete three-level hierarchy visible in logs
- Server IP and component information tracked in spans

#### Files to Modify

- `src/remote_actions/mod.rs`
- `src/remote_actions/cloud_init.rs`
- `src/remote_actions/docker.rs`
- `src/remote_actions/docker_compose.rs`

---

### Phase 4: Optimization & Documentation

**Status**: 🔴 **Not Started**  
**Priority**: Low  
**Estimated Effort**: 1-2 days

#### Documentation

- [ ] **4.1** Update contributing guidelines

  - File: `docs/contributing/README.md`
  - Add span instrumentation guidelines
  - Document field naming conventions

- [ ] **4.2** Create logging development guide

  - File: `docs/contributing/logging-guidelines.md`
  - Span usage examples
  - Field naming standards
  - Debugging techniques

- [ ] **4.3** Update testing documentation
  - File: `docs/contributing/testing.md`
  - How to test with spans
  - Log assertion examples

#### Code Optimization

- [ ] **4.4** Review and optimize span fields

  - Remove redundant fields
  - Ensure consistent naming across components
  - Add missing contextual information

- [ ] **4.5** Performance testing

  - Measure span overhead
  - Optimize high-frequency operations
  - Document performance impact

- [ ] **4.6** Integration testing
  - Test with different tracing subscribers
  - Verify JSON output format
  - Test log filtering capabilities

#### Acceptance Criteria

- Complete documentation for span usage
- Performance impact documented and acceptable
- All logging guidelines updated
- Integration tests pass with various subscribers

---

## 📊 Progress Tracking

### Overall Progress

- **Total Tasks**: 25
- **Completed**: 0 (0%)
- **In Progress**: 0 (0%)
- **Not Started**: 25 (100%)

### Phase Progress

| Phase                   | Status         | Tasks Complete | Progress |
| ----------------------- | -------------- | -------------- | -------- |
| Phase 1: Commands       | 🔴 Not Started | 0/4            | 0%       |
| Phase 2: Steps          | 🔴 Not Started | 0/11           | 0%       |
| Phase 3: Remote Actions | 🔴 Not Started | 0/4            | 0%       |
| Phase 4: Optimization   | 🔴 Not Started | 0/6            | 0%       |

## 🛠️ Technical Implementation Details

### Span Naming Conventions

```rust
// Commands (Level 1)
#[instrument(
    name = "{operation}_command",
    skip_all,
    fields(
        command_type = "{type}"
    )
)]

// Steps (Level 2)
#[instrument(
    name = "{action}_{component}",
    skip_all,
    fields(
        step_type = "{category}",
        operation = "{operation}"
    )
)]

// Remote Actions (Level 3)
#[instrument(
    name = "{component}_{operation}",
    skip(self),
    fields(
        action_type = "{type}",
        component = "{component}",
        server_ip = %server_ip
    )
)]
```

### Field Standards

- **command_type**: `provision`, `configure`, `test`
- **step_type**: `infrastructure`, `rendering`, `system`, `connectivity`, `software`, `validation`
- **operation**: `init`, `plan`, `apply`, `render`, `install`, `validate`, `wait`
- **component**: `opentofu`, `ansible`, `docker`, `cloud_init`, `ssh`

### Dependencies

```toml
[dependencies]
tracing = { version = "0.1", features = ["attributes"] }
```

## 🧪 Testing Strategy

### Unit Tests

- Verify spans are created correctly
- Test span field values
- Ensure nested span relationships

### Integration Tests

- Test full workflow span hierarchy
- Verify log output structure
- Test with different tracing subscribers

### E2E Tests

- Run existing E2E tests with spans enabled
- Verify performance impact is acceptable
- Test log correlation and filtering

## 📈 Success Metrics

- [ ] **Hierarchical Structure**: Clear three-level nesting in logs
- [ ] **Performance**: <10% overhead in E2E test execution time
- [ ] **Compatibility**: Works with console and JSON subscribers
- [ ] **Debugging**: Faster issue identification and resolution
- [ ] **Monitoring**: Better observability for production deployments

## 🔄 Review Process

### Code Review Checklist

- [ ] Span names follow naming conventions
- [ ] Required fields are present and consistent
- [ ] `skip` parameters used appropriately
- [ ] Documentation updated for new spans
- [ ] Tests updated and passing

### Testing Requirements

- [ ] Unit tests pass for instrumented functions
- [ ] E2E tests show proper span hierarchy
- [ ] Performance tests show acceptable overhead
- [ ] Multiple subscriber formats tested

---

**Next Steps**: Begin Phase 1 implementation with `ProvisionCommand` instrumentation.

**Dependencies**: None - can start immediately with existing tracing setup.

**Risk Assessment**: Low - Non-breaking changes, backward compatible, can be implemented incrementally.
