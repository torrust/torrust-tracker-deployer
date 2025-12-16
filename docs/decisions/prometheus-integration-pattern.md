# Decision: Prometheus Integration Pattern - Enabled by Default with Opt-Out

## Status

Accepted

## Date

2025-01-22

## Context

The tracker deployment system needed to add Prometheus as a metrics collection service. Several design decisions were required:

1. **Enablement Strategy**: Should Prometheus be mandatory, opt-in, or enabled-by-default?
2. **Template Rendering**: How should Prometheus templates be rendered in the release workflow?
3. **Service Validation**: How should E2E tests validate optional services like Prometheus?

The decision impacts:

- User experience (ease of getting started with monitoring)
- System architecture (template rendering patterns)
- Testing patterns (extensibility for future optional services)

## Decision

### 1. Enabled-by-Default with Opt-Out

Prometheus is **included by default** in generated environment templates but can be disabled by removing the configuration section.

**Implementation**:

```rust
pub struct UserInputs {
    pub prometheus: Option<PrometheusConfig>, // Some by default, None to disable
}
```

**Configuration**:

```json
{
  "prometheus": {
    "scrape_interval": 15
  }
}
```

**Disabling**: Remove the entire `prometheus` section from the environment config.

**Rationale**:

- Monitoring is a best practice - users should get it by default
- Opt-out is simple - just remove the config section
- No complex feature flags or enablement parameters needed
- Follows principle of least surprise (monitoring expected for production deployments)

### 2. Independent Template Rendering Pattern

Each service renders its templates **independently** in the release handler, not from within other service's template rendering.

**Architecture**:

```text
ReleaseCommandHandler::execute()
├─ Step 1: Create tracker storage
├─ Step 2: Render tracker templates (tracker/*.toml)
├─ Step 3: Deploy tracker configs
├─ Step 4: Create Prometheus storage (if enabled)
├─ Step 5: Render Prometheus templates (prometheus.yml) - INDEPENDENT STEP
├─ Step 6: Deploy Prometheus configs
├─ Step 7: Render Docker Compose templates (docker-compose.yml)
└─ Step 8: Deploy compose files
```

**Rationale**:

- Each service is responsible for its own template rendering
- Docker Compose templates only define service orchestration, not content generation
- Environment configuration is the source of truth for which services are enabled
- Follows Single Responsibility Principle (each step does one thing)
- Makes it easy to add future services (Grafana, Alertmanager, etc.)

**Anti-Pattern Avoided**: Rendering Prometheus templates from within Docker Compose template rendering step.

### 3. ServiceValidation Struct for Extensible Testing

E2E validation uses a `ServiceValidation` struct with boolean flags instead of function parameters.

**Implementation**:

```rust
pub struct ServiceValidation {
    pub prometheus: bool,
    // Future: pub grafana: bool,
    // Future: pub alertmanager: bool,
}

pub fn run_release_validation(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
    services: Option<ServiceValidation>,
) -> Result<(), String>
```

**Rationale**:

- Extensible for future services without API changes
- More semantic than boolean parameters
- Clear intent: `ServiceValidation { prometheus: true }`
- Follows Open-Closed Principle (open for extension, closed for modification)

**Anti-Pattern Avoided**: `run_release_validation_with_prometheus_check(addr, creds, true)` - too specific and not extensible.

## Consequences

### Positive

1. **Better User Experience**:

   - Users get monitoring by default without manual setup
   - Simple opt-out (remove config section)
   - Production-ready deployments out of the box

2. **Cleaner Architecture**:

   - Each service manages its own templates independently
   - Clear separation of concerns in release handler
   - Easy to add future services (Grafana, Alertmanager, Loki, etc.)

3. **Extensible Testing**:

   - ServiceValidation struct easily extended for new services
   - Consistent pattern for optional service validation
   - Type-safe validation configuration

4. **Maintenance Benefits**:
   - Independent template rendering simplifies debugging
   - Each service's templates can be modified independently
   - Clear workflow steps make issues easier to trace

### Negative

1. **Default Overhead**:

   - Users who don't want monitoring must manually remove the section
   - Prometheus container always included in default deployments
   - Slightly more disk/memory usage for minimal deployments

2. **Configuration Discovery**:
   - Users must learn that removing the section disables the service
   - Not immediately obvious from JSON schema alone
   - Requires documentation of the opt-out pattern

### Risks

1. **Breaking Changes**: Future Prometheus config schema changes require careful migration planning
2. **Service Dependencies**: Adding services that depend on Prometheus requires proper ordering logic
3. **Template Complexity**: As services grow, need to ensure independent rendering doesn't duplicate logic

## Alternatives Considered

### Alternative 1: Mandatory Prometheus

**Approach**: Always deploy Prometheus, no opt-out.

**Rejected Because**:

- Forces monitoring on users who don't want it
- Increases minimum resource requirements
- Violates principle of least astonishment for minimal deployments

### Alternative 2: Opt-In with Feature Flag

**Approach**: Prometheus disabled by default, enabled with `"prometheus": { "enabled": true }`.

**Rejected Because**:

- Requires users to discover and enable monitoring manually
- Most production deployments should have monitoring - opt-in makes it less likely
- Adds complexity with enabled/disabled flags

### Alternative 3: Render Prometheus Templates from Docker Compose Step

**Approach**: Docker Compose template rendering step also renders Prometheus templates.

**Rejected Because**:

- Violates Single Responsibility Principle
- Makes Docker Compose step dependent on Prometheus internals
- Harder to add future services independently
- Couples service orchestration with service configuration

### Alternative 4: Boolean Parameters for Service Validation

**Approach**: `run_release_validation(addr, creds, check_prometheus: bool)`.

**Rejected Because**:

- Not extensible - adding Grafana requires API change
- Less semantic - what does `true` mean?
- Becomes unwieldy with multiple services
- Violates Open-Closed Principle

## Related Decisions

- [Template System Architecture](../technical/template-system-architecture.md) - Project Generator pattern
- [Environment Variable Injection](environment-variable-injection-in-docker-compose.md) - Configuration passing
- [DDD Layer Placement](../contributing/ddd-layer-placement.md) - Module organization

## References

- Issue: [#238 - Prometheus Slice - Release and Run Commands](../issues/238-prometheus-slice-release-run-commands.md)
- Manual Testing Guide: [Prometheus Verification](../e2e-testing/manual/prometheus-verification.md)
- Prometheus Documentation: https://prometheus.io/docs/
- torrust-demo Reference: Existing Prometheus integration patterns
