# Clarifying Questions for Linter Parallel Execution Feature

## 🤔 Questions for Implementation

**Note**: This feature has been assessed and **deferred** as low priority. These questions are documented for future reference if/when this feature is reconsidered.

Please answer these questions if you decide to implement this feature:

---

### 1. **Performance Requirements**

**Current State**:

- Sequential execution: ~13 seconds
- Parallel execution potential: ~9 seconds (~30% faster, 4s improvement)

**Questions**:

- Is the current ~13s execution time causing problems for developers?
- Have users complained about linting speed?
- Is CI/CD pipeline time a concern where this 4s would matter?
- What performance threshold would justify the implementation complexity?

**Your Answer:**

- Is the current ~13s execution time causing problems for developers?

Not really, it's acceptable for a pre-commit hook.

- Have users complained about linting speed?

No. I'm the only developer using it for now.

- Is CI/CD pipeline time a concern where this 4s would matter?

No, it's not a concern.

- What performance threshold would justify the implementation complexity?

If we had more linters added in the future, making the total time significantly longer (e.g., >25s), then parallel execution would be more justified.

### 2. **Output Handling Strategy**

**Challenge**: Current linters print errors immediately using `println!()` and `eprintln!()`. Parallel execution would create messy, interleaved output.

**Solution Options**:

- **Option A**: Refactor all linters to buffer output, display sequentially after parallel execution
- **Option B**: Use a synchronized output mechanism (mutex-protected stdout)
- **Option C**: Implement structured logging that handles concurrent output (e.g., `tracing` with proper formatting)

**Questions**:

- Which output handling approach do you prefer?
- Should we maintain the current output format exactly, or can we redesign it?
- Is the current `println!()`/`eprintln!()` approach acceptable to keep (which would rule out naive parallelization)?

**Your Answer:**

- Which output handling approach do you prefer?

I prefer Option B.

- Should we maintain the current output format exactly, or can we redesign it?

I would keep the current one unless it's too complex to implement as-is.

- Is the current `println!()`/`eprintln!()` approach acceptable to keep (which would rule out naive parallelization)?

Mixing metadata (e.g., "Running linter X") is not a problem. The issue is mixing error reporting from different linters. We can collect the output from each linter and print it sequentially after all linters finish.

### 3. **Grouping Strategy**

**Proposed Groups**:

- **Group 1 (Parallel)**: markdown, yaml, toml, shellcheck, cspell
- **Group 2 (Sequential)**: clippy → rustfmt

**Questions**:

- Is this grouping strategy correct?
- Should cspell (read-only, checks all files) be in a separate group?
- Are there any other file conflicts we haven't identified?
- Should we make grouping configurable?

**Your Answer:**

- Is this grouping strategy correct?

We can include "rustfmt" in the first group since it only modifies `*.rs` files, and clippy is the only one that needs to be in the sequential group.

- Should cspell (read-only, checks all files) be in a separate group?

Yes, cspell can be in its own group since it doesn't modify any files and can run safely in parallel with others.

- Are there any other file conflicts we haven't identified?

No, I do not see any other conflicts.

- Should we make grouping configurable?

No, the current grouping is sufficient for now.

### 4. **Async Runtime Choice**

**Options**:

- **Option A**: `tokio` - Most popular, full-featured
- **Option B**: `async-std` - Alternative async runtime
- **Option C**: `rayon` - Data parallelism (simpler, no async/await)

**Questions**:

- Which async/parallel runtime should we use?
- Is adding `tokio` as a dependency acceptable?
- Would `rayon` be simpler since we don't need async I/O, just parallel execution?

**Your Answer:**

- Which async/parallel runtime should we use?

I think we should use `tokio` since it's the most popular and full-featured.

- Is adding `tokio` as a dependency acceptable?

Yes, adding `tokio` is acceptable.

- Would `rayon` be simpler since we don't need async I/O, just parallel execution?

It could be simpler, but I prefer `tokio` for its ecosystem and flexibility.

### 5. **Refactoring Scope**

**Required Changes**:

- All 7 linter modules need refactoring
- Create output buffering system
- Update error handling for parallel scenarios
- Add comprehensive testing

**Questions**:

- Should we refactor all linters at once, or incrementally?
- Start with a proof-of-concept (1-2 linters) first?
- What's the acceptable timeline for this refactoring?

**Your Answer:**

- Should we refactor all linters at once, or incrementally?

No, I think we should refactor them incrementally, one at a time, to ensure each linter works correctly before moving to the next. And commit the changes incrementally.

- Start with a proof-of-concept (1-2 linters) first?

Yes, starting with a proof-of-concept using 1-2 linters would be a good approach to validate the output handling and performance improvement.

- What's the acceptable timeline for this refactoring?

One day.

### 6. **Compatibility with Auto-fix**

**Integration**:

```rust
pub async fn run_all_linters_parallel(fix: bool) -> Result<()> {
    // Parallel execution with optional auto-fix
    // Group 1: Parallel with fix support
    // Group 2: Sequential (clippy, rustfmt)
}
```

**Questions**:

- Should parallel execution support auto-fix from the start, or add it later?
- Any concerns about auto-fix + parallel execution interaction?
- Should parallel execution be opt-in (flag) or default behavior?

**Your Answer:**

- Should parallel execution support auto-fix from the start, or add it later?

Yes, auto-fix should be supported from the start. Auto-fix is going to be implemented first, and it's a more valuable feature.

- Any concerns about auto-fix + parallel execution interaction?

No, I don't foresee any major issues. As long as each linter's output is properly isolated, there shouldn't be any conflicts.

- Should parallel execution be opt-in (flag) or default behavior?

No, I think it should be the default behavior once implemented.

### 7. **Testing Strategy**

**Test Scenarios**:

- Parallel execution with all linters passing
- Parallel execution with some linters failing
- Output ordering and formatting
- Race conditions and file conflicts
- Integration with auto-fix (if applicable)

**Questions**:

- What level of testing is required before merging?
- Should we test on different machines/OSes for timing issues?
- How do we verify output is clean and not interleaved?

**Your Answer:**

- What level of testing is required before merging?

Testing parallel execution is hard. I would add an extra option to run the linters sequentially for testing purposes.

- Should we test on different machines/OSes for timing issues?

No, not necessary.

- How do we verify output is clean and not interleaved?

We will test it manually and visually verify the output.

### 8. **Error Handling**

**Scenarios**:

- One linter fails in parallel group - continue with others?
- Async task panics - how to handle?
- Output buffering fails - fallback to sequential?

**Questions**:

- How should errors in parallel tasks be aggregated?
- Should one failure stop all linters, or continue and report all failures?
- What's the fallback strategy if parallel execution fails?

**Your Answer:**

- One linter fails in parallel group - continue with others?

Yes, continue with others.

- Async task panics - how to handle?

We should catch panics and report them as errors without crashing the entire process.
Although panics should be rare if we handle errors properly.

- Output buffering fails - fallback to sequential?

Yes, if output buffering fails, we can fallback to sequential execution as a safe fallback.

### 9. **Configuration**

**Options**:

- Make parallelization configurable via CLI flag: `--parallel`/`--sequential`
- Configuration file setting
- Environment variable
- Always parallel (no option)

**Questions**:

- Should parallel execution be opt-in or default?
- Do we need a way to disable it for debugging?
- Should users be able to configure grouping strategy?

**Your Answer:**

- Should parallel execution be opt-in or default?

It should be default, but optionally can be disabled with a flag.

- Do we need a way to disable it for debugging?

Yes, we should have a way to disable it for debugging purposes.

- Should users be able to configure grouping strategy?

Not necessary for now.

### 10. **Priority and Timeline**

**Current Decision**: Deferred as low priority

**Questions**:

- What would need to change to make this a higher priority?
- Is there a specific timeline when this might be reconsidered?
- What other features should be completed before this?

**Your Answer:**

- What would need to change to make this a higher priority?

If the auto-fix process takes too long, then we might reconsider this feature to speed it up.
Or if we add more linters that increase the total linting time significantly.

- Is there a specific timeline when this might be reconsidered?

When the execution time goes over 25 seconds.

- What other features should be completed before this?

The auto-fix feature should be completed first.

## 📋 Summary of Current Assessment

Based on the cost-benefit analysis in the specification:

- ✅ Current performance (~13s) is acceptable
- ❌ Implementation complexity is significant
- ❌ Risk of bugs during refactoring
- ✅ YAGNI principle applies - implement only if needed
- ✅ Focus on auto-fix feature first (higher value)

**Recommendation**: Keep this feature deferred unless circumstances change (more linters added, performance complaints, CI/CD optimization becomes critical, etc.)

---

## 🚀 Next Steps (If Feature is Prioritized)

Once you've answered these questions and decided to implement:

1. Create proof-of-concept with 1-2 linters
2. Validate output handling approach
3. Measure actual performance improvement
4. Create detailed implementation plan
5. Begin incremental refactoring
6. Test thoroughly before merging

## 🔗 Related Documentation

- [Linter Parallel Execution Specification](./specification.md)
- [Linter Auto-fix Feature](../linter-auto-fix/specification.md)
- [Linting Guide](../../contributing/linting.md)
