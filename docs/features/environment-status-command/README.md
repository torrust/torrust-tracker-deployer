# Environment Status Command

A new console command to display environment information with state-aware details.

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

- Command naming decision (status vs show)
- Output format requirements
- State-specific information to display
- Future extensibility for JSON output

## 📋 Status

**Current Phase**: Planning

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ⏳ Answer clarifying questions
4. ⏳ Update specification based on answers
5. ⏳ Begin implementation

**Next Steps**:

1. Answer clarifying questions (command name, output format)
2. Refine specification with detailed output format
3. Implement application layer StatusCommand/ShowCommand
4. Implement presentation layer console subcommand
5. Add E2E tests

## 🎯 Quick Summary

Add a new console command to display environment information with state-aware details. The command will show basic information like environment name and current state, with additional details based on the state (e.g., IP address and SSH port for provisioned environments).

**Key Points**:

- **Problem**: Users need visibility into environment state and details without inspecting JSON files
- **Solution**: New console command that loads environment and displays human-friendly information
- **Status**: Planning phase - need to decide on command name and output format
- **Command Name Options**: `status` or `show`
- **Usage**: `torrust-tracker-deployer {status|show} <environment-name>`

## 🔗 Related Documentation

- [Console Commands Overview](../../console-commands.md)
- [Command Architecture](../../codebase-architecture.md)
- [User Guide - Commands](../../user-guide/commands.md)
- [Development Principles](../../development-principles.md)
- [Output Handling Guide](../../contributing/output-handling.md)
