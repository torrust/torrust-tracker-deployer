# Clarifying Questions for Linter Auto-fix Feature

## 🤔 Questions for Implementation

Please answer these questions by editing this file directly. Add your answers after each question.

---

### 1. **Scope - Which Linters Get Auto-fix?**

Based on the research, I identified these linters with auto-fix:

- ✅ **markdown**: `npx markdownlint-cli --fix`
- ✅ **yaml**: Should we use `prettier --write` or is there another tool?
- ✅ **clippy**: `cargo clippy --fix --allow-dirty --allow-staged`
- ✅ **rustfmt**: Already auto-formats (no change needed)
- ✅ **taplo**: `taplo fmt` for TOML files
- ❌ **shellcheck**: No auto-fix (skip)
- ❌ **cspell**: No auto-fix (skip)

**Questions:**

- Is this the correct list of linters?
- For YAML, should we use `prettier` or another tool? (I see prettier might not be installed)
- Are there any other linters I'm missing?

**Your Answer:**

- Is this the correct list of linters?

Yes, it is.

- For YAML, should we use `prettier` or another tool? (I see prettier might not be installed)

We can use [yamlfmt](https://github.com/google/yamlfmt) if it's compatible with the YAML linter we are using.

- Are there any other linters I'm missing?

No, there are no other linters to add.

### 2. **Fix Behavior - What Gets Fixed?**

When running `cargo run --bin linter all --fix`, should we:

- **Option A**: Fix ALL files in the project (even those not staged/modified)
- **Option B**: Only fix files that the linter would check anyway (current linter behavior)
- **Option C**: Only fix staged files (git staged files only)

**My assumption:** Option B (same scope as current linting) - is that correct?

**Your Answer:**

Yes, you are right. Option B is the correct one.

### 3. **Output Verbosity and Logging**

You mentioned:

- "We only need to provide the remaining errors. Developers can check what was changed with git."
- "The output should follow the current pattern. We are only using logging as output (tracing crate)."
- "Logs can be nested using tracing instrumentation (spans) to group events related to one linter."

**Confirmed approach:**

- Use `tracing` crate for all output (consistent with current implementation)
- Use tracing spans to group operations per linter (optional enhancement)
- Log auto-fix operations at INFO level
- Only show errors that remain after auto-fix
- Maintain current logging targets (e.g., `target: "markdown"`)

**Example with tracing:**

```rust
#[tracing::instrument(name = "markdown", skip_all)]
pub fn run_markdown_linter_with_fix(fix: bool) -> Result<()> {
    if fix {
        info!("Applying auto-fix...");
        // Run fix command
    }
    info!("Scanning markdown files...");
    // Run check
    // ...
}
```

**Question:** Do you want tracing spans for grouping linter operations, or is the current flat logging with targets sufficient?

**Your Answer:**

The current flat logging with targets is sufficient. Regarding "Log auto-fix operations at INFO level", we should not be too verbose. We can show only a summary of the number of files fixed per linter.

### 4. **Exit Code Behavior**

What should the exit code be in these scenarios?

- All linters pass without fixes needed: **0** ✅
- Some files auto-fixed, all linters pass after fix: **0** ✅
- Some files auto-fixed, but errors remain: **non-zero** ✅
- Auto-fix fails (command error): **non-zero** ✅

**Question:** Is this correct?

**Your Answer:**

Yes, it's correct.

### 5. **Git Integration**

Should we:

- **Option A**: Just run fix commands, let git track changes naturally (unstaged)
- **Option B**: Automatically stage fixed files
- **Option C**: Show git diff after fixes

**My assumption:** Option A (just fix, don't auto-stage) - correct?

**Your Answer:**

Yes, it's correct.

### 6. **Error Handling**

If auto-fix command fails (e.g., `npx` not found, network error):

- **Option A**: Fail immediately, don't continue with other linters
- **Option B**: Log error, skip that linter's auto-fix, continue with checking
- **Option C**: Log error, skip that linter entirely (no fix or check)

**My suggestion:** Option B (log error, still run check) - agree?

**Your Answer:**

I want a different option: **Option D - Auto-install missing tools**. Automatically install the missing tool (e.g., npm package) if it's not found, the same way we do for the linters. After installation, proceed with auto-fix and checking.

### 7. **Definition of Done - Testing**

What level of testing do you expect?

- Unit tests for CLI parsing? ✅
- Integration tests for each linter? ✅
- E2E test with actual broken files that get fixed? ✅
- Manual testing checklist? ✅

**Question:** Is this sufficient?

**Your Answer:**

Yes, that should be enough.

### 8. **Documentation Updates**

Which documents need updating?

- `docs/contributing/commit-process.md` - Pre-commit checklist ✅
- `docs/contributing/linting.md` - Linting documentation ✅
- `README.md` - Main readme (if it mentions linting) ✅
- Any others?

**Your Answer:**

No, that's enough.

### 9. **Backward Compatibility**

Should the behavior without `--fix` remain **exactly** the same as current?

- Same output format? ✅
- Same exit codes? ✅
- Same file paths checked? ✅

**Question:** Confirm this is correct?

**Your Answer:**

Yes, that's correct.

### 10. **Timeline & Priority**

Should we implement this as:

- **Option A**: Implement all linters at once
- **Option B**: Start with markdown only, iterate with others
- **Option C**: Core infrastructure first, then add linters incrementally

**My suggestion:** Option A (all at once, it's not that complex) - agree?

**Your Answer:**

Yes, we can do it all at once. However, during the implementation, I want to work on one linter at a time so we can test each one properly and commit the changes incrementally.

## 📋 Summary of Assumptions

Based on your feedback, here are my assumptions (please confirm or correct):

1. ✅ Add `--fix` flag to existing `cargo run --bin linter` binary
2. ✅ Fix same files that linter checks (project-wide)
3. ✅ **UPDATED**: Show only remaining errors after fix, not what was fixed
4. ✅ Exit 0 only if all errors resolved (fixed or none)
5. ✅ Don't auto-stage files, just fix them (developers use git to see changes)
6. ✅ If fix command fails, log error and continue with check
7. ✅ Implement all linters with auto-fix support at once
8. ✅ Comprehensive testing (unit + integration + manual)
9. ✅ Update documentation in commit-process and linting guides

**Your Confirmation:**

Regarding point 6: It's OK, but if the command fails because the tool is missing, I want it to be installed automatically (same as we do for linters).

Regarding point 7: Yes, but during the implementation, I want to work on one linter at a time so we can test each one properly and commit the changes incrementally.

### 11. **Parallel Execution (Moved to Separate Feature)**

**Note**: Parallel execution has been moved to a separate feature specification.

**See**: [Parallel Linter Execution Feature](../linter-parallel-execution/specification.md)

**Decision for auto-fix feature**: Use sequential execution (current approach)

**Rationale**:

- Current performance (~13s) is acceptable for pre-commit workflow
- Parallel execution is a separate optimization that can be added later
- Auto-fix works correctly with sequential execution
- Focus on implementing auto-fix functionality first (higher priority)

**Compatibility**: Auto-fix will work correctly with parallel execution if that feature is implemented in the future.

---

## 🚀 Next Steps

Once you've answered these questions:

1. I'll update the specification based on your answers
2. Create a detailed implementation plan
3. You'll review the plan
4. Commit the documentation
5. Start implementing the feature
