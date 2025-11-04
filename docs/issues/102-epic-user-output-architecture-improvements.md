# User Output Architecture Improvements

**Issue**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102)
**Parent Epic**: N/A (Standalone refactoring epic)
**Roadmap**: N/A (Code quality improvement)
**Related**: [Refactoring Plan](../refactors/plans/user-output-architecture-improvements.md)

## Overview

This Epic refactors the `UserOutput` module (`src/presentation/user_output.rs`) to improve clarity, testability, maintainability, and sustainability. The current implementation mixes concerns (formatting, routing, verbosity control) and lacks extensibility for different output styles and destinations.

## Refactoring Plan

See comprehensive refactoring plan: [docs/refactors/plans/user-output-architecture-improvements.md](../refactors/plans/user-output-architecture-improvements.md)

## Goals

- [x] **Separate Concerns**: Extract verbosity filtering, theme configuration, and formatting logic
- [x] **Simplify Testing**: Improve test infrastructure and reduce duplication
- [x] **Enable Extensibility**: Support different output styles (emoji, plain text, JSON) and destinations
- [x] **Improve Maintainability**: Reduce code duplication and establish clear abstractions
- [x] **Maintain Quality**: All refactorings must pass pre-commit checks and maintain test coverage

## Proposals Summary

### Phase 0: Quick Wins (High Impact, Low Effort)

- **Proposal #0**: Extract Verbosity Filtering Logic
- **Proposal #1**: Simplify Test Infrastructure
- **Proposal #2**: Add Theme/Configuration Support

### Phase 1: Strategic Improvements (High Impact, Medium Effort)

- **Proposal #3**: Use Message Trait for Extensibility
- **Proposal #5**: Parameterized Test Cases

### Phase 2: Polish & Extensions (Medium Impact, Low-Medium Effort)

- **Proposal #4**: Add Alternative Formatters (optional enhancement)
- **Proposal #6**: Type-Safe Channel Routing
- **Proposal #7**: Add Buffering Control
- **Proposal #8**: Builder Pattern for Complex Messages
- **Proposal #9**: Output Sink Abstraction

## Key Architectural Decision

**Trait-Based Message System**: Each message type (`ProgressMessage`, `SuccessMessage`, etc.) implements the `OutputMessage` trait with its own formatting, verbosity requirements, and channel routing. This achieves true Open/Closed Principle - new message types can be added without modifying existing code.

**Alternative Considered**: Enum-based messages with centralized formatter. Discarded because pattern matching on enum variants requires modifying the formatter for each new message type, violating the Open/Closed Principle.

## Implementation Strategy

**Incremental Approach**: Create subissues for each proposal as work progresses. This allows adapting the implementation to the current codebase state and adjusting for any discoveries made during implementation.

## Sub-Tasks

### Phase 0: Quick Wins

- [x] [#103](https://github.com/torrust/torrust-tracker-deployer/issues/103) - Proposal 0: Extract Verbosity Filtering Logic
- [x] [#123](https://github.com/torrust/torrust-tracker-deployer/issues/123) - Proposal 1: Simplify Test Infrastructure
- [x] [#124](https://github.com/torrust/torrust-tracker-deployer/issues/124) - Proposal 2: Add Theme/Configuration Support

### Phase 1: Strategic Improvements

- [x] [#127](https://github.com/torrust/torrust-tracker-deployer/issues/127) - Proposal 3: Use Message Trait for Extensibility
- [x] [#128](https://github.com/torrust/torrust-tracker-deployer/issues/128) - Proposal 5: Parameterized Test Cases

### Phase 2: Polish & Extensions

- [x] [#133](https://github.com/torrust/torrust-tracker-deployer/issues/133) - Proposal 4: Add Formatter Override Support
- [x] [#135](https://github.com/torrust/torrust-tracker-deployer/issues/135) - Proposal 6: Type-Safe Channel Routing
- [ ] [#138](https://github.com/torrust/torrust-tracker-deployer/issues/138) - Proposal 7: Add Buffering Control
- [ ] [#139](https://github.com/torrust/torrust-tracker-deployer/issues/139) - Proposal 8: Builder Pattern for Complex Messages
- [ ] [#140](https://github.com/torrust/torrust-tracker-deployer/issues/140) - Proposal 9: Output Sink Abstraction

### Other Related Work

- [x] [#107](https://github.com/torrust/torrust-tracker-deployer/issues/107) - Centralize UserOutput via Dependency Injection

## Timeline

- **Estimated Duration**: 3-4 weeks (with parallel development possible)
- **Target Completion**: End of November 2025

## Related Documentation

- [Development Principles](../development-principles.md) - Core principles including testability and maintainability
- [Contributing Guidelines](../contributing/README.md) - General contribution process
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Testing Conventions](../contributing/testing/) - Testing best practices
