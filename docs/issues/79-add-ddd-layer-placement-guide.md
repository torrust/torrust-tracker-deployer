# Issue: Add DDD Layer Placement Guide to Contributing Documentation

**Issue**: [#79](https://github.com/torrust/torrust-tracker-deployer/issues/79)
**Related**: [#75](https://github.com/torrust/torrust-tracker-deployer/issues/75) - Move config module to correct DDD layer

## 📋 Issue Information

- **Type**: Documentation Enhancement
- **Priority**: High
- **Related Issue**: #75 - Move config module to correct DDD layer
- **Parent Epic**: None (standalone improvement)

## 🎯 Problem Statement

The project lacks clear guidelines on which code belongs in which DDD layer (Domain, Application, Infrastructure, Presentation). This has led to violations like the config module (#75) being placed in the domain layer when it clearly belongs in the application layer.

Without explicit documentation, contributors (including AI assistants) may:

- Place DTOs in the domain layer
- Mix infrastructure concerns (file I/O, HTTP) with domain logic
- Create unclear boundaries between layers
- Make code harder to maintain and test

### Why This Matters

The issue that prompted this work (#75) showed how easy it is to place code in the wrong layer without clear guidelines. The config module had several red flags:

1. Used `serde` on DTOs with `String` primitives (data transfer, not domain entities)
2. Had `generate_template_file()` with file I/O (infrastructure concern)
3. Used `String` primitives instead of domain value objects
4. Documentation admitted it "sits at the boundary" (not domain!)

**Important Distinction**: Domain entities with serde derives for persistence (like `Environment<S>`) are acceptable because:

- Serde derives are code generation, not implementation
- Actual serialization logic lives in the serde library
- Domain code doesn't contain persistence logic
- If custom serialization needed, implement in infrastructure layer

This guide will help distinguish between:

- ✅ Domain entities with serde for persistence
- ❌ DTOs masquerading as domain entities

## 💡 Proposed Solution

Create a comprehensive guide `docs/contributing/ddd-layer-placement.md` that clearly documents:

1. **Purpose of each layer** - What belongs where and why
2. **Rules and red flags** - Clear indicators of wrong placement
3. **Nuanced guidance** - When seemingly "wrong" patterns are actually acceptable (e.g., serde on entities)
4. **Examples from codebase** - Real code showing correct placement
5. **Decision flowchart** - Quick reference for "where does this go?"

### Layer Specification

#### Domain Layer (`src/domain/`)

**Purpose**: Pure business logic, entities, value objects, domain events

**Rules**:

- ✅ YES: Value objects with validation (`EnvironmentName`, `SshCredentials`)
- ✅ YES: Entities with business logic (`Environment<S>`)
- ✅ YES: Domain traits defining contracts (`Clock`, `EnvironmentRepository`)
- ✅ YES: Serde derives on entities for persistence (pragmatic trade-off)
- ❌ NO: File I/O operations (`std::fs`, `tokio::fs`)
- ❌ NO: HTTP clients, external APIs
- ❌ NO: DTOs with `String` primitives and serde (these are application layer)
- ❌ NO: Manual `impl Serialize` (belongs in infrastructure)

**Red Flags**:

- Using `serde` on DTOs with `String` primitives (not domain entities)
- Importing `std::fs`, `tokio::fs`, `reqwest`, `hyper`
- Methods that read/write files directly
- Raw `String` types without domain semantics

**Nuance - Serde on Domain Entities:**

Using `#[derive(Serialize, Deserialize)]` on domain entities for **persistence** is a pragmatic trade-off:

- ✅ OK: Domain entities with serde derives for repository persistence
- ✅ OK: Value objects that need to be stored
- ❌ NOT OK: DTOs pretending to be domain entities
- ❌ NOT OK: Manual `impl Serialize` in domain code (put in infrastructure)

**Rationale**: Serde derives are code generation (external to your domain code). The actual serialization logic lives in the serde library, not your domain layer. Many Rust projects make this pragmatic choice to avoid boilerplate infrastructure code.

If you need **custom serialization logic**, implement it in the infrastructure layer and keep domain types pure.

#### Application Layer (`src/application/`)

**Purpose**: Use cases, DTOs, command handlers, orchestration

**Rules**:

- ✅ YES: Command handlers that orchestrate domain and infrastructure
- ✅ YES: DTOs for data transfer (`EnvironmentCreationConfig`)
- ✅ YES: Serde for JSON/TOML deserialization
- ✅ YES: Use cases that coordinate multiple domain operations
- ✅ YES: Application-level error aggregation
- ❌ NO: Business logic (belongs in domain)
- ❌ NO: Direct file I/O (use infrastructure traits)
- ❌ NO: Direct external API calls (use infrastructure traits)

**Examples**:

- `ProvisionCommand` - Orchestrates provisioning steps
- `EnvironmentCreationConfig` - DTO for create command
- `CommandError` types - Application-level error aggregation

#### Infrastructure Layer (`src/infrastructure/`)

**Purpose**: External integrations, persistence, I/O operations

**Rules**:

- ✅ YES: File I/O implementations
- ✅ YES: HTTP clients, external API integrations
- ✅ YES: Repository implementations
- ✅ YES: OpenTofu, Ansible, SSH clients
- ✅ YES: Custom serialization implementations
- ❌ NO: Business rules or validation
- ❌ NO: Domain entities (reference them, don't define them)

**Examples**:

- `OpenTofuClient` - Wraps OpenTofu CLI
- `JsonEnvironmentRepository` - File-based persistence
- `SshClient` - SSH operations

#### Presentation Layer (`src/presentation/`)

**Purpose**: CLI, user interaction, command parsing

**Rules**:

- ✅ YES: Clap command definitions
- ✅ YES: User input validation and parsing
- ✅ YES: Output formatting and display
- ✅ YES: Routing to application layer
- ❌ NO: Business logic
- ❌ NO: Direct infrastructure calls (go through application)

**Examples**:

- `Cli` struct with clap derives
- Command handlers that route to application layer

### Decision Flowchart

```text
Does it contain business rules or validation?
├─ YES → Domain Layer
│   └─ Does it need persistence?
│       ├─ YES → Add #[derive(Serialize, Deserialize)]
│       └─ NO → Pure domain type
│
└─ NO → Is it data transfer or orchestration?
    ├─ Data Transfer (DTO) → Application Layer
    ├─ Orchestration (Use Case) → Application Layer
    ├─ External Integration → Infrastructure Layer
    └─ User Interface → Presentation Layer
```

## 📝 Implementation Plan

### Deliverable

Create `docs/contributing/ddd-layer-placement.md` with:

1. **Introduction**

   - Why DDD matters for this project (reference to `docs/vision-infrastructure-as-software.md`)
   - Overview of the four layers
   - Benefits of proper layer separation

2. **Layer Specifications** (detailed rules for each layer)

   - Domain Layer - with nuanced serde guidance
   - Application Layer
   - Infrastructure Layer
   - Presentation Layer

3. **Decision Flowchart**

   - Quick reference diagram
   - "Where does my code belong?" guide

4. **Real Examples**

   - Correct placements from current codebase
   - Common mistakes to avoid
   - Refactoring examples (like config module move)

5. **Integration**
   - Link from `docs/contributing/README.md`
   - Reference in `.github/copilot-instructions.md`

### Steps

1. Create the guide document with all sections
2. Add real code examples from the codebase
3. Link from `docs/contributing/README.md`
4. Update `.github/copilot-instructions.md` to reference the guide
5. Run linters and commit

### Integration Points

- **Contributing Guide**: Add link from `docs/contributing/README.md`
- **AI Instructions**: Reference in `.github/copilot-instructions.md`
- **Related Docs**: Reference from module organization guide

## ✅ Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

- [ ] Document created at `docs/contributing/ddd-layer-placement.md`
- [ ] All four layers documented with clear rules and examples
- [ ] Nuanced guidance on serde usage included (entities vs DTOs)
- [ ] Decision flowchart included for quick reference
- [ ] Real code examples from the codebase
- [ ] Linked from `docs/contributing/README.md`
- [ ] Referenced in `.github/copilot-instructions.md`
- [ ] All linters pass (markdownlint, cspell)
- [ ] Document follows project markdown conventions

## 🔗 Related Issues

- #75 - Move config module to correct DDD layer (the issue that revealed this need)

## 📚 Reference Materials

### Internal Documentation

- Project vision for Infrastructure as Software (`docs/vision-infrastructure-as-software.md`)
- Current codebase architecture (`docs/codebase-architecture.md`)
- Module organization guide (`docs/contributing/module-organization.md`)

### External Resources

- **Herberto Graça - Explicit Architecture**

  - [DDD, Hexagonal, Onion, Clean, CQRS - How I Put It All Together](https://herbertograca.com/2017/11/16/explicit-architecture-01-ddd-hexagonal-onion-clean-cqrs-how-i-put-it-all-together/)
  - Comprehensive synthesis of DDD patterns with visual diagrams and concrete examples

- **Microsoft - DDD Microservices Guide**

  - [Design a DDD-oriented microservice](https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice)
  - Practical guidance on DDD layers with .NET examples, clear layer responsibilities

- **DDD Crew - Starter Modelling Process**

  - [GitHub: DDD Starter Modelling Process](https://github.com/ddd-crew/ddd-starter-modelling-process)
  - Step-by-step guide with EventStorming, Context Maps, decision flowcharts

- **Eric Evans - Domain Language**

  - [Domain Language - DDD Resources](https://www.domainlanguage.com/ddd/)
  - Original DDD resources including free DDD Reference guide

- **Martin Fowler - DDD Overview**
  - [Domain Driven Design](https://martinfowler.com/bliki/DomainDrivenDesign.html)
  - High-level overview and key concepts explanation

## 🏷️ Labels

- `documentation`
- `enhancement`
- `DDD`
- `contributing-guide`
