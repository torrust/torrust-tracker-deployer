# Linter Auto-fix Feature

This folder contains the complete documentation for the linter auto-fix feature implementation.

## 📄 Documents

### [specification.md](./specification.md)

The main feature specification including:

- Overview and goals
- Feature description with `--fix` flag usage
- Decision rationale (why Option 3 was chosen)
- Linter auto-fix support matrix
- Expected behavior and output format using tracing
- Safety considerations
- Definition of done
- Testing strategy

### [questions.md](./questions.md)

Clarifying questions that need to be answered before implementation:

- Linter scope and tools
- Fix behavior (what gets fixed)
- Output verbosity using tracing
- Exit code behavior
- Git integration
- Error handling
- Testing requirements
- Documentation updates
- Backward compatibility
- Timeline and priority

## 📋 Status

**Current Phase**: Specification Complete

**Next Steps**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ✅ Answer clarifying questions in `questions.md`
4. ✅ Update specification based on answers
5. ⏳ Create detailed implementation plan
6. ⏳ Review implementation plan
7. ⏳ Commit documentation
8. ⏳ Begin implementation

## 🎯 Quick Summary

Add a `--fix` flag to the linter binary that:

- Automatically fixes issues for linters that support auto-fix
- Uses **yamlfmt** for YAML formatting (not prettier)
- Uses tracing crate with **minimal verbosity** (show only "Fixed N files" summaries)
- Shows only remaining errors after auto-fix
- Allows developers to use git to see what changed
- **Auto-installs missing tools** (matches current linter behavior)
- Maintains backward compatibility (no `--fix` = current behavior)
- **Incremental implementation**: One linter at a time for easier testing and commits

**Usage**:

```bash
# Check only (current behavior)
cargo run --bin linter all

# Fix and check
cargo run --bin linter all --fix

# Individual linters
cargo run --bin linter markdown --fix
```

## � Key Decisions

Based on answers in [questions.md](./questions.md), the following key decisions were made:

1. **YAML Tool**: Use **yamlfmt** (not prettier) for YAML formatting

   - Reason: More focused tool, simpler to integrate

2. **Fix Scope**: Fix the same files that current linter checks (Option B)

   - No expansion to additional file types
   - Consistent with existing linter behavior

3. **Output Verbosity**: **Minimal** - show only "Fixed N files" summaries

   - Keep output clean and focused
   - Users can see detailed changes via `git diff`
   - Only display errors that still need attention

4. **Error Handling**: **Auto-install missing tools** (Option D)

   - Matches current linter behavior
   - Seamless developer experience
   - Logs installation for transparency

5. **Testing Strategy**: Unit + Integration + E2E + Manual

   - Comprehensive but focused
   - No property-based testing needed for initial version

6. **Implementation**: Incremental - one linter at a time

   - Easier to review, test, and commit
   - Reduces risk and complexity
   - Can deploy partial functionality

7. **Git Integration**: Don't auto-stage changes
   - Let git track changes naturally
   - Developers review with `git diff` before committing

## �🔗 Related Documentation

- [Error Handling Guide](../../contributing/error-handling.md)
- [Development Principles](../../development-principles.md)
- [Linting Guide](../../contributing/linting.md)
- [Logging Guide](../../contributing/logging-guide.md)
