# Clarifying Questions for Exists Command

This document contains questions to clarify requirements, scope, and priorities before implementation begins. Product owners or stakeholders should answer these questions directly in the document.

---

## 🔍 Scope and Requirements

### 1. **Command Name**

**Question**: Do you agree with using `exists` (third-person) as the command name?

**Context**: The name `exists` was chosen because:

- It matches the SDK method: `Deployer::exists()`
- It matches the repository trait method: `EnvironmentRepository::exists()`
- It reads naturally in shell conditionals: `if deployer exists my-env; then …`
- Alternative candidates: `exist` (bare infinitive), `check` (too generic), `has` (unclear object)

**Your Answer**: [To be filled by product owner]

YEs, I agree.

### 2. **Core Functionality — What Constitutes "Exists"?**

**Question**: Should `exists` check only for the presence of `environment.json` (current `EnvironmentRepository::exists()` behavior), or should it also verify that the file is valid/loadable?

**Option A — File existence only** (recommended):

- Fast (`Path::exists()`, no IO beyond stat)
- Answers "has an environment been created with this name?"
- A corrupt file still means "yes, it exists" — use `show` to check readability

**Option B — File existence + loadability**:

- Slightly slower (reads and deserializes JSON)
- A corrupt file would report "does not exist" — which may be misleading
- Conflates existence with health

**Your Answer**: [To be filled by product owner]

It should only check for the file existence.

### 3. **Exit Code Semantics**

**Question**: Do you agree with the proposed exit code scheme?

**Proposed scheme**:

| Scenario                   | Exit Code |
| -------------------------- | --------- |
| Environment exists         | 0         |
| Environment does not exist | 1         |
| Invalid name / IO error    | 2         |

**Context**: This breaks the current convention where all errors exit with code 1. The rationale is that `exists` is a boolean query (like POSIX `test` and `grep -q`), where exit codes 0 and 1 carry semantic meaning (true/false) and code 2 signals an actual error.

**Alternative**: Keep exit code 1 for both "not found" and errors (consistent with other commands, but loses scriptability — callers cannot distinguish "no" from "broken").

**Your Answer**:

After analysis, we decided **not** to use the POSIX boolean exit code pattern (0=true, 1=false, 2=error). Instead, the command uses the same convention as all other commands:

- **Exit 0** = success (both `true` and `false` results) — the boolean value is output to stdout
- **Exit 1** = error (invalid name, IO failure, etc.)

**Rationale**:

1. It respects the existing project conventions (exit 0 = success, exit 1 = error)
2. It forces the user to explicitly handle all scenarios (exists, does not exist, error) rather than conflating "false" with "error" — this follows Rust's `Result<bool, Error>` philosophy
3. It's more readable because the caller knows the command returns a boolean value on stdout
4. It fits well with returning bare `true` or `false` as output (valid JSON values), same format for both human-readable and JSON output

The POSIX boolean pattern (used by `test`, `grep -q`) has a fundamental flaw: `if/else` constructs conflate "the answer is false" with "the command failed". Modern shells (Nushell, PowerShell, Fish) have moved past this 1970s limitation by separating the data channel from the error channel.

---

### 4. **Quiet Mode**

**Question**: Should the command support a `--quiet` / `-q` flag that suppresses stdout?

**Decision**: **Deferred.** Quiet mode will be implemented as a transversal feature for all commands in the future, rather than as a per-command flag. Users can redirect stdout to `/dev/null` in the meantime.

---

### 5. **Output When Environment Does Not Exist — Stdout or Stderr?**

**Question**: When the environment does not exist, should the message go to stdout or stderr?

**Option A — Stdout** (recommended):

- "Does not exist" is the _answer_ to the question, not an error
- Consistent: both `true` and `false` answers go to the same stream
- Easier to capture/parse programmatically

**Option B — Stderr**:

- Consistent with how other commands report "environment not found"
- Feels more "error-like" even though it is a valid result

**Your Answer**: [To be filled by product owner]

Option A.

### 6. **JSON Output Format**

**Question**: Should JSON output be supported from the start, or deferred?

**Proposed JSON structure**:

```json
{ "name": "my-environment", "exists": true }
```

**Context**: The global `--format json` flag already exists. Supporting it from day one is low effort since the output is trivially simple. Deferring it would be inconsistent with other commands that already support `--format json`.

**Your Answer**: [To be filled by product owner]

If I'm not wrong `true` or `false` are valid JSON values, so we can just return `true` or `false` instead of wrapping it in an object. This would be more concise and still valid JSON. So the output would be:

```json
true
```

or

```json
false
```

## 🎯 Technical Approach

### 7. **Router Exit Code Handling**

**Question**: How should the router handle the `exists` result?

**Context**: Since the command always exits 0 on success (both `true` and `false` results), the router does not need special exit code handling. The controller prints the boolean result to stdout and returns `Ok(())`. Errors propagate normally and cause exit code 1.

**Router code**:

```rust
Commands::Exists { environment } => {
    context
        .container()
        .create_exists_controller()
        .execute(&environment, context.output_format())?;
    Ok(())
}
```

This is simpler than the original proposal because there is no need to inspect the boolean result to set the exit code.

**Your Answer**:

Agreed. The router does not need special handling. This is consistent with all other commands and avoids the complexity of the POSIX boolean pattern.

---

### 8. **ProgressReporter — Use or Skip?**

**Question**: Should the `exists` command use the `ProgressReporter` step-tracking pattern?

**Context**: Other commands define step enums (e.g., `ShowStep::ValidateName`, `ShowStep::LoadEnvironment`) with progress reporting. The `exists` command is so fast (sub-millisecond) that progress reporting adds overhead and complexity without user benefit.

**Recommendation**: Skip `ProgressReporter` for this command. Use direct `UserOutput` calls for the single result line.

**Your Answer**: [To be filled by product owner]

I would skip it for this command, as it is very fast and does not have multiple steps that would benefit from progress reporting. The overhead of setting up progress reporting for a single-step command may not be justified.

### 9. **UserOutput vs Direct Println**

**Question**: Should the result be printed via the `UserOutput` system or directly to stdout?

**Context**: `UserOutput` provides structured output with formatting, colors, and verbosity awareness. For a single-line boolean result, it may be overkill. However, using it maintains consistency with other commands.

**Recommendation**: Use `UserOutput` for consistency, even if the output is minimal.

**Your Answer**: [To be filled by product owner]

Use `UserOutput` for consistency with other commands, even if the output is minimal. This allows for future enhancements (e.g., adding colors or additional info) without changing the output mechanism.

### 10. **SDK Integration**

**Question**: Should the existing SDK `Deployer::exists()` be updated to use the new `ExistsCommandHandler` instead of wrapping `show()`?

**Current SDK implementation**:

```rust
pub fn exists(&self, env_name: &EnvironmentName) -> Result<bool, ShowCommandHandlerError> {
    match self.show(env_name) {
        Ok(_) => Ok(true),
        Err(ShowCommandHandlerError::EnvironmentNotFound { .. }) => Ok(false),
        Err(e) => Err(e),
    }
}
```

**Proposed change**: Use `ExistsCommandHandler` directly (faster, cleaner, correct error type):

```rust
pub fn exists(&self, env_name: &EnvironmentName) -> Result<bool, ExistsCommandHandlerError> {
    let handler = ExistsCommandHandler::new(self.repository.clone());
    Ok(handler.execute(env_name)?.exists)
}
```

**Note**: This would be a breaking change to the SDK's `exists()` return type.

**Your Answer**: [To be filled by product owner]

Yes, there is no problem yet with breaking changes in the SDK, since we have not released a stable version. So we can update the SDK `exists()` method to use the new `ExistsCommandHandler` for better performance and cleaner implementation. We just need to make sure to update the return type and error handling accordingly.

## 📊 Priority and Timeline

### 11. **Priority Level**

**Question**: What is the priority of this feature? (High | Medium | Low)

**Context**: The SDK already has `exists()`, so programmatic users are served. The CLI gap primarily affects shell scripting and CI/CD pipelines. This is a small, well-scoped feature.

**Your Answer**: [To be filled by product owner]

High, because I want to include it in the first release to provide a complete CLI experience and enable scripting use cases.

### 12. **Timeline Expectations**

**Question**: Is there a target date for completion?

**Estimated effort**: 3-4 days (implementation + testing + documentation)

**Your Answer**: [To be filled by product owner]

1 hour.

### 13. **Dependencies**

**Question**: Does this feature depend on other work being completed first?

**Known Dependencies**:

- Requires `EnvironmentRepository::exists()` — already implemented
- Requires `EnvironmentName` validation — already implemented
- No blocking dependencies identified

**Your Answer**: [To be filled by product owner]

No.

## ✅ Success Criteria

### 14. **Definition of Done**

**Question**: Do you agree with the proposed acceptance criteria?

- [ ] `exists <env>` exits 0 when environment exists, with `true` on stdout
- [ ] `exists <env>` exits 0 when environment does not exist, with `false` on stdout
- [ ] `--format json` produces valid JSON (`true` or `false`)
- [ ] Invalid environment names produce a clear error (exit 1)
- [ ] Unit tests cover all handler paths
- [ ] E2E tests verify exit codes and output
- [ ] Documentation updated

**Your Answer**: [To be filled by product owner]

---

### 15. **Testing Requirements**

**Question**: What level of testing is expected?

**Proposed**:

- **Unit Tests**: Handler logic (exists, not exists, repository error)
- **E2E Tests**: Full command execution with exit code verification
- **No integration tests needed** (the handler is trivially thin)

**Your Answer**: [To be filled by product owner]

---

## ⚠️ Risk Assessment

### 16. **Exit Code Convention Break**

**Question**: Are you comfortable with this command using exit code 1 for "does not exist" (instead of treating it as an error)?

**Risk**: Other tooling or scripts that assume exit code 1 always means "error" could misinterpret the result.

**Mitigation**: Document the exit code semantics clearly. This convention is well-established in POSIX tools (`test`, `grep -q`, `diff --quiet`).

**Your Answer**:

This question is no longer applicable. After analysis, we decided to keep the standard convention: exit 0 = success, exit 1 = error. The boolean result (`true`/`false`) is communicated via stdout, not via exit codes. This means there is no convention break — the `exists` command behaves identically to all other commands regarding exit codes.

---

### 17. **Permission Edge Case**

**Question**: How should the command behave when the user lacks read permissions on the data directory?

**Option A**: Report `false` (environment effectively does not exist for this user)
**Option B**: Report an error (exit 2) explaining the permission issue

**Recommendation**: Option B — silently reporting `false` when the real issue is permissions could lead to data loss (e.g., user runs `create` thinking the environment does not exist, overwriting state).

**Note**: This depends on whether `EnvironmentRepository::exists()` propagates permission errors. Currently it uses `Path::exists()`, which returns `false` on permission denied. We may need to adjust the implementation to use `std::fs::metadata()` and handle `PermissionDenied` explicitly.

**Your Answer**: [To be filled by product owner]

---

### 18. **Backward Compatibility**

**Question**: Confirm that backward compatibility is not a concern?

**Context**: This is a new command. The only potential concern is updating the SDK `exists()` method's return type from `ShowCommandHandlerError` to `ExistsCommandHandlerError` — which is a breaking SDK change.

**Your Answer**: [To be filled by product owner]

---

## 💡 Additional Questions

### 19. **Should `exists` Also Report the Environment State?**

**Question**: Should the command optionally report the state (Created, Provisioned, etc.) when the environment exists?

**Example**: `torrust-tracker-deployer exists my-env --show-state`

**Output**: `Environment 'my-env' exists (state: Provisioned).`

**Consideration**: This would require loading the environment JSON (losing the performance advantage), but could be useful as a quick-check alternative to `show`. This could also be a future enhancement.

**Your Answer**: [To be filled by product owner]

---

### 20. **Multiple Environment Names?**

**Question**: Should the command accept multiple environment names in a single invocation?

**Example**: `torrust-tracker-deployer exists env1 env2 env3`

**Consideration**: This adds complexity (what exit code when some exist and some do not?) and can be achieved with a shell loop. Recommend deferring this.

**Your Answer**: [To be filled by product owner]

---

## 📝 Notes

- The SDK already has `Deployer::exists()` which wraps `show()` — this feature brings the same capability to the CLI
- The command is deliberately thin — essentially a CLI frontend for `EnvironmentRepository::exists()`
- Exit code semantics follow the project's standard convention (0 = success, 1 = error), with the boolean result communicated via stdout (`true`/`false`)
