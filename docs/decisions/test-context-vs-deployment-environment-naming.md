# Decision: Test Context vs Deployment Environment Naming

## Status

Accepted

## Date

2025-09-29

## Context

The Torrust Tracker Deploy project has two distinct concepts that both could be called "environment":

1. **Testing Context (Current `TestEnvironment`)**: A test harness managing temporary resources, SSH keys, services, and cleanup for end-to-end testing.

2. **Multi-Environment Deployment (Planned Feature)**: Different deployment targets (dev, staging, production) with separate state, build artifacts, and persistent infrastructure.

This naming collision could mislead contributors and create ambiguity in the codebase.

## Decision

### Final Choice

- **`TestEnvironment` → `TestContext`**: Better represents a complete testing context with multiple resources
- **Reserve `Environment` for multi-environment feature**: Aligns with industry standards (GitHub, AWS, Railway, Vercel)

### Alternatives Discarded

**For Testing Context:**

- `TestFixture`: Too narrow, implies single resource rather than complete context
- `TestHarness`: Less familiar to some developers
- `TestSetup`: Generic, less descriptive
- `TestSandbox`: Could suggest security focus rather than resource management

**For Multi-Environment Feature:**

- `Context`: Used by Docker/Kubernetes but less intuitive for deployment targets
- `Workspace`: Used by Terraform differently, could conflict with IDE workspaces
- `Target`: Less descriptive of full environment concept
- `Stage`: Implies sequence, less flexible
- `Stack`: Implies specific architecture

## Why

- **TestContext** accurately describes managing a complete testing context with multiple resources and services
- **Environment** is the industry standard for deployment targets, making it immediately intuitive for users
- Clear conceptual separation: `TestContext` for temporary test infrastructure, `Environment` for persistent deployment targets
  | `TestContext` ✅ | Conveys complete context management | Slightly longer | Testing frameworks, .NET |
  | `TestSandbox` | Implies isolation | Could suggest security focus | Containerization |
  | `TestWorkspace` | Suggests temporary work area | Could conflict with IDE workspaces | IDE terminology |

### For Multi-Environment Feature (New Concept)

| Name             | Pros                         | Cons                                  | Industry Usage               |
| ---------------- | ---------------------------- | ------------------------------------- | ---------------------------- |
| `Environment` ✅ | Industry standard, intuitive | Could conflict with test environment  | GitHub, AWS, Railway, Vercel |
| `Context`        | Used by Docker/Kubernetes    | Less intuitive for deployment targets | Docker, Kubernetes           |
| `Workspace`      | Clear isolation concept      | Used by Terraform differently         | Terraform                    |
| `Target`         | Clear deployment meaning     | Less descriptive of full environment  | Build systems                |
| `Stage`          | Common in CI/CD              | Implies sequence, less flexible       | AWS, CI/CD pipelines         |
| `Stack`          | Complete infrastructure set  | Implies specific architecture         | Pulumi, CloudFormation       |

## Decision

We will adopt the following naming strategy:

### 1. Rename `TestEnvironment` → `TestContext`

**Rationale:**

- `TestContext` better represents what the struct actually does: managing a complete testing context with multiple resources, configuration, and services
- It's more accurate than "TestFixture" which implies a single resource
- Common in testing frameworks and clearly distinguishes from deployment environments
- Conveys the idea of a complete context that encompasses multiple concerns

### 2. Use `Environment` for Multi-Environment Feature

**Rationale:**

- `Environment` is the industry standard term used by GitHub Actions, AWS, Railway, Vercel, and most DevOps tools
- Users will immediately understand commands like `deploy --environment dev` or `deploy --environment staging`
- Aligns with common DevOps terminology and user expectations
- Most intuitive for the target use case

### 3. Related Renames

- `TestEnvironmentError` → `TestContextError`
- `TestEnvironmentType` → `TestContextType`
- Update field names: `environment_type` → `context_type`
- Update documentation and comments throughout

## Consequences

### Positive

- **Clear Conceptual Separation**: `TestContext` for temporary test infrastructure, `Environment` for persistent deployment targets
- **Industry Alignment**: Users familiar with other DevOps tools will immediately understand the Environment concept
- **Accurate Naming**: `TestContext` better describes the actual functionality
- **Future-Proof**: Leaves `Environment` available for the more important multi-environment feature

### Negative

- **Refactoring Required**: Need to update existing code, tests, and documentation
- **Temporary Disruption**: Developers working on branches will need to update their code
- **Documentation Updates**: All references in docs and comments need updating

### Neutral

- **Learning Curve**: Minimal - `TestContext` is intuitive for developers familiar with testing frameworks

## Implementation Plan

1. Create this decision record
2. Update `TestEnvironment` → `TestContext` in `src/e2e/environment.rs`
3. Search for and update all references in other files
4. Update documentation and module-level comments
5. Update error types and messages
6. Run tests to ensure no breakage

## Examples After Implementation

```rust
// Test context usage
let test_context = TestContext::initialized(
    false,
    "templates",
    &ssh_user,
    instance_name,
    ssh_private_key_path,
    ssh_public_key_path,
    TestContextType::Container,
)?;

// Future environment usage (planned)
deploy --environment dev
deploy --environment staging
deploy --environment production

struct Environment {
    name: String,
    build_dir: PathBuf,
    data_dir: PathBuf,
    config: EnvironmentConfig,
}
```

## Related Decisions

This decision enables future work on:

- Multi-environment deployment feature
- Environment-specific configuration management
- Independent state management per environment

## References

- Industry examples: GitHub Environments, AWS Environments, Docker Contexts, Terraform Workspaces
- Testing terminology: xUnit TestFixture, .NET TestContext
- Current codebase: `src/e2e/environment.rs`
