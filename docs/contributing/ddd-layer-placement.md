# DDD Layer Placement Guide

This guide provides clear guidelines on which code belongs in which Domain-Driven Design (DDD) layer. Following these guidelines ensures proper separation of concerns, maintainability, and testability.

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

### Examples from the Codebase

#### ✅ Correct: Value Object with Validation

```rust
// src/domain/environment/name.rs

/// Validated environment name following restricted format rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnvironmentName(String);

impl EnvironmentName {
    /// Creates a new `EnvironmentName` from a string with validation.
    pub fn new(name: String) -> Result<Self, EnvironmentNameError> {
        Self::validate(&name)?;
        Ok(Self(name))
    }

    fn validate(name: &str) -> Result<(), EnvironmentNameError> {
        // Business rules for environment names
        if name.is_empty() {
            return Err(EnvironmentNameError::Empty);
        }
        // ... more validation
        Ok(())
    }
}
```

**Why this is domain:**

- Contains business rules (validation)
- Uses serde for persistence (pragmatic trade-off)
- No infrastructure concerns

#### ✅ Correct: Domain Entity with Type-State Pattern

```rust
// src/domain/environment/mod.rs

/// Environment entity with type-state pattern for lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment<S> {
    context: EnvironmentContext,
    state: S,
}

impl Environment<Created> {
    pub fn new(
        name: EnvironmentName,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
    ) -> Self {
        // Domain logic for creating environments
        Self {
            context: EnvironmentContext::new(name, ssh_credentials, ssh_port),
            state: Created,
        }
    }
}
```

**Why this is domain:**

- Core business entity
- Type-safe state transitions (compile-time safety)
- Contains business logic
- Serde for persistence (pragmatic choice)

#### ✅ Correct: Domain Trait

```rust
// src/domain/environment/repository/environment_repository.rs

/// Repository interface for environment persistence
#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    async fn save(&self, environment: &Environment<DynState>) -> Result<(), RepositoryError>;
    async fn find_by_name(&self, name: &EnvironmentName) -> Result<Option<Environment<DynState>>, RepositoryError>;
    async fn delete(&self, name: &EnvironmentName) -> Result<(), RepositoryError>;
}
```

**Why this is domain:**

- Defines contract for persistence (interface)
- No implementation details
- Infrastructure layer implements this trait

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

### Examples from the Codebase

#### ✅ Correct: Command Handler

```rust
// src/application/command_handlers/provision/handler.rs

/// Command handler orchestrating the complete provisioning workflow
pub struct ProvisionCommandHandler {
    pub(crate) tofu_template_renderer: Arc<TofuTemplateRenderer>,
    pub(crate) ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    pub(crate) ansible_client: Arc<AnsibleClient>,
    pub(crate) opentofu_client: Arc<OpenTofuClient>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) repository: TypedEnvironmentRepository,
}

impl ProvisionCommandHandler {
    /// Execute the complete provisioning workflow
    pub async fn execute(
        &self,
        environment: Environment<Created>,
    ) -> Result<Environment<Provisioned>, ProvisionCommandHandlerError> {
        // Orchestrates steps without containing business logic
        let env = self.transition_to_provisioning(environment).await?;
        let env = self.render_templates(env).await?;
        let env = self.initialize_infrastructure(env).await?;
        // ... more steps
        Ok(env)
    }
}
```

**Why this is application:**

- Orchestrates domain and infrastructure services
- No business logic (delegates to domain)
- Uses infrastructure implementations
- Coordinates workflow steps

#### ✅ Correct: DTO (Data Transfer Object)

```rust
// src/application/command_handlers/create/config/environment_config.rs

/// Configuration for creating a deployment environment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentCreationConfig {
    pub environment: EnvironmentSection,
    pub ssh_credentials: SshCredentialsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSection {
    /// Raw string that will be validated when converted to EnvironmentName
    pub name: String,
}

impl EnvironmentCreationConfig {
    /// Convert to domain types with validation
    pub fn to_domain_params(&self) -> Result<(EnvironmentName, SshCredentials), CreateConfigError> {
        let name = EnvironmentName::new(self.environment.name.clone())?;
        let credentials = self.ssh_credentials.to_domain_credentials()?;
        Ok((name, credentials))
    }
}
```

**Why this is application:**

- Data transfer object (not domain entity)
- Uses raw primitives (`String`) for deserialization
- Converts to domain types
- Serde for JSON/TOML parsing

#### ✅ Correct: Step (Unit of Work)

```rust
// src/application/steps/infrastructure/apply.rs

/// Step that applies OpenTofu configuration to provision infrastructure
pub struct ApplyInfrastructureStep {
    client: Arc<OpenTofuClient>,
}

impl ApplyInfrastructureStep {
    pub async fn execute(
        &self,
        environment: &Environment<Provisioning>,
    ) -> Result<(), StepError> {
        // Coordinates infrastructure without business logic
        self.client.apply(environment.build_dir()).await?;
        Ok(())
    }
}
```

**Why this is application:**

- Unit of work in a workflow
- Coordinates infrastructure calls
- No business logic

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

### Examples from the Codebase

#### ✅ Correct: Repository Implementation

```rust
// src/infrastructure/persistence/json_environment_repository.rs

/// JSON-based file system implementation of EnvironmentRepository
pub struct JsonEnvironmentRepository {
    base_data_dir: PathBuf,
}

#[async_trait]
impl EnvironmentRepository for JsonEnvironmentRepository {
    async fn save(&self, environment: &Environment<DynState>) -> Result<(), RepositoryError> {
        let file_path = self.environment_file_path(environment.name());
        let json = serde_json::to_string_pretty(environment)?;
        tokio::fs::write(file_path, json).await?;
        Ok(())
    }

    async fn find_by_name(&self, name: &EnvironmentName) -> Result<Option<Environment<DynState>>, RepositoryError> {
        let file_path = self.environment_file_path(name);
        if !file_path.exists() {
            return Ok(None);
        }
        let json = tokio::fs::read_to_string(file_path).await?;
        let environment = serde_json::from_str(&json)?;
        Ok(Some(environment))
    }
}
```

**Why this is infrastructure:**

- Implements domain interface (`EnvironmentRepository`)
- Contains file I/O operations
- Handles serialization/deserialization
- No business logic

#### ✅ Correct: External Tool Client

```rust
// src/adapters/tofu/client.rs

/// Client for executing OpenTofu CLI commands
pub struct OpenTofuClient {
    command_executor: Arc<dyn CommandExecutor>,
}

impl OpenTofuClient {
    pub async fn init(&self, working_dir: &Path) -> Result<(), OpenTofuError> {
        self.command_executor
            .execute("tofu", &["init"], working_dir)
            .await?;
        Ok(())
    }

    pub async fn apply(&self, working_dir: &Path) -> Result<(), OpenTofuError> {
        self.command_executor
            .execute("tofu", &["apply", "-auto-approve"], working_dir)
            .await?;
        Ok(())
    }
}
```

**Why this is infrastructure:**

- Wraps external tool (OpenTofu)
- Executes system commands
- No business logic
- Provides interface for application layer

#### ✅ Correct: Template Renderer

```rust
// src/infrastructure/external_tools/tofu/template/renderer/mod.rs

/// Renderer for OpenTofu templates using Tera
pub struct TofuTemplateRenderer {
    tera: Tera,
    source_dir: PathBuf,
}

impl TofuTemplateRenderer {
    pub fn render_templates(
        &self,
        environment: &Environment<impl Send + Sync>,
    ) -> Result<(), TemplateRenderError> {
        let context = self.build_context(environment);
        for template in self.tera.get_template_names() {
            let rendered = self.tera.render(template, &context)?;
            self.write_output(template, &rendered, environment)?;
        }
        Ok(())
    }
}
```

**Why this is infrastructure:**

- Uses external library (Tera)
- Handles file I/O
- No business logic

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

### Examples from the Codebase

#### ✅ Correct: CLI Definition

```rust
// src/presentation/cli/mod.rs

/// Command-line interface for Torrust Tracker Deployer
#[derive(Parser, Debug)]
#[command(name = "torrust-tracker-deployer")]
#[command(about = "Automated deployment infrastructure for Torrust Tracker")]
#[command(version)]
pub struct Cli {
    /// Global arguments (logging configuration)
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new deployment environment
    Create {
        #[command(subcommand)]
        action: CreateAction,
    },
    /// Destroy an existing deployment environment
    Destroy {
        environment: String,
    },
}
```

**Why this is presentation:**

- Uses clap for CLI parsing
- Defines user interface
- No business logic

#### ✅ Correct: Command Handler (Presentation)

```rust
// src/presentation/commands/destroy/handler.rs

/// Presentation-layer handler for destroy command
pub struct DestroyCommandPresenter {
    handler: Arc<DestroyCommandHandler>,
}

impl DestroyCommandPresenter {
    pub async fn handle(&self, environment_name: &str) -> Result<(), PresentationError> {
        // Parse user input
        let name = EnvironmentName::new(environment_name.to_string())
            .map_err(|e| PresentationError::InvalidEnvironmentName(e))?;

        // Call application layer
        self.handler.execute(name).await
            .map_err(PresentationError::from)?;

        // Format success output
        println!("✅ Environment '{}' destroyed successfully", environment_name);
        Ok(())
    }
}
```

**Why this is presentation:**

- Handles user input parsing
- Routes to application layer
- Formats output for users
- No business logic

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
