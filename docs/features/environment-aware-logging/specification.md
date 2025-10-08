# Environment-Aware Logging Feature Specification

## 📋 Overview

The application supports multi-environment deployments, allowing users to deploy the tracker to multiple environments (e.g., `e2e-full`, `e2e-config`, `e2e-provision`). Each environment's state is stored in the `data/` folder. However, the current logging system doesn't consistently show the environment name in all log records, making it difficult to identify which environment a log entry belongs to in some cases.

## 🔍 Problem Statement

### Current Situation

Commands already use tracing spans with environment context. All commands use the `#[instrument]` macro with the environment name as a field:

```rust
#[instrument(
    name = "provision_command",
    skip_all,
    fields(
        command_type = "provision",
        environment = %environment.name()
    )
)]
pub async fn execute(
    &self,
    environment: Environment<Created>,
) -> Result<Environment<Provisioned>, ProvisionCommandError> {
```

This means **all logs executed within the command span are already mapped to the environment**. The tracing infrastructure captures this hierarchical relationship:

```text
2025-10-08T07:37:21.466853Z  INFO torrust_tracker_deploy::infrastructure::external_tools::tofu::adapter::client: Applying infrastructure changes in directory: build/e2e-full/tofu/lxd
  at src/infrastructure/external_tools/tofu/adapter/client.rs:195
  in torrust_tracker_deploy::application::steps::infrastructure::apply::apply_infrastructure with step_type: "infrastructure", operation: "apply", auto_approve: true
  in torrust_tracker_deploy::application::commands::provision::provision_command with command_type: "provision", environment: e2e-full
```

The span hierarchy shows `environment: e2e-full` in the command span context.

### The Real Issues

Based on log analysis, the visibility is inconsistent:

**Clear environment visibility**:

```text
2025-10-08T09:35:40.731158Z  INFO torrust_tracker_deploy::application::steps::software::docker: Installing Docker via Ansible
  ...
  in torrust_tracker_deploy::application::commands::configure::configure_command with command_type: "configure", environment: e2e-full
```

**Missing environment visibility**:

```text
2025-10-08T09:36:02.850501Z  WARN torrust_tracker_deploy::shared::ssh::client: SSH warning detected
  ...
  in torrust_tracker_deploy::application::commands::test::test_command with command_type: "test"
  # ❌ Missing environment field!
```

**Root causes**:

1. **Some commands missing environment field** - e.g., `TestCommand` doesn't include environment in span instrumentation
2. **E2E test utilities** - Cleanup and setup logs are outside command spans
3. **Visibility depends on span depth** - Nested logs might not prominently show environment

### Important Context

- **Single environment usage**: Most users will deploy to a single environment at a time, so this is not critical for typical usage
- **E2E testing impact**: The main impact is on E2E tests that run multiple environments concurrently
- **Future UI considerations**: Once a UI is implemented, stdout/stderr will be needed for user output, requiring log migration to files anyway

## 🎯 Goals

- **Improve visibility**: Make it easier to identify which environment any log entry belongs to
- **Debugging support**: Enable efficient debugging of environment-specific issues
- **Minimal complexity**: Simple solution - don't over-engineer

## 💡 Decision

**Selected Approach**: **Hybrid - Improve Visibility**

Combine existing span-based infrastructure (already working) with targeted improvements:

1. **Add environment field to command spans that are missing it** (e.g., `TestCommand`)
2. **Add environment field to application/domain layer logs** where environment is available and makes sense
3. **Keep infrastructure layers environment-agnostic** (proper abstraction maintained)

### Reasoning

Based on the analysis of current logs:

- Span-based approach (Option 3) is already working well but has visibility gaps
- Some logs clearly show environment (configure command), others don't (test command)
- Root cause: Not all commands include environment in their span instrumentation
- Solution: Fill the gaps rather than redesign the system

### What We're NOT Doing (Deferred)

- ❌ Custom formatter development (not needed - current formatter works)
- ❌ Separate log files per environment (deferred until UI implementation decision)
- ❌ Changing command dispatch or entry points (wait for production code)

### Priority

**Medium** - Nice-to-have for debugging, not blocking any features

### Timeline

Can be implemented incrementally as part of regular development

## 🔧 Implementation Details

### Scope

The environment field should be added to:

- ✅ **Application layer commands**: Add to command spans that are missing it
- ✅ **Application layer logs**: Add `environment` field to info/debug/warn logs where available
- ✅ **Domain layer logs**: Add environment field where it makes sense
- ❌ **Infrastructure layer**: Adapters and external tool clients (remain environment-agnostic)
- ❌ **Shared utilities**: Generic utilities that don't operate on environments
- ❌ **E2E test helpers**: Cleanup and setup utilities (not part of production code - deferred)

### Specific Changes Required

#### 1. Command Spans Missing Environment Field

**TestCommand** - Add environment to span instrumentation:

```rust
// Current (missing environment)
#[instrument(
    name = "test_command",
    skip_all,
    fields(
        command_type = "test"
    )
)]

// Add environment field
#[instrument(
    name = "test_command",
    skip_all,
    fields(
        command_type = "test",
        environment = %environment.name()  // ADD THIS
    )
)]
```

**Check other commands** for missing environment fields:

- `ProvisionCommand` ✅ (already has it)
- `ConfigureCommand` ✅ (already has it)
- `DeployCommand` - verify and add if missing
- `DestroyCommand` - verify and add if missing
- `TestCommand` ❌ (needs to be added)

#### 2. Application/Domain Layer Logs

Add `environment = %env.name()` field to logs where environment is available:

```rust
// Example: Step-level logs
info!(
    environment = %environment.name(),  // ADD THIS
    step = "install_docker",
    "Installing Docker via Ansible"
);
```

**Target areas**:

- Command-level logs (start/completion messages)
- Step-level logs where environment context is available
- Domain operations that work with environment objects

**Do NOT add to**:

- Infrastructure adapters (TofuClient, AnsibleClient, etc.)
- SSH operations (remain generic)
- External tool wrappers (remain environment-agnostic)

#### 3. Create Command - Use Environment Name Being Created

The `CreateCommand` should include the environment name in its span:

```rust
#[instrument(
    name = "create_command",
    skip_all,
    fields(
        command_type = "create",
        environment = %environment_name  // Use the name being created
    )
)]
pub fn execute(&self, environment_name: &str) -> Result<Environment<Created>> {
    // ...
}
```

### Commands Requiring Environment Context

| Command        | Requires Environment? | Reason                                               | Status    |
| -------------- | --------------------- | ---------------------------------------------------- | --------- |
| `provision`    | ✅ Yes                | Provisions infrastructure for environment            | ✅ Has it |
| `configure`    | ✅ Yes                | Configures software for environment                  | ✅ Has it |
| `test`         | ✅ Yes                | Tests environment deployment                         | ✅ Has it |
| `deploy`       | ✅ Yes                | Deploys application to environment                   | N/A       |
| `destroy`      | ✅ Yes                | Destroys environment infrastructure                  | N/A       |
| `create`       | ✅ Yes                | Creates new environment (use name being created)     | N/A       |
| `check`        | ❌ No                 | Checks system dependencies (generic, no environment) | N/A       |
| Internal tools | ❌ No                 | Linters, tests, etc.                                 | N/A       |

**Note**: Only `provision`, `configure`, and `test` commands exist currently (Oct 8, 2025). Other commands listed are planned but not yet implemented.

### Abstraction Layers

**Application Layer** (knows about environments):

- Commands: `ProvisionCommand`, `DeployCommand`, etc.
- Steps: Infrastructure setup, connectivity checks, etc.
- Should include environment context

**Infrastructure Layer** (environment-agnostic):

- Adapters: `TofuClient`, `AnsibleClient`, `SshClient`
- External tool wrappers
- Should NOT include environment context (maintains proper abstraction)

## 📊 Impact Analysis

### Hybrid Approach: Improve Visibility

**Files to Modify**: ~10-20 files (much smaller scope than originally estimated)

#### Phase 1: Fix Command Spans

**Files**:

- `src/application/commands/test.rs` - Add environment to TestCommand span
- `src/application/commands/deploy.rs` - Verify and add if missing
- `src/application/commands/destroy.rs` - Verify and add if missing

**Effort**: Low - Simple span field additions

#### Phase 2: Add Environment to Key Logs (Optional)

**Files**: Application layer command and step implementations where environment is available

**Target**: 10-20 strategic log statements where environment context adds value

**Effort**: Low-Medium - Incremental improvement

### Migration Strategy

1. **Audit commands**: Check all command `#[instrument]` macros for environment field
2. **Fix missing spans**: Add environment field to TestCommand and other commands missing it
3. **Identify key logs**: Find 10-20 high-value logs that would benefit from environment field
4. **Add environment field**: Systematically add `environment = %env.name()` to selected logs
5. **Test visibility**: Run E2E tests and verify environment appears in logs where expected
6. **Document**: Update logging guide with examples of when to include environment field

### Implementation Approach

**Incremental**: This can be done gradually without blocking other work

- ✅ Start with command spans (quick wins)
- ✅ Add to high-visibility logs incrementally
- ✅ No need for big-bang migration
- ✅ Can be done as part of regular development

## ✅ Definition of Done

### Phase 1: Command Spans (Priority) ✅ COMPLETED

- [x] All commands audited for environment field in `#[instrument]` macro
- [x] `TestCommand` updated with environment field
- [x] Other commands verified/updated as needed (only 3 commands exist: provision, configure, test - all have environment field)
- [x] E2E tests run successfully with improved environment visibility
- [x] Manual verification that logs show environment where expected

### Phase 2: Strategic Log Enhancement (Optional/Incremental)

- [ ] Identify 10-20 high-value logs for environment field addition
- [ ] Add `environment = %env.name()` to selected logs
- [ ] Verify environment field doesn't leak to infrastructure layers
- [ ] Document when to include environment field in logs

### Documentation

- [ ] Update [Logging Guide](../../contributing/logging-guide.md) with:
  - [ ] When to include environment field in command spans
  - [ ] When to include environment field in logs
  - [ ] Examples of proper vs improper environment usage
  - [ ] Abstraction layer guidelines (infrastructure stays environment-agnostic)
- [ ] Update this specification with final implementation details
- [ ] Update README with completion status

### Testing

- [ ] Existing tests continue to pass
- [ ] Manual testing of E2E logs confirms visibility improvements
- [ ] (Optional) Add unit tests for environment field presence if valuable

## 🧪 Testing Strategy

### Manual Testing (Primary)

**E2E Test Verification**:

1. Run `cargo run --bin e2e-tests-full`
2. Review log output for environment visibility
3. Verify commands show environment in span context
4. Confirm key logs include environment where expected

**Success Criteria**:

- All command logs show environment name in span context
- High-value logs explicitly show environment field
- Infrastructure layers remain environment-agnostic (no environment in adapter logs)

### Automated Testing (Optional)

**Unit Tests**: Not required for this feature - manual verification is sufficient

**Rationale**: Based on answers to questions:

- Focus on "what to expect" rather than "what not to do"
- Testing log output can be complex with infrastructure layers
- Manual review during E2E tests provides adequate coverage
- Can add tests later if patterns emerge that benefit from automation

## 📚 Related Documentation

- [Logging Guide](../../contributing/logging-guide.md)
- [Development Principles](../../development-principles.md)
- [Error Handling Guide](../../contributing/error-handling.md)
- [Codebase Architecture](../../codebase-architecture.md)

## 🔗 References

- [Tracing Spans Documentation](https://docs.rs/tracing/latest/tracing/#spans)
- [Tracing Fields Documentation](https://docs.rs/tracing/latest/tracing/#recording-fields)
