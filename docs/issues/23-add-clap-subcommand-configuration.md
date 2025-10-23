# Add Clap Subcommand Configuration

**GitHub Issue**: [#23](https://github.com/torrust/torrust-tracker-deployer/issues/23)  
**Type**: Task  
**Priority**: High  
**Parent Epic**: #10 ([`10-epic-ui-layer-destroy-command.md`](./10-epic-ui-layer-destroy-command.md))
**Dependencies**: Issue #21 ([`21-fix-e2e-infrastructure-preservation.md`](./21-fix-e2e-infrastructure-preservation.md)) - **MUST BE COMPLETED FIRST**
**Related**: [Epic #9: App Layer Destroy Command](https://github.com/torrust/torrust-tracker-deployer/issues/9), Issue 10.2: Rename App Commands to Command Handlers
**Estimated Effort**: 3-4 hours

## Overview

Implement the `destroy` subcommand in the CLI with basic functionality and UserOutput scaffolding. This creates the user-facing interface that calls the DestroyCommandHandler from the application layer.

This task establishes the foundation for user-friendly CLI interactions by introducing the UserOutput system with verbosity levels, while keeping the implementation focused on MVP functionality.

## 🔗 Dependencies

**Blocking Dependency**: Issue #21 ([Fix E2E Infrastructure Preservation](./21-fix-e2e-infrastructure-preservation.md))

This issue **CANNOT** be implemented until Issue 10.1 is completed because:

1. **Manual Testing Required**: The destroy CLI command must be manually tested to ensure it works correctly
2. **E2E Infrastructure Needed**: Manual testing requires preserving E2E infrastructure using `--keep` flag
3. **Verification Workflow**: Developers need to:
   - Run E2E tests with `--keep` to provision infrastructure
   - Test the destroy CLI command on that preserved infrastructure
   - Verify complete cleanup using LXD commands (`lxc list | grep e2e-provision`)

**Without Issue 10.1 completed**, the manual testing workflow is broken and this destroy CLI implementation cannot be properly validated.

## Goals

- [ ] Add `destroy` subcommand to Clap configuration
- [ ] Wire up subcommand to call `DestroyCommandHandler` from application layer
- [ ] Implement `VerbosityLevel` enum and `UserOutput` struct
- [ ] Add essential progress messages using `UserOutput`
- [ ] Ensure separation between user output and internal logging

## Specifications

### Clap Subcommand Structure

Add to `src/app.rs`:

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Destroy an existing deployment environment
    Destroy {
        /// Name of the environment to destroy
        environment: String,
    },
    // Other commands (Provision, Configure) will be added in future issues
}
```

### VerbosityLevel Implementation

Create `src/presentation/user_output.rs`:

```rust
/// Verbosity levels for user output
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    /// Minimal output - only errors and final results
    Quiet,
    /// Default level - essential progress and results
    Normal,
    /// Detailed progress including intermediate steps
    Verbose,
    /// Very detailed including decisions and retries
    VeryVerbose,
    /// Maximum detail for troubleshooting
    Debug,
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        Self::Normal
    }
}
```

### Output Channel Strategy

Based on [console output patterns research](../research/UX/console-app-output-patterns.md), we follow the **Progress/Results Separation** pattern used by modern CLI tools (cargo, docker, npm):

#### Channel Usage for Torrust Deployer

1. **stdout (Results Channel)**:

   - Deployment results (instance IPs, configuration summaries)
   - Final status reports
   - Structured data (JSON output for piping)
   - Data that users want to pipe/redirect

2. **stderr (Progress/Operational Channel)**:
   - Step progress ("Provisioning instance...", "Tearing down infrastructure...")
   - Status updates ("Instance ready", "Environment destroyed successfully")
   - Non-critical warnings
   - Error messages with actionable guidance

#### Benefits of This Approach

- **Clean Piping**: `torrust-tracker-deployer destroy env | jq .status` works correctly
- **Automation Friendly**: Scripts can redirect progress to /dev/null while capturing results
- **Unix Convention**: Follows established patterns from cargo, docker, and other modern tools
- **User Experience**: Progress feedback doesn't interfere with result data

### UserOutput Implementation

- **stdout**: Final results, data that users want to pipe/redirect
- **stderr**: Progress updates, status messages, operational info, errors

```rust
use std::io::Write;

/// Handles user-facing output separate from internal logging
/// Uses dual channels following Unix conventions and modern CLI best practices
pub struct UserOutput {
    verbosity: VerbosityLevel,
    stdout_writer: Box<dyn Write + Send + Sync>,
    stderr_writer: Box<dyn Write + Send + Sync>,
}

impl UserOutput {
    /// Create new UserOutput with default stdout/stderr channels
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self {
            verbosity,
            stdout_writer: Box::new(std::io::stdout()),
            stderr_writer: Box::new(std::io::stderr()),
        }
    }

    /// Create UserOutput for testing with custom writers
    pub fn with_writers(
        verbosity: VerbosityLevel,
        stdout_writer: Box<dyn Write + Send + Sync>,
        stderr_writer: Box<dyn Write + Send + Sync>
    ) -> Self {
        Self { verbosity, stdout_writer, stderr_writer }
    }

    /// Display progress message to stderr (Normal level and above)
    /// Progress messages go to stderr following cargo/docker patterns
    pub fn progress(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            writeln!(self.stderr_writer, "⏳ {}", message).unwrap_or(());
        }
    }

    /// Display success message to stderr (Normal level and above)
    /// Success status goes to stderr to allow clean result piping
    pub fn success(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            writeln!(self.stderr_writer, "✅ {}", message).unwrap_or(());
        }
    }

    /// Display warning message to stderr (Normal level and above)
    pub fn warn(&mut self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            writeln!(self.stderr_writer, "⚠️  {}", message).unwrap_or(());
        }
    }

    /// Display error message to stderr (all levels)
    pub fn error(&mut self, message: &str) {
        writeln!(self.stderr_writer, "❌ {}", message).unwrap_or(());
    }

    /// Output final results to stdout for piping/redirection
    /// This is where deployment results, configuration summaries, etc. go
    pub fn result(&mut self, message: &str) {
        writeln!(self.stdout_writer, "{}", message).unwrap_or(());
    }

    /// Output structured data to stdout (JSON, etc.)
    pub fn data(&mut self, data: &str) {
        writeln!(self.stdout_writer, "{}", data).unwrap_or(());
    }
}
```

### Unit Testing Strategy

#### Testing UserOutput with Buffer Writers

Use buffer writers to capture and assert output in tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_user_output(verbosity: VerbosityLevel) -> (UserOutput, Vec<u8>, Vec<u8>) {
        let stdout_buffer = Vec::new();
        let stderr_buffer = Vec::new();

        let stdout_writer = Box::new(Cursor::new(stdout_buffer.clone()));
        let stderr_writer = Box::new(Cursor::new(stderr_buffer.clone()));

        let output = UserOutput::with_writers(verbosity, stdout_writer, stderr_writer);

        (output, stdout_buffer, stderr_buffer)
    }

    #[test]
    fn it_should_write_progress_messages_to_stderr() {
        let (mut output, stdout_buf, stderr_buf) = create_test_user_output(VerbosityLevel::Normal);

        output.progress("Testing progress message");

        // Verify message went to stderr
        let stderr_content = String::from_utf8(stderr_buf).unwrap();
        assert_eq!(stderr_content, "⏳ Testing progress message\n");

        // Verify stdout is empty
        let stdout_content = String::from_utf8(stdout_buf).unwrap();
        assert_eq!(stdout_content, "");
    }

    #[test]
    fn it_should_write_results_to_stdout() {
        let (mut output, stdout_buf, stderr_buf) = create_test_user_output(VerbosityLevel::Normal);

        output.result("Test result data");

        // Verify message went to stdout
        let stdout_content = String::from_utf8(stdout_buf).unwrap();
        assert_eq!(stdout_content, "Test result data\n");

        // Verify stderr is empty
        let stderr_content = String::from_utf8(stderr_buf).unwrap();
        assert_eq!(stderr_content, "");
    }

    #[test]
    fn it_should_respect_verbosity_levels() {
        let (mut output, _, stderr_buf) = create_test_user_output(VerbosityLevel::Quiet);

        output.progress("This should not appear");

        // Verify no output at Quiet level
        let stderr_content = String::from_utf8(stderr_buf).unwrap();
        assert_eq!(stderr_content, "");
    }

    #[test]
    fn it_should_always_show_errors_regardless_of_verbosity() {
        let (mut output, _, stderr_buf) = create_test_user_output(VerbosityLevel::Quiet);

        output.error("Critical error message");

        // Verify error appears even at Quiet level
        let stderr_content = String::from_utf8(stderr_buf).unwrap();
        assert_eq!(stderr_content, "❌ Critical error message\n");
    }
}
```

#### Testing CLI Integration

Test the CLI argument parsing and integration:

```rust
#[cfg(test)]
mod cli_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn it_should_parse_destroy_subcommand() {
        let args = vec!["torrust-tracker-deployer", "destroy", "test-env"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Destroy { environment } => {
                assert_eq!(environment, "test-env");
            }
        }
    }

    #[test]
    fn it_should_require_environment_parameter() {
        let args = vec!["torrust-tracker-deployer", "destroy"];
        let result = Cli::try_parse_from(args);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("required"));
    }
}
```

### Command Handler Integration

Update command execution in `src/app.rs`:

```rust
async fn handle_destroy_command(environment: String) -> anyhow::Result<()> {
    use crate::application::command_handlers::destroy::DestroyCommandHandler;
    use crate::presentation::user_output::{UserOutput, VerbosityLevel};

    // Create UserOutput with default stdout/stderr channels
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    // Display initial progress (to stderr)
    output.progress(&format!("Destroying environment '{}'...", environment));

    // Create environment (simplified for MVP)
    let env_name = EnvironmentName::new(environment.clone())
        .map_err(|e| anyhow::anyhow!("Invalid environment name '{}': {}", environment, e))?;

    // Note: This is simplified - full implementation will use dependency injection
    let environment = Environment::new_with_default_paths(env_name, create_default_ssh_credentials()?);

    // Execute destroy command handler
    output.progress("Tearing down infrastructure...");

    let command_handler = create_destroy_command_handler().await?;
    let result = command_handler.execute(environment).await;

    match result {
        Ok(_destroyed_env) => {
            output.progress("Cleaning up resources...");
            output.success(&format!("Environment '{}' destroyed successfully", environment));

            // Future enhancement: Output structured result data to stdout
            // output.result(&format!(r#"{{"environment": "{}", "status": "destroyed"}}"#, environment));

            Ok(())
        }
        Err(e) => {
            output.error(&format!("Failed to destroy environment '{}': {}", environment, e));
            Err(anyhow::anyhow!("Destroy command failed: {}", e))
        }
    }
}
```

### Example Usage and Output

**Command:**

```bash
torrust-tracker-deployer destroy my-env
```

**Expected Output (Normal verbosity):**

**To stderr (progress/status):**

```text
⏳ Destroying environment 'my-env'...
⏳ Tearing down infrastructure...
⏳ Cleaning up resources...
✅ Environment 'my-env' destroyed successfully
```

**To stdout (results - for future enhancement):**

```text
(Empty for basic destroy - future versions may output summary data)
```

**Example with piping (future enhancement):**

```bash
# Progress goes to stderr, results to stdout - clean for automation
torrust-tracker-deployer destroy my-env > /dev/null   # Suppress results, show progress
torrust-tracker-deployer destroy my-env 2>/dev/null  # Suppress progress, show results
```

**Parallel Internal Logging (always present in log files):**

```text
2025-10-21T10:15:00.123Z INFO destroy_command_handler: Starting environment destruction
    environment="my-env" command_type="destroy"
2025-10-21T10:15:01.456Z INFO opentofu_client: Executing destroy operation
    workspace="/path/to/env" operation="destroy"
2025-10-21T10:15:15.789Z INFO destroy_command_handler: Environment destruction completed
    environment="my-env" duration=15.666s
```

### Error Handling

All errors go to stderr following Unix conventions:

```rust
// Invalid environment name (stderr)
output.error("Invalid environment name 'invalid name!': Environment names must be alphanumeric with hyphens");

// Environment not found (stderr)
output.error("Environment 'nonexistent' not found. Use 'list' command to see available environments");

// Infrastructure failure (stderr)
output.error("Failed to destroy environment 'my-env': OpenTofu destroy operation failed");
```

## Implementation Plan

### Subtask 1: Implement VerbosityLevel and UserOutput (1.5 hours)

- [x] Create `src/presentation/user_output.rs` (moved from shared to presentation layer)
- [x] Implement `VerbosityLevel` enum with 5 levels
- [x] Implement `UserOutput` struct with dual writers (stdout/stderr)
- [x] Add methods: `progress`, `success`, `warn`, `error` (stderr), `result`, `data` (stdout)
- [x] Add constructor for default channels and testing with custom writers
- [x] Document channel usage strategy based on console patterns research
- [x] Update `src/presentation/mod.rs` to export new module

### Subtask 2: Add Destroy Subcommand to Clap (45 minutes)

- [ ] Add `Destroy` variant to `Commands` enum in `src/app.rs`
- [ ] Add `environment` parameter as required String
- [ ] Add help text and description for the subcommand
- [ ] Update match statement to handle `Commands::Destroy` case

### Subtask 3: Implement Command Handler Integration (1.5 hours)

- [ ] Create `handle_destroy_command` function in `src/app.rs`
- [ ] Integrate with `DestroyCommandHandler` from application layer
- [ ] Add environment name validation
- [ ] Add basic error handling with user-friendly messages
- [ ] Use `UserOutput` for progress messages

### Subtask 4: Add Unit Tests (1.5 hours)

- [ ] Test CLI argument parsing for destroy subcommand
- [ ] Test `VerbosityLevel` enum ordering and default
- [ ] Test `UserOutput` methods with buffer writers for output capture
- [ ] Test proper channel separation (stdout vs stderr) using separate buffers
- [ ] Test error handling for invalid environment names
- [ ] Test integration between CLI and command handler
- [ ] Verify channel usage matches documented strategy

### Subtask 5: Documentation and Integration (30 minutes)

- [ ] Update help text and CLI documentation
- [ ] Add code examples in rustdoc comments
- [ ] Ensure all imports are properly organized
- [ ] Run linting and fix any issues

## Acceptance Criteria

**Prerequisites**: Issue 10.1 (Fix E2E Infrastructure Preservation) must be completed before this can be fully tested and accepted.

- [ ] `destroy` subcommand added to Clap configuration
- [ ] Subcommand accepts required `environment` parameter
- [ ] `VerbosityLevel` enum implemented with 5 levels (Quiet, Normal, Verbose, VeryVerbose, Debug)
- [ ] `UserOutput` struct implemented with dual writers and proper channel separation
- [ ] Channel strategy follows console output patterns research (progress to stderr, results to stdout)
- [ ] Essential destroy messages implemented using appropriate channels
- [ ] Subcommand successfully calls `DestroyCommandHandler` from application layer
- [ ] User-friendly error messages for invalid environment names
- [ ] Help text provides clear usage information
- [ ] Unit tests implemented with buffer writers for output capture and assertion
- [ ] Unit tests verify proper channel separation (stdout vs stderr)
- [ ] Unit tests cover CLI argument parsing and error handling
- [ ] Manual testing completed using E2E test infrastructure (--keep flag)
- [ ] Manual testing verifies complete infrastructure, data, and build cleanup
- [ ] User output completely separated from internal logging
- [ ] All linting passes successfully
- [ ] Integration works end-to-end with existing command handler

## Related Documentation

- [Console Output Patterns Research](../research/UX/console-app-output-patterns.md) - Channel usage strategy
- [User Output vs Logging Separation](../research/UX/user-output-vs-logging-separation.md)
- [Application Commands](../codebase-architecture.md#application-layer)
- [CLI Design Principles](../development-principles.md)
- [Epic #10](./10-epic-ui-layer-destroy-command.md) - Parent epic context

## Notes

**Estimated Time**: 5-5.5 hours

**Channel Strategy**: This implementation follows the **Progress/Results Separation** pattern from the console output patterns research:

- **stderr**: Progress messages, status updates, warnings, errors
- **stdout**: Final results and data for piping/redirection
- **Future**: After implementation, create an ADR documenting the channel usage strategy

**MVP Focus**: This implementation focuses on basic functionality:

- Single verbosity level (Normal) for now - no CLI verbosity flags yet
- Essential progress messages to stderr
- Basic error handling to stderr
- Clear separation between user output channels and internal logging

**Future Enhancements** (out of scope for this issue):

- Clap verbosity flags (`-v`, `-vv`, `-q`)
- Interactive confirmations
- Colored output
- Progress bars or spinners
- Advanced error recovery

**Testing Strategy**:

### Unit Testing

- Use buffer writers to capture and assert UserOutput messages
- Test CLI argument parsing with clap
- Verify channel separation (stdout/stderr)
- Test verbosity level behavior

### Manual Testing Procedure

**⚠️ Prerequisite**: Issue 10.1 ([Fix E2E Infrastructure Preservation](./fix-e2e-infrastructure-preservation.md)) must be completed first. The `--keep` flag functionality is required for this testing procedure.

#### Step 1: Set up test infrastructure using E2E tests

```bash
# Build the project
cargo build

# Run E2E provision and destroy test but keep infrastructure (don't auto-destroy)
cargo run --bin e2e_provision_and_destroy_tests -- --keep

# This creates an environment named "e2e-provision" with infrastructure ready for testing
```

#### Step 2: Test the new destroy CLI command

```bash
# Test help output
cargo run -- help destroy
cargo run -- destroy --help

# Test basic destroy command with the E2E environment
cargo run -- destroy e2e-provision

# Test invalid environment names
cargo run -- destroy invalid-name!
cargo run -- destroy ""
cargo run -- destroy "name with spaces"
```

#### Step 3: Verify complete cleanup

After running destroy, verify that:

```bash
# Check infrastructure is destroyed
lxc list  # Should not show the environment's containers

# Check data directory is removed
ls -la data/  # Should not contain environment directory

# Check build directory is removed
ls -la build/  # Should not contain environment directory

# Verify no leftover OpenTofu state
ls -la build/*/tofu/  # Should be clean or non-existent
```

**Expected Test Results**:

- Help commands show proper usage information
- Valid destroy commands show progress messages to stderr
- Invalid commands show error messages to stderr
- Infrastructure, data, and build directories are completely removed
- Commands are idempotent (safe to run multiple times)

**Key Dependencies**:

- Issue 10.1 must be completed first (command handler renaming)
- Depends on existing `DestroyCommandHandler` from Epic #9
- Uses existing `EnvironmentName` validation from domain layer
