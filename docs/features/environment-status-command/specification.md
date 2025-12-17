# Environment Status Command Specification

## 📋 Overview

This feature adds a new console command to display environment information with state-aware details. Users can quickly view essential information about their deployment environments without manually inspecting JSON files.

### Context

Currently, users must manually inspect environment JSON files in `data/{environment-name}/environment.json` to check environment state and details. This is not user-friendly and doesn't provide a clear view of the environment's current status.

### Problem Statement

Users need a convenient way to:

- Check the current state of an environment
- View basic environment information (name, state, creation time)
- See state-specific details (IP address for provisioned environments, service URLs for running environments)
- Quickly understand environment status without reading JSON files

## 🎯 Goals

### Primary Goals

- **Goal 1**: Provide a console command to display environment information
- **Goal 2**: Show state-aware details (different information based on environment state)
- **Goal 3**: Use human-friendly output formatting consistent with existing commands
- **Goal 4**: Handle errors gracefully (environment not found, invalid state, etc.)

### Secondary Goals (Nice-to-Have)

- Future support for JSON output format
- Verbosity levels for more or less detail
- Provider-specific information display

### Non-Goals

What this feature explicitly does NOT aim to do:

- Real-time monitoring (CPU, memory, network usage)
- Historical state transition tracking
- Log viewing or tailing
- Multiple environment listing (that's the separate `list` command)
- Infrastructure verification and smoke tests (use the `test` command instead)
- Remote service health checks or connectivity validation

## 💡 Proposed Solution

### Approach

Implement a new read-only command that:

1. Loads environment data from the persisted JSON file
2. Extracts relevant information based on current state
3. Formats and displays information using UserOutput system
4. Provides clear error messages for common failure scenarios

The command follows the existing command architecture pattern:

- **Application Layer**: `ShowCommand` (or `StatusCommand`) with command handler
- **Presentation Layer**: Console subcommand for CLI interface
- **Read-Only**: No state modifications, only information display

### Command Name

**Decision**: Use `show`

**Rationale**:

- The command displays stored environment information without verification
- Aligns with common CLI patterns (`kubectl get`, `docker inspect`, `git show`)
- Reserves `status` for future infrastructure/service status checking
- Clearer intent: "show me information about this environment"

**Future Command Separation**:

- **`show`** (this feature) - Display stored environment data (fast, read-only, no network calls)
- **`test`** (already implemented) - Validate infrastructure components (cloud-init, Docker, Docker Compose)
- **`status`** (future) - Check runtime status of deployed services (health checks, connectivity validation)

### Design Overview

```text
┌─────────────────────────────────────────────────────┐
│  User                                               │
└───────────────────────┬─────────────────────────────┘
                        │
                        │ torrust-tracker-deployer show <env>
                        ↓
┌─────────────────────────────────────────────────────┐
│  Presentation Layer                                 │
│  - Parse CLI arguments                              │
│  - Create ExecutionContext                          │
│  - Dispatch to ShowCommand                          │
└───────────────────────┬─────────────────────────────┘
                        │
                        ↓
┌─────────────────────────────────────────────────────┐
│  Application Layer                                  │
│  - ShowCommandHandler                               │
│  - Load environment from persistence                │
│  - Extract state-specific information               │
│  - Format output via UserOutput                     │
└───────────────────────┬─────────────────────────────┘
                        │
                        ↓
┌─────────────────────────────────────────────────────┐
│  Domain Layer                                       │
│  - Environment<S> entity with state                 │
│  - Read-only access to environment data             │
└─────────────────────────────────────────────────────┘
                        ↑
                        │
┌─────────────────────────────────────────────────────┐
│  Infrastructure Layer                               │
│  - EnvironmentLoader (read JSON from filesystem)    │
│  - FileSystemRepository                             │
└─────────────────────────────────────────────────────┘
```

### Output Format Examples

#### Created State

```text
Environment: my-environment
State: Created
Provider: LXD

The environment configuration is ready. Run 'provision' to create infrastructure.
```

**Note**: Creation timestamp will be added in Phase 7 of the implementation.

#### Provisioned State

```text
Environment: my-environment
State: Provisioned
Provider: LXD

Infrastructure:
  Instance IP: 10.140.190.14
  SSH Port: 22
  SSH User: ubuntu
  SSH Key: /home/user/.ssh/torrust_deployer_key

Connection:
  ssh -i /home/user/.ssh/torrust_deployer_key ubuntu@10.140.190.14

Next step: Run 'configure' to set up the system.
```

#### Configured State

```text
Environment: my-environment
State: Configured
Provider: LXD

Infrastructure:
  Instance IP: 10.140.190.14
  SSH Port: 22
  SSH User: ubuntu
  SSH Key: /home/user/.ssh/torrust_deployer_key

System:
  Docker: Installed
  Docker Compose: Installed
  Firewall: Configured

Connection:
  ssh -i /home/user/.ssh/torrust_deployer_key ubuntu@10.140.190.14

Next step: Run 'release' to deploy application configuration.
```

#### Running State

```text
Environment: my-environment
State: Running
Provider: Hetzner Cloud

Infrastructure:
  Instance IP: 157.10.23.45
  SSH Port: 22
  SSH User: root

Tracker Services:
  UDP Trackers:
    - udp://157.10.23.45:6868/announce
    - udp://157.10.23.45:6969/announce
  HTTP Tracker:
    - http://157.10.23.45:7070/announce
    - Health: http://157.10.23.45:7070/health_check
  API Endpoint:
    - http://157.10.23.45:1212/api
    - Health: http://157.10.23.45:1212/api/health_check

Status: ✓ All services running
```

### Key Design Decisions

1. **Command Name: `show`**

   - Displays stored environment information only
   - Consistent with industry CLI patterns for information display
   - Reserves `status` for future service health/runtime status checks
   - Clear separation of concerns: `show` (data), `test` (infrastructure), `status` (services)

2. **Read-Only Operation**

   - No state modifications
   - Fast and safe to run repeatedly
   - No side effects on infrastructure

3. **Human-Friendly Output First**

   - Initial implementation uses pretty-printed output with formatting
   - JSON output can be added later with `--format json` flag
   - Consistent with existing command output patterns

4. **State-Aware Information**

   - Different details shown based on environment state
   - Provides relevant, actionable information for each state
   - Clear next-step guidance

5. **No Remote Verification**
   - Display stored state information only
   - No network calls to remote infrastructure
   - Fast and reliable operation
   - Infrastructure verification is handled by the separate `test` command

### Alternatives Considered

#### Option 1: Call it `status` instead of `show`

- **Pros**: Common for checking service state, familiar to users
- **Cons**: Implies dynamic health checking, may cause confusion with service status
- **Decision**: Not chosen - `show` is clearer for information display

#### Option 2: Include remote verification by default

- **Pros**: More accurate real-time status
- **Cons**: Slower, can fail on network issues, adds complexity, duplicates the `test` command functionality
- **Decision**: Not chosen - infrastructure verification is the responsibility of the `test` command

#### Option 3: Support multiple formats from the start

- **Pros**: Maximum flexibility immediately
- **Cons**: More implementation work, delays delivery
- **Decision**: Not chosen - implement human-friendly output first, add JSON later

## 🔧 Implementation Details

### Architecture Changes

No major architectural changes required. This follows the existing command pattern:

- New application layer command handler
- New presentation layer console subcommand
- Reuses existing environment loading infrastructure

### Component Design

#### Component 1: ShowCommandHandler

**Purpose**: Application layer command handler that loads environment and formats output

**Location**: `src/application/commands/show/mod.rs`

**Interface**:

```rust
use crate::domain::environment::{Environment, EnvironmentName};
use crate::infrastructure::persistence::EnvironmentLoader;
use crate::presentation::views::UserOutput;
use std::sync::Arc;

pub struct ShowCommandHandler {
    environment_loader: Arc<dyn EnvironmentLoader>,
}

impl ShowCommandHandler {
    pub fn new(environment_loader: Arc<dyn EnvironmentLoader>) -> Self {
        Self { environment_loader }
    }

    /// Execute the show command for the given environment name
    pub async fn execute(
        &self,
        environment_name: &EnvironmentName,
        user_output: &Arc<UserOutput>,
    ) -> Result<(), ShowCommandError> {
        // 1. Load environment (any state)
        let environment = self.environment_loader.load(environment_name)?;

        // 2. Extract information based on state
        let info = self.extract_info(&environment);

        // 3. Format and display via UserOutput
        self.display_info(&info, user_output);

        Ok(())
    }

    fn extract_info(&self, environment: &Environment) -> EnvironmentInfo {
        // Extract state-specific information
        todo!()
    }

    fn display_info(&self, info: &EnvironmentInfo, user_output: &Arc<UserOutput>) {
        // Format and display information
        todo!()
    }
}
```

**Dependencies**: EnvironmentLoader, UserOutput

#### Component 2: Console Subcommand

**Purpose**: Presentation layer CLI interface for the show command

**Location**: `src/presentation/input/cli/subcommands/show.rs`

**Interface**:

```rust
use clap::Args;

#[derive(Args)]
pub struct ShowCommand {
    /// Name of the environment to show
    #[arg(value_name = "ENVIRONMENT")]
    pub environment: String,

    // Future options
    // #[arg(long, value_name = "FORMAT")]
    // pub format: Option<OutputFormat>,
    //
    // #[arg(long)]
    // pub check_remote: bool,
}
```

**Integration Point**: Add to main CLI enum in `src/presentation/input/cli/commands.rs`

#### Component 3: EnvironmentInfo DTO

**Purpose**: Data transfer object for environment information

**Location**: `src/application/commands/show/info.rs`

**Interface**:

```rust
use crate::domain::environment::EnvironmentState;
use std::net::IpAddr;

pub struct EnvironmentInfo {
    pub name: String,
    pub state: EnvironmentState,
    pub provider: String,
    pub created_at: Option<String>,
    pub infrastructure: Option<InfrastructureInfo>,
    pub services: Option<ServicesInfo>,
    pub next_step: String,
}

pub struct InfrastructureInfo {
    pub instance_ip: IpAddr,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub ssh_key_path: String,
}

pub struct ServicesInfo {
    pub udp_trackers: Vec<url::Url>,
    pub http_tracker: Option<url::Url>,
    pub http_tracker_health: Option<url::Url>,
    pub api_endpoint: Option<url::Url>,
    pub api_health: Option<url::Url>,
}
```

### Data Model

No new persistent data structures required. The command reads existing environment JSON:

```json
{
  "environment": {
    "name": "my-environment",
    "state": "Provisioned",
    "provider_config": { ... },
    "runtime_outputs": {
      "instance_ip": "10.140.190.14"
    }
  }
}
```

### API Changes

New public API in application layer:

```rust
// src/application/commands/mod.rs
pub mod show;
pub use show::ShowCommandHandler;
```

New console subcommand:

```rust
// src/presentation/input/cli/commands.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    Show(ShowCommand),
}
```

### Configuration

No new configuration required. Command uses existing environment data directory structure.

## 📊 Impact Analysis

### Files to Create

| File Path                                        | Purpose                    | Effort |
| ------------------------------------------------ | -------------------------- | ------ |
| `src/application/commands/show/mod.rs`           | ShowCommandHandler         | Medium |
| `src/application/commands/show/handler.rs`       | Command execution logic    | Medium |
| `src/application/commands/show/info.rs`          | EnvironmentInfo DTOs       | Low    |
| `src/application/commands/show/formatter.rs`     | Output formatting logic    | Medium |
| `src/application/commands/show/error.rs`         | Command-specific errors    | Low    |
| `src/presentation/input/cli/subcommands/show.rs` | CLI subcommand definition  | Low    |
| `tests/e2e/commands/show_command.rs`             | E2E tests for show command | Medium |
| `docs/user-guide/commands/show.md`               | User documentation         | Low    |

### Files to Modify

| File Path                                | Changes Required                  | Effort |
| ---------------------------------------- | --------------------------------- | ------ |
| `src/application/commands/mod.rs`        | Export show command module        | Low    |
| `src/presentation/input/cli/commands.rs` | Add Show variant to Commands enum | Low    |
| `src/presentation/dispatch/mod.rs`       | Add show command dispatch         | Low    |
| `docs/console-commands.md`               | Add show command documentation    | Low    |
| `docs/user-guide/commands.md`            | Add command reference             | Low    |
| `docs/roadmap.md`                        | Mark task 5.1 as complete         | Low    |

### Breaking Changes

None. This is a new command with no impact on existing functionality.

### Performance Impact

- **Positive**: Fast read-only operation, no network calls (initially)
- **Neutral**: Performance comparable to other read-only operations
- **Impact**: Minimal - just file read and output formatting

### Security Considerations

- Read-only operation - no security risks
- Displays IP addresses and SSH information - user should have access to this
- No credential exposure - SSH keys not displayed
- Environment data already stored locally with file permissions

## 🗓️ Implementation Plan

### Implementation Strategy

**Top-Down Approach**: Implement from presentation layer down to domain layer for iterative, testable development.

**Why Top-Down**:

- Command is runnable from the very beginning
- E2E tests can be written early and validate incrementally
- Features appear progressively in output as implementation progresses
- Faster feedback loop - see results immediately
- Natural workflow matches user interaction

**Development Flow**:

1. **Presentation Layer** - CLI subcommand skeleton (shows "Not implemented")
2. **Application Layer** - Command handler skeleton with error handling (shows basic info)
3. **Add state-specific details** - Progressively add Provisioned, Configured, Running state info using existing environment data (with error handling for missing data)
4. **Refine formatting** - Improve output appearance and user experience
5. **Testing and documentation** - Comprehensive testing and user guides
6. **Extend internal state** - Add new fields to domain model (creation timestamp, service URLs)
7. **Enhanced display** - Show newly available information

Each step produces a working, testable command with incrementally more features. Error handling is integrated at each phase rather than being deferred. Start with existing data, then extend what we store to display more information.

### Phase 1: Presentation Layer - CLI Skeleton

- [ ] Create CLI subcommand definition in `src/presentation/input/cli/subcommands/show.rs`
- [ ] Add Show variant to Commands enum in `src/presentation/input/cli/commands.rs`
- [ ] Add basic command dispatch in `src/presentation/dispatch/mod.rs`
- [ ] Add placeholder handler that returns "Not implemented" message
- [ ] Verify command appears in `--help` output
- [ ] Create initial E2E test in `tests/e2e/commands/show_command.rs` (create environment, run show command)
- [ ] Manual CLI testing - command is runnable

**Estimated Duration**: 1-2 hours

**Result**: Command is runnable from the CLI, shows "Not implemented" placeholder message. E2E test validates basic command execution.

**Testing**: E2E test will evolve throughout implementation as we add more features. Initially validates command doesn't crash and produces output.

### Phase 2: Application Layer Foundation

- [ ] Create `ShowCommandHandler` structure in `src/application/commands/show/mod.rs`
- [ ] Create `ShowCommandError` enum in `src/application/commands/show/error.rs`
- [ ] Implement environment loading logic with error handling
- [ ] Handle environment not found errors with clear messages
- [ ] Create `EnvironmentInfo` DTO in `src/application/commands/show/info.rs`
- [ ] Add basic state extraction logic (name, state, provider)
- [ ] Display basic information via UserOutput
- [ ] Add comprehensive unit tests for command handler (including error scenarios)
- [ ] Update E2E test to validate basic info display (name, state, provider)

**Estimated Duration**: 2-3 hours

**Result**: Command displays environment name, state, and provider for all environments.

**Testing**: Unit tests cover handler logic, error scenarios, and DTO creation. E2E test validates end-to-end basic info display.

### Phase 3: State-Aware Information Extraction

- [ ] Implement state-specific info extraction using existing environment data
- [ ] Add infrastructure details (IP, SSH user, SSH port) for Provisioned state from runtime_outputs
- [ ] Handle missing runtime_outputs gracefully with clear error messages
- [ ] Extract service configuration for Running state from tracker config
- [ ] Handle all environment states (Created, Configured, Released, etc.)
- [ ] Handle invalid or corrupted environment state data
- [ ] Add comprehensive unit tests for info extraction (including edge cases and missing data)
- [ ] Update E2E test to validate state-specific details

**Estimated Duration**: 3-4 hours

**Note**: This phase uses only data already stored in environment JSON - no new fields added yet.

**Testing**: Unit tests cover all state transitions, missing data scenarios, and edge cases. E2E test validates state-specific information extraction.

### Phase 4: Output Formatting

- [ ] Implement output formatter with UserOutput
- [ ] Add state-aware formatting
- [ ] Include next-step guidance
- [ ] Add visual improvements (colors, spacing)
- [ ] Add unit tests for output formatting logic
- [ ] Update E2E test to validate output formatting and next-step guidance
- [ ] Manual testing of output on terminal

**Estimated Duration**: 2-3 hours

**Testing**: Unit tests cover formatting logic for all states and visual elements. E2E test validates complete formatted output.

### Phase 5: Testing Strategy Analysis and Documentation

- [ ] Analyze and decide on E2E testing strategy for different environment states:
  - **Strategy 1**: Call show command in existing E2E workflow tests (`e2e_complete_workflow_tests`, `e2e_deployment_workflow_tests`, `e2e_infrastructure_lifecycle_tests`) after each state transition
  - **Strategy 2**: Mock internal state in dedicated E2E test before running show command
  - **Strategy 3**: Test different states only via unit tests for message building logic (decoupled from internal state)
- [ ] Implement chosen testing strategy
- [ ] Add additional E2E tests for error scenarios (non-existent environment, corrupted data)
- [ ] Verify test coverage meets requirements (unit + E2E)
- [ ] Write user documentation in `docs/user-guide/commands/show.md`
- [ ] Update `docs/console-commands.md` with show command documentation
- [ ] Update `docs/user-guide/commands.md` with command reference
- [ ] Update `docs/roadmap.md` to mark task 5.1 as complete

**Estimated Duration**: 2-3 hours

**Testing Decision Factors**:

- **Strategy 1**: Most realistic, tests actual state transitions, but requires running through full workflows (slower, more complex)
- **Strategy 2**: More isolated, faster, but requires mocking infrastructure (may not catch integration issues)
- **Strategy 3**: Fastest, best for unit testing message logic, but doesn't validate end-to-end state reading

**Recommended Approach**: Combination of strategies - Strategy 3 for message formatting unit tests, Strategy 1 for realistic E2E validation in existing workflow tests.

### Phase 6: Add Creation Timestamp

- [ ] Add `created_at` field to Environment domain model
- [ ] Update environment JSON serialization to include timestamp
- [ ] Update `create` command to populate creation timestamp
- [ ] Update `show` command to display creation timestamp
- [ ] Add unit tests for timestamp persistence and display
- [ ] Update E2E test to validate timestamp display

**Estimated Duration**: 1-2 hours

**Note**: This extends the internal state with new information. Since this is early development with no production users, no backward compatibility or migration logic is needed.

**Testing**: Unit tests cover timestamp creation, serialization, and display. E2E test validates timestamp appears in output.

### Phase 7: Add Service URLs to RuntimeOutputs

- [ ] Add `service_endpoints` field to RuntimeOutputs domain model
- [ ] Define ServiceEndpoints struct with URL fields and health check endpoints
- [ ] Update `run` command to populate service URLs after successful startup
- [ ] Update `show` command to read URLs from RuntimeOutputs (with fallback to construction)
- [ ] Add SSH key path to infrastructure info
- [ ] Add comprehensive unit tests for service endpoints persistence and display
- [ ] Update E2E test to validate service URLs display

**Estimated Duration**: 2-3 hours

**Note**: This enhancement makes service URLs a first-class part of the deployment state, available to all commands without rebuilding.

**Testing**: Unit tests cover ServiceEndpoints creation, serialization, URL construction, and fallback logic. E2E test validates service URLs appear in output for Running state.

**Testing**: Unit tests cover ServiceEndpoints creation, serialization, URL construction, and fallback logic. E2E test validates service URLs appear in output for Running state.

**Total Estimated Duration**: 14-22 hours (2-3 days) for complete implementation including enhancements

**Notes**:

- Error handling is integrated throughout all phases rather than being a separate phase, ensuring robust error management at each development step.
- Unit tests are written incrementally at each phase for code added or modified in that phase.
- E2E test is created in Phase 1 and evolves throughout implementation, providing continuous validation of the feature.
- Testing strategy for different environment states is analyzed and decided in Phase 5.

## ✅ Definition of Done

### Functional Requirements

- [ ] Command displays environment name and state for all state types
- [ ] Command shows IP address, SSH port, SSH user, and SSH key path for Provisioned environments
- [ ] Command includes ready-to-use SSH connection command for Provisioned environments
- [ ] Command shows service URLs for Running environments
- [ ] Command provides next-step guidance based on state
- [ ] Command handles non-existent environments with clear error message
- [ ] Command handles invalid or corrupted environment data gracefully
- [ ] Output is human-friendly and easy to read
- [ ] Command execution is fast (< 100ms for typical environment)

### Technical Requirements

- [ ] Code follows project conventions and style guidelines
- [ ] All linters pass (clippy, rustfmt, shellcheck, etc.)
- [ ] No compiler warnings
- [ ] Uses UserOutput for all output (no println! or eprintln!)
- [ ] Follows DDD layer placement (Application and Presentation layers)
- [ ] Error types use explicit enums (not anyhow)
- [ ] Performance is acceptable (< 100ms)

### Testing Requirements

- [ ] Unit tests cover ShowCommandHandler logic
- [ ] Unit tests cover info extraction for all states
- [ ] Unit tests cover output formatting
- [ ] Unit tests cover error scenarios
- [ ] E2E test validates command with Created environment
- [ ] E2E test validates command with Provisioned environment
- [ ] E2E test validates command with Running environment
- [ ] E2E test validates error handling for non-existent environment
- [ ] Edge cases tested (missing data, invalid state, etc.)

### Documentation Requirements

- [ ] `docs/features/environment-status-command/` - Feature documentation complete
- [ ] `docs/console-commands.md` - Show command section added
- [ ] `docs/user-guide/commands.md` - Show command reference added
- [ ] `docs/user-guide/commands/show.md` - Detailed command guide created
- [ ] `docs/roadmap.md` - Task 5.1 marked as complete
- [ ] Inline code documentation (doc comments)
- [ ] Command help text in CLI (--help)

### Code Quality

- [ ] No code duplication (DRY principle)
- [ ] Clear separation of concerns (formatting vs logic)
- [ ] Meaningful variable and function names
- [ ] Proper error context and messages
- [ ] Follows module organization conventions

## 🔮 Future Enhancements

### JSON Output Format

Add support for machine-readable JSON output:

```bash
torrust-tracker-deployer show my-environment --format json
```

Output:

```json
{
  "name": "my-environment",
  "state": "Running",
  "provider": "hetzner",
  "infrastructure": {
    "ip_address": "157.10.23.45",
    "ssh_port": 22,
    "ssh_user": "root"
  },
  "services": {
    "udp_trackers": ["udp://157.10.23.45:6868", "udp://157.10.23.45:6969"],
    "http_tracker": "http://157.10.23.45:7070",
    "api_endpoint": "http://157.10.23.45:1212/api"
  },
  "created_at": "2025-12-16T10:30:00Z",
  "updated_at": "2025-12-16T14:22:30Z"
}
```

## 📝 Implementation Notes

### UserOutput Usage

Follow output handling guidelines in `docs/contributing/output-handling.md`:

```rust
// ✓ Correct
user_output.lock().borrow_mut().info("Environment: my-environment");
user_output.lock().borrow_mut().success("State: Running");

// ✗ Wrong - NEVER use these
println!("Environment: my-environment");
eprintln!("Error loading environment");
```

### Error Handling

Follow error handling principles in `docs/contributing/error-handling.md`:

```rust
// Define explicit error types
#[derive(Debug, thiserror::Error)]
pub enum ShowCommandError {
    #[error("Environment '{0}' not found")]
    EnvironmentNotFound(EnvironmentName),

    #[error("Failed to load environment: {0}")]
    LoadError(#[from] EnvironmentLoadError),

    #[error("Invalid environment state: {0}")]
    InvalidState(String),
}
```

### Module Organization

Follow module organization conventions in `docs/contributing/module-organization.md`:

- Public items first
- High-level abstractions before implementation details
- Imports at the top of file
- Prefer imports over full paths

### Testing Conventions

Follow unit testing conventions in `docs/contributing/testing/unit-testing.md`:

- Use behavior-driven naming: `it_should_display_ip_when_environment_is_provisioned()`
- Never use `test_` prefix
- Follow What-When-Then structure

### Service URLs in RuntimeOutputs

**Implementation Note**: Currently, service URLs are constructed on-demand from environment configuration. For better performance and consistency, consider updating the `RuntimeOutputs` domain model (in `src/domain/environment/runtime_outputs.rs`) to store service URLs after the `run` command successfully starts services.

**Benefits**:

- URLs available to all commands without rebuilding
- Ensures consistency across commands
- Single source of truth for deployed service endpoints
- Aligns with the RuntimeOutputs purpose (data generated during deployment)

**Proposed RuntimeOutputs Extension**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    pub udp_trackers: Vec<url::Url>,
    pub http_tracker: Option<url::Url>,
    pub http_tracker_health: Option<url::Url>,
    pub api_endpoint: Option<url::Url>,
    pub api_health: Option<url::Url>,
}

pub struct RuntimeOutputs {
    pub instance_ip: Option<IpAddr>,
    pub provision_method: Option<ProvisionMethod>,
    // Add after run command succeeds:
    pub service_endpoints: Option<ServiceEndpoints>,
}
```

**Implementation Strategy**:

1. Update `RuntimeOutputs` with `service_endpoints` field
2. Modify `run` command to populate URLs after successful service startup
3. Update `show` command to read from `RuntimeOutputs` when available
4. Fall back to constructing URLs if `service_endpoints` is `None` (for backward compatibility)

This enhancement can be implemented as part of this feature or as a separate preliminary task.
