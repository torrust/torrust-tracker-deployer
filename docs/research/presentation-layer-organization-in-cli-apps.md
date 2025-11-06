# Research: Presentation Layer Organization in CLI Applications

## Overview

This document researches how CLI applications organize their presentation layer (user-facing code), examining patterns from popular Rust CLI tools, web frameworks, and industry standards.

**Research Goal**: Identify common patterns, best practices, and architectural approaches for organizing presentation layer code in command-line applications.

**Date**: November 6, 2025

## Scope

This research covers:

1. **Popular Rust CLI Tools** - How they structure their codebase
2. **Web Framework Patterns** - MVC/MVT patterns as architectural reference
3. **Industry Standards** - Common patterns across different ecosystems
4. **CLI Design Guidelines** - Best practices for CLI architecture

## 1. Popular Rust CLI Tools Analysis

### 1.1 cargo (Rust's Package Manager)

**Repository**: <https://github.com/rust-lang/cargo>

**Structure**:

```text
src/cargo/
├── core/
│   ├── compiler/
│   ├── package.rs
│   └── workspace.rs
├── ops/                    # Operations (like controllers)
│   ├── cargo_compile.rs
│   ├── cargo_fetch.rs
│   ├── cargo_install.rs
│   └── ...
├── sources/
├── util/
│   ├── command_prelude.rs
│   ├── config/
│   └── errors.rs
└── cli.rs                 # CLI definitions
```

**Key Patterns**:

- **Flat `ops/` directory** - All operations (commands) in one namespace
- **No explicit routing layer** - Commands call ops directly
- **`util/command_prelude.rs`** - Common imports and setup for commands
- **Separation**: CLI parsing (cli.rs) → Operations (ops/) → Core logic

**Observations**:

- ✅ Clear separation between CLI and operations
- ✅ Operations are well-isolated
- ❌ No explicit dispatcher/router pattern
- ❌ Output formatting scattered across operations

### 1.2 ripgrep (Fast Search Tool)

**Repository**: <https://github.com/BurntSushi/ripgrep>

**Structure**:

```text
crates/
└── core/
    └── app.rs              # Application setup and coordination
grep-cli/                   # CLI utilities
grep-printer/               # Output formatting
grep-searcher/              # Search logic
```

**Key Patterns**:

- **Separate crates** - Modular design with clear boundaries
- **Dedicated printer crate** - Output formatting isolated
- **App coordinator** - Central orchestration point

**Observations**:

- ✅ Excellent separation via crates
- ✅ Dedicated output handling (printer)
- ✅ Clear orchestration layer (app.rs)
- ⚠️ Multi-crate approach may be overkill for smaller projects

### 1.3 rustup (Rust Toolchain Manager)

**Repository**: <https://github.com/rust-lang/rustup>

**Structure**:

```text
src/
├── cli/                    # CLI parsing
│   ├── common.rs
│   ├── help.rs
│   └── self_update/
├── command.rs              # Command dispatcher
├── toolchain.rs            # Core logic
├── config.rs
└── utils/
```

**Key Patterns**:

- **Explicit `command.rs`** - Central command dispatcher
- **CLI module** - Dedicated to argument parsing
- **Flat structure** - Most logic at top level

**Observations**:

- ✅ Explicit dispatcher pattern
- ✅ Clear CLI separation
- ⚠️ Some commands mixed with core logic
- ✅ Simple, easy to navigate

### 1.4 bat (Cat Clone with Syntax Highlighting)

**Repository**: <https://github.com/sharkdp/bat>

**Structure**:

```text
src/
├── bin/                    # Binary entry points
│   └── bat/
│       └── main.rs
├── clap_app.rs            # CLI definitions (clap)
├── config.rs              # Configuration
├── controller.rs          # Main controller/orchestrator
├── input.rs               # Input handling
├── printer.rs             # Output rendering
├── assets.rs
└── pretty_printer.rs
```

**Key Patterns**:

- **Controller pattern** - Explicit `controller.rs` for orchestration
- **Separate input/output** - `input.rs` and `printer.rs`
- **Configuration layer** - Dedicated config handling

**Observations**:

- ✅ Clear MVC-like pattern (controller, input, printer)
- ✅ Single responsibility for each module
- ✅ Explicit orchestration layer
- ✅ Clean separation of concerns

### 1.5 fd (Find Alternative)

**Repository**: <https://github.com/sharkdp/fd>

**Structure**:

```text
src/
├── main.rs
├── app.rs                  # Application setup
├── config.rs               # Configuration
├── exec.rs                 # Execution logic
├── output.rs               # Output formatting
└── walk.rs                 # Core logic
```

**Key Patterns**:

- **Flat structure** - Simple, direct organization
- **Dedicated output module** - `output.rs` for formatting
- **App coordinator** - `app.rs` for setup

**Observations**:

- ✅ Simple and effective
- ✅ Clear output separation
- ⚠️ No explicit controller/dispatcher
- ✅ Good for smaller CLIs

### 1.6 Summary: Common Rust CLI Patterns

| Pattern                     | cargo | ripgrep    | rustup | bat | fd         |
| --------------------------- | ----- | ---------- | ------ | --- | ---------- |
| **CLI Module**              | ✓     | ✓          | ✓      | ✓   | ✗          |
| **Explicit Dispatcher**     | ✗     | ✗          | ✓      | ✗   | ✗          |
| **Controller Pattern**      | ✗     | ✓ (app.rs) | ✗      | ✓   | ✓ (app.rs) |
| **Dedicated Output**        | ✗     | ✓ (crate)  | ✗      | ✓   | ✓          |
| **Operations/Commands Dir** | ✓     | ✗          | ✗      | ✗   | ✗          |

**Key Takeaways**:

1. **No single standard** - Different approaches for different scales
2. **CLI separation universal** - All separate argument parsing
3. **Controller pattern common** - Many use explicit orchestration layer
4. **Output isolation valuable** - Dedicated printer/output modules
5. **Flat vs. nested** - Smaller tools use flat, larger use nested

## 2. Web Framework Patterns (MVC/MVT)

### 2.1 Model-View-Controller (MVC)

**Origin**: Smalltalk-80 (1970s), popularized by Ruby on Rails

**Structure**:

```text
Request → Router → Controller → Model
                      ↓
                    View → Response
```

**Responsibilities**:

- **Router**: Maps requests to controllers
- **Controller**: Orchestrates logic, coordinates models and views
- **Model**: Business logic and data
- **View**: Presentation and rendering

**Application to CLI**:

```text
CLI Args → Dispatcher → Controller → Domain
                          ↓
                        View → Output
```

**Mapping**:

| Web MVC    | CLI Equivalent           | Purpose                    |
| ---------- | ------------------------ | -------------------------- |
| Router     | Dispatcher               | Route commands to handlers |
| Controller | Command Handler          | Orchestrate execution      |
| Model      | Domain/Application Layer | Business logic             |
| View       | Output Renderer          | Format output              |

### 2.2 Model-View-Template (MVT) - Django

**Structure**:

```text
Request → URL Dispatcher → View (Controller) → Model
                             ↓
                          Template → Response
```

**Key Difference**: "View" acts as controller, "Template" is the view

**Relevance to CLI**:

- Django's URL dispatcher ≈ CLI command dispatcher
- Django's View ≈ CLI command handler
- Django's Template ≈ CLI output formatter

### 2.3 Presentation-Abstraction-Control (PAC)

**Structure**: Hierarchical agents, each with:

- **Presentation**: User interface
- **Abstraction**: Business logic
- **Control**: Coordination between layers

**Relevance to CLI**:

- **Presentation**: CLI parsing + output rendering
- **Abstraction**: Application/domain logic
- **Control**: Command dispatcher + handlers

### 2.4 Key Insights from Web Frameworks

1. **Routing Separation**: Always separate routing from handling
2. **Controller Pattern**: Orchestration layer between input and logic
3. **View Isolation**: Rendering logic separate from business logic
4. **Clear Boundaries**: Each layer has distinct responsibility
5. **Testability**: Layers can be tested independently

## 3. Industry Standards and Common Patterns

### 3.1 POSIX CLI Conventions

**Standard patterns**:

- Argument parsing (getopt)
- Stdout for results
- Stderr for diagnostics
- Exit codes for status

**Architectural implications**:

- Need clear stdout/stderr separation
- Output formatting must respect conventions
- Error handling separate from results

### 3.2 Command Pattern (Design Pattern)

**Structure**:

```text
Invoker → Command Interface → Concrete Command → Receiver
```

**Application to CLI**:

```text
Dispatcher → Command Trait → Specific Command → Application Logic
```

**Benefits**:

- Uniform interface for all commands
- Easy to add new commands
- Supports undo/redo (if needed)
- Request queuing and logging

### 3.3 Chain of Responsibility Pattern

**Structure**:

```text
Request → Handler 1 → Handler 2 → ... → Handler N
```

**Application to CLI**:

- Middleware pattern for cross-cutting concerns
- Pre-processing (validation, auth, setup)
- Post-processing (cleanup, logging)

**Example**:

```text
Input → Validation → Dispatcher → Handler → Output Formatter → User
```

### 3.4 The Twelve-Factor App

**Relevant principles**:

1. **Codebase**: One codebase, many deploys
2. **Dependencies**: Explicitly declare dependencies
3. **Config**: Store config in environment
4. **Logs**: Treat logs as event streams

**CLI Architecture Impact**:

- Configuration separate from code
- Logging infrastructure at presentation layer
- Environment variable handling
- Clear separation of concerns

### 3.5 SOLID Principles Application

**Single Responsibility**:

- Each module/layer has ONE reason to change
- CLI parsing ≠ command execution ≠ output rendering

**Open/Closed**:

- Easy to add new commands without modifying dispatcher
- New output formats without changing handlers

**Liskov Substitution**:

- Command implementations interchangeable
- Output renderers swappable

**Interface Segregation**:

- Small, focused interfaces (Command, Renderer, etc.)
- Clients don't depend on unused methods

**Dependency Inversion**:

- Handlers depend on abstractions (traits)
- Not concrete implementations

## 4. CLI Design Best Practices

### 4.1 CLI Guidelines (clig.dev)

**Key recommendations**:

1. **Output**: Stdout for results, stderr for messages
2. **Consistency**: Follow established patterns
3. **Discoverability**: Clear help and documentation
4. **Robustness**: Handle errors gracefully

**Architectural requirements**:

- Dual-channel output system
- Consistent command structure
- Built-in help generation
- Structured error handling

### 4.2 Command-Line Interface Guidelines (Microsoft)

**Principles**:

1. **Predictability**: Consistent command structure
2. **Composability**: Commands work together
3. **Feedback**: Progress and status information
4. **Error handling**: Clear, actionable errors

**Architecture needs**:

- Uniform command interface
- Progress reporting system
- Error context and suggestions

### 4.3 The Art of Command Line (GitHub Guide)

**Best practices**:

- Use standard streams correctly
- Provide machine-readable output
- Support piping and redirection
- Graceful degradation

**Design implications**:

- Output abstraction layer
- Format options (JSON, plain text)
- Channel awareness (TTY detection)

## 5. Pattern Synthesis

### 5.1 Layered Architecture for CLI

Based on research, an effective CLI presentation layer should have:

```text
┌─────────────────────────────────────┐
│     Input Layer (Parsing)           │
│  - CLI argument parsing             │
│  - Input validation                 │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│   Dispatch Layer (Routing)          │
│  - Command routing                  │
│  - Context management               │
│  - Middleware hooks                 │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  Controller Layer (Orchestration)   │
│  - Command handlers                 │
│  - Application coordination         │
│  - Error handling                   │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│    View Layer (Rendering)           │
│  - Output formatting                │
│  - Progress indicators              │
│  - Channel management (stdout/err)  │
└─────────────────────────────────────┘
```

### 5.2 Key Principles Identified

1. **Separation of Concerns**

   - Input parsing separate from execution
   - Execution separate from output
   - Each layer has single responsibility

2. **Explicit Routing**

   - Central dispatcher for command routing
   - Clear entry point for all commands
   - Easy to add middleware

3. **Controller Pattern**

   - Orchestration layer for business logic coordination
   - No direct I/O in controllers
   - Testable without CLI infrastructure

4. **Output Abstraction**

   - Dedicated rendering layer
   - Format-agnostic (text, JSON, etc.)
   - Channel-aware (stdout vs stderr)

5. **Standard Terminology**
   - Use familiar names (Controller, View, Dispatcher)
   - Industry-standard patterns
   - Easy onboarding for contributors

### 5.3 Recommended Module Structure

```text
presentation/
├── input/              # Input parsing and validation
│   └── cli/           # Clap definitions
├── dispatch/          # Command routing and context
│   ├── router.rs      # Main dispatcher
│   └── middleware/    # Cross-cutting concerns
├── controllers/       # Command handlers
│   └── {command}/     # Individual command modules
└── views/            # Output rendering
    ├── formatters/    # Output formats (JSON, text)
    ├── messages/      # Message types
    ├── progress/      # Progress indicators
    └── terminal/      # Channel management
```

### 5.4 Pattern Comparison

| Pattern                     | Complexity | Testability | Scalability | Best For                              |
| --------------------------- | ---------- | ----------- | ----------- | ------------------------------------- |
| **Flat** (fd-style)         | Low        | Medium      | Low         | Small CLIs (< 5 commands)             |
| **Two-Level** (cargo-style) | Medium     | Medium      | Medium      | Medium CLIs (5-15 commands)           |
| **Layered** (MVC-inspired)  | High       | High        | High        | Large CLIs (15+ commands)             |
| **Multi-Crate** (ripgrep)   | Very High  | Very High   | Very High   | Complex CLIs with reusable components |

## 6. Recommendations

### 6.1 For Small CLIs (< 5 commands)

**Structure**: Flat with minimal layers

```text
src/
├── cli.rs        # Argument parsing
├── commands/     # Command implementations
├── output.rs     # Output formatting
└── main.rs
```

**Why**: Simplicity over architecture

### 6.2 For Medium CLIs (5-15 commands)

**Structure**: Two-level with explicit separation

```text
src/presentation/
├── cli/          # CLI parsing
├── commands/     # Command handlers
└── output/       # Output rendering
```

**Why**: Balance between organization and complexity

### 6.3 For Large CLIs (15+ commands)

**Structure**: Layered architecture (MVC-inspired)

```text
src/presentation/
├── input/        # Input layer
├── dispatch/     # Routing layer
├── controllers/  # Handler layer
└── views/        # Rendering layer
```

**Why**: Clear boundaries, high testability, easy to extend

### 6.4 Universal Recommendations

Regardless of size:

1. ✅ **Separate CLI parsing** from execution
2. ✅ **Isolate output rendering** from business logic
3. ✅ **Use standard streams** correctly (stdout/stderr)
4. ✅ **Provide structured errors** with actionable messages
5. ✅ **Support multiple output formats** when useful
6. ✅ **Test each layer** independently

## 7. Conclusion

### Key Findings

1. **No universal standard** - Different scales need different approaches
2. **Common patterns exist** - Input parsing, orchestration, output rendering
3. **MVC translates well** - Web framework patterns apply to CLI
4. **Layering valuable** - Clear boundaries improve maintainability
5. **Industry practices** - Follow POSIX conventions and CLI guidelines

### Applicability

- **Small projects**: Keep it simple, focus on clarity
- **Growing projects**: Introduce layers as complexity increases
- **Large projects**: Full layered architecture with explicit routing

### Further Reading

- **CLI Guidelines**: <https://clig.dev/>
- **Command Line Interface Guidelines**: <https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/windows-commands>
- **Rust API Guidelines**: <https://rust-lang.github.io/api-guidelines/>
- **The Twelve-Factor App**: <https://12factor.net/>
- **Design Patterns**: Gang of Four (Command, Chain of Responsibility)

---

**Research conducted by**: GitHub Copilot  
**Date**: November 6, 2025  
**Status**: Complete
