# Progress Reporting in Application Layer

> **⚠️ DRAFT PROPOSAL**: This feature documentation is a draft and has not yet gone through the full feature definition process (Questions, Specification, etc.). It serves as a preliminary proposal for the implemented changes.

## 1. Current Implementation: Controller-Level Progress Reporting

We have implemented a **Controller-Level Progress Reporting** mechanism using Enums. This addresses the immediate issue of incorrect step counts in the CLI.

### The Solution: Enum-based Step Definition

Instead of hardcoded constants (e.g., `const WORKFLOW_STEPS: usize = 9;`), we now define workflow steps as an `enum` within each command handler.

#### Implementation Details

Each command handler (e.g., `ConfigureCommandHandler`) defines a `Step` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    ValidateEnvironment,
    RunPlaybook,
}

impl Step {
    /// Returns the total number of steps in the workflow
    fn count() -> usize {
        2 // Updated to match variants
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateEnvironment => "Validating environment",
            Self::RunPlaybook => "Running configuration playbook",
        }
    }
}
```

#### Why this approach?

1. **Single Source of Truth**: The steps and their count are defined in one place.
2. **Type Safety**: The compiler ensures all steps are handled.
3. **Maintainability**: Adding a step requires updating the Enum, making it obvious that the count needs to change.

## 2. Future Work: Application Layer Progress Reporting

The current implementation only reports progress at the **Controller** level. This means:

- The Controller knows _which_ high-level command is running (e.g., "Running Playbook").
- The Controller _does not_ know the internal progress of that command (e.g., "Playbook task 5/20").

**The Challenge**: Long-running operations inside the Application Layer (like Ansible playbooks, Terraform runs, or file downloads) appear as a single "frozen" step to the user until they complete.

**The Goal**: We need to introduce a mechanism to report progress _from inside_ the Application Layer back to the Presentation Layer. This will likely involve:

- Passing a `ProgressReporter` trait or callback into the Application Layer.
- Having Domain services publish progress events.

## 3. Verbosity Levels

This feature is closely related to the concept of **Verbosity Levels**. We aim to support different levels of detail in the user output, as defined in our UX research.

- **Definition**: See [User Output vs Internal Logging: Architectural Decision](../../research/UX/user-output-vs-logging-separation.md) for the definition of `VerbosityLevel` (Quiet, Normal, Verbose, VeryVerbose, Debug).
- **Goal**: The progress reporter should respect these levels (e.g., showing detailed sub-steps only in `Verbose` mode).

## 4. Roadmap

This feature is part of the implementation of the following roadmap task:

- [docs/roadmap.md](../../roadmap.md): **1.9** Add levels of verbosity as described in the UX research.
