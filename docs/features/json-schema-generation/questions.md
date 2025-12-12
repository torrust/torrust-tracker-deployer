# Clarifying Questions for JSON Schema Generation

This document contains questions to clarify requirements, scope, and priorities before implementation begins. Product owners or stakeholders should answer these questions directly in the document.

---

## 🔍 Scope and Requirements

### 1. **Core Functionality**

**Question**: What is the minimum viable functionality for this feature?

**Your Answer**:

- Generate valid JSON Schema from Rust configuration types
- Provide CLI command `create schema` with optional output path
- Print schema to stdout if no path provided
- Write schema to file if path provided
- Update `create template` command output to mention schema generation

All of them.

### 2. **Out of Scope**

**Question**: What is explicitly NOT included in this feature?

**Your Answer**:

- Schema validation during `create template` (validation already exists in Rust deserialization)
- IDE plugins or extensions for schema integration
- Schema versioning or migration
- Online schema hosting/registry
- Schema for other configuration formats (TOML, YAML)

Correct, all of them.

### 3. **User Experience**

**Question**: How should users interact with this feature? What's the expected workflow?

**Your Answer**:

1. User runs `cargo run create schema` to see schema or `cargo run create schema ./schema.json` to save it
2. User configures their IDE/editor to use the schema for validation (e.g., VS Code settings)
3. When editing environment JSON files, IDE provides validation, auto-completion, and documentation
4. Optional: User can commit schema file to repository for team consistency

Correct, all of them.

## 🎯 Technical Approach

### 4. **Implementation Strategy**

**Question**: Are there specific technical approaches or patterns we should follow?

**Your Answer**:

- Use Schemars crate to derive JSON Schema from Rust types
- Add `#[derive(JsonSchema)]` to all config types in `src/application/command_handlers/create/config/`
- Create new command handler in application layer
- Follow existing three-level pattern (Command → Step → Action)
- Ensure schema includes descriptions from Rust doc comments

Correct, all of them.

### 5. **Integration Points**

**Question**: How does this feature integrate with existing components?

**Your Answer**:

- Uses existing config types in `src/application/command_handlers/create/config/`
- Adds new subcommand to CLI parser in presentation layer
- Updates `create template` command to mention schema availability
- No changes to environment creation, provisioning, or deployment logic

Correct, all of them.

### 6. **Performance Requirements**

**Question**: Are there specific performance requirements or constraints?

**Your Answer**:

- Schema generation should complete in <100ms (it's a simple reflection operation)
- Schema file size expected to be <100KB
- No runtime performance impact on other commands

No specific performance requirements. Only need to ensure it doesn't hang or crash or it takes an unreasonable amount of time.

## 📊 Priority and Timeline

### 7. **Priority Level**

**Question**: What is the priority of this feature? (High | Medium | Low)

**Your Answer**: **Medium** - Improves developer experience but not blocking for core functionality

It's high priority as it significantly enhances user experience when configuring environments.
And specifically for users who are using AI agents to help them write configuration files, having a JSON Schema is very beneficial.

### 8. **Timeline Expectations**

**Question**: Is there a target date or sprint for completion?

**Your Answer**: No specific deadline. Can be implemented incrementally alongside other work.

Correct, no specific deadline.

### 9. **Dependencies**

**Question**: Does this feature depend on other work being completed first?

**Your Answer**:

- No blockers - existing config types are stable
- Should review if Hetzner provider config is finalized before implementation
- May want to coordinate with any ongoing config refactoring work

There are no dependencies or blockers for this feature.

## ✅ Success Criteria

### 10. **Definition of Done**

**Question**: How do we know this feature is complete and working correctly?

**Your Answer**:

- [ ] Schemars crate added as dependency
- [ ] All config types derive `JsonSchema`
- [ ] CLI accepts `create schema [PATH]` command
- [ ] Schema prints to stdout when no path provided
- [ ] Schema writes to file when path provided
- [ ] `create template` output mentions schema generation
- [ ] Generated schema validates example JSON files
- [ ] Unit tests for schema generation
- [ ] Documentation updated (user guide, commands.md)
- [ ] Example IDE configuration provided (VS Code settings.json)

All of them.

### 11. **Testing Requirements**

**Question**: What level of testing is expected? (Unit | Integration | E2E)

**Your Answer**:

- **Unit Tests**: Test schema generation from config types
- **Integration Tests**: Test CLI command with stdout and file output
- **E2E Tests**: Not required - this is a utility command that doesn't affect deployment
- **Manual Testing**: Verify schema validates against actual environment JSON files

All of them. For the E2E tests, we can follow the example of other commands like `tests/e2e/create_command.rs`.

## 🛡️ Risk Assessment

### 12. **Potential Risks**

**Question**: What risks or challenges should we be aware of?

**Your Answer**:

- Schemars may not handle all Rust types perfectly (enums, generics)
- Schema may become out of sync if config types change
- Breaking changes to config structure require schema updates
- Documentation burden - users need to know how to use schema in their IDE

I think the main risk is that Schemars may not perfectly represent all Rust types we use in our configuration structs, especially more complex types like enums or generics. We need to test and verify the generated schema to ensure it meets our needs.

I'm not worry about breaking changes to config structure because the deployer app is meant to be used during some minutes while deploying an environment, so users will likely regenerate the schema as needed.

## 📖 Documentation Requirements

### 13. **Documentation Needs**

**Question**: What documentation should be created or updated?

**Your Answer**:

- Update `docs/user-guide/commands/create.md` with schema command
- Add example `.vscode/settings.json` for VS Code users
- Update `docs/console-commands.md` with new command
- Add troubleshooting section for IDE integration
- Include schema example in repository (e.g., `examples/environment-schema.json`)

Correct, all of them. But notice that the documentation about the new command should be added to `docs/user-guide/commands/create.md`. We have included the documentation for its subcommands in the same file.

## 🔄 Future Enhancements

### 14. **Follow-up Work**

**Question**: Are there related enhancements to consider for future iterations?

**Your Answer**:

- Auto-generate schema during build/CI and commit to repository
- Provide schema URL for remote referencing (`$schema` field)
- Schema validation command (separate from generation)
- Support for multiple schema formats (OpenAPI, GraphQL schema)
- Schema diff tool to detect breaking changes

That would be nice to have in the future, but out of scope for the initial implementation.

**Status**: ✅ All questions answered by product owner
**Last Updated**: December 12, 2025
