# Requirements Analysis

> **📝 Q&A Session**  
> Questions and answers that defined the Environment State Management feature specification.

## 🔍 Clarifying Questions and Responses

This document captures the clarifying questions asked during feature specification and the refined responses that guided the implementation approach.

### 1. State Granularity & Transitions

**Question**: Should we track intermediate states during command execution, or just track the final state after each command completes?

**Response**: We want to track **intermediate states**. Without them, we won't know if a command has started execution. For example, if the provision command fails after creating the VM during cloud-init execution, we would not even know the VM was created.

**Question**: For failed states, should we include context about what specifically failed?

**Response**: Since commands are composed of steps, we can store the name of the step that failed for now.

### 2. State Persistence Strategy

**Question**: Should the state file include just the current state enum, timestamp, transition history, or error details?

**Response**: It should include all the information in the Environment type, including the current state. Instead of storing transitions as part of the domain, we can keep it simple for now and log transitions with tracing so we can check logs to see the transitions between states. If that information becomes critical for the domain in the future, we can transition to an event sourcing model.

**Question**: Should we validate that the actual infrastructure matches the stored state when loading?

**Response**: Not for now. Maybe we can do it in the future with the "status" command or "test" command.

### 3. Command Integration Points

**Question**: Should state transitions happen at the beginning of a command, at the end of a command, or both?

**Response**: Both. We need to track when commands start and when they complete.

**Question**: If a command is interrupted (Ctrl+C, system crash), should we have a way to detect and handle "stuck" states?

**Response**: It's enough to know the environment was in the intermediate state (e.g., "provisioning"). In the future, we can track progress for steps or include sub-states for transitions between steps.

### 4. Status Query Implementation

**Question**: Should the status command show just the current state, state + timestamp + last command run, or all environments at once?

**Response**: Only the current state for now. We can get the rest via logging for now.

**Question**: Should the status command also validate the actual infrastructure state against the stored state?

**Response**: Not for now.

### 5. Repository Pattern & Storage

**Question**: Should we create a generic StateRepository trait that could support different storage backends in the future?

**Response**: Yes.

**Question**: Should state changes be atomic (write to temp file, then rename) to avoid corruption during write failures?

**Response**: Yes.

### 6. Error Recovery Considerations

**Question**: Should we store enough information in failed states to help users understand what went wrong and what they might try manually?

**Response**: Yes, but not in the first iteration. We can improve that later.

**Question**: Should we allow manual state transitions for recovery scenarios?

**Response**: Not for now. In this first iteration, we can only inform about the error and the user can destroy the environment and start from scratch.

## 📊 Key Decisions Made

Based on the responses above, the following key decisions were made for the feature specification:

### State Management Approach

1. **Intermediate State Tracking**: Track both command start (intermediate states) and completion (final states) for full visibility
2. **Error Context**: Store step name that failed within commands
3. **Simple Persistence**: Store complete Environment object in JSON format
4. **Logging-Based History**: Use tracing for state transition history instead of domain persistence

### Storage Strategy

1. **Repository Pattern**: Generic trait with JSON file implementation
2. **Atomic Operations**: Use temp file + rename pattern for data integrity
3. **No Validation**: Don't validate infrastructure state matches stored state in initial iteration
4. **Simple Status**: Display only current state, additional info via logging

### Error Handling Philosophy

1. **Fail-Safe Approach**: Prioritize state visibility over automatic recovery
2. **Manual Recovery**: Users destroy and recreate environments when errors occur
3. **Future Enhancement**: Plan for detailed error information and recovery suggestions
4. **Graceful Degradation**: Interrupted commands remain in intermediate states

### Implementation Priorities

1. **Core Functionality First**: Focus on basic state tracking and persistence
2. **Testing Foundation**: Ensure state management doesn't break existing functionality
3. **Incremental Enhancement**: Plan for future improvements without over-engineering
4. **User Experience**: Prioritize clarity and actionability in error states

## 🎯 Scope Boundaries

### In Scope for Initial Implementation

- Basic state machine with valid transitions
- JSON file-based state persistence
- Command integration for state updates
- Simple status query functionality
- Error state tracking with step context

### Out of Scope for Initial Implementation

- Infrastructure state validation
- Manual state transitions for recovery
- Detailed error recovery suggestions
- Step-level progress tracking
- State transition history persistence
- Automatic error recovery mechanisms

### Future Enhancements Planned

- Enhanced error recovery with detailed guidance
- Step-level progress tracking within commands
- Infrastructure state validation capabilities
- Event sourcing for complete audit trail
- Manual state reset and recovery tools

## 🔍 Critical Gap Analysis & Responses

After the initial feature specification, a critical gap analysis was performed to identify potential issues and missing considerations. Below are the gaps identified and the user's responses:

### 1. Type Erasure Implementation Detail

**Gap**: The `into_any()` method implementation is marked as `todo!()`. This is a critical piece for the type-state pattern to work with persistence.

**User Response**: Document it as a known implementation detail that needs completion during development.

**Action**: Add clear documentation in implementation plan noting this requires completion by the development team.

### 2. Error Recovery Strategy

**Gap**: While failure states exist (`ProvisionFailed`, `ConfigureFailed`, etc.), the documentation doesn't clearly define recovery workflows.

**User Response**: For now, in this first iteration, we only inform the user via internal logging and the user can destroy the environment and start from scratch. Notice that there is no command yet to destroy the environment, so it must be done manually by running the underlying OpenTofu commands.

**Action**: Document manual recovery process using OpenTofu commands and plan destroy command for future enhancement.

### 3. Concurrent Access Protection

**Gap**: JSON file persistence doesn't address concurrent access, file locking, or race conditions.

**User Response**: We have to implement a lock mechanism to avoid race conditions. Add a mechanism to lock the state file during read/write operations. Yes, we can add the process ID to the lock file to identify which process holds the lock.

**Action**: Implement file locking mechanism with process ID tracking for lock ownership identification.

### 4. State File Migration/Versioning

**Gap**: Missing consideration for schema changes, backward compatibility, and migration strategy.

**User Response**: Since this deployer is expected to be used once and only for some minutes, we can avoid versioning for now. The deployer only helps users to set up the initial environment. If in the future the application evolves to manage long-lived environments, we can add versioning and migration strategies at that time. We only need to document why we are not doing versioning for now.

**Action**: Document rationale for deferring versioning due to short-lived deployment usage pattern.

### 5. Timestamp/Audit Trail

**Gap**: Current design lacks timestamp tracking, duration metrics, and audit trail capabilities.

**User Response**: Tracing logging will be enough for now. Logs have timestamps and can be used for audit trails. We have to log (info level) each state transition with timestamps.

**Action**: Ensure all state transitions are logged at info level with timestamps for audit trail purposes.

### 6. Type Erasure Pattern Implementation

**Recommendation**: Complete type erasure pattern for all states.

**User Response**: OK, go ahead.

**Action**: Add complete implementation example for type erasure pattern to documentation.

### 7. Recovery Strategy Documentation

**Recommendation**: Add section explaining how to handle failed states and define recovery workflows.

**User Response**: OK, go ahead. We can explain how to destroy the environment manually using OpenTofu commands.

**Action**: Add detailed recovery documentation with manual OpenTofu cleanup procedures.

## 📝 Updated Implementation Priorities

Based on the gap analysis feedback, the following priorities have been established:

### Must Have (Critical)

1. **File Locking Mechanism**: Implement state file locking with process ID tracking
2. **State Transition Logging**: Log all state transitions at info level with timestamps
3. **Type Erasure Pattern**: Complete implementation for all state types
4. **Recovery Documentation**: Document manual cleanup using OpenTofu commands

### Should Have (Important)

1. **Lock File Management**: Clean up stale locks and handle process crashes
2. **Error Context Enhancement**: Provide clear error messages with actionable guidance
3. **Manual Cleanup Guide**: Step-by-step instructions for environment destruction

### Won't Have (Deferred)

1. **State Versioning**: Deferred due to short-lived deployment usage pattern
2. **Automatic Recovery**: Users manually destroy and recreate on failures
3. **Destroy Command**: Manual OpenTofu usage sufficient for initial iteration
