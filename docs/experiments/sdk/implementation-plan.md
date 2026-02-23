# SDK Proof of Concept: Implementation Plan

## Objective

Build a minimal, working SDK facade that demonstrates the concept end-to-end:
a Rust consumer can create a deployment environment, query it, validate a
config file, and destroy it — all through typed Rust APIs, without touching
the CLI.

## Scope

### In scope (PoC)

- `Deployer` facade struct with builder
- Subset of operations: `create_environment`, `show`, `list`, `validate`,
  `destroy`, `purge`
- Re-exports of necessary domain and config types
- One complete example program (`examples/sdk_basic_usage.rs`)

### Out of scope (PoC)

- Operations requiring real infrastructure: `provision`, `configure`,
  `release`, `run`, `test`, `register` (these work but need LXD/SSH — the
  PoC focuses on operations testable without infrastructure)
- Custom `EnvironmentRepository` implementations
- Custom `Clock` implementations
- Stability guarantees

## Phase 1: Define Exposed Types

### 1.1 SDK Module Structure

```text
src/presentation/sdk/
├── mod.rs          # Public API surface: Deployer, DeployerBuilder, re-exports
├── deployer.rs     # Deployer facade implementation
└── builder.rs      # DeployerBuilder implementation
```

### 1.2 The `Deployer` Facade

````rust
// src/presentation/sdk/deployer.rs

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::application::command_handlers::{
    CreateCommandHandler, DestroyCommandHandler, ListCommandHandler,
    PurgeCommandHandler, ShowCommandHandler, ValidateCommandHandler,
};
use crate::application::traits::CommandProgressListener;
use crate::domain::{
    AnyEnvironmentState, Environment, EnvironmentName,
};
use crate::domain::environment::state::{Created, Destroyed};
use crate::infrastructure::persistence::RepositoryFactory;
use crate::shared::Clock;

/// The main entry point for SDK consumers.
///
/// Provides typed access to all deployer operations without requiring
/// manual dependency wiring.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::presentation::sdk::Deployer;
///
/// let deployer = Deployer::builder()
///     .working_dir("/path/to/workspace")
///     .build()
///     .expect("Failed to initialize deployer");
///
/// let environments = deployer.list().expect("Failed to list environments");
/// ```
pub struct Deployer {
    working_dir: PathBuf,
    repository: Arc<dyn crate::domain::environment::repository::EnvironmentRepository + Send + Sync>,
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
    data_directory: Arc<Path>,
}
````

### 1.3 The `DeployerBuilder`

````rust
// src/presentation/sdk/builder.rs

use std::path::PathBuf;

/// Builder for constructing a [`Deployer`] with sensible defaults.
///
/// # Required
///
/// - `working_dir` — the workspace root where `data/` and `build/` live
///
/// # Optional
///
/// - `clock` — custom clock implementation (defaults to system clock)
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::presentation::sdk::Deployer;
///
/// let deployer = Deployer::builder()
///     .working_dir("/home/user/deployer-workspace")
///     .build()
///     .unwrap();
/// ```
pub struct DeployerBuilder {
    working_dir: Option<PathBuf>,
    // Future: custom clock, custom repository, progress listener, etc.
}

impl DeployerBuilder {
    pub fn new() -> Self { ... }
    pub fn working_dir(mut self, path: impl Into<PathBuf>) -> Self { ... }
    pub fn build(self) -> Result<Deployer, DeployerBuildError> { ... }
}
````

### 1.4 Deployer Methods (PoC Subset)

```rust
impl Deployer {
    /// Create a new builder.
    pub fn builder() -> DeployerBuilder;

    /// Create a new deployment environment from a configuration.
    ///
    /// Equivalent to `torrust-tracker-deployer create environment --env-file <path>`.
    pub fn create_environment(
        &self,
        config: EnvironmentCreationConfig,
    ) -> Result<Environment<Created>, CreateCommandHandlerError>;

    /// Show information about an existing environment.
    ///
    /// Equivalent to `torrust-tracker-deployer show <name>`.
    pub fn show(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<EnvironmentInfo, ShowCommandHandlerError>;

    /// List all environments in the workspace.
    ///
    /// Equivalent to `torrust-tracker-deployer list`.
    pub fn list(&self) -> Result<EnvironmentList, ListCommandHandlerError>;

    /// Validate an environment configuration file.
    ///
    /// Equivalent to `torrust-tracker-deployer validate <path>`.
    pub fn validate(
        &self,
        config_path: &Path,
    ) -> Result<ValidationResult, ValidateCommandHandlerError>;

    /// Destroy the infrastructure for an environment.
    ///
    /// Equivalent to `torrust-tracker-deployer destroy <name>`.
    pub fn destroy(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Destroyed>, DestroyCommandHandlerError>;

    /// Purge all local data for an environment.
    ///
    /// Equivalent to `torrust-tracker-deployer purge <name>`.
    pub fn purge(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<(), PurgeCommandHandlerError>;
}
```

### 1.5 Re-exported Types

The SDK module will re-export the types that consumers need, organized by
purpose:

```rust
// src/presentation/sdk/mod.rs

// === Core facade ===
pub use self::deployer::Deployer;
pub use self::builder::{DeployerBuilder, DeployerBuildError};

// === Domain types (inputs/outputs) ===
pub use crate::domain::{
    Environment,
    EnvironmentName,
    EnvironmentNameError,
    AnyEnvironmentState,
    InstanceName,
    ProfileName,
    Provider,
    ProviderConfig,
    LxdConfig,
    HetznerConfig,
    BackupConfig,
};

// === Environment states ===
pub use crate::domain::environment::state::{
    Created,
    Provisioned,
    Configured,
    Released,
    Running,
    Destroyed,
};

// === Configuration types (for create_environment) ===
pub use crate::application::command_handlers::create::config::EnvironmentCreationConfig;

// === Result types ===
pub use crate::application::command_handlers::show::EnvironmentInfo;
pub use crate::application::command_handlers::list::EnvironmentList;
pub use crate::application::command_handlers::validate::ValidationResult;

// === Error types ===
pub use crate::application::command_handlers::create::CreateCommandHandlerError;
pub use crate::application::command_handlers::destroy::DestroyCommandHandlerError;
pub use crate::application::command_handlers::show::ShowCommandHandlerError;
pub use crate::application::command_handlers::list::ListCommandHandlerError;
pub use crate::application::command_handlers::validate::ValidateCommandHandlerError;
pub use crate::application::command_handlers::purge::PurgeCommandHandlerError;

// === Extension points ===
pub use crate::application::traits::{CommandProgressListener, NullProgressListener};
```

## Phase 2: Implement the Facade

### 2.1 Tasks

| #   | Task                                         | Description                                      |
| --- | -------------------------------------------- | ------------------------------------------------ |
| 1   | Create `src/presentation/sdk/mod.rs`         | Module declaration with re-exports               |
| 2   | Create `src/presentation/sdk/builder.rs`     | `DeployerBuilder` with validation                |
| 3   | Create `src/presentation/sdk/deployer.rs`    | `Deployer` struct delegating to command handlers |
| 4   | Register module in `src/presentation/mod.rs` | Add `pub mod sdk;`                               |
| 5   | Verify it compiles                           | `cargo build`                                    |

### 2.2 Implementation Notes

The `Deployer` methods are thin wrappers — each one:

1. Constructs the appropriate command handler (injecting `self.repository`,
   `self.clock`, etc.)
2. Calls `handler.execute(...)` with the user-provided arguments
3. Returns the result directly — no transformation

Example implementation sketch:

```rust
impl Deployer {
    pub fn create_environment(
        &self,
        config: EnvironmentCreationConfig,
    ) -> Result<Environment<Created>, CreateCommandHandlerError> {
        let handler = CreateCommandHandler::new(
            Arc::clone(&self.repository),
            Arc::clone(&self.clock),
        );
        handler.execute(config, &self.working_dir)
    }

    pub fn show(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<EnvironmentInfo, ShowCommandHandlerError> {
        let handler = ShowCommandHandler::new(Arc::clone(&self.repository));
        handler.execute(env_name)
    }

    pub fn list(&self) -> Result<EnvironmentList, ListCommandHandlerError> {
        let handler = ListCommandHandler::new(
            Arc::clone(&self.repository_factory),
            Arc::clone(&self.data_directory),
        );
        handler.execute()
    }

    pub fn destroy(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Destroyed>, DestroyCommandHandlerError> {
        let handler = DestroyCommandHandler::new(
            Arc::clone(&self.repository),
            Arc::clone(&self.clock),
        );
        handler.execute(env_name)
    }
}
```

## Phase 3: Example Program

### 3.1 Create `examples/sdk_basic_usage.rs`

A runnable example that demonstrates the SDK API. This example does NOT
require real infrastructure — it uses `create_environment`, `show`, `list`,
and `purge` which only operate on local files.

```rust
//! Basic SDK usage example.
//!
//! Demonstrates how to use the Torrust Tracker Deployer SDK to:
//! 1. Validate an environment configuration file
//! 2. Create a deployment environment
//! 3. List all environments
//! 4. Show environment details
//! 5. Purge the environment
//!
//! Run with:
//!   cargo run --example sdk_basic_usage

use std::path::PathBuf;

use torrust_tracker_deployer_lib::presentation::sdk::{
    Deployer,
    EnvironmentCreationConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize the deployer SDK
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let deployer = Deployer::builder()
        .working_dir(&workspace)
        .build()?;

    // 2. Validate a configuration file
    let config_path = workspace.join("envs/e2e-infrastructure.json");
    match deployer.validate(&config_path) {
        Ok(result) => {
            println!("Configuration valid!");
            println!("  Environment: {}", result.environment_name);
            println!("  Provider: {}", result.provider);
            println!("  Prometheus: {}", result.has_prometheus);
            println!("  Grafana: {}", result.has_grafana);
        }
        Err(e) => {
            eprintln!("Validation failed: {e}");
            return Ok(());
        }
    }

    // 3. Load the config and create an environment
    let config_json = std::fs::read_to_string(&config_path)?;
    let config: EnvironmentCreationConfig = serde_json::from_str(&config_json)?;
    let env_name = config.environment.name.clone();

    println!("\nCreating environment '{env_name}'...");
    let environment = deployer.create_environment(config)?;
    println!("Created environment: {}", environment.name());

    // 4. List all environments
    println!("\nAll environments:");
    let env_list = deployer.list()?;
    for env in env_list.environments() {
        println!("  - {} (state: {})", env.name(), env.state());
    }

    // 5. Show environment details
    let info = deployer.show(environment.name())?;
    println!("\nEnvironment details:");
    println!("  Name: {}", info.name);
    println!("  Provider: {}", info.provider);
    println!("  State: {}", info.state);

    // 6. Clean up — purge the environment data
    println!("\nPurging environment...");
    deployer.purge(environment.name())?;
    println!("Environment purged.");

    Ok(())
}
```

### 3.2 Example: AI Agent Workflow

A more advanced example showing how an AI agent might use the SDK to build
a custom deployment pipeline with error handling and decision-making:

```rust
//! AI Agent deployment workflow example.
//!
//! Shows how an AI agent could use the SDK for conditional deployment logic.

use torrust_tracker_deployer_lib::presentation::sdk::{
    Deployer,
    EnvironmentCreationConfig,
    EnvironmentName,
};

async fn deploy_tracker(
    deployer: &Deployer,
    config: EnvironmentCreationConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let env_name_str = config.environment.name.clone();

    // Step 1: Check if environment already exists
    let env_name = EnvironmentName::new(&env_name_str)?;
    if let Ok(_info) = deployer.show(&env_name) {
        // Environment exists — agent decides to destroy and recreate
        println!("Environment exists, destroying first...");
        deployer.destroy(&env_name)?;
        deployer.purge(&env_name)?;
    }

    // Step 2: Create environment
    let created = deployer.create_environment(config)?;
    println!("Created: {}", created.name());

    // Step 3: Provision (requires real infrastructure)
    // let provisioned = deployer.provision(created.name(), None).await?;

    // Step 4: Configure
    // let configured = deployer.configure(provisioned.name(), None).await?;

    // Step 5: Release
    // let released = deployer.release(configured.name(), None).await?;

    // Step 6: Run
    // let running = deployer.run(released.name()).await?;

    Ok(format!("Deployed environment: {env_name_str}"))
}
```

## Phase 4: Validate

| Check        | Command                               | Expectation               |
| ------------ | ------------------------------------- | ------------------------- |
| Compiles     | `cargo build`                         | No errors                 |
| Tests pass   | `cargo test`                          | Existing tests unaffected |
| Example runs | `cargo run --example sdk_basic_usage` | Completes without panic   |
| Clippy clean | `cargo clippy`                        | No new warnings           |
| Docs build   | `cargo doc --no-deps`                 | SDK types documented      |

## Future Work (Post-PoC)

After the PoC validates the approach, subsequent work would include:

1. **Add async operations** — `provision`, `configure`, `release`, `run`,
   `test`, `register` methods on `Deployer` (these require `async`)
2. **Progress listener integration** — allow SDK consumers to pass
   `CommandProgressListener` implementations for real-time feedback
3. **Custom repository support** — expose `EnvironmentRepository` trait
   for alternative storage backends
4. **Separate crate** — extract into `torrust-tracker-deployer-sdk` with
   independent versioning
5. **`#[non_exhaustive]` annotations** — on public enums and structs for
   forward compatibility
6. **Integration tests** — SDK-specific tests that exercise the facade
7. **Published documentation** — `docs.rs`-ready API docs with examples
