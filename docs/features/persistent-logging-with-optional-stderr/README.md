# Persistent Logging with Optional Stderr Redirection

Add persistent file-based logging with optional stderr output for development and testing workflows.

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
3. ✅ Answer clarifying questions (21/21 answered)
4. ✅ Update specification based on answers
5. ⏳ Begin implementation

**Next Steps**:

1. Implement Phase 1: Core logging functionality with enum-based API
2. Implement Phase 2: Update all callers (main app + E2E tests)
3. Implement Phase 3: Documentation and cleanup
4. Manual E2E testing to verify behavior

## 🎯 Quick Summary

This feature introduces persistent file-based logging to prepare for the production CLI application while maintaining current E2E test visibility.

**Key Points**:

- **Problem**: Need persistent logs for production use, but E2E tests currently rely on stderr visibility
- **Solution**: Always write logs to `./data/logs/log.txt` + optional stderr output controlled by `LogOutput` enum
- **API**: `init(LogOutput::FileOnly)` for production, `init(LogOutput::FileAndStderr)` for tests
- **Status**: Ready for implementation - all questions answered, specification complete
- **Purpose**: Preparatory refactor to enable clean separation of user output and internal logging

## 🔍 Context

This feature is a preparatory step for introducing user-facing output in the main application. Currently:

- E2E tests use `tracing` logs directly for visibility
- No persistent log files exist
- Main application is minimal (just shows a message)

Future state:

- Production CLI will show user-friendly output to stdout/stderr
- Internal logs will be persistent in files
- E2E tests can optionally see logs on stderr during development
- Full E2E tests will eventually call the production app (current tests become integration tests)

## 🔗 Related Documentation

- [User Output vs Internal Logging Separation](../../research/UX/user-output-vs-logging-separation.md) - Architectural decision on output separation
- [Development Principles](../../development-principles.md) - Observability and traceability principles
- [Logging Guide](../../contributing/logging-guide.md) - Internal logging conventions
