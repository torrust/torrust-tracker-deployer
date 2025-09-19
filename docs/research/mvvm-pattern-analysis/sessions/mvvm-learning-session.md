# MVVM Pattern Learning Session

**Version**: 0.1.0  
**Date**: September 19, 2025  
**Session Type**: Interactive Q&A  
**Purpose**: Deep dive into MVVM pattern concepts to better understand its applicability to the Torrust Tracker Deploy project

**Note**: This session documents the learning phase of MVVM research. The exploratory conclusions here suggested potential benefits for CLI applications. However, subsequent detailed architectural analysis (see `application-mvvm-analysis.md`) determined that MVVM is not suitable for this specific application's domain and requirements.

---

## 🎯 Learning Objectives

- Gain deeper understanding of MVVM pattern fundamentals
- Clarify specific aspects of MVVM implementation
- Build knowledge foundation for architectural analysis
- Document insights for team reference

---

## 📝 Q&A Session

### Question 1

**Your Question**: It seems that the MVVM pattern is not an architectural pattern but a design pattern. It looks that it's applied only to the presentation layer, and not to the whole app architecture. Is that right?

**My Answer**:

Excellent observation! This is a really important distinction that often causes confusion. You're absolutely correct - MVVM is fundamentally a **presentation layer pattern**, not a full application architecture pattern.

Here's the breakdown:

#### **MVVM as a Presentation Pattern**

- **Scope**: MVVM specifically addresses the organization of the **UI/presentation layer**
- **Focus**: It's concerned with how the user interface interacts with business logic
- **Boundary**: MVVM typically stops at the edge of the presentation layer

#### **The Three MVVM Components (Presentation Layer Only)**

1. **View**: The UI elements (buttons, text fields, etc.)
2. **ViewModel**: The presentation logic and state management
3. **Model**: _Within MVVM context_, this often refers to **presentation models** or **data transfer objects** that represent the data as the UI needs it

#### **What MVVM Doesn't Cover**

- **Business Logic Layer**: Domain services, business rules, use cases
- **Data Access Layer**: Repositories, database connections, APIs
- **Infrastructure Layer**: File systems, external services, configuration
- **Application Services**: Cross-cutting concerns, orchestration

#### **MVVM in the Larger Architecture**

In a complete application architecture, MVVM sits as the **presentation layer pattern** within larger architectural patterns like:

- **Clean Architecture** (Uncle Bob)
- **Hexagonal Architecture** (Ports & Adapters)
- **Layered Architecture**
- **Domain-Driven Design (DDD)**

#### **Example in Practice**

```text
┌──────────────────────────────────────────┐
│           Presentation Layer             │
│  ┌─────────────────────────────────────┐ │
│  │              MVVM                   │ │
│  │  View ↔ ViewModel ↔ Model           │ │
│  └─────────────────────────────────────┘ │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│          Application Layer               │
│         (Use Cases, Services)            │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│            Domain Layer                  │
│      (Business Rules, Entities)          │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│         Infrastructure Layer             │
│    (Database, APIs, File System)         │
└──────────────────────────────────────────┘
```

#### **Key Insight**

When people say "our app follows MVVM architecture," they usually mean:

1. The **presentation layer** uses MVVM pattern
2. The overall app might use Clean Architecture, DDD, etc.
3. MVVM handles the UI concerns within that larger architectural context

This distinction is crucial for your Torrust Tracker Deploy analysis - you'll want to identify if the presentation aspects could benefit from MVVM organization, while recognizing that the overall application architecture likely follows different patterns (possibly Clean Architecture or layered architecture).

---

### Question 2

**Your Question**: It seems this pattern was introduced originally for desktop applications. I understand that the main goal was to remove logic from views. I guess the context was the views were getting more and more complex at that time and there was some "business" logic for the views that developers did not want to duplicate or mix with the view templates. For example, enabling or not an option might depend on some previous user's decisions. That's a logic we should pull from the view, so we can test it independently or even reuse between views. Am I right?

**My Answer**:

Absolutely spot on! You've captured the historical context and core motivation perfectly. This is exactly why MVVM emerged and why it became so influential.

#### **Historical Context - Desktop GUI Applications**

MVVM was indeed born from the desktop application world, specifically:

- **Microsoft's WPF (Windows Presentation Foundation)** around 2005-2006
- **John Gossman** (Microsoft architect) coined the term and formalized the pattern
- **Rich desktop applications** with complex UIs were becoming common
- **Data binding capabilities** in frameworks like WPF made MVVM practical

#### **The Core Problem You Identified**

Your example is perfect - **conditional UI logic** was a major pain point:

```text
Traditional View-heavy approach:
┌─────────────────────────────────────┐
│             View (XAML/UI)          │
│  ┌─────────────────────────────────┐│
│  │ if (user.hasPermission &&       ││  ❌ Logic in View
│  │     previousChoice == "admin")  ││  ❌ Hard to test
│  │   enableButton()                ││  ❌ Hard to reuse
│  │ else                            ││  ❌ Mixed concerns
│  │   disableButton()               ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

#### **What MVVM Solved**

1. **Separation of Concerns**:

   ```text
   View:      "Show button enabled/disabled based on ViewModel state"
   ViewModel: "Button should be enabled if user.hasPermission && previousChoice == 'admin'"
   Model:     "Here's the user data and previous choices"
   ```

2. **Testability**:

   ```rust
   // You can test this logic without any UI!
   #[test]
   fn it_should_enable_button_for_admin_users_with_permission() {
       let viewmodel = UserSettingsViewModel::new(
           User { has_permission: true },
           PreviousChoice::Admin
       );
       assert_eq!(viewmodel.is_button_enabled(), true);
   }
   ```

3. **Reusability**:
   - Same ViewModel logic could drive multiple Views (desktop, mobile, web)
   - Same conditional logic could be reused across different screens

#### **The Specific "Business Logic" You Mentioned**

This is what we call **Presentation Logic** (not pure business logic):

- **Form validation rules**: "Submit button enabled only when all fields valid"
- **Navigation logic**: "Show advanced options only after basic setup complete"
- **UI state management**: "Loading spinner while data fetches"
- **Conditional visibility**: "Show admin panel only for admin users"

#### **Classic Desktop MVVM Example**

```text
User Settings Screen:
┌─────────────────────────────────────┐
│  View (WPF/XAML)                    │
│  ┌─────────┐ ┌──────────────────┐   │
│  │ [Save]  │ │ [Advanced Panel] │   │  ← UI Elements
│  └─────────┘ └──────────────────┘   │
└─────────────────────────────────────┘
            ↕ Data Binding
┌─────────────────────────────────────┐
│  ViewModel                          │
│  • IsSaveEnabled: bool              │  ← Presentation Logic
│  • ShowAdvancedPanel: bool          │
│  • ValidationErrors: List<string>   │
└─────────────────────────────────────┘
            ↕
┌─────────────────────────────────────┐
│  Model (Data)                       │
│  • User: { name, email, role }      │  ← Pure Data
│  • Settings: { theme, language }    │
└─────────────────────────────────────┘
```

#### **Key Insight for Your Analysis**

Even though your Rust CLI application isn't a desktop GUI, you might have similar **presentation logic** concerns:

- When to show certain configuration options
- How to format output based on user choices
- What commands to make available based on current state
- Validation feedback and error presentation

The pattern's core value - **separating presentation logic from views** - can apply beyond traditional GUIs!

You've really grasped the essential motivation behind MVVM. This understanding will be crucial when evaluating if similar separation would benefit your CLI application's presentation layer.

---

### Question 3

**Your Question**: I want some more modern examples of the pattern. For example, for me this pattern is what you do when you use ReactJS+Store. You can centralize the state of the presentation layer so the different views can access this state. Although maybe this example is not complete because the store usually does not have any kind of "presentation" logic. But at least it stores the data that the views need. It could be the ViewModel or it could also be considered the DTO for the business model (backend). It's probably a mix of both in this case (domain model cache + presentation state model).

**My Answer**:

Fantastic observation! You've identified a really important evolution of MVVM into modern web development. Your React + Store example is spot-on, and your nuanced analysis about it being "not complete" shows you're thinking deeply about the pattern boundaries.

#### **Modern MVVM Incarnations**

You're absolutely right that **React + Redux/Zustand/Context** represents a modern MVVM-like approach:

```javascript
// Modern React MVVM-ish Structure
┌─────────────────────────────────────┐
│  React Component (View)             │  ← Pure rendering, minimal logic
│  function UserProfile() {           │
│    const { user, isLoading,         │
│             updateProfile } =       │
│      useUserStore();                │
│    return <div>...</div>            │
│  }                                  │
└─────────────────────────────────────┘
            ↕ Store/Hooks
┌─────────────────────────────────────┐
│  Custom Hook/Store (ViewModel-ish)  │  ← Presentation logic + state
│  const useUserStore = () => ({      │
│    user: state.user,                │
│    isLoading: state.loading,        │
│    updateProfile: (data) => {       │
│      validateInput(data);           │  ← Presentation logic
│      dispatch(updateUser(data));    │
│    }                                │
│  });                                │
└─────────────────────────────────────┘
            ↕
┌─────────────────────────────────────┐
│  API/Backend Data (Model-ish)       │  ← Pure data from backend
│  { id, name, email, avatar_url }    │
└─────────────────────────────────────┘
```

#### **Your Insight About the "Mix"**

You've identified a key characteristic of modern implementations - the store often serves **dual purposes**:

1. **ViewModel aspects**:

   - Derived state (computed values)
   - UI state management (loading, errors)
   - Presentation logic (formatting, validation)

2. **Model Cache aspects**:
   - Raw backend data
   - Domain entities
   - API response DTOs

#### **Modern Examples Across Frameworks**

**Vue.js + Pinia/Vuex**:

```javascript
// Store acts as ViewModel
const userStore = useUserStore()
computed: {
  displayName() {  // ← Presentation logic
    return `${user.firstName} ${user.lastName}`.trim() || 'Anonymous'
  },
  canEdit() {      // ← UI conditional logic
    return user.role === 'admin' || user.id === currentUser.id
  }
}
```

**Angular + Services/NgRx**:

```typescript
// Service acts as ViewModel
@Injectable()
class UserViewModel {
  private readonly userSubject = new BehaviorSubject<User | null>(null);

  readonly user$ = this.userSubject.asObservable();
  readonly displayName$ = this.user$.pipe(
    map((user) => (user ? `${user.firstName} ${user.lastName}` : "Anonymous"))
  );

  updateProfile(data: UserUpdateData): void {
    // Validation logic (presentation concern)
    if (!this.validateInput(data)) return;

    // Update backend (model interaction)
    this.userService.updateUser(data);
  }
}
```

**Svelte + Stores**:

```javascript
// Store with derived values (ViewModel-like)
export const userStore = writable(null);
export const displayName = derived(userStore, ($user) =>
  $user ? `${$user.firstName} ${$user.lastName}` : "Anonymous"
);
```

#### **The "Hybrid" Nature You Identified**

Modern stores are indeed **hybrids** because:

```text
Traditional MVVM:
Model ← ViewModel ← View

Modern Web MVVM:
Backend/API ← Store/State ← Component
     ↑            ↑           ↑
   Pure Data    Hybrid:     Pure UI
               Model Cache
                  +
            Presentation Logic
```

#### **Why This Evolution Happened**

1. **Network Reality**: Web apps need to cache backend data locally
2. **Component Reusability**: Multiple components need access to same data
3. **Performance**: Avoid prop drilling and unnecessary re-renders
4. **DevTools**: Time-travel debugging and state inspection

#### **Your Analysis Applied to CLI Context**

This modern understanding is super relevant for your Rust CLI application:

```rust
// Potential CLI "Store/ViewModel" approach
pub struct DeploymentState {
    // Model cache (like React store)
    infrastructure_config: InfrastructureConfig,
    deployment_status: DeploymentStatus,

    // Presentation state (ViewModel aspects)
    current_step: DeploymentStep,
    progress_percentage: u8,
    error_messages: Vec<String>,
}

impl DeploymentState {
    // Presentation logic methods
    pub fn should_show_advanced_options(&self) -> bool {
        self.current_step >= DeploymentStep::ConfigurationComplete
    }

    pub fn format_status_message(&self) -> String {
        match self.deployment_status {
            DeploymentStatus::InProgress(step) =>
                format!("Deploying: {} ({}%)", step, self.progress_percentage),
            DeploymentStatus::Complete => "✅ Deployment successful!".to_string(),
            DeploymentStatus::Failed(err) => format!("❌ Failed: {}", err),
        }
    }
}
```

You've really captured how MVVM has evolved and adapted to modern development paradigms! The "hybrid" nature you identified is exactly what makes modern implementations so powerful and practical.

---

### Question 4

**Your Question**: How might this pattern apply specifically to command-line tools?

**My Answer**:

Excellent question! CLI applications present a fascinating challenge for MVVM because they don't have traditional "views" with persistent UI elements, but they absolutely have presentation concerns that can benefit from MVVM principles.

#### **Redefining MVVM Components for CLI Context**

**View (CLI Output Layer)**:

- Terminal output formatting
- Progress indicators, spinners, status messages
- Error display and user prompts
- Interactive menus and confirmation dialogs
- Colored/styled text presentation

**ViewModel (CLI Presentation Logic)**:

- Command validation and parsing
- Output formatting decisions (JSON vs table vs plain text)
- Progress tracking and status management
- User interaction flow control
- Conditional command availability

**Model (Data/Business Layer)**:

- Configuration data, deployment state
- File system interactions, API calls
- Business logic execution (infrastructure provisioning)
- Domain entities and services

#### **CLI MVVM in Practice**

Here's how your Torrust Tracker Deploy could apply MVVM:

```rust
// View Layer - Terminal Output
pub struct TerminalView {
    output: Box<dyn Write>,
}

impl TerminalView {
    pub fn show_deployment_status(&mut self, status: &DeploymentDisplayState) {
        match status.phase {
            DeploymentPhase::Provisioning => {
                writeln!(self.output, "{} Provisioning infrastructure... ({}%)",
                    status.spinner_char, status.progress_percentage);
            }
            DeploymentPhase::Complete => {
                writeln!(self.output, "✅ {}", status.success_message);
            }
            DeploymentPhase::Failed => {
                writeln!(self.output, "❌ Error: {}", status.error_message);
            }
        }
    }

    pub fn show_configuration_options(&mut self, options: &ConfigurationOptions) {
        if options.show_advanced {
            writeln!(self.output, "Advanced Options:");
            for option in &options.advanced_options {
                writeln!(self.output, "  --{} {}", option.name, option.description);
            }
        }
    }
}

// ViewModel Layer - CLI Presentation Logic
pub struct DeploymentViewModel {
    // State
    deployment_state: DeploymentState,
    user_preferences: UserPreferences,
    command_history: Vec<String>,

    // Dependencies (would be injected)
    deployment_service: Box<dyn DeploymentService>,
}

impl DeploymentViewModel {
    // Presentation logic methods
    pub fn get_display_state(&self) -> DeploymentDisplayState {
        DeploymentDisplayState {
            phase: self.deployment_state.current_phase(),
            progress_percentage: self.calculate_progress_percentage(),
            spinner_char: self.get_current_spinner_char(),
            success_message: self.format_success_message(),
            error_message: self.format_error_message(),
        }
    }

    pub fn get_configuration_options(&self) -> ConfigurationOptions {
        ConfigurationOptions {
            show_advanced: self.should_show_advanced_options(),
            advanced_options: self.get_filtered_advanced_options(),
        }
    }

    pub fn should_show_advanced_options(&self) -> bool {
        self.user_preferences.show_advanced ||
        self.deployment_state.basic_setup_complete()
    }

    pub fn handle_provision_command(&mut self, args: &ProvisionArgs) -> CommandResult {
        // Validate command (presentation concern)
        if let Some(validation_error) = self.validate_provision_args(args) {
            return CommandResult::ValidationError(validation_error);
        }

        // Show immediate feedback (presentation concern)
        let feedback = self.format_provision_start_message(args);

        // Execute business logic (delegate to service)
        match self.deployment_service.provision(args.into()) {
            Ok(deployment_id) => {
                self.deployment_state.set_active_deployment(deployment_id);
                CommandResult::Success(feedback)
            }
            Err(e) => CommandResult::Error(self.format_provision_error(e))
        }
    }

    // Formatting methods (presentation logic)
    fn format_success_message(&self) -> String {
        match &self.deployment_state {
            DeploymentState::Complete { target, duration, .. } => {
                format!("Deployment to {} completed in {:.1}s", target, duration.as_secs_f64())
            }
            _ => "Deployment completed successfully".to_string()
        }
    }

    fn calculate_progress_percentage(&self) -> u8 {
        // Complex presentation logic for progress calculation
        let total_steps = self.deployment_state.total_steps();
        let completed_steps = self.deployment_state.completed_steps();
        ((completed_steps as f64 / total_steps as f64) * 100.0) as u8
    }
}

// Model Layer - Business Logic
pub struct DeploymentService {
    infrastructure_provider: Box<dyn InfrastructureProvider>,
    config_manager: ConfigManager,
}

impl DeploymentService {
    pub fn provision(&self, request: ProvisionRequest) -> Result<DeploymentId> {
        // Pure business logic - no presentation concerns
        let config = self.config_manager.load_config(&request.config_path)?;
        let deployment = self.infrastructure_provider.create_deployment(config)?;
        Ok(deployment.id)
    }
}
```

#### **CLI MVVM Benefits**

**1. Testable Presentation Logic**:

```rust
#[test]
fn it_should_show_advanced_options_when_basic_setup_complete() {
    let mut viewmodel = DeploymentViewModel::new_for_test();
    viewmodel.deployment_state.mark_basic_setup_complete();

    let options = viewmodel.get_configuration_options();
    assert!(options.show_advanced);
}

#[test]
fn it_should_format_progress_message_correctly() {
    let viewmodel = create_viewmodel_with_progress(3, 5);
    let display_state = viewmodel.get_display_state();
    assert_eq!(display_state.progress_percentage, 60);
}
```

**2. Flexible Output Formats**:

```rust
// Same ViewModel can drive different Views
impl DeploymentViewModel {
    pub fn get_json_output(&self) -> serde_json::Value {
        json!({
            "status": self.deployment_state.current_phase(),
            "progress": self.calculate_progress_percentage(),
            "message": self.format_status_message()
        })
    }

    pub fn get_table_output(&self) -> Vec<Vec<String>> {
        // Format for table display
    }
}
```

**3. Command Flow Control**:

```rust
impl DeploymentViewModel {
    pub fn get_available_commands(&self) -> Vec<Command> {
        let mut commands = vec![Command::Help, Command::Status];

        match self.deployment_state.current_phase() {
            DeploymentPhase::NotStarted => {
                commands.push(Command::Provision);
                commands.push(Command::Configure);
            }
            DeploymentPhase::InProgress => {
                commands.push(Command::Cancel);
                commands.push(Command::Logs);
            }
            DeploymentPhase::Complete => {
                commands.push(Command::Deploy);
                commands.push(Command::Destroy);
            }
            DeploymentPhase::Failed => {
                commands.push(Command::Retry);
                commands.push(Command::Debug);
            }
        }

        commands
    }
}
```

#### **CLI-Specific MVVM Patterns**

**Interactive CLI (TUI-style)**:

- ViewModel manages screen state and navigation
- View handles keyboard input and screen rendering
- Model provides data and executes commands

**Batch/Script CLI**:

- ViewModel formats output for different consumers (human vs machine)
- View handles different output channels (stdout, stderr, files)
- Model executes business operations

**Progressive CLI (like your deployment tool)**:

- ViewModel manages multi-step workflow state
- View provides step-by-step feedback and progress indication
- Model handles each deployment phase independently

#### **Key Insight for Your Project**

Your Torrust Tracker Deploy application could benefit from MVVM by:

1. **Separating formatting logic** from business logic
2. **Making output formatting testable** and configurable
3. **Supporting multiple output formats** (human-readable, JSON, etc.)
4. **Managing complex command flows** and state transitions
5. **Providing consistent user experience** across different command contexts

The pattern works especially well for CLI tools that have:

- Complex multi-step workflows
- Different output format requirements
- Rich user interaction and feedback
- State-dependent command availability

---

_[Questions and answers will be added as we proceed with the session]_

---

## 💡 Key Insights Discovered

_[Will be populated with key takeaways at the end of the session]_

---

## 🎓 Learning Summary

_[Will be completed at the end of our session with main concepts learned]_
