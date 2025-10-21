# Add Developer Documentation for Destroy Command

**GitHub Issue**: [#13](https://github.com/torrust/torrust-tracker-deployer/issues/13)  
**Issue Type**: Sub-issue (9.3)  
**Parent Epic**: #9 ([`epic-app-layer-destroy-command.md`](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/issues/9-epic-app-layer-destroy-command.md))  
**Related Roadmap**: [Section 1.2](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-create-command-torrust-tracker-deployer-destroy)  
**Dependencies**: Issue 9.1 and 9.2 should be completed first  
**Priority**: Medium  
**Estimated Effort**: 2-3 hours

---

## 📋 Issue Overview

Document the destroy command implementation for developers. This includes architecture decisions, usage examples, error handling patterns, and E2E testing integration.

The documentation will help current and future developers understand how to use, maintain, and extend the destroy command functionality.

## 🎯 Goals

1. Document destroy command architecture and design decisions
2. Provide code examples for using `DestroyCommand` in development
3. Document error handling patterns and best practices
4. Update E2E testing documentation with destroy functionality
5. Ensure all documentation passes linting requirements

## 📦 Scope

### Core Documentation

The documentation will be organized across three distinct locations:

1. **Internal Contributors**: `docs/contributing/` - Developer-focused implementation details
2. **Decision Records**: `docs/decisions/` - Architectural decisions and rationale
3. **User-Facing**: `docs/user-guide/` - End-user command documentation

#### Documentation Locations

- **`docs/contributing/commands.md`** - Single developer guide covering all commands (update existing or create new)
- **`docs/decisions/`** - Architectural decision records (if significant decisions are made)
- **`docs/user-guide/commands/destroy.md`** - User-facing command documentation
- **`docs/user-guide/commands.md`** - Command index with descriptions and links
- Updated E2E testing documentation

### Content Structure

1. **Architecture Overview**: How the destroy command fits into DDD layers
2. **Usage Examples**: Code examples for developers
3. **Error Handling**: Patterns and best practices
4. **Testing Integration**: How destroy command integrates with E2E tests
5. **Troubleshooting**: Common issues and solutions

## 🏗️ Documentation Structure

### New Documentation Files

#### 1. Internal Contributors Documentation: `docs/contributing/commands.md`

Single developer guide covering all commands (add destroy command section):

- **Command Architecture**: How all commands fit into DDD Application Layer patterns
- **Destroy Command Implementation**: Architecture, error handling, testing patterns
- **Code Usage**: Internal API usage examples for developers (not CLI usage)
- **Testing Strategies**: Unit test patterns, mock strategies, integration approaches
- **Debugging and Development**: Internal troubleshooting for contributors

#### 2. User-Facing Documentation: `docs/user-guide/commands/destroy.md`

End-user command reference covering:

- Command syntax and options
- Usage examples and workflows
- Common use cases and scenarios
- Error messages and troubleshooting
- Safety considerations and best practices

#### 3. Command Index: `docs/user-guide/commands.md`

Master command reference covering:

- List of all available commands with short descriptions
- Links to detailed command documentation
- Command categories and workflows
- Getting started guide

### Updated Files

#### `docs/contributing/testing.md`

Add section on:

- Destroy command testing approaches
- E2E integration patterns
- Mock strategies for unit tests

#### `docs/e2e-testing.md`

Update with:

- New destroy functionality in E2E tests
- Updated test flow documentation
- Troubleshooting destroy-related test issues

#### `docs/decisions/` (if needed)

Create ADRs for any significant architectural decisions made during implementation.

## 📋 Content Requirements

### 1. Internal Contributors Documentation (`docs/contributing/commands.md`)

#### Add Destroy Command Section

Add a new section to the existing (or create new) commands developer guide:

#### Architecture Documentation

Document how `DestroyCommand` integrates with existing command patterns:

- How it follows established DDD Application Layer patterns (like `ProvisionCommand`, `ConfigCommand`)
- Integration with existing infrastructure services (OpenTofu client, state management)
- Error handling and recovery patterns specific to destroy operations
- Command composition and orchestration within the application layer

#### Internal API Usage Examples

Provide code examples for **developers** working on the codebase:

```rust
// How to use DestroyCommand in application layer code
use crate::application::commands::destroy::DestroyCommand;

async fn destroy_environment(environment_name: &str) -> Result<(), DestroyError> {
    let destroy_command = DestroyCommand::new(environment_name)?;
    destroy_command.execute().await
}

// How to integrate with E2E tests
async fn cleanup_test_environment(env: &Environment) -> Result<(), DestroyError> {
    let destroy_cmd = DestroyCommand::for_environment(env);
    destroy_cmd.execute().await
}
```

#### Error Handling Patterns for Developers

Document internal error handling (not user-facing):

- Error types and their meanings for developers
- Recovery strategies for partial failures in code
- Logging patterns for debugging and development
- How internal errors map to user-facing messages
- Testing error scenarios

#### Testing Integration for Contributors

Explain testing approaches for developers:

- How destroy command integrates with E2E test infrastructure
- Unit testing strategies and mock patterns
- Integration testing approaches
- CI/CD considerations for contributors

### 2. User-Facing Documentation (`docs/user-guide/commands/destroy.md`)

#### Command Reference

Document:

- Command syntax: `torrust-tracker-deployer destroy <ENVIRONMENT_NAME>`
- Available options and flags
- Output format and progress indicators
- Exit codes and their meanings

#### Usage Examples

Provide practical examples:

```bash
# Basic usage
torrust-tracker-deployer destroy my-environment

# Example with verbose output (future)
torrust-tracker-deployer destroy my-environment --verbose
```

#### User Scenarios

Cover common use cases:

- Cleaning up after testing
- Removing failed deployments
- Scheduled environment cleanup
- Emergency teardown procedures

#### Troubleshooting

User-focused troubleshooting:

- Common error messages and solutions
- What to do when destroy fails
- How to verify complete cleanup
- When to contact support

### 3. Command Index (`docs/user-guide/commands.md`)

#### Command Overview

Provide a structured command reference:

```markdown
# Available Commands

## Environment Management

- **[`provision`](commands/provision.md)** - Deploy new environments with infrastructure and applications
- **[`destroy`](commands/destroy.md)** - Remove environments and clean up all resources
- **[`configure`](commands/configure.md)** - Update configuration for existing environments

## Getting Started

For first-time users, we recommend starting with the `provision` command to create your first environment...
```

## 📋 Acceptance Criteria

### Internal Contributors Documentation

- [ ] Destroy command section added to `docs/contributing/commands.md`
- [ ] Architecture integration with existing command patterns documented
- [ ] Internal API code examples provided for developers
- [ ] Error handling patterns for contributors documented
- [ ] Testing strategies and mock patterns for development documented

### User-Facing Documentation

- [ ] User command reference created in `docs/user-guide/commands/destroy.md`
- [ ] Command index updated in `docs/user-guide/commands.md`
- [ ] User scenarios and troubleshooting documented
- [ ] Command syntax and examples provided

### Updated Documentation

- [ ] E2E testing guide updated with destroy functionality
- [ ] Testing conventions updated in `docs/contributing/testing.md`

### Quality Standards

- [ ] All markdown linting passes
- [ ] Documentation follows project style guidelines
- [ ] All links are valid and functional
- [ ] Examples are accurate and tested

## 🧪 Documentation Quality Standards

### Writing Guidelines

- Follow existing documentation style and structure
- Use clear, concise language
- Provide practical examples
- Include troubleshooting information
- Ensure accessibility for developers at different experience levels

### Code Examples

- All code examples must be syntactically correct
- Include complete, runnable examples where possible
- Show both successful and error scenarios
- Follow project coding conventions

### Linking and Cross-References

- Link to related documentation sections
- Reference relevant ADRs (Architectural Decision Records)
- Cross-reference with existing guides
- Ensure all links are valid and up-to-date

## 🔗 Dependencies

- **Requires**: Issue 9.1 (Add DestroyCommand in Application Layer) - for accurate technical documentation
- **Requires**: Issue 9.2 (Update E2E Provision Tests) - for complete E2E testing documentation
- **References**: Existing documentation structure and style guides

## 📝 Implementation Notes

### Documentation Organization by Audience

#### Internal Contributors (`docs/contributing/commands.md`)

- Add destroy command section to existing (or create new) single commands developer guide
- Follow established pattern with clear headings and subheadings
- Provide quick reference sections for developers working on the codebase
- Use code blocks with proper syntax highlighting for internal API usage
- Focus on implementation details, architecture patterns, and contributor concerns
- **Exclude** user-facing information like CLI usage examples, command syntax, etc.

#### User-Facing (`docs/user-guide/`)

- Use simple, clear language suitable for end users
- Provide step-by-step instructions and examples
- Focus on practical usage and common scenarios
- Include troubleshooting sections with solutions
- Minimize technical jargon

#### Decision Records (`docs/decisions/`)

- Follow ADR template if significant architectural decisions are made
- Document rationale and alternatives considered
- Include consequences and future implications

### File Structure to Create

```text
docs/
├── contributing/
│   ├── commands.md (update existing or create new - single doc for all commands)
│   └── testing.md (updated)
├── user-guide/
│   ├── commands.md (new/updated)
│   └── commands/
│       └── destroy.md (new)
├── decisions/
│   └── [any-new-ADRs].md (if needed)
└── e2e-testing.md (updated)
```

### Integration with Existing Docs

Ensure seamless integration with:

- Existing command documentation patterns
- Error handling guides and conventions
- Testing conventions and E2E documentation
- Development principles and style guides
- Cross-reference with provision and configure command docs

### Examples and Snippets

Include practical examples for:

- Basic destroy command usage
- Error handling and recovery
- Integration with existing services
- Testing patterns and mocks

## 🚀 Next Steps

After completing this documentation:

1. Review documentation with team for accuracy and completeness
2. Integrate feedback and revisions
3. Ensure documentation stays up-to-date as destroy command evolves
4. Consider adding video tutorials or interactive examples for complex topics

## 📊 Related Documentation

- [Parent Epic #9](https://github.com/torrust/torrust-tracker-deployer/issues/9)
- [Development Principles](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/development-principles.md)
- [Error Handling Guide](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/error-handling.md)
- [Testing Conventions](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/testing.md)
- [Module Organization](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/module-organization.md)

---

**Issue Document**: [docs/issues/epic-app-layer-destroy-command-add-developer-documentation.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/issues/epic-app-layer-destroy-command-add-developer-documentation.md)
