---
name: place-code-in-ddd-layers
description: Guide for placing code in the correct DDD (Domain-Driven Design) layer in this Rust project. Covers the four-layer architecture (Domain, Application, Infrastructure, Presentation), dependency rules, what belongs in each layer, red flags for wrong placement, and a decision flowchart. Use when writing new code, refactoring, or moving code between layers. Triggers on "DDD layer", "which layer", "where does this code go", "domain layer", "application layer", "infrastructure layer", "presentation layer", "place code", or "layer placement".
metadata:
  author: torrust
  version: "1.0"
---

# Placing Code in the Correct DDD Layer

## Layer Overview

```text
Presentation  →  Application  →  Domain
                 Infrastructure  →  Domain
```

**Dependency rule**: Dependencies flow inward toward Domain. Domain never depends on
Application or Infrastructure.

| Layer             | Path                  | Purpose                                         |
| ----------------- | --------------------- | ----------------------------------------------- |
| `domain/`         | `src/domain/`         | Business logic, entities, value objects, traits |
| `application/`    | `src/application/`    | Commands, steps, use cases, DTOs                |
| `infrastructure/` | `src/infrastructure/` | File I/O, SSH, OpenTofu, Ansible, HTTP          |
| `presentation/`   | `src/presentation/`   | CLI definitions, output formatting, dispatch    |

## What Belongs Where

### Domain (`src/domain/`)

✅ Value objects with validation (`EnvironmentName`, `Username`, `TraceId`)
✅ Domain entities (`Environment<S>` type-state pattern)
✅ Domain traits/interfaces (`Clock`, `EnvironmentRepository`)
✅ Business rules and domain constraints
✅ `#[derive(Serialize, Deserialize)]` on entities for persistence (pragmatic)

❌ File I/O (`std::fs`, `tokio::fs`)
❌ HTTP clients (`reqwest`, `hyper`)
❌ External tools (OpenTofu, Ansible, SSH)
❌ DTOs with raw `String` primitives

### Application (`src/application/`)

✅ Command handlers (`CreateCommand`, `ProvisionCommand`)
✅ Steps orchestrated by commands
✅ DTOs for data transfer across boundaries
✅ Application services and use case logic

❌ Business rules (belong in Domain)
❌ External I/O (belongs in Infrastructure)
❌ UI rendering (belongs in Presentation)

### Infrastructure (`src/infrastructure/`)

✅ SSH client, file system operations
✅ OpenTofu and Ansible executors
✅ Template renderers
✅ Repository implementations
✅ Custom serialization logic

❌ Business logic (belongs in Domain)
❌ Use case orchestration (belongs in Application)

### Presentation (`src/presentation/`)

✅ Clap CLI command definitions
✅ `UserOutput` and `ProgressReporter`
✅ Command dispatch / routing
✅ Error display formatting

❌ Business logic (belongs in Domain)
❌ External I/O (belongs in Infrastructure)

## Decision Flowchart

```text
Is it a business rule or invariant?
├─ YES → Domain
└─ NO → Does it orchestrate a use case?
    ├─ YES → Application
    └─ NO → Does it talk to external systems/files?
        ├─ YES → Infrastructure
        └─ NO → Is it CLI/UI concern?
            └─ YES → Presentation
```

## Red Flags

- `std::fs` or `tokio::fs` import in `domain/` → move to `infrastructure/`
- Raw `String` DTO deriving `Deserialize` in `domain/` → move to `application/`
- Business logic in `presentation/` → move to `domain/` or `application/`
- `application/` importing from `presentation/` → forbidden dependency

## Reference

Full guide with examples: [`docs/contributing/ddd-layer-placement.md`](../../docs/contributing/ddd-layer-placement.md)
Architecture overview: [`docs/codebase-architecture.md`](../../docs/codebase-architecture.md)
