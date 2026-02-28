# Exists Command

A new console command to check whether a deployment environment exists, providing a fast, unambiguous, scriptable boolean result.

## 📄 Documents

### [specification.md](./specification.md)

The main feature specification including:

- Overview and problem statement
- Feature goals and non-goals
- Proposed solution with exit code semantics
- Implementation details (component design, architecture)
- Edge cases analysis (corrupt files, permissions, race conditions)
- Definition of done
- Testing strategy

### [questions.md](./questions.md)

Clarifying questions that need to be answered before implementation:

- What constitutes "exists" (file presence vs. loadability)
- Exit code semantics and output format
- SDK integration and backward compatibility
- Permission edge cases

## 📋 Status

**Current Phase**: Planning

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ⏳ Answer clarifying questions
4. ⏳ Update specification based on answers
5. ⏳ Begin implementation

**Next Steps**:

1. Answer remaining clarifying questions
2. Refine specification with final decisions
3. Implement application layer `ExistsCommandHandler`
4. Implement presentation layer CLI controller and routing
5. Add unit tests and E2E tests
6. Update documentation

## 🎯 Quick Summary

Add a new `exists` console command that checks whether a named deployment environment exists and returns a clear boolean result. The command outputs `true` or `false` to stdout and always exits 0 on success (exit 1 only for errors), following the project's standard exit code convention.

**Key Points**:

- **Problem**: No unambiguous, scriptable way to check if an environment exists from the CLI
- **Solution**: New `exists` command that outputs `true`/`false` to stdout
- **Exit codes**: 0 = success (both `true` and `false` results), 1 = error
- **Status**: Planning phase
- **Usage**: `torrust-tracker-deployer exists <environment-name> [--format json]`
- **Performance**: Sub-millisecond — file existence check only, no JSON deserialization
- **SDK parity**: The SDK already has `Deployer::exists()` — this brings the same capability to the CLI

## 🔗 Related Documentation

- [Console Commands Overview](../../console-commands.md)
- [Command Architecture](../../codebase-architecture.md)
- [User Guide - Commands](../../user-guide/commands.md)
- [Show Command Feature](../environment-status-command/specification.md) — related command for displaying environment info
- [Development Principles](../../development-principles.md)
