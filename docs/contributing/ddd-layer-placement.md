# DDD Layer Placement Guide

This guide provides clear guidelines on which code belongs in which Domain-Driven Design (DDD) layer. Following these guidelines ensures proper separation of concerns, maintainability, and testability.

> **Note on Examples**: This guide uses illustrative code patterns to demonstrate layer placement principles. These are generic examples designed to show common patterns rather than exact code from the repository, making them more maintainable and easier to understand.

## 🎯 Why DDD Matters

The Torrust Tracker Deployer follows Domain-Driven Design principles to maintain clear separation between business logic, use cases, external integrations, and user interfaces. This architecture ensures:

- **Clear boundaries** - Each layer has a specific responsibility
- **Better testability** - Components can be tested in isolation
- **Easier maintenance** - Changes in one layer don't cascade to others
- **Type safety** - Compile-time guarantees for valid state transitions
- **Team collaboration** - Consistent patterns make onboarding easier

For a comprehensive overview of the architecture, see [Codebase Architecture](../codebase-architecture.md).

## 📚 Layer Overview

The project uses a four-layer architecture with strict dependency rules:

```text
┌─────────────────────────────────────┐
│     Presentation Layer              │  User Interface
│  (CLI, User Output, Command         │  - Clap command definitions
│   Dispatch, Error Display)          │  - User input parsing
│  src/presentation/                  │  - Output formatting
└────────────┬────────────────────────┘
             │ depends on
             ↓
┌─────────────────────────────────────┐
│      Application Layer              │  Use Cases & Orchestration
│  (Commands, Use Cases, Steps)       │  - Command handlers
│  src/application/                   │  - DTOs for data transfer
└────────────┬────────────────────────┘  - Application services
             │ depends on
             ↓
┌─────────────────────────────────────┐
│       Domain Layer                  │  Business Logic
│  (Business Logic, Entities,         │  - Domain entities
│   Value Objects)                    │  - Value objects
│  src/domain/                        │  - Domain traits
└─────────────────────────────────────┘
             ↑
             │ depends on
             │
┌─────────────────────────────────────┐
│    Infrastructure Layer             │  External Integrations
│  (External Tools, File System,      │  - File I/O
│   SSH, Templates, Trace Writers)    │  - SSH, HTTP clients
│  src/infrastructure/                │  - OpenTofu, Ansible
└─────────────────────────────────────┘
```

### Dependency Rule

**Dependencies flow inward toward the domain**:

- ✅ Presentation → Application → Domain
- ✅ Infrastructure → Domain
- ❌ Domain → Application (Forbidden)
- ❌ Domain → Infrastructure (Forbidden)
- ❌ Application → Presentation (Forbidden)

This ensures the domain layer remains pure business logic, free from technical implementation details.

## 🏛️ Domain Layer (`src/domain/`)

### Purpose

The domain layer contains pure business logic, entities, value objects, and domain events. It represents the core problem space and business rules without any technical implementation details.

### What Belongs Here

- ✅ **Value Objects** with validation - `EnvironmentName`, `Username`, `TraceId`
- ✅ **Domain Entities** - `Environment<S>` with type-state pattern
- ✅ **Domain Traits** - `Clock`, `EnvironmentRepository` (interfaces)
- ✅ **Business Rules** - Validation logic, domain constraints
- ✅ **Domain Events** - Events representing business occurrences
- ✅ **Serde derives on entities** - For persistence (pragmatic trade-off, see below)

### What Does NOT Belong Here

- ❌ **File I/O operations** - `std::fs`, `tokio::fs`
- ❌ **HTTP clients** - `reqwest`, `hyper`
- ❌ **External APIs** - OpenTofu, Ansible, SSH clients
- ❌ **DTOs with primitives** - `String` types meant for deserialization
- ❌ **Manual serialization** - Custom `impl Serialize` (put in infrastructure)

### Red Flags

Watch for these indicators that code might be in the wrong layer:

- Using `serde` on DTOs with raw `String` primitives (not domain entities)
- Importing `std::fs`, `tokio::fs`, `reqwest`, `hyper`
- Methods that read/write files directly
- Raw `String` types without domain semantics
- Database queries or external API calls

### Nuance: Serde on Domain Entities

Using `#[derive(Serialize, Deserialize)]` on domain entities for **persistence** is a pragmatic trade-off:

**✅ Acceptable Use:**

```rust
use serde::{Deserialize, Serialize};

/// Domain entity with serde for persistence
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentName(String);

/// Domain entity with business logic and persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment<S> {
    context: EnvironmentContext,
    state: S,
}
```

**❌ Not Acceptable:**

```rust
/// DTO masquerading as domain entity
#[derive(Serialize, Deserialize)]
pub struct ConfigDto {
    pub name: String,  // Raw primitive, no domain semantics
    pub path: String,  // Should be PathBuf or domain type
}
```

**Rationale:**

- Serde derives are code generation (external to your domain code)
- Actual serialization logic lives in the serde library
- Many Rust projects make this pragmatic choice to avoid boilerplate
- Domain entities have business logic; DTOs are just data containers

**When NOT to use serde in domain:**

- If you need **custom serialization logic**, implement it in the infrastructure layer
- Keep domain types pure and let infrastructure handle the serialization details

### Examples and Patterns

#### ✅ Pattern: Value Object with Validation

Value objects encapsulate primitives with business rules and validation:

```rust
/// Validated value object (e.g., EnvironmentName, Username, Email)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidatedName(String);

impl ValidatedName {
    /// Constructor with validation - enforces business rules
    pub fn new(value: String) -> Result<Self, ValidationError> {
        Self::validate(&value)?;
        Ok(Self(value))
    }

    fn validate(value: &str) -> Result<(), ValidationError> {
        // Business rules go here
        if value.is_empty() {
            return Err(ValidationError::Empty);
        }
        if !value.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidCharacters);
        }
        Ok(())
    }
}
```

**Why this is domain:**

- Contains business rules (validation logic)
- Uses serde for persistence (pragmatic trade-off)
- No infrastructure concerns (file I/O, HTTP, etc.)

#### ✅ Pattern: Domain Entity with Business Logic

Domain entities are core business objects with identity and lifecycle:

```rust
/// Domain entity with business logic (e.g., Environment, Deployment, Order)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    id: EntityId,
    name: ValidatedName,
    status: EntityStatus,
}

impl Entity {
    pub fn new(name: ValidatedName) -> Self {
        // Domain logic for creating entities
        Self {
            id: EntityId::generate(),
            name,
            status: EntityStatus::Created,
        }
    }

    pub fn activate(&mut self) -> Result<(), DomainError> {
        // Business rules for state transitions
        match self.status {
            EntityStatus::Created => {
                self.status = EntityStatus::Active;
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition),
        }
    }
}
```

**Why this is domain:**

- Core business object with identity
- Contains business logic and state transitions
- Serde for persistence (pragmatic choice)
- No external dependencies

#### ✅ Pattern: Repository Trait (Domain Interface)

Domain defines persistence contracts without implementation:

```rust
/// Repository interface - domain defines the contract
#[async_trait]
pub trait EntityRepository: Send + Sync {
    async fn save(&self, entity: &Entity) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<Entity>, RepositoryError>;
    async fn delete(&self, id: &EntityId) -> Result<(), RepositoryError>;
}
```

**Why this is domain:**

- Defines contract for persistence (interface only)
- No implementation details (infrastructure will implement)
- Domain stays independent of persistence technology

## 📋 Application Layer (`src/application/`)

### Purpose

The application layer orchestrates domain and infrastructure services to implement use cases. It contains command handlers, DTOs for data transfer, and application services.

### What Belongs Here

- ✅ **Command Handlers** - `ProvisionCommandHandler`, `DestroyCommandHandler`
- ✅ **DTOs** - Data transfer objects like `EnvironmentCreationConfig`
- ✅ **Serde** - For JSON/TOML deserialization
- ✅ **Use Cases** - Orchestration of domain operations
- ✅ **Application Services** - Coordination between layers
- ✅ **Steps** - Individual units of work in command workflows
- ✅ **Application Errors** - Error types for application-level failures

### What Does NOT Belong Here

- ❌ **Business Logic** - Belongs in domain layer
- ❌ **Direct File I/O** - Use infrastructure traits instead
- ❌ **Direct External APIs** - Use infrastructure abstractions

### Examples and Patterns

#### ✅ Pattern: Command Handler (Use Case Orchestrator)

Command handlers orchestrate workflows by coordinating domain and infrastructure:

```rust
/// Command handler orchestrating a complete workflow
pub struct CommandHandler {
    domain_service: Arc<dyn DomainService>,
    infrastructure_client: Arc<dyn InfrastructureClient>,
    repository: Arc<dyn Repository>,
}

impl CommandHandler {
    /// Execute the workflow - orchestration only, no business logic
    pub async fn execute(&self, input: Input) -> Result<Output, CommandError> {
        // 1. Load domain entity
        let entity = self.repository.find_by_id(&input.id).await?;
        
        // 2. Delegate business logic to domain
        let updated_entity = entity.perform_business_operation(input.params)?;
        
        // 3. Use infrastructure services
        self.infrastructure_client.execute_external_operation(&updated_entity).await?;
        
        // 4. Persist changes
        self.repository.save(&updated_entity).await?;
        
        Ok(Output::from(updated_entity))
    }
}
```

**Why this is application:**

- Orchestrates domain and infrastructure (no business logic itself)
- Coordinates workflow steps in sequence
- Delegates business rules to domain layer
- Uses infrastructure through interfaces

#### ✅ Pattern: DTO (Data Transfer Object)

DTOs handle deserialization and convert to domain types:

```rust
/// DTO for external input (JSON, TOML, API requests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDto {
    /// Raw string - will be validated when converted to domain type
    pub name: String,
    pub port: u16,
    pub enabled: bool,
}

impl ConfigDto {
    /// Convert DTO to validated domain types
    pub fn to_domain(&self) -> Result<DomainConfig, ValidationError> {
        // Validation happens in domain layer
        let name = ValidatedName::new(self.name.clone())?;
        Ok(DomainConfig::new(name, self.port, self.enabled))
    }
}
```

**Why this is application:**

- Data transfer (not a domain entity with business logic)
- Uses raw primitives for deserialization
- Converts to domain types with validation
- Bridge between external format and domain

#### ✅ Pattern: Application Step

Steps are reusable units of work within workflows:

```rust
/// Reusable step in a command handler workflow
pub struct WorkflowStep {
    infrastructure_client: Arc<dyn Client>,
}

impl WorkflowStep {
    pub async fn execute(&self, entity: &Entity) -> Result<StepResult, StepError> {
        // Coordinate infrastructure call - no business logic
        let data = entity.get_required_data();
        self.infrastructure_client.perform_action(data).await?;
        Ok(StepResult::Success)
    }
}
```

**Why this is application:**

- Reusable component in workflows
- Coordinates infrastructure without business logic
- Can be composed into larger workflows

## 🔧 Infrastructure Layer (`src/infrastructure/` and `src/adapters/`)

### Purpose

The infrastructure layer provides technical implementations for external integrations, file I/O, persistence, and other external concerns. It implements interfaces defined in the domain layer.

**Note:** The project organizes infrastructure code into two directories:

- `src/adapters/` - Direct wrappers for external tools (OpenTofu, Ansible, SSH, Docker, LXD)
- `src/infrastructure/` - Other infrastructure concerns (persistence, tracing, template rendering)

### What Belongs Here

- ✅ **File I/O implementations** - Reading/writing files
- ✅ **HTTP clients** - External API integrations
- ✅ **Repository implementations** - `JsonEnvironmentRepository`
- ✅ **External tool wrappers** - OpenTofu, Ansible, SSH clients
- ✅ **Template rendering** - Tera template engines
- ✅ **Custom serialization** - When domain can't use derives
- ✅ **Trace writers** - Writing trace files

### What Does NOT Belong Here

- ❌ **Business Rules** - Belongs in domain
- ❌ **Domain Entities** - Reference them, don't define them
- ❌ **Use Cases** - Belongs in application

### Examples and Patterns

#### ✅ Pattern: Repository Implementation

Repositories implement domain persistence interfaces using specific technologies:

```rust
/// File-based repository implementing domain interface
pub struct FileSystemRepository {
    base_dir: PathBuf,
}

#[async_trait]
impl EntityRepository for FileSystemRepository {
    async fn save(&self, entity: &Entity) -> Result<(), RepositoryError> {
        let file_path = self.entity_file_path(entity.id());
        let json = serde_json::to_string_pretty(entity)?;
        tokio::fs::write(file_path, json).await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &EntityId) -> Result<Option<Entity>, RepositoryError> {
        let file_path = self.entity_file_path(id);
        if !file_path.exists() {
            return Ok(None);
        }
        let json = tokio::fs::read_to_string(file_path).await?;
        let entity = serde_json::from_str(&json)?;
        Ok(Some(entity))
    }
}
```

**Why this is infrastructure:**

- Implements domain repository interface
- Contains file I/O operations (tokio::fs)
- Handles serialization/deserialization
- No business logic (just persistence mechanics)

#### ✅ Pattern: External Tool Adapter

Adapters wrap external tools and provide a clean interface:

```rust
/// Adapter for external CLI tool (e.g., Terraform, Ansible, Docker)
pub struct ExternalToolClient {
    command_executor: Arc<dyn CommandExecutor>,
}

impl ExternalToolClient {
    pub async fn initialize(&self, working_dir: &Path) -> Result<(), ToolError> {
        self.command_executor
            .execute("tool", &["init"], working_dir)
            .await?;
        Ok(())
    }

    pub async fn apply_changes(&self, working_dir: &Path) -> Result<(), ToolError> {
        self.command_executor
            .execute("tool", &["apply", "--auto-approve"], working_dir)
            .await?;
        Ok(())
    }
}
```

**Why this is infrastructure:**

- Wraps external tool (CLI, API, SDK)
- Executes system commands or API calls
- No business logic (just integration mechanics)
- Provides clean interface for application layer

#### ✅ Pattern: HTTP Client Adapter

HTTP clients integrate with external APIs:

```rust
/// HTTP client for external API integration
pub struct ApiClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl ApiClient {
    pub async fn fetch_data(&self, id: &str) -> Result<ApiResponse, ApiError> {
        let url = format!("{}/api/data/{}", self.base_url, id);
        let response = self.http_client
            .get(&url)
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;
        Ok(response)
    }
}
```

**Why this is infrastructure:**

- HTTP client for external API
- Handles network I/O
- No business logic (just API integration)
- Returns data for application layer to process

## 🎨 Presentation Layer (`src/presentation/`)

### Purpose

The presentation layer handles user interaction through the command-line interface. It parses user input, dispatches commands, formats output, and displays errors in a user-friendly manner.

### What Belongs Here

- ✅ **Clap command definitions** - CLI argument parsing
- ✅ **User input validation** - Parsing command-line arguments
- ✅ **Output formatting** - User-friendly messages and progress indicators
- ✅ **Command dispatch** - Routing to application layer
- ✅ **Error display** - Formatting errors for users with help systems

### What Does NOT Belong Here

- ❌ **Business Logic** - Belongs in domain
- ❌ **Direct Infrastructure Calls** - Go through application layer

### Examples and Patterns

#### ✅ Pattern: CLI Definition with Clap

CLI structures define the user interface and parse arguments:

```rust
/// Main CLI structure with global args and subcommands
#[derive(Parser, Debug)]
#[command(name = "app-name")]
#[command(about = "Application description")]
#[command(version)]
pub struct Cli {
    /// Global flags available to all subcommands
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Available subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new resource
    Create {
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    /// Delete an existing resource
    Delete {
        name: String,
    },
}
```

**Why this is presentation:**

- Uses clap for argument parsing
- Defines user-facing interface
- No business logic (just data structure)
- Routes to application layer for execution

#### ✅ Pattern: Command Dispatcher

Dispatchers route CLI commands to application layer handlers:

```rust
/// Dispatcher that routes commands to application handlers
pub struct CommandDispatcher {
    command_handler: Arc<CommandHandler>,
}

impl CommandDispatcher {
    pub async fn dispatch(&self, command: Commands) -> Result<(), DispatchError> {
        match command {
            Commands::Create { name, force } => {
                // Parse/validate user input
                let validated_name = ValidatedName::new(name)
                    .map_err(DispatchError::InvalidInput)?;
                
                // Route to application layer
                self.command_handler.create(validated_name, force).await?;
                
                // Format user output
                println!("✅ Resource created successfully");
                Ok(())
            }
            Commands::Delete { name } => {
                let validated_name = ValidatedName::new(name)
                    .map_err(DispatchError::InvalidInput)?;
                
                self.command_handler.delete(validated_name).await?;
                println!("✅ Resource deleted successfully");
                Ok(())
            }
        }
    }
}
```

**Why this is presentation:**

- Routes commands to application layer
- Parses and validates user input
- Formats output for users
- No business logic (delegates to application/domain)

## 🧭 Decision Flowchart

Use this flowchart to quickly determine where code belongs:

```text
Does it contain business rules or validation?
├─ YES → Domain Layer
│   └─ Does it need persistence?
│       ├─ YES → Add #[derive(Serialize, Deserialize)]
│       └─ NO → Pure domain type
│
└─ NO → What is its primary responsibility?
    │
    ├─ Data Transfer (DTO) → Application Layer
    │   └─ Raw primitives (String, i32) for deserialization
    │
    ├─ Orchestration (Use Case) → Application Layer
    │   └─ Coordinates domain + infrastructure
    │
    ├─ External Integration → Infrastructure Layer
    │   └─ File I/O, HTTP, SSH, OpenTofu, Ansible
    │
    └─ User Interface → Presentation Layer
        └─ CLI parsing, output formatting, error display
```

## 🔍 Common Mistakes to Avoid

### ❌ DTOs in Domain Layer

```rust
// WRONG: This belongs in application layer
// src/domain/config.rs

#[derive(Serialize, Deserialize)]
pub struct ConfigDto {
    pub name: String,  // Raw primitive, no domain validation
    pub path: String,  // Should be PathBuf or domain type
}
```

**Why wrong:**

- Uses raw primitives without domain semantics
- No business logic or validation
- Purpose is data transfer, not domain modeling

**Correct placement:**

```rust
// src/application/config/environment_config.rs

#[derive(Serialize, Deserialize)]
pub struct EnvironmentCreationConfig {
    pub name: String,  // Will be validated when converted to EnvironmentName
}

impl EnvironmentCreationConfig {
    pub fn to_domain_params(&self) -> Result<EnvironmentName, ConfigError> {
        EnvironmentName::new(self.name.clone())
    }
}
```

### ❌ File I/O in Domain Layer

```rust
// WRONG: This belongs in infrastructure layer
// src/domain/environment/mod.rs

impl Environment {
    pub fn save_to_file(&self) -> Result<(), Error> {
        let json = serde_json::to_string(self)?;
        std::fs::write("environment.json", json)?;  // ❌ File I/O in domain!
        Ok(())
    }
}
```

**Why wrong:**

- Domain should not know about file systems
- Breaks dependency rule (domain → infrastructure)
- Hard to test

**Correct approach:**

```rust
// Domain defines interface
// src/domain/environment/repository.rs

#[async_trait]
pub trait EnvironmentRepository {
    async fn save(&self, environment: &Environment) -> Result<(), RepositoryError>;
}

// Infrastructure implements
// src/infrastructure/persistence/json_repository.rs

pub struct JsonEnvironmentRepository;

#[async_trait]
impl EnvironmentRepository for JsonEnvironmentRepository {
    async fn save(&self, environment: &Environment) -> Result<(), RepositoryError> {
        let json = serde_json::to_string(environment)?;
        tokio::fs::write("environment.json", json).await?;  // ✅ File I/O in infrastructure
        Ok(())
    }
}
```

### ❌ Business Logic in Application Layer

```rust
// WRONG: This belongs in domain layer
// src/application/command_handlers/create.rs

impl CreateCommandHandler {
    pub fn validate_environment_name(&self, name: &str) -> Result<(), Error> {
        // Business rules in application layer - WRONG!
        if name.is_empty() {
            return Err(Error::EmptyName);
        }
        if name.starts_with(char::is_numeric) {
            return Err(Error::StartsWithNumber);
        }
        Ok(())
    }
}
```

**Why wrong:**

- Business rules belong in domain
- Violates single responsibility
- Not reusable across use cases

**Correct approach:**

```rust
// Domain contains business rules
// src/domain/environment/name.rs

impl EnvironmentName {
    pub fn new(name: String) -> Result<Self, EnvironmentNameError> {
        Self::validate(&name)?;  // ✅ Business rules in domain
        Ok(Self(name))
    }

    fn validate(name: &str) -> Result<(), EnvironmentNameError> {
        if name.is_empty() {
            return Err(EnvironmentNameError::Empty);
        }
        if name.starts_with(char::is_numeric) {
            return Err(EnvironmentNameError::StartsWithNumber);
        }
        Ok(())
    }
}

// Application uses domain validation
// src/application/command_handlers/create.rs

impl CreateCommandHandler {
    pub fn execute(&self, name_str: String) -> Result<Environment, CreateError> {
        let name = EnvironmentName::new(name_str)?;  // ✅ Delegates to domain
        // ... rest of use case
    }
}
```

## 📚 Related Documentation

- [Codebase Architecture](../codebase-architecture.md) - Comprehensive architecture overview
- [Module Organization](./module-organization.md) - How to organize code within modules
- [Error Handling](./error-handling.md) - Error handling principles and patterns
- [Development Principles](../development-principles.md) - Core development principles

## 🔗 External Resources

- [Herberto Graça - Explicit Architecture](https://herbertograca.com/2017/11/16/explicit-architecture-01-ddd-hexagonal-onion-clean-cqrs-how-i-put-it-all-together/)
- [Microsoft - DDD Microservices Guide](https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice)
- [DDD Crew - Starter Modelling Process](https://github.com/ddd-crew/ddd-starter-modelling-process)
- [Eric Evans - Domain Language](https://www.domainlanguage.com/ddd/)
- [Martin Fowler - Domain Driven Design](https://martinfowler.com/bliki/DomainDrivenDesign.html)

## 💡 Summary

When in doubt, ask yourself:

1. **Does it contain business rules?** → Domain
2. **Is it orchestrating a use case?** → Application
3. **Does it integrate with external systems?** → Infrastructure
4. **Does it handle user interaction?** → Presentation

Remember: The goal is **clear separation of concerns**. Each layer should have one job and do it well.
