# Infrastructure Module Organization: Execution Context Separation

**Status**: Accepted  
**Date**: 2025-12-11  
**Deciders**: Development Team  
**Issue**: [#220](https://github.com/torrust/torrust-tracker-deployer/issues/220)

## Context

The infrastructure layer contains components that interact with external systems. However, there are two fundamentally different types of external interactions:

1. **SSH-based operations**: Commands executed **inside the VM** via SSH connection
2. **External validation**: HTTP requests made **from outside the VM** to test end-to-end functionality

Previously, both types were mixed in `infrastructure/remote_actions/`, creating architectural confusion:

- `remote_actions/validators/docker.rs` - Executes `docker --version` inside VM via SSH
- `remote_actions/validators/running_services.rs` - Makes HTTP requests to services from outside VM

This mixing obscured the critical distinction of **where the code executes** and **what it validates**.

## Decision

We separate infrastructure modules by execution context:

```text
src/infrastructure/
├── remote_actions/          # SSH-based operations executed INSIDE the VM
│   └── validators/
│       ├── cloud_init.rs
│       ├── docker.rs
│       └── docker_compose.rs
└── external_validators/     # E2E validation from OUTSIDE the VM
    └── running_services.rs
```

### Module Purposes

**`remote_actions/`** (SSH-based, inside VM):

- Execute commands via SSH connection inside the VM
- Validate internal VM state and configuration
- Examples: Check if Docker is installed, verify cloud-init completion
- Scope: Internal system state

**`external_validators/`** (HTTP-based, outside VM):

- Make HTTP requests from test runner/deployment machine
- Validate end-to-end service accessibility
- Test network configuration and firewall rules
- Examples: Health check endpoints, service availability tests
- Scope: External accessibility and E2E functionality

## Rationale

### Why Both Remain in Infrastructure Layer (DDD)

Both modules are infrastructure concerns because they:

- Interact with external systems (VMs, networks, services)
- Provide technical capabilities for application layer
- Depend on adapters (SSH client, HTTP client)
- Are not business logic or domain concepts

The distinction is **execution context**, not **DDD layer**.

### Why Separation Improves Architecture

1. **Clarity**: Developers immediately understand where code executes
2. **Testability**: Different testing strategies for SSH vs HTTP operations
3. **Documentation**: Module names self-document their purpose
4. **Maintainability**: Related code grouped by execution context
5. **Discoverability**: New validators know which module to use

### Comparison with Remote Actions Module

| Aspect             | `remote_actions/`                 | `external_validators/`        |
| ------------------ | --------------------------------- | ----------------------------- |
| Execution location | Inside VM via SSH                 | Outside VM (test runner)      |
| Connection type    | SSH                               | HTTP/HTTPS                    |
| Validates          | Internal state                    | External accessibility        |
| Examples           | Docker version, cloud-init status | Service health, API endpoints |
| Firewall impact    | Not validated                     | Implicitly validated          |

## Consequences

### Positive

- **Clear architectural boundaries**: Execution context is explicit
- **Better code organization**: Related validators grouped together
- **Improved documentation**: Module purpose is self-evident
- **Easier testing**: Different strategies for SSH vs HTTP
- **Scalable**: Future validators know which module to use

### Neutral

- **Module proliferation**: More top-level infrastructure modules
- **Import paths change**: Code needs import updates (one-time cost)

### Negative

- **None identified**: This is a pure improvement in organization

## Alternatives Considered

### Alternative 1: Keep Everything in `remote_actions/`

**Rejected because**:

- Mixes fundamentally different execution contexts
- "Remote actions" implies SSH operations, confusing for HTTP validators
- Harder to understand what code does without reading implementation

### Alternative 2: Move to Application Layer Services

**Rejected because**:

- Not business logic or use cases
- Depends on infrastructure adapters (SSH, HTTP clients)
- Violates DDD layer boundaries (application depends on infrastructure)
- `RunningServicesValidator` performs infrastructure concerns (external system validation)

### Alternative 3: Create `e2e_validators/` Instead

**Rejected because**:

- "E2E" describes testing strategy, not execution context
- Less clear than "external" for where code runs
- Could be confused with test helpers

## Implementation

### File Reorganization

1. Create `src/infrastructure/external_validators/mod.rs`
2. Move `running_services.rs` from `remote_actions/validators/` to `external_validators/`
3. Update infrastructure module exports
4. Update all import paths in application and testing code

### Documentation Updates

1. Update `docs/codebase-architecture.md` with new structure
2. Add module-level documentation explaining execution context
3. Update validator documentation to reference execution context

## Related Decisions

- [Port Zero Not Supported](port-zero-not-supported.md) - Validates port configuration
- [DDD Layer Placement](../contributing/ddd-layer-placement.md) - Explains infrastructure layer

## Notes

This refactoring maintains all existing functionality while improving code organization and clarity. The change is purely structural - no behavior changes.
