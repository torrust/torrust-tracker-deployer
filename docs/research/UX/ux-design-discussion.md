# UX Design Research for Torrust Deployer

> **📋 Research Document Status**  
> This document contains UX research and alternative design proposals that differ from the current planned implementation. These are **research findings only** - the final UX design is still being decided and may combine elements from multiple approaches.

## Context

This document summarizes research and discussion on user experience design for the Torrust Tracker deployer. The focus is on creating a console application that is accessible to developers who are dabbling in self-hosting but are not professional system administrators.

## Relationship to Current Project Design

The current project design (documented in [Console Commands](../../console-commands.md) and [Deployment Overview](../../deployment-overview.md)) follows a **granular command approach** with individual commands for each deployment step:

```bash
# Current planned implementation
torrust-tracker-deployer create <env>      # Initialize environment
torrust-tracker-deployer provision <env>   # Create infrastructure
torrust-tracker-deployer configure <env>   # Setup system
torrust-tracker-deployer release <env>     # Deploy application
torrust-tracker-deployer run <env>         # Start services
```

This research document explores **alternative UX approaches**, including:

- Single-command wizard mode (`torrust-tracker-deployer up`)
- Resumable state management (`torrust-tracker-deployer continue`)
- Hybrid approaches that combine both patterns

The **final implementation may combine elements from both approaches**, offering individual step commands for experienced users alongside simplified workflows for beginners.

## Project Overview

- **Purpose**: Building a deployer for the Torrust Tracker
- **First version**: CLI tool with sequential deployment steps
- **Process flow**: `provision → configure → install → release → run → test`
- **Target audience**: Developers interested in self-hosting (not professional sysadmins)

## UX Goals

The user experience design is guided by these core objectives:

- **Reduce friction and confusion** for non-expert users
- **Provide a guided experience** while still allowing control for advanced users
- **Make errors actionable** with clear next steps and recovery instructions

## Proposed UX Improvements

### 1. Single Entrypoint / Wizard Mode

**Command**: `torrust-tracker-deployer up`

**Functionality**:

- Runs all steps in sequential order with progress indicators
- Pauses on errors, explains the issue clearly
- Suggests specific fixes with copy-paste commands
- Continues automatically after user resolves issues

**Benefits**:

- Eliminates need to remember command sequences
- Provides guided experience for beginners
- Maintains momentum through the deployment process

### 2. Resumable State Management

**State Storage**: `.torrust-tracker-deployer/state.json`

**Resume Command**: `torrust-tracker-deployer continue`

**Features**:

- Tracks progress through deployment steps
- Remembers configuration and choices
- Enables recovery from interruptions
- Prevents re-running completed steps unnecessarily

### 3. Interactive Configuration

**Approach**: Prompt users for settings instead of requiring manual config file editing

**Features**:

- Auto-generate `torrust.toml` based on user responses
- Validate inputs in real-time
- Provide sensible defaults with explanation
- Allow review and modification before proceeding

**Benefits**:

- Reduces configuration errors
- Eliminates need to understand complex config file formats
- Guides users through required vs optional settings

### 4. User Mode System

#### Beginner Mode (Default)

- **Characteristics**: Guided, interactive experience
- **Prompts**: Asks for user input when needed
- **Explanations**: Provides context for each step
- **Error handling**: Verbose, educational error messages

#### Expert Mode

- **Command**: `--non-interactive --config ./torrust.toml`
- **Characteristics**: Minimal prompts, assumes expertise
- **Use cases**: Automation, CI/CD pipelines, experienced users
- **Error handling**: Concise, technical error messages

### 5. Enhanced Error Handling

**Visual Design**:

- **Colors and symbols**: `✅` success, `⚠` warning, `❌` error
- **Consistent formatting**: Clear visual hierarchy
- **Progress indicators**: Show current step and overall progress

**Error Content**:

- **Friendly language**: Avoid technical jargon when possible
- **Actionable hints**: Provide copy-paste commands for fixes
- **Context preservation**: Maintain user's place in the workflow
- **Recovery guidance**: Clear instructions on how to continue

### 6. Hybrid Command Architecture (Research Proposal)

**Concept**: Combine single-command convenience with granular control for different user types.

**Beginner-Friendly Commands**:

- `torrust-tracker-deployer up <env>` - Full deployment wizard with progress tracking
- `torrust-tracker-deployer continue <env>` - Resume from last completed step

**Expert/CI Commands** (matching current design):

- `torrust-tracker-deployer create <env>` - Individual step control
- `torrust-tracker-deployer provision <env>`
- `torrust-tracker-deployer configure <env>`
- `torrust-tracker-deployer release <env>`
- `torrust-tracker-deployer run <env>`

**Benefits of Hybrid Approach**:

- **Progressive complexity**: Beginners can start with `up`, learn the process, then graduate to individual commands
- **CI/Automation friendly**: Individual commands work better for automated pipelines
- **Error recovery flexibility**: Users can choose between `continue` (easy) or specific step retry (precise)
- **Best of both worlds**: No need to choose between approaches

### 7. Resume Strategy

**Hybrid Approach** offering multiple recovery paths:

1. **Automatic resume**: `torrust-tracker-deployer continue` (recommended for beginners)
2. **Step-specific restart**: `torrust-tracker-deployer provision` (for advanced users)

**Benefits**:

- Accommodates different user preferences
- Provides learning path from beginner to expert usage
- Maintains flexibility for different scenarios

## Example Error Flow

```text
[1/6] Provisioning...
✅ Done

[2/6] Configuring...
✅ Done

[3/6] Installing...
❌ Error: Docker not found in PATH

Hint: Install Docker by running:
  sudo apt install docker.io

----------------------------------------
Your deployment stopped at step [3/6]: install
----------------------------------------

You have two options:

  1. Fix the issue and resume where you left off:
       torrust-tracker-deployer continue

  2. Re-run only the failed step (install):
       torrust-tracker-deployer install

Use --verbose for more details about the error.
```

## Design Principles

### Progressive Disclosure

- **Simple by default**: Hide complexity unless user requests it
- **Expandable detail**: Verbose modes available for troubleshooting
- **Learn as you go**: Introduce advanced concepts gradually

### Familiar Patterns

- **Industry standards**: Follow conventions from Cargo, Ansible, Terraform
- **Predictable behavior**: Consistent command patterns and output formatting
- **Standard conventions**: Use established CLI patterns and terminology

### Recovery-Oriented Design

- **Assume interruptions**: Design for partial completion and resumption
- **Clear state tracking**: Always know where the user is in the process
- **Multiple paths forward**: Provide options for different skill levels

## Implementation Considerations

### Technical Requirements

- **State persistence**: Reliable storage and retrieval of deployment progress
- **Error classification**: Different handling for different types of errors
- **Configuration validation**: Early detection of configuration issues
- **Progress tracking**: Accurate reporting of step completion

### User Testing Priorities

- **First-time user flow**: How intuitive is the initial experience?
- **Error recovery**: How well do users handle and recover from failures?
- **Expert transition**: Can beginners grow into expert usage patterns?
- **Automation integration**: Does expert mode work well in scripts/CI?

## Future Enhancements

### Potential TUI Features

- **Real-time progress bars**: Visual progress indication
- **Interactive task lists**: Expandable/collapsible step details
- **Log panel widgets**: Separate areas for logs vs user interaction
- **Status dashboards**: Overview of deployment health and progress

### Advanced Features

- **Configuration templates**: Pre-built configs for common scenarios
- **Rollback capabilities**: Undo partial deployments
- **Health monitoring**: Post-deployment validation and monitoring
- **Update workflows**: Streamlined processes for upgrading deployments

## Research Status and Outcome

This UX research explored alternative approaches to the console interface design. Based on this research, **a concrete design decision has been made**.

**Decision**: The project will implement a **hybrid command architecture** combining both approaches:

- **Plumbing commands** (low-level): Individual commands for precise control (`provision`, `configure`, `release`, `run`)
- **Porcelain commands** (high-level): Orchestration command for simplified workflows (`deploy`)

**Implementation**: Plumbing commands will be implemented first, followed by porcelain commands built on top of the stable plumbing foundation.

**Key Differences from Research**:

- **Single porcelain command**: `deploy` instead of `up` and `continue`
- **Limited scope**: Porcelain only automates deployment workflow, not management commands
- **State-aware**: The `deploy` command intelligently continues from current environment state

For complete specification details, see:

- [Hybrid Command Architecture Feature](../../features/hybrid-command-architecture/README.md) - Detailed feature specification
- [Console Commands](../../console-commands.md) - Updated implementation documentation

## Related Documentation

- [Console Commands](../../console-commands.md) - Current planned command structure
- [Deployment Overview](../../deployment-overview.md) - Current implementation design
- [Console Output & Logging Strategy](./console-output-logging-strategy.md) - Complementary research on output handling

This UX design research provides a foundation for creating a deployment tool that can serve both newcomers to self-hosting and experienced system administrators, with clear growth paths between different usage patterns.
