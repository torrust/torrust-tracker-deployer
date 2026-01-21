# Config Validation Command

A command to validate environment configuration files without producing any side effects on the application state.

## 📄 Documents

### [specification.md](./specification.md)

The main feature specification including:

- Overview and problem statement
- Feature goals
- Proposed solution
- Implementation details
- Definition of done
- Testing strategy

### [questions.md](./questions.md)

Clarifying questions that need to be answered before implementation:

- Scope and requirements
- Technical approach
- Priority and timeline
- Success criteria
- Risk assessment

## 📋 Status

**Current Phase**: Ready for Implementation

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ✅ Answer clarifying questions
4. ✅ Update specification based on answers
5. ⏳ Begin implementation

**Next Steps**:

1. Extract validation logic from `CreateCommandHandler`
2. Create `ValidateCommandHandler`
3. Add CLI subcommand
4. Write E2E tests

## 🎯 Quick Summary

Users and AI agents need a way to validate environment configuration files without modifying application state or checking state-dependent conditions.

**Key Points**:

- **Problem**: The `create` command validates config but also persists it to internal state - there's no way to validate without side effects
- **Solution**: Standalone `validate` command that checks config-intrinsic validity only
- **Command**: `torrust-tracker-deployer validate --env-file envs/config.json`
- **Status**: Ready for implementation

**Validation Scope** (three levels):

| Level | Type                                                           | Included? |
| ----- | -------------------------------------------------------------- | --------- |
| 1     | Syntactic (JSON valid, types correct)                          | ✅ Yes    |
| 2     | Config-intrinsic semantics (e.g., Grafana requires Prometheus) | ✅ Yes    |
| 3     | State-dependent semantics (e.g., name already exists)          | ❌ No     |

**Design Decision**: Chose `validate` command over `--dry-run` flag because:

- `validate` = "Is this configuration intrinsically valid?" (pure, deterministic)
- `--dry-run` = "Will `create` succeed right now?" (state-dependent - different promise)
- The same config may be valid in one data directory but conflict in another
- `create` has minimal side effects (internal state only, easily reversible with `destroy`)

## 🔗 Related Documentation

- [Environment Configuration Schema](../../../schemas/environment-config.json)
- [User Guide - Commands](../../user-guide/commands/)
- [Console Commands](../../console-commands.md)
- [Development Principles](../../development-principles.md)
