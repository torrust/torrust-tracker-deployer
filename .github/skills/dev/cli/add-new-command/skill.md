---
name: add-new-command
description: Guide for implementing new CLI commands in the deployer using outside-in development. Covers presentation layer scaffolding, application handler, confirmation prompts, E2E testing, and documentation. Use when adding new commands like validate, render, backup, or any console subcommand. Triggers on "add command", "new command", "implement command", "create subcommand", or "add CLI command".
metadata:
  author: torrust
  version: "1.0"
---

# Adding New Commands to the Deployer

This skill guides you through implementing new CLI commands for the Torrust Tracker Deployer using an **outside-in** (presentation → application → domain) development approach.

## Why Outside-In?

**Start from the outer layers (user interface) and work inward:**

```text
Presentation (CLI) → Application (handlers) → Domain (business logic)
```

**Benefits for infrastructure applications:**

1. **Test immediately** - Run command and check output after each step
2. **Validate UX early** - Verify command interface before business logic
3. **Defer hard problems** - Infrastructure code (filesystems, APIs, databases) is hard to unit test
4. **E2E focus** - Integration tests work from day one

**Alternative (inside-out)**: Starting from domain/business logic requires mocking external dependencies or waiting until full implementation to test real behavior.

## Implementation Phases

### Phase 1: Presentation Layer Stub

**Goal**: Make command runnable with proper routing and empty implementation.

**What to build**:

```rust
// 1. Add CLI command variant
// src/presentation/input/cli/commands.rs
pub enum Commands {
    // ... existing commands
    YourCommand {
        name: String,
        #[arg(long)]
        force: bool,  // If needed
    },
}

// 2. Add routing
// src/presentation/dispatch/router.rs
Commands::YourCommand { name, force } => {
    self.handle_your_command(name, force).await
}

// 3. Create controller stub
// src/presentation/controllers/your_command/handler.rs
pub struct YourCommandController {
    // Dependencies (injected later)
}

impl YourCommandController {
    pub fn execute(&mut self, name: &str) -> Result<()> {
        // Stub: just show progress steps
        self.progress.start("Step 1: Validate input")?;
        self.progress.complete("Validation complete")?;
        Ok(())
    }
}

// 4. Define presentation errors
// src/presentation/controllers/your_command/errors.rs
#[derive(Error, Debug)]
pub enum YourCommandError {
    #[error("Invalid name: {0}")]
    InvalidName(String),
}

impl YourCommandError {
    pub fn help(&self) -> Option<String> {
        match self {
            Self::InvalidName(_) => Some(
                "Use lowercase alphanumeric with hyphens".to_string()
            ),
        }
    }
}

// 5. Wire in container
// src/bootstrap/container.rs
pub fn your_command_controller(&self) -> YourCommandController {
    YourCommandController::new(/* dependencies */)
}
```

**Test Phase 1**:

```bash
# Test 1: Help text displays correctly
cargo run -- your-command --help

# Test 2: Error handling works (file not found, invalid input)
cargo run -- your-command /path/to/nonexistent

# Test 3: Success path (stub shows progress)
cargo run -- your-command valid-input
```

**Expected Behavior Phase 1**:

✅ Help text shows:

- Command description
- Arguments and flags
- Usage examples
- Options documentation

✅ Error handling shows:

- Clear error message
- Troubleshooting help (via `.help()` method)
- Actionable guidance

✅ Success path shows:

- Progress steps (1/N, 2/N, 3/N)
- Step descriptions
- Success message
- Stub completion (no real work yet)

**What Validates**:

- CLI registration works
- Routing is correct
- Controller is wired
- Error types have `.help()` methods
- Progress reporting displays properly

**What Doesn't Validate**:

- Real business logic (Phase 2)
- Domain validation (Phase 2)
- Actual operations (Phase 2)

**Commit**: `feat: [#ISSUE] add your-command presentation layer stub`

> **Query commands (commands that return data)**: If your command returns a value rather than just performing an action (e.g., `exists`, `show`, `list`), the presentation layer needs a view layer in addition to the controller. Create:
>
> - `src/presentation/cli/views/commands/your_command/view_data/your_details.rs` — DTO populated with the result
> - `src/presentation/cli/views/commands/your_command/views/text_view.rs` — `impl Render<YourResult>`
> - `src/presentation/cli/views/commands/your_command/views/json_view.rs` — `impl Render<YourResult>`
>
> See [`src/presentation/cli/views/commands/exists/`](../../../src/presentation/cli/views/commands/exists/) as a reference.

### Phase 2: Application Handler

**Goal**: Implement business logic that actually does the work.

**What to build**:

```rust
// 1. Create application handler
// src/application/command_handlers/your_command/handler.rs
pub struct YourCommandHandler {
    repository: Arc<dyn EnvironmentRepository>,
    working_directory: Arc<Path>,
}

impl YourCommandHandler {
    pub fn execute(&self, name: &EnvironmentName) -> Result<()> {
        // 1. Verify preconditions
        if !self.repository.exists(name)? {
            return Err(YourCommandError::NotFound);
        }

        // 2. Perform the actual work
        self.do_the_work(name)?;

        // 3. Update state if needed
        // repository.update(...)?;

        Ok(())
    }

    fn do_the_work(&self, name: &EnvironmentName) -> Result<()> {
        // Real implementation:
        // - Call external tools (Ansible, OpenTofu)
        // - Modify filesystem
        // - Update database/registry
        // - Generate artifacts

        tracing::info!("Executing work for {}", name);
        Ok(())
    }
}

// 2. Create application errors
// src/application/command_handlers/your_command/errors.rs
#[derive(Error, Debug)]
pub enum YourCommandError {
    #[error("Environment not found: {0}")]
    NotFound(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

// 3. Update controller to use handler
// src/presentation/controllers/your_command/handler.rs
pub struct YourCommandController {
    handler: YourCommandHandler,  // Inject handler
    progress: ProgressReporter,
}

impl YourCommandController {
    pub fn execute(&mut self, name: &str) -> Result<()> {
        let env_name = EnvironmentName::try_from(name)?;

        self.progress.start("Validating environment")?;
        // Validation...

        self.progress.start("Performing operation")?;
        self.handler.execute(&env_name)?;  // Delegate to handler

        self.progress.complete("Operation complete")?;
        Ok(())
    }
}
```

**Test Phase 2**:

```bash
# Test 1: Valid input performs real work
cargo run -- your-command valid-input

# Test 2: Invalid preconditions fail gracefully
cargo run -- your-command nonexistent

# Test 3: Domain validation errors are caught
cargo run -- your-command invalid-format

# Test 4: Business rule violations show helpful errors
cargo run -- your-command edge-case-input
```

**Expected Behavior Phase 2**:

✅ **Valid Input** - Real operations execute:

- Progress steps complete with actual work
- State changes occur (files created/updated, database modified)
- Detailed success message with results
- Operation completes successfully

Example (validate command):

```text
⏳ [1/3] Loading configuration file...
⏳   ✓ Configuration file loaded (took 0ms)
⏳ [2/3] Validating JSON schema...
⏳   ✓ Schema validation passed (took 0ms)
⏳ [3/3] Validating configuration fields...
⏳   ✓ Field validation passed (took 0ms)

✅ Configuration file 'envs/lxd-local-example.json' is valid

Environment Details:
  • Name: lxd-local-example
  • Provider: lxd
  • Prometheus: Enabled
  • Grafana: Enabled
```

✅ **Invalid Preconditions** - Clear presentation errors:

- Error message explains what's wrong
- `.help()` provides actionable guidance
- Exit code is non-zero

Example:

```text
❌ Command failed: Configuration file not found: /tmp/nonexistent.json

For detailed troubleshooting:
Verify the file path is correct: /tmp/nonexistent.json
Use 'create template' to generate a valid configuration file.
```

✅ **Domain Validation Errors** - Application layer catches issues:

- Specific error message (not generic)
- Context about what was validated
- Helpful troubleshooting steps

Example (JSON parsing):

```text
❌ Validation failed for configuration file: /tmp/invalid.json

For detailed troubleshooting:
JSON parsing failed for file '/tmp/invalid.json'.

Error details:
key must be a string at line 1 column 3

Common issues:
- Missing or extra commas
- Unmatched braces or brackets
- Invalid escape sequences
```

✅ **Business Rule Violations** - Domain layer enforces constraints:

- Detailed error about which rule failed
- Why the rule exists
- How to fix the problem

Example (missing SSH keys):

```text
❌ Validation failed: SSH private key file not found: /tmp/nonexistent-key

This means the configuration file has valid JSON syntax but violates
domain constraints or business rules.

Common issues:
- SSH key files don't exist at specified paths
- Invalid environment name (must be lowercase with dashes)
- Invalid port numbers or IP addresses
```

**What Validates**:

- Real business logic executes
- Application handler performs actual work
- Domain validation catches constraint violations
- Error propagation works (domain → application → presentation)
- State changes occur correctly

**What Doesn't Validate**:

- User confirmation prompts (Phase 3)
- E2E integration across commands (Phase 4)
- Full error coverage (Phase 4)

**Verification Checklist**:

```bash
# 1. Compilation succeeds
cargo check

# 2. Code quality passes
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt

# 3. Manual tests cover all scenarios
# - Valid input success path
# - Presentation errors (file not found, wrong type)
# - Application errors (JSON parsing, schema validation)
# - Domain errors (business rules, constraints)

# 4. Verify state changes
# - Check filesystem modifications
# - Verify database/registry updates
# - Confirm generated artifacts
```

**Commit**: `feat: [#ISSUE] add your-command application layer handler`

#### Unit Tests for the Handler

Write unit tests directly alongside the handler in `src/application/command_handlers/your_command/tests/mod.rs`. Cover:

- Success path (e.g., operation succeeds when environment exists/does not exist)
- Error path (repository failure propagates correctly)

Use `FileEnvironmentRepository` (via `EnvironmentTestBuilder`) or a simple inline stub:

```rust
struct FailingRepository;
impl EnvironmentRepository for FailingRepository {
    fn find(...) -> Result<...> { Err(RepositoryError::...) }
}
```

Add `#[cfg(test)] mod tests;` to `src/application/command_handlers/your_command/mod.rs`.

#### SDK Layer Update

If the command is useful for programmatic workflows, expose it through the SDK:

1. Add a method `your_command(...)` in both:
   - `packages/sdk/src/deployer.rs`
   - `src/presentation/sdk/deployer.rs`

2. Add a `From<YourCommandHandlerError>` impl in `packages/sdk/src/error.rs` (new `SdkError` variant). **Without this, doc examples using the `?` operator will fail doc tests**:

   ```rust
   // packages/sdk/src/error.rs
   /// [`super::deployer::Deployer::your_command`] failed.
   #[error(transparent)]
   YourCommand(#[from] YourCommandHandlerError),
   ```

3. Verify doc tests pass: `cargo test --doc --workspace`

### Phase 3: User Confirmation (Optional)

**Goal**: Add interactive confirmation for destructive/important operations.

**When to add**:

- ✅ Destructive operations (delete, purge, destroy)
- ✅ Operations that modify infrastructure
- ✅ Operations that cost money
- ✅ Operations that are hard to undo

**When to SKIP**:

- ❌ Read-only operations (list, show, validate)
- ❌ Dry-run commands that don't change state
- ❌ Operations with `--dry-run` mode
- ❌ Commands that generate artifacts without side effects

**Decision**: If your command is read-only or non-destructive, **skip to Phase 4** (E2E Tests).

**What to build** (if needed):

```rust
// src/presentation/controllers/your_command/handler.rs
impl YourCommandController {
    pub fn execute(&mut self, name: &str, force: bool) -> Result<()> {
        let env_name = EnvironmentName::try_from(name)?;

        // Show confirmation unless --force
        if !force && !self.confirm_operation(&env_name)? {
            return Err(YourCommandError::UserCancelled);
        }

        // Continue with operation...
        self.handler.execute(&env_name)?;
        Ok(())
    }

    fn confirm_operation(&mut self, name: &EnvironmentName) -> Result<bool> {
        self.progress.blank_line()?;
        self.progress.output().lock().borrow_mut().warning(
            "⚠️  WARNING: This operation will [describe impact]"
        );
        self.progress.output().lock().borrow_mut().warning(
            "This action cannot be undone!"
        );
        self.progress.blank_line()?;

        print!("Continue? [y/N]: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| YourCommandError::IoError(e.to_string()))?;

        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}
```

**Test Phase 3**:

```bash
# Interactive mode
cargo run -- your-command test-name
# User prompted: Continue? [y/N]:

# Automated mode with --force
cargo run -- your-command test-name --force
# No prompt, executes immediately
```

**Commit**: `feat: [#ISSUE] add your-command confirmation prompt`

### Phase 4: E2E Tests

**Goal**: Black-box testing validating end-to-end behavior.

**Note**: Application handler unit tests belong in Phase 2 (`src/application/command_handlers/your_command/tests/`). Phase 4 E2E tests verify the complete command as a black box through the CLI binary.

**Why E2E over unit tests for infrastructure code**: Infrastructure code interacts with external systems (filesystems, databases, APIs, VMs). E2E tests validate real behavior without complex mocking.

**What to build**:

```rust
// tests/e2e/your_command.rs
use crate::support::*;

#[test]
fn it_should_execute_successfully_when_conditions_met() {
    let runner = ProcessRunner::new();
    let env_name = "e2e-your-command-test";

    // Setup: Create prerequisites
    runner.run_create_command(env_name, &create_config());

    // Execute: Run your command
    let result = runner.run_your_command(env_name, &["--force"]);

    // Assert: Verify outcomes
    assert!(result.success());
    assert!(result.output().contains("Operation complete"));
    assert_expected_state_changes(env_name);
}

#[test]
fn it_should_fail_when_preconditions_not_met() {
    let runner = ProcessRunner::new();

    let result = runner.run_your_command("nonexistent", &["--force"]);

    assert!(!result.success());
    assert!(result.output().contains("not found"));
}

#[test]
fn it_should_handle_edge_cases() {
    // Test with custom working directory
    // Test with unusual but valid inputs
    // Test idempotency (run twice, same result)
    // Test isolation (multiple environments don't interfere)
}

// Add helper to ProcessRunner
// src/testing/e2e/process_runner.rs
impl ProcessRunner {
    pub fn run_your_command(&self, name: &str, flags: &[&str]) -> ExecutionResult {
        let mut args = vec!["your-command", name];
        args.extend_from_slice(flags);
        self.run(&args)
    }
}

// Add assertions
// tests/support/assertions.rs
pub fn assert_expected_state_changes(env_name: &str) {
    // Check filesystem changes
    // Check registry state
    // Check generated artifacts
}
```

**Test Phase 4**:

```bash
# Run all E2E tests for your command
cargo test --test e2e_integration your_command -- --test-threads=1

# Run with output for debugging
cargo test --test e2e_integration your_command -- --nocapture --test-threads=1
```

**Expected Behavior Phase 4**:

✅ **All Test Scenarios Pass**:

```text
running 5 tests
test e2e::your_command::test_scenario_1 ... ok
test e2e::your_command::test_scenario_2 ... ok
test e2e::your_command::test_scenario_3 ... ok
test e2e::your_command::test_scenario_4 ... ok
test e2e::your_command::test_scenario_5 ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

**Test Coverage Checklist**:

- ✅ Success scenario with valid input
- ✅ Presentation layer errors (file not found, invalid path)
- ✅ Application layer errors (parsing failures, schema validation)
- ✅ Domain layer errors (constraint violations, business rules)
- ✅ Read-only verification (no side effects for dry-run commands)

**Example Test Cases** (validate command):

1. **File Not Found** - Reports missing configuration file
2. **Invalid JSON** - Shows JSON parsing error with line numbers
3. **Missing SSH Keys** - Catches domain validation errors
4. **Valid Configuration** - Succeeds and displays environment details
5. **No Deployment Created** - Verifies read-only behavior

**What Validates**:

- Complete end-to-end workflow
- Error propagation through all layers
- User-facing messages are helpful
- No unintended side effects
- Command works in realistic scenarios

**What Doesn't Validate**:

- User documentation (Phase 5)
- Integration with other commands (Phase 6)
- Full production deployment (manual testing)

**Commit**: `feat: [#ISSUE] add your-command E2E tests`

### Phase 5: Documentation

**Goal**: Comprehensive user documentation for the new command.

**What to create**:

````markdown
<!-- docs/user-guide/commands/your-command.md -->

# Your Command

Brief description of what the command does and when to use it.

## Command Syntax

\```bash
torrust-tracker-deployer your-command <NAME> [OPTIONS]
\```

### Arguments

- `<NAME>` - Environment name (required)

### Options

- `--force` - Skip confirmation prompt (optional)

## Usage Examples

### Basic Usage

\```bash
torrust-tracker-deployer your-command my-env
\```

### Automated/CI Usage

\```bash
torrust-tracker-deployer your-command my-env --force
\```

## What This Command Does

1. [Step 1 description]
2. [Step 2 description]
3. [Step 3 description]

## When to Use

- [Use case 1]
- [Use case 2]
- [Use case 3]

## When NOT to Use

- [Anti-pattern 1]
- [Anti-pattern 2]

## Common Scenarios

### Scenario 1: [Description]

\```bash

# Commands...

\```

### Scenario 2: [Description]

\```bash

# Commands...

\```

## Troubleshooting

### Error: [Common Error Message]

**Cause**: [Why this happens]

**Solution**: [How to fix]

\```bash

# Fix command

\```

## Related Commands

- [`other-command`](./other-command.md) - [When to use instead]
- [`related-command`](./related-command.md) - [Use before/after]

## See Also

- [Feature documentation](../../features/your-feature/)
- [Architecture decisions](../../decisions/)
````

**Also update**:

```markdown
<!-- docs/user-guide/commands/README.md -->

## [Category] Commands

- **[`your-command`](./your-command.md)** - Brief description
```

**Also update `docs/console-commands.md`** — this file lists every command with its full CLI syntax and is separate from the user guide. Add your command in the appropriate section.

**Also update the command workflow** (if appropriate):

````markdown
<!-- docs/user-guide/commands/README.md -->

The typical command sequence for a complete deployment:

```text
1. create template    → Generate configuration template
2. (edit template)    → Customize your settings
3. your-command       → [Where your command fits in workflow]
4. create environment → Create environment from config
...
```
````

**Test Phase 5**:

```bash
# 1. Verify documentation exists
ls docs/user-guide/commands/your-command.md

# 2. Verify command index updated
grep "your-command" docs/user-guide/commands/README.md

# 3. Verify workflow updated (if applicable)
grep "your-command" docs/user-guide/commands/README.md -A 5

# 4. Run markdown linter
cargo run --bin linter markdown
```

✅ **Expected Behaviors**:

| Aspect                   | Expected Result                                                                           |
| ------------------------ | ----------------------------------------------------------------------------------------- |
| **Documentation File**   | `docs/user-guide/commands/your-command.md` exists with 200+ lines                         |
| **Content Sections**     | All required sections present (Syntax, Examples, When to Use, Scenarios, Troubleshooting) |
| **Command Index**        | `your-command` listed in `docs/user-guide/commands/README.md`                             |
| **Workflow Integration** | Command appears in workflow sequence if it's part of main deployment path                 |
| **Markdown Linting**     | ✅ All checks pass with proper formatting                                                 |
| **Links Work**           | All internal links to other commands/docs are valid                                       |

**Documentation Quality Checklist**:

- ✅ Clear command syntax with all arguments and options
- ✅ At least 3 common scenarios with complete code
- ✅ Error examples showing both error message and solution
- ✅ "When to Use" and "When NOT to Use" sections
- ✅ Troubleshooting section with 3+ common issues
- ✅ Related commands section with appropriate cross-links
- ✅ Examples use realistic names (not "foo", "bar")

**Example Documentation** (validate command - 280+ lines):

```markdown
# Validate Command

Validates a Torrust Tracker Deployer configuration file without creating an environment.

## Command Syntax

[...]

## Common Scenarios

### Scenario 1: Pre-creation Validation

[Complete workflow with commands]

### Scenario 2: CI/CD Pipeline Check

[Automated validation example]

### Scenario 3: Troubleshooting Invalid Config

[Debug workflow]

## Troubleshooting

### Error: Configuration file not found

**Cause**: File path is incorrect
**Solution**: Verify file exists: `ls -la envs/your-config.json`
[...]
```

**What Phase 5 Validates**:

- User-facing documentation is comprehensive
- All command aspects are documented
- Examples are practical and realistic
- Troubleshooting covers common issues
- Integration with command index is complete

**What Phase 5 Doesn't Validate**:

- Whether users find the docs helpful (requires user testing)
- Documentation accuracy over time (requires maintenance)
- Completeness of edge cases (evolves with usage)

**Commit**: `docs: [#ISSUE] add your-command user documentation`

### Phase 6: Integration Polish (Optional)

**Goal**: Improve discoverability and user guidance.

**Ideas**:

- Add hints in related command outputs
- Update help text to mention new command
- Add to command workflow diagrams
- Cross-link documentation

**Example** (from purge command):

```rust
// After destroy completes, hint about purge
self.progress.output().lock().borrow_mut().result(&format!(
    "💡 Local data preserved for debugging. To completely remove:\n   \
     torrust-tracker-deployer purge {name} --force"
));
```

**Commit**: `feat: [#ISSUE] improve your-command discoverability`

## Implementation Checklist

Use this for tracking progress:

```markdown
- [ ] Phase 1: Presentation layer stub
  - [ ] CLI command variant
  - [ ] Router integration
  - [ ] Controller skeleton
  - [ ] Error types with help()
  - [ ] Container wiring
  - [ ] Manual test: command runs
  - [ ] Commit presentation stub
- [ ] Phase 2: Application handler
  - [ ] Handler with business logic
  - [ ] Application error types
  - [ ] Unit tests for handler (tests/mod.rs)
  - [ ] Controller delegates to handler
  - [ ] SDK method added (if applicable)
  - [ ] SdkError variant + From impl added (if applicable)
  - [ ] `cargo test --doc --workspace` passes
  - [ ] Manual test: real behavior
  - [ ] Commit application handler
- [ ] Phase 3: Confirmation (if needed)
  - [ ] Interactive prompt
  - [ ] --force flag support
  - [ ] Stdin reading
  - [ ] Manual test: both modes
  - [ ] Commit confirmation
- [ ] Phase 4: E2E tests
  - [ ] Success scenario test
  - [ ] Error handling test
  - [ ] Edge case tests
  - [ ] Helper methods in ProcessRunner
  - [ ] Assertion methods
  - [ ] All tests pass
  - [ ] Commit E2E tests
- [ ] Phase 5: Documentation
  - [ ] Create docs/user-guide/commands/your-command.md
  - [ ] Update docs/user-guide/commands/README.md
  - [ ] Update docs/console-commands.md
  - [ ] Update docs/features/active-features.md status
  - [ ] Pass markdown linting
  - [ ] Commit documentation
- [ ] Phase 6: Polish (optional)
  - [ ] Add discoverability hints
  - [ ] Update related commands
  - [ ] Commit polish
- [ ] Final verification
  - [ ] cargo run --bin linter all
  - [ ] All E2E tests pass
  - [ ] Pre-commit checks pass
  - [ ] Create PR
```

## Code Organization

Follow DDD layer placement rules from [`docs/contributing/ddd-layer-placement.md`](../../../docs/contributing/ddd-layer-placement.md):

### Presentation Layer (`src/presentation/`)

- CLI command definitions
- Controllers (orchestration, user interaction)
- User output formatting
- Confirmation prompts
- Progress reporting
- Presentation-specific errors

### Application Layer (`src/application/`)

- Command handlers (business workflows)
- Use case orchestration
- Application service coordination
- Application-specific errors
- Command DTOs (if needed)

### Domain Layer (`src/domain/`)

- Business entities (EnvironmentName, etc.)
- Value objects
- Domain errors
- Business rules and invariants
- Domain services (if pure logic)

### Infrastructure Layer (`src/infrastructure/`)

- External tool wrappers (Ansible, OpenTofu)
- Repository implementations
- File system operations
- API clients
- Database access

## Example: The Purge Command

**Real implementation reference**: [PR #323](https://github.com/torrust/torrust-tracker-deployer/pull/323)

**Phases executed**:

1. Presentation stub (6841d94b) - 671 lines
2. Application handler (e053e57b) - 521 lines
3. Confirmation prompt (45513e99) - 85 lines
4. E2E tests (1aaf7573) - 449 lines, 5 scenarios
5. Documentation (13f773f2) - 555 lines (426-line user guide)
6. Destroy hint (be2a5a74) - Small UX improvement

**Total**: ~2,224 insertions, production-ready feature

**Key files to reference**:

- Presentation: [`src/presentation/controllers/purge/handler.rs`](../../../src/presentation/controllers/purge/handler.rs)
- Application: [`src/application/command_handlers/purge/handler.rs`](../../../src/application/command_handlers/purge/handler.rs)
- Tests: [`tests/e2e/purge_command.rs`](../../../tests/e2e/purge_command.rs)
- Docs: [`docs/user-guide/commands/purge.md`](../../../docs/user-guide/commands/purge.md)

### Example: `validate` Command (This Skill Guide's Validation)

**Read-only command following this skill guide (Phases 1-2, 4-5):**

- Presentation: [`src/presentation/controllers/validate/handler.rs`](../../../src/presentation/controllers/validate/handler.rs)
- Application: [`src/application/command_handlers/validate/handler.rs`](../../../src/application/command_handlers/validate/handler.rs)
- Tests: [`tests/e2e/validate_command.rs`](../../../tests/e2e/validate_command.rs)
- Docs: [`docs/user-guide/commands/validate.md`](../../../docs/user-guide/commands/validate.md)
- Implementation commit: [272847e3](https://github.com/torrust/torrust-tracker-deployer/commit/272847e3)

**Key learnings from validate implementation:**

- Phase 3 skipped (read-only command needs no confirmation)
- Phase 6 skipped (integration polish deferred to future iteration)
- Domain validation reuses `EnvironmentParams` conversion (SSH key checking, constraints)
- E2E tests verify read-only behavior (no deployment created)
- Documentation includes "When NOT to Use" section for anti-patterns

## Tips & Best Practices

### Start Simple, Iterate Fast

- Phase 1 stub can be 10 lines - just make it runnable
- Test after every phase before moving to the next
- Commit after each phase (small, focused commits)

### Defer Complexity

- Don't worry about error handling in Phase 1
- Don't implement full business logic until Phase 2
- Don't add confirmation until basic flow works

### Leverage E2E Tests

- Write tests that match how users invoke the command
- Don't mock external dependencies - test real behavior
- Use `--force` flag in tests to skip interactive prompts

### Focus on User Experience

- Clear progress messages at each step
- Helpful error messages with actionable `.help()` suggestions
- Show what changed after command completes
- Provide examples in help text

### Follow Project Conventions

- Read [`docs/contributing/`](../../../docs/contributing/) before starting
- Follow commit message format: `type: [#ISSUE] description`
- Run pre-commit checks: `./scripts/pre-commit.sh`
- Update roadmap after completion

### Watch for Clippy Pedantic Gotchas

This project enables `clippy::pedantic`. Two patterns that commonly trigger warnings when adding commands:

1. **`unused_self`**: Validation helper methods that don't use `self` must be associated functions:

   ```rust
   // ❌ Triggers unused_self
   fn validate_name(&self, name: &str) -> Result<()> { ... }

   // ✅ Correct: associated function, call as Self::validate_name(name)?
   fn validate_name(name: &str) -> Result<()> { ... }
   ```

2. **`doc_link_with_quotes`**: Shell command substitutions in `///` doc comments (e.g., `` `"$(torrust-tracker-deployer your-command my-env)"` ``) trigger this lint. Place the allow attribute **before** the entire doc block, not inside it:

   ```rust
   // ✅ Place BEFORE the doc comment block
   #[allow(clippy::doc_link_with_quotes)]
   /// Check whether an environment exists.
   ///
   /// Shell usage: `"$(torrust-tracker-deployer your-command my-env)"`
   YourCommand { ... },
   ```

## Related Documentation

- **Architecture**: [`docs/codebase-architecture.md`](../../../docs/codebase-architecture.md)
- **DDD Layer Placement**: [`docs/contributing/ddd-layer-placement.md`](../../../docs/contributing/ddd-layer-placement.md)
- **Error Handling**: [`docs/contributing/error-handling.md`](../../../docs/contributing/error-handling.md)
- **Output Handling**: [`docs/contributing/output-handling.md`](../../../docs/contributing/output-handling.md)
- **Testing Guide**: [`docs/contributing/testing/`](../../../docs/contributing/testing/)
- **Development Principles**: [`docs/development-principles.md`](../../../docs/development-principles.md)

## Next Steps After Implementation

1. **Update feature status**: Change the command's row from `📋 Specified` → `✅ Implemented` in [`docs/features/active-features.md`](../../../docs/features/active-features.md). This is the per-command tracking file; it is separate from the general [`docs/roadmap.md`](../../../docs/roadmap.md).
2. **Remove issue spec**: Delete from `docs/issues/` after PR merge
3. **Update GitHub issue**: Close linked issue or update epic progress
4. **Consider skill**: If command pattern is reusable, document in skills/
