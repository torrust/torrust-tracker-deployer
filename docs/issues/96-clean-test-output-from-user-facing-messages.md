# Clean Test Output from User-Facing Messages

**Issue**: [#96](https://github.com/torrust/torrust-tracker-deployer/issues/96)
**Parent Epic**: N/A (standalone improvement)
**Related**: Development Principles - User Friendliness, Observability

> **⚠️ BLOCKED: DO NOT IMPLEMENT YET**
>
> This issue depends on a UserOutput refactoring issue that needs to be defined and implemented first.
> Once the refactoring issue is created, update this document with:
>
> - Link to the blocking issue
> - Any updated implementation approach based on the refactoring

## 📋 Overview

When running `cargo test`, user-facing output messages (emojis, progress indicators, error messages with tips) appear in stderr, making test output noisy and difficult to read. This issue addresses the need to keep test output clean by default while maintaining observability for debugging.

## 🎯 Problem Statement

### Current Behavior

Running `cargo test` produces mixed output in stderr containing:

1. **Cargo compilation/test progress** (normal, should remain)
2. **User-facing application output** (problematic, should be silenced in tests)
3. **Test names and results** (normal, goes to stdout)

### Evidence

From captured stderr during `cargo test`, we see user-facing messages that should not appear:

```text
⏳ First message
✅ Second message
❌ Third message
⏳ Test progress
✅ Test success
⏳ Loading configuration from '/tmp/.tmpQ4cTH1/config.json'...
⏳ Loading configuration from '/tmp/.tmptuHLvU/config.json'...
⏳ Loading configuration from '/tmp/.tmpbwrMDD/config.json'...
⏳ Creating environment 'test-create-env'...
⏳ Validating configuration and creating environment...
[... many more messages ...]
✅ Configuration template generated: /tmp/.tmpelqHWs/test.json

Next steps:
1. Edit the template file and replace placeholder values:
   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')
   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key
   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key
2. Review default values:
   - username: 'torrust' (can be changed if needed)
   - port: 22 (standard SSH port)
3. Create the environment:
   torrust-tracker-deployer create environment --env-file /tmp/.tmpelqHWs/test.json
```

**Note**: Cargo's own progress messages should remain untouched:

- Compilation progress: `Compiling ring v0.17.14`
- Test execution: `Running unittests src/lib.rs (target/debug/deps/...)`

### Root Causes

The issue has **two distinct sources**:

#### 1. Direct UserOutput Usage in Tests

Tests directly call command handlers that create `UserOutput` with real stderr:

```rust
// src/presentation/commands/context.rs
#[test]
fn it_should_allow_accessing_output_multiple_times() {
    let mut ctx = CommandContext::new(working_dir);

    // These write directly to stderr!
    ctx.output().progress("First message");
    ctx.output().success("Second message");
    ctx.output().error("Third message");
}
```

**Solution**: Use `VerbosityLevel::Silent` in test contexts.

#### 2. Subprocess Execution in E2E Tests

E2E tests spawn the application as a subprocess, which writes to its own stderr:

```rust
// tests/ai_enforcement.rs or similar
ProcessRunner::new()
    .working_dir(temp_workspace.path())
    .run_create_command("./environment.json")
    .expect("Failed to run create command");
```

The subprocess writes to stderr independently of the test harness.

**Possible Solutions**:

- Add `--silent` CLI flag to suppress all user output
- Redirect subprocess stderr to file and only show on failure
- Add `--log-output` option to control where output goes

## 🎯 Goals

- [ ] Test output is clean by default (no user-facing messages in stderr)
- [ ] User-facing output still available when needed for debugging
- [ ] Production behavior unchanged (normal verbose output)
- [ ] E2E test output can be captured/silenced when needed
- [ ] Solution maintains observability principles

## � Implementation Strategy

This issue will be addressed **incrementally, one message at a time**. Each message source will be:

1. **Investigated** - Find where the message originates
2. **Analyzed** - Determine the best solution for that specific case
3. **Fixed** - Apply the appropriate solution
4. **Verified** - Confirm the message no longer appears in test output

The solution approaches described below are **guidelines, not prescriptions**. Each case may require a different approach or a combination of approaches. New patterns may emerge during implementation.

### Progress Tracking

Track each message category as it's addressed:

- [ ] Progress messages (⏳) from direct test usage
- [ ] Success messages (✅) from direct test usage
- [ ] Error messages (❌) from direct test usage
- [ ] Template generation output from subprocess tests
- [ ] Environment creation output from subprocess tests
- [ ] Destroy command output from subprocess tests
- [ ] Other messages discovered during implementation

## �📐 Solution Approaches (Guidelines)

### Approach 1: Silent Mode for Direct Test Usage

**For**: Tests that directly call command handlers

**Changes Needed**:

1. Add `VerbosityLevel::Silent` variant
2. Update `UserOutput` to check verbosity before writing
3. Update test helpers to use silent mode by default
4. Allow tests to override when they need to verify output

**Pros**:

- ✅ Simple and direct
- ✅ Maintains existing test structure
- ✅ Easy to implement
- ✅ No changes to production code paths

**Cons**:

- ❌ Doesn't solve subprocess output issue
- ❌ Requires updating many test contexts

### Approach 2: CLI Flag for Subprocess Tests

**For**: E2E tests that spawn the application as subprocess

**Option A: `--silent` flag**

```bash
torrust-tracker-deployer --silent create environment --env-file config.json
```

**Option B: `--quiet` flag (following cargo convention)**

```bash
torrust-tracker-deployer --quiet create environment --env-file config.json
```

#### Option C: Redirect stderr to file, show only on failure

```rust
let result = ProcessRunner::new()
    .capture_stderr()  // Capture but don't display
    .run_create_command("./environment.json");

if !result.success() {
    eprintln!("Command failed:\n{}", result.stderr());
}
```

**Pros**:

- ✅ Works for subprocess execution
- ✅ Users can enable silence when needed
- ✅ Aligns with standard CLI conventions (--quiet/--silent)
- ✅ Option C shows errors when they occur

**Cons**:

- ❌ Adds new CLI surface area
- ❌ Requires documentation
- ❌ Option C more complex to implement

### Approach 3: Hybrid Solution

Combine both approaches:

1. **For unit/integration tests**: Use `VerbosityLevel::Silent` in test contexts
2. **For E2E subprocess tests**: Add `--quiet` CLI flag + capture stderr
3. **Progressive enhancement**: Show stderr only on subprocess failure

**Note**: This is a suggested approach. During implementation, we may discover that different messages require different solutions, or that a simpler approach works for all cases.

## 🔧 Implementation Plan

**Strategy**: Iterative, message-by-message investigation and fixing.

### Workflow for Each Message

For each message category identified in stderr:

1. **Locate Source**

   - Search codebase for the exact message text
   - Identify if it comes from direct test calls or subprocess execution
   - Document the call stack leading to the output

2. **Analyze Context**

   - Determine why the message appears
   - Check if it's from unit tests, integration tests, or E2E tests
   - Identify if it's intentional debug output or unintended leakage

3. **Choose Solution**

   - Select the appropriate approach from the guidelines below
   - Consider complexity vs benefit
   - Document the reasoning for the chosen approach

4. **Implement Fix**

   - Apply the minimal change needed
   - Add tests to prevent regression
   - Verify no other messages appear

5. **Verify**
   - Run `cargo test > /tmp/test_stdout.txt 2> /tmp/test_stderr.txt`
   - Check stderr for any remaining unwanted output
   - Move to next message category

### Phase 1: Investigation and Foundation (Time: Variable)

**First Target**: "⏳ First message" and related progress messages from direct test usage

**Tasks**:

- [ ] Locate source of "First message" output
- [ ] Identify test(s) producing this output
- [ ] Document call stack and context
- [ ] Decide on solution approach
- [ ] Implement fix (may include adding Silent mode if not yet implemented)
- [ ] Verify fix with captured stderr
- [ ] Document findings and solution in PR or issue comments

**Files likely to modify** (based on initial analysis):

- `src/presentation/user_output.rs` - May need Silent verbosity level
- `src/presentation/commands/context.rs` - May need test-specific context
- Test files calling `CommandContext::new()` directly

**Acceptance Criteria**:

- [ ] "First message", "Second message", "Third message" no longer appear in stderr
- [ ] Test still passes
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Time estimate**: 2-4 hours (includes investigation, design, and implementation)

### Subsequent Targets: Iterative Progression

After completing Phase 1, continue with the message-by-message workflow for remaining output:

- Template generation messages in subprocess tests
- Other progress indicators (⏳)
- Success messages (✅)
- Error messages with tips (❌)
- Multi-line guidance sections

Each message will be investigated individually using the same workflow as Phase 1.

**Note**: Implementation order and solutions will be determined based on findings during investigation. This ensures flexibility to handle unexpected cases or complexities as they arise.

## ✅ Acceptance Criteria

### Functional Requirements

- [ ] Running `cargo test` produces clean stderr (no user-facing messages)
- [ ] Cargo's own progress messages remain visible
- [ ] Tests can suppress user-facing output without breaking functionality
- [ ] CLI provides mechanism to control output verbosity (if needed for E2E tests)
- [ ] Error messages still appear when commands fail
- [ ] E2E test failures show subprocess stderr for debugging

### Quality Requirements

- [ ] All existing tests pass
- [ ] Tests added to verify clean test output behavior
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] Documentation updated as needed for any new features or patterns
- [ ] No regression in production output behavior

### User Experience

- [ ] Test output is easy to read (no noise)
- [ ] Failed tests show sufficient context for debugging
- [ ] CLI help documentation updated if new flags or features added
- [ ] Error messages remain clear and actionable in all output modes

## 📚 Related Documentation

- [Development Principles](../development-principles.md) - User Friendliness and Observability
- [Error Handling Guide](../contributing/error-handling.md) - Error message requirements
- [Testing Conventions](../contributing/testing/) - Test output patterns
- [Cargo Book - Test Output](https://doc.rust-lang.org/cargo/commands/cargo-test.html) - Understanding test capture

## 📊 Messages to Clean

### Categories of Output to Silence

**Progress Messages** (⏳):

```text
⏳ Loading configuration from '/tmp/.tmpQ4cTH1/config.json'...
⏳ Creating environment 'test-create-env'...
⏳ Validating configuration and creating environment...
⏳ Generating configuration template...
⏳ Destroying environment 'test-env'...
⏳ Tearing down infrastructure...
```

**Success Messages** (✅):

```text
✅ Environment 'custom-location-env' created successfully
✅ Environment 'duplicate-env' created successfully
✅ Configuration template generated: /tmp/.tmpelqHWs/test.json
```

**Error Messages with Tips** (❌):

```text
❌ Configuration file not found: /tmp/.tmpKiDu4O/nonexistent.json
Tip: Check that the file path is correct: ls -la /tmp/.tmpKiDu4O/nonexistent.json

❌ Failed to parse configuration file '/tmp/.tmp6CUY8O/invalid.json' as JSON: ...
Tip: Validate JSON syntax with: jq . < /tmp/.tmp6CUY8O/invalid.json

❌ Configuration validation failed: Invalid environment name: ...
Tip: Review the validation error and fix the configuration file
```

**Multi-line Guidance**:

```text
Next steps:
1. Edit the template file and replace placeholder values:
   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')
   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key
   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key
2. Review default values:
   - username: 'torrust' (can be changed if needed)
   - port: 22 (standard SSH port)
3. Create the environment:
   torrust-tracker-deployer create environment --env-file /tmp/.tmpelqHWs/test.json
```

### Messages to Keep (Cargo Output)

These are normal cargo progress messages and should remain:

```text
Compiling ring v0.17.14
Compiling rustls v0.23.32
Finished `test` profile [unoptimized + debuginfo] target(s) in 14.30s
Running unittests src/lib.rs (target/debug/deps/torrust_tracker_deployer_lib-24017c1290c2d59f)
Running tests/e2e_create_command.rs (target/debug/deps/e2e_create_command-735e701736556ea8)
Doc-tests torrust_tracker_deployer_lib
```

## 🔍 Investigation Checklist

When implementing, investigate and document:

- [ ] Which tests write to stderr directly vs via subprocess?
- [ ] Are there other sources of test output we haven't identified?
- [ ] Do any tests intentionally verify output content?
- [ ] Should `Drop` implementations respect silent mode?
- [ ] How do other CLI tools handle test output (cargo, git, docker)?
- [ ] Should there be a `RUST_TEST_QUIET` environment variable?

## 📝 Notes

### Design Decisions to Make

1. **Flag naming**: `--quiet` (cargo convention) vs `--silent`
2. **Flag precedence**: What if both `--quiet` and `--verbose` are set?
3. **Error verbosity**: Should errors be more verbose even in quiet mode?
4. **Test environment variable**: Should `RUST_TEST=1` automatically enable silent mode?

### Open Questions

- Should we add a `--test-mode` internal flag for subprocess tests?
- Do we need separate verbosity for stdout vs stderr?
- Should `Drop` implementations check verbosity level?
- Is there value in keeping progress messages but hiding success messages?

## ⏱️ Time Estimate

**Phase 1 (First Target)**: 2-4 hours

**Total Project**: Unknown at this stage. Will be estimated after Phase 1 completion and assessment of remaining complexity.

**Note**: This iterative approach allows for better time estimation as each message case is investigated and understood. Early cases will inform time estimates for similar patterns in remaining messages.
