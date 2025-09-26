# UX Design Research for Torrust Deployer

> **📋 Research Document Status**  
> This document contains UX research and alternative design proposals that differ from the current planned implementation. These are **research findings only** - the final UX design is still being decided and may combine elements from multiple approaches.

## Context

This document summarizes research and discussion on user experience design for the Torrust Tracker deployer. The focus is on creating a console application that is accessible to developers who are dabbling in self-hosting but are not professional system administrators.

## Relationship to Current Project Design

The current project design (documented in [Console Commands](../../console-commands.md) and [Deployment Overview](../../deployment-overview.md)) follows a **granular command approach** with individual commands for each deployment step:

```bash
# Current planned implementation
torrust-deploy create <env>      # Initialize environment
torrust-deploy provision <env>   # Create infrastructure
torrust-deploy configure <env>   # Setup system
torrust-deploy release <env>     # Deploy application
torrust-deploy run <env>         # Start services
```

This research document explores **alternative UX approaches**, including:

- Single-command wizard mode (`torrust-deploy up`)
- Resumable state management (`torrust-deploy continue`)
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

**Command**: `torrust-deploy up`

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

**State Storage**: `.torrust-deploy/state.json`

**Resume Command**: `torrust-deploy continue`

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

- `torrust-deploy up <env>` - Full deployment wizard with progress tracking
- `torrust-deploy continue <env>` - Resume from last completed step

**Expert/CI Commands** (matching current design):

- `torrust-deploy create <env>` - Individual step control
- `torrust-deploy provision <env>`
- `torrust-deploy configure <env>`
- `torrust-deploy release <env>`
- `torrust-deploy run <env>`

**Benefits of Hybrid Approach**:

- **Progressive complexity**: Beginners can start with `up`, learn the process, then graduate to individual commands
- **CI/Automation friendly**: Individual commands work better for automated pipelines
- **Error recovery flexibility**: Users can choose between `continue` (easy) or specific step retry (precise)
- **Best of both worlds**: No need to choose between approaches

### 7. Resume Strategy

**Hybrid Approach** offering multiple recovery paths:

1. **Automatic resume**: `torrust-deploy continue` (recommended for beginners)
2. **Step-specific restart**: `torrust-deploy provision` (for advanced users)

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
       torrust-deploy continue

  2. Re-run only the failed step (install):
       torrust-deploy install

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

## Research Status and Next Steps

This UX research explores alternative approaches to the console interface design. The **current project implementation** follows the individual command approach documented in [Console Commands](../../console-commands.md).

**Key Decision Points**:

- **Command structure**: Individual commands vs. wizard mode vs. hybrid approach
- **State management**: How to track and resume deployment progress
- **User modes**: Whether to implement beginner/expert mode differentiation
- **Error recovery**: Granular vs. automatic resume strategies

The **final implementation may combine elements from multiple approaches**, potentially offering:

- Individual step commands for experienced users and CI/CD integration
- Simplified wizard commands (`up`, `continue`) for beginner-friendly workflows
- Flexible error recovery options accommodating different user preferences

## Related Documentation

- [Console Commands](../../console-commands.md) - Current planned command structure
- [Deployment Overview](../../deployment-overview.md) - Current implementation design
- [Console Output & Logging Strategy](./console-output-logging-strategy.md) - Complementary research on output handling

This UX design research provides a foundation for creating a deployment tool that can serve both newcomers to self-hosting and experienced system administrators, with clear growth paths between different usage patterns.
