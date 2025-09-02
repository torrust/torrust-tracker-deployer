# Decision: Removal of Meson Build System

## Status

**REMOVED** - September 2, 2025

## Context

This project initially adopted [Meson](https://mesonbuild.com/) as a task runner to wrap common development commands and provide a unified interface for development workflows. Meson was chosen to:

- Provide shorter, memorable commands for complex operations
- Ensure consistent execution environments
- Offer potential IDE integration
- Standardize development workflows across team members

### Initial Implementation

The project included:

- `meson.build` configuration file with custom targets
- `builddir/` for Meson build artifacts
- Single target: comprehensive Rust clippy linting
- Documentation in `docs/tech-stack/meson.md`

## Problems Encountered

### 1. **Verbose Output**

Meson's output was verbose and made error messages from underlying tools (like clippy) harder to read and debug. The extra layer of abstraction obscured the actual tool output.

### 2. **Limited Adoption**

Despite being set up, only one meson target was ever created (`clippy`), indicating limited practical value for the team's workflow.

### 3. **Redundancy with Existing Tools**

- **Cargo**: Already provides excellent task management for Rust projects
- **Shell Scripts**: Project already had sophisticated cross-platform dependency management scripts
- **Multiple Build Systems**: Adding meson created another layer on top of cargo, tofu, and ansible

### 4. **Maintenance Overhead**

- Additional configuration files to maintain
- Another tool dependency for contributors to install
- Documentation overhead for minimal benefit

## Decision

**Remove meson entirely** and rely on:

1. **Cargo** for Rust-specific tasks (build, test, clippy, etc.)
2. **Shell Scripts** for cross-platform tooling and dependency management
3. **Direct tool invocation** for cleaner output and better debugging

## Implementation

### Removed Files

- `meson.build`
- `builddir/` directory
- `docs/tech-stack/meson.md`
- `docs/meson.md`

### Migrated Functionality

- **Clippy target** → `scripts/lint/clippy.sh` with identical arguments
- **Task organization** → Organized shell scripts in `scripts/lint/` directory
- **Unified interface** → `scripts/lint.sh` wrapper script

### Updated Documentation

- README.md development tasks section
- Repository structure documentation
- Linting workflow documentation

## Consequences

### Positive

- ✅ **Cleaner output**: Direct tool execution shows clear error messages
- ✅ **Reduced complexity**: Fewer tools and configuration files to maintain
- ✅ **Better debugging**: No abstraction layer hiding tool behavior
- ✅ **Rust-native workflow**: Leverages cargo's excellent task management
- ✅ **Maintained functionality**: All original capabilities preserved

### Negative

- ❌ **Longer commands**: Some commands are longer to type (mitigated by wrapper scripts)
- ❌ **No IDE integration**: Lost potential meson IDE support (minimal impact observed)

## When to Reconsider

Meson might be worth reconsidering if:

### Project Scope Changes

- Project becomes multi-language (C/C++, Python, etc.)
- Complex build orchestration needs across multiple subsystems
- Need for sophisticated dependency management beyond current shell scripts

### Team Scale Changes

- Large team with diverse skill levels needs more standardized workflows
- Complex CI/CD pipelines requiring build system integration
- Need for advanced build caching and incremental builds

### Tool Ecosystem Changes

- Meson significantly improves output clarity and debugging
- Integration requirements with tools that work better with meson
- Performance benefits become significant for the project scale

## Alternative Solutions

Before reintroducing meson, consider:

### Task Runners

- **Just**: Rust-native task runner with simple syntax
- **Make**: Universal, well-understood by most developers
- **npm scripts**: If the project adopts Node.js tooling

### Build Systems

- **Bazel**: For very large, multi-language projects
- **CMake**: If C/C++ components are added
- **Native cargo workspaces**: For complex Rust project organization

## References

- [Meson Documentation](https://mesonbuild.com/)
- [Just Task Runner](https://github.com/casey/just)
- [Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- Original meson implementation: commit `[hash]` (before removal)
- Removal implementation: commit `[hash]` (September 2, 2025)

---

_This decision document should be updated if the context significantly changes or if meson is reconsidered for future use._
