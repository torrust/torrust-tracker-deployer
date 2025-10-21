# EPIC: UI Layer Destroy Command

**GitHub Issue**: [#10](https://github.com/torrust/torrust-tracker-deployer/issues/10)  
**Epic Type**: Child Epic #10 (Phase 2 of Task 1.2)
**Parent Epic**: #8 ([`8-epic-destroy-command.md`](./8-epic-destroy-command.md))
**Dependencies**: Child Epic #9 ([`9-epic-app-layer-destroy-command.md`](./9-epic-app-layer-destroy-command.md)) must be completed first
**Related Roadmap**: [Section 1.2](../roadmap.md#1-add-scaffolding-for-main-app)
**Parent Issue**: #2 (Scaffolding for main app)

---

## 📋 Epic Overview

Implement the user-facing CLI interface for the destroy command. This epic builds on top of the application layer implementation from Epic #9 to provide a complete, production-ready CLI experience.

This follows an **incremental, lean approach**: start with basic functionality and add improvements iteratively.

## 🎯 Goals

1. Rename existing app commands to command handlers for clarity
2. Add Clap subcommand configuration for `destroy`
3. Implement user-friendly progress messages and feedback
4. Provide comprehensive user documentation

**Note**: Force flags (`--force`, `--yes`) and skip confirmation features are **out of scope for MVP**. These are easy to implement later and not essential for the initial destroy command functionality.

## 🚫 Non-Goals

- Application layer logic (completed in Epic #9)
- Complex interactive wizards (future improvement)
- Selective resource destruction (future improvement)

## 📦 Sub-Issues

### Issue 10.1: Rename App Commands to Command Handlers

**Description**: Refactor terminology to distinguish between UI commands (Clap) and Application Layer commands (DDD).

**Scope**:

- Rename structs/modules in `src/application/commands/` for clarity
- Update documentation to use "command handler" terminology
- Ensure consistent naming across the codebase

**Rationale**: As we introduce UI-level commands (Clap subcommands), we need clear terminology:

- **UI Command**: Clap subcommand (e.g., `destroy`, `provision`)
- **Command Handler**: DDD Application Layer command (e.g., `DestroyCommand`, `ProvisionCommand`)

**Acceptance Criteria**:

- [ ] Command handlers renamed consistently
- [ ] Documentation updated with new terminology
- [ ] No breaking changes to functionality
- [ ] All tests pass after refactoring
- [ ] Code follows project conventions

**Estimated Effort**: 2-3 hours

---

### Issue 10.2: Add Clap Subcommand Configuration

**Description**: Implement the `destroy` subcommand in the CLI with basic functionality and UserOutput scaffolding.

**Scope**:

- Add `destroy` subcommand to `src/app.rs`
- Wire up subcommand to call `DestroyCommand` handler
- Add environment name parameter (required)
- Add `UserOutput` type and `VerbosityLevel` enum (following `docs/research/UX/user-output-vs-logging-separation.md`)
- Implement basic progress messages using `UserOutput`
- Basic command structure without advanced options

**UserOutput Integration**:

- Add `VerbosityLevel` enum (Quiet, Normal, Verbose, VeryVerbose, Debug)
- Add `UserOutput` struct with methods for different message types
- **Note**: Only enum definition for now, no Clap verbosity flags yet
- Use `UserOutput` for essential destroy command messages

**Example Usage**:

```bash
torrust-tracker-deployer destroy <ENVIRONMENT_NAME>
```

**Example Output** (Normal verbosity level):

```text
⏳ Destroying environment 'my-env'...
⏳ Tearing down infrastructure...
⏳ Cleaning up resources...
✅ Environment 'my-env' destroyed successfully
```

**Acceptance Criteria**:

- [ ] `destroy` subcommand added to Clap configuration
- [ ] Subcommand calls `DestroyCommand` from Application Layer
- [ ] Environment name parameter required
- [ ] `VerbosityLevel` enum implemented (5 levels)
- [ ] `UserOutput` struct implemented with basic methods (`progress`, `success`, `warn`)
- [ ] Essential destroy messages implemented using `UserOutput`
- [ ] Help text provides clear usage information
- [ ] Basic error handling for missing environment
- [ ] Unit tests for CLI argument parsing
- [ ] User output separated from internal logging

**Estimated Effort**: 3-4 hours

---

### Issue 10.3: Add User Documentation

**Description**: Create comprehensive user-facing documentation for the destroy command.

**Scope**:

- Add section to `docs/user-guide/` about destroy command
- Document command usage and examples
- Document flags and options
- Add troubleshooting section
- Document safety considerations

**Acceptance Criteria**:

- [ ] User guide created/updated
- [ ] Usage examples provided
- [ ] Flags and options documented
- [ ] Troubleshooting section added
- [ ] Safety considerations documented
- [ ] All markdown linting passes

**Estimated Effort**: 2-3 hours

---

## 📊 Epic Summary

**Total Estimated Effort**: 7-10 hours

**Sub-Issues**:

1. Issue 10.1: Rename App Commands to Command Handlers (2-3h)
2. Issue 10.2: Add Clap Subcommand Configuration with Basic Progress (3-4h)
3. Issue 10.3: Add User Documentation (2-3h)

**Out of Scope for MVP**: Force flags (`--force`, `--yes`) and skip confirmation features - these can be implemented as separate improvements later.

## 🔗 Dependencies

- **Requires**: Epic #9 (App Layer Destroy Command) - must be completed first
- **Blocks**: None (future improvements can build on this)

## 📝 Technical Notes

### MVP Focus

This epic focuses on the essential functionality for MVP:

- Basic destroy subcommand with clear interface
- UserOutput type and VerbosityLevel enum (scaffolding for future verbosity flags)
- Essential progress messages using Normal verbosity level
- User-friendly feedback with proper separation from logging
- Basic user documentation
- Clean separation between UI and Application layers

**Future Improvements** (out of scope for MVP):

- Clap verbosity flags (`-v`, `-vv`, `-q`)
- Force and skip confirmation flags
- Interactive mode with detailed prompts
- Selective resource destruction
- Advanced verbosity level implementations

### UI Command vs. Command Handler

After this epic, we'll have clear separation:

```rust
// UI Layer (src/app.rs) - Clap subcommands
#[derive(Subcommand)]
enum Commands {
    Provision { /* ... */ },
    Destroy { environment: String },
    Configure { /* ... */ },
}

// Application Layer (src/application/commands/) - DDD command handlers
pub struct DestroyCommandHandler { /* ... */ }
pub struct ProvisionCommandHandler { /* ... */ }
pub struct ConfigureCommandHandler { /* ... */ }
```

### User Output vs. Logging

Follow project principles established in `docs/research/UX/user-output-vs-logging-separation.md`:

- **User Output**: Directed to stdout via `UserOutput` type, clear and concise
- **Logging**: Directed to files/stderr via tracing, detailed for debugging
- Complete separation between user-facing messages and internal logging

### UserOutput Implementation

```rust
// Verbosity levels (enum only for MVP, no CLI flags yet)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    Quiet,      // Minimal output
    Normal,     // Default: Essential progress (MVP focus)
    Verbose,    // Detailed progress
    VeryVerbose, // Including decisions & retries
    Debug,      // Maximum detail for troubleshooting
}

// UserOutput type for destroy command
pub struct UserOutput {
    verbosity: VerbosityLevel,
}

impl UserOutput {
    pub fn progress(&self, message: &str) { /* ... */ }
    pub fn success(&self, message: &str) { /* ... */ }
    pub fn warn(&self, message: &str) { /* ... */ }
}
```

### Example Destroy Output

**Normal Verbosity** (MVP implementation):

```text
⏳ Destroying environment 'my-env'...
⏳ Tearing down infrastructure...
⏳ Cleaning up resources...
✅ Environment 'my-env' destroyed successfully
```

**Parallel Internal Logging** (always present):

```text
2025-10-21T10:15:00.123Z INFO destroy_command: Starting environment destruction
    environment="my-env" command_type="destroy"
2025-10-21T10:15:01.456Z INFO opentofu_client: Executing destroy operation
    workspace="/path/to/env" operation="destroy"
2025-10-21T10:15:15.789Z INFO destroy_command: Environment destruction completed
    environment="my-env" duration=15.666s
```

## 🚀 Next Steps After Completion

After completing this epic:

1. User testing with real environments
2. Gather feedback on UX
3. Consider future improvements:
   - Interactive confirmation with resource preview
   - Selective destruction (only OpenTofu, only Ansible, etc.)
   - Batch destruction for multiple environments
   - Integration with CI/CD pipelines

---

## 📋 Related Documentation

- [Roadmap](../roadmap.md)
- [Parent Issue #2](https://github.com/torrust/torrust-tracker-deployer/issues/2)
- [Epic #9: App Layer Destroy Command](https://github.com/torrust/torrust-tracker-deployer/issues/9)
- [Development Principles](../development-principles.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md)
