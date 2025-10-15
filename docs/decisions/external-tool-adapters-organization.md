# Decision: Consolidate External Tool Adapters in src/adapters/

## Status

Accepted

## Date

2025-10-15

## Context

The project uses multiple external CLI tools (SSH, Docker, Ansible, LXD, OpenTofu) through thin wrapper clients. These wrappers are currently scattered across two locations:

- **`src/shared/`**: SSH, Docker, and the CommandExecutor base abstraction
- **`src/infrastructure/external_tools/`**: Ansible, LXD, OpenTofu (with their template renderers)

This split was based on an assumption about reusability:

- SSH and Docker were placed in `shared/` because they were thought to be generic and reusable
- Ansible, LXD, and OpenTofu were placed in `infrastructure/` because they seemed application-specific

However, this organization creates several issues:

### Problems with Current Organization

1. **Inconsistent Discoverability**: External tool wrappers are split across modules, making them hard to find
2. **Semantic Confusion**: `src/shared/` mixes pure utilities (Clock, Username) with infrastructure adapters (SSH, Docker)
3. **Artificial Split**: The distinction is based on _assumed reusability_ rather than _nature_ of the code
4. **Pattern Inconsistency**: All wrappers follow the same pattern (thin clients using CommandExecutor) but are organized differently

### What These Wrappers Are

All external tool wrappers share these characteristics:

- **Thin adapters**: Minimal business logic, just command builders
- **Common pattern**: Use CommandExecutor for actual command execution
- **Infrastructure concerns**: All interact with external systems
- **Consistent API**: Client struct with domain-specific methods returning typed results

The key insight is that these are all **infrastructure adapters**, not pure utilities, regardless of whether they're currently used in production code or just tests.

## Decision

**Consolidate all external tool adapters into a new top-level `src/adapters/` module.**

This creates a three-tier semantic organization:

```text
src/
├── adapters/              # External tool adapters (thin wrappers, reusable)
│   ├── ansible/          # Ansible CLI wrapper
│   ├── docker/           # Docker CLI wrapper
│   ├── lxd/              # LXD CLI wrapper
│   ├── ssh/              # SSH client wrapper
│   └── tofu/             # OpenTofu CLI wrapper
├── infrastructure/        # Application-specific infrastructure
│   └── external_tools/   # Application-specific logic for external tools
│       ├── ansible/
│       │   └── template/ # Ansible template rendering (app-specific)
│       ├── lxd/          # LXD-specific logic if any
│       └── tofu/
│           └── template/ # OpenTofu template rendering (app-specific)
│   ├── remote_actions/   # High-level remote operations
│   └── persistence/      # State management
├── shared/               # Pure utilities (Clock, Username, error types)
│   ├── command/          # CommandExecutor (used by all adapters)
│   └── ...               # Port checkers, error types, etc.
└── ...                   # domain/, application/, testing/
```

### What Goes Where

**`src/adapters/`** - Infrastructure adapters for external tools:

- Thin wrapper clients (AnsibleClient, DockerClient, etc.)
- Minimal business logic
- Generic, reusable across projects
- Examples: SSH authentication, Docker commands, LXD operations

**`src/infrastructure/external_tools/`** - Application-specific logic for external tools:

- Ansible template renderers and inventory builders
- OpenTofu template renderers and cloud-init generation
- Application-specific orchestration and configuration
- Tightly coupled to this project's requirements

**`src/shared/`** - Pure utilities with no external dependencies:

- CommandExecutor (used by all adapters)
- Clock, Username, error types
- Port checkers
- Generic abstractions

### Migration Path

| From                                                  | To                          |
| ----------------------------------------------------- | --------------------------- |
| `src/shared/ssh/`                                     | `src/adapters/ssh/`         |
| `src/shared/docker/`                                  | `src/adapters/docker/`      |
| `src/shared/command/`                                 | STAY (pure utility)         |
| `src/infrastructure/external_tools/ansible/adapter/`  | `src/adapters/ansible/`     |
| `src/infrastructure/external_tools/lxd/adapter/`      | `src/adapters/lxd/`         |
| `src/infrastructure/external_tools/tofu/adapter/`     | `src/adapters/tofu/`        |
| `src/infrastructure/external_tools/ansible/template/` | STAY (application-specific) |
| `src/infrastructure/external_tools/tofu/template/`    | STAY (application-specific) |

### Why "adapters" and Not Other Names

**Considered alternatives:**

- **`src/clients/`**: Matches class names (\*Client), but implies network/API clients specifically
- **`src/packages/`**: Suggests future extraction, but conflicts with workspace-level `packages/` directory
- **`src/infrastructure/adapters/`**: Keeps within DDD layers, but doesn't emphasize reusability
- **`src/wrappers/`** or **`src/tools/`**: Too informal or vague

**Why "adapters" is best:**

- ✅ Well-known design pattern (Adapter/Wrapper pattern)
- ✅ Common in port-adapter architecture (Hexagonal Architecture)
- ✅ Top-level placement signals reusability intent
- ✅ Clear semantic: adapts external tools to project's needs
- ✅ Future-proof: easy to extract to `packages/adapters/` workspace package later

## Consequences

### Positive Consequences

1. **Improved Discoverability**: All external tool adapters in one predictable location
2. **Semantic Clarity**: Clear distinction between pure utilities, adapters, and application infrastructure
3. **Consistent Conventions**: Easier to apply uniform documentation, testing, and error handling patterns
4. **Code Sharing**: Common adapter patterns can be extracted and shared more easily
5. **Better Mental Model**: "Adapters wrap external tools" is clearer than current split
6. **Future Extraction Ready**: Clean separation makes it easier to extract to workspace packages when needed
7. **Alignment with Architecture Patterns**: Follows port-adapter (Hexagonal) architecture principles

### Negative Consequences

1. **Migration Effort**: Need to move files and update all imports across the codebase
2. **New Structure**: Contributors need to learn the new organization
3. **Breaking Changes**: External consumers (if any) would need to update imports
4. **Git History**: File moves may complicate `git blame` (mitigated by `git log --follow`)

### Neutral Consequences

1. **Partial Module Reorganization**: Only adapters move from `external_tools/*/adapter/` to top-level `adapters/`
   - Template logic stays in `infrastructure/external_tools/*/template/` (application-specific)
   - Creates clear distinction between generic adapters and application-specific usage
   - `external_tools/` becomes purely about application-specific tool configuration

### Risks and Mitigations

**Risk**: Missing imports during migration  
**Mitigation**: Use `cargo check` and `cargo test` to catch all broken imports

**Risk**: Test integration complexity  
**Mitigation**: Integration tests (like `tests/ssh_client_integration.rs`) stay in `tests/` directory

**Risk**: Confusion with workspace packages  
**Mitigation**: Clear documentation distinguishing `src/adapters/` (internal) from `packages/` (workspace)

## Alternatives Considered

### Alternative 1: Keep Current Split (Status Quo)

**Rejected** - Maintains semantic confusion between pure utilities and infrastructure adapters.

### Alternative 2: Move Everything to `src/infrastructure/external_tools/`

**Rejected** - Mixes generic reusable adapters with application-specific infrastructure, doesn't signal reusability.

### Alternative 3: Create Workspace Package `packages/external-tools/`

**Rejected** - Premature abstraction. No other Torrust projects currently need these adapters. Would add complexity (versioning, publishing, dependency management) without proven need. Decision: wait for concrete reuse case before extracting to workspace package.

### Alternative 4: Use `src/clients/` Instead of `src/adapters/`

**Considered viable** - Would work equally well, matches class naming convention (\*Client). Rejected in favor of "adapters" for clearer architectural semantics and broader applicability (adapters can include non-client patterns).

### Alternative 5: Keep Adapters in Infrastructure with Subdirectory

**Rejected** - Using `src/infrastructure/adapters/` keeps them in DDD infrastructure layer but doesn't emphasize reusability as clearly as top-level placement.

## Related Decisions

- [Repository Rename to Deployer](./repository-rename-to-deployer.md) - Project identity and scope
- [LXD VM over Containers](./lxd-vm-over-containers.md) - LXD adapter usage
- [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md) - Template separation rationale

## References

- **Hexagonal Architecture**: Ports and Adapters pattern by Alistair Cockburn
- **DDD Infrastructure Layer**: Domain-Driven Design by Eric Evans
- **Adapter Pattern**: Gang of Four Design Patterns
- [Module Organization Guide](../contributing/module-organization.md) - Project organization conventions
- [Development Principles](../development-principles.md) - Maintainability and clarity principles
