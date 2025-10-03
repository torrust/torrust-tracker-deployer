# Command Refactoring for State Management Integration

> **📋 Preparatory Refactoring**
> Clean up and restructure commands to make Phase 5 (State Management Integration) easier and safer to implement.

## 🎯 Refactoring Goals

**Primary Goal**: Prepare `ProvisionCommand` and `ConfigureCommand` for seamless integration with the type-state pattern and persistence layer.

**Why This Matters**: The commands need significant changes for Phase 5:

- Accept `Environment<S>` as input (not individual parameters)
- Return `Environment<S'>` as output (not void or primitive types)
- Integrate state transitions at multiple points
- Call repository for persistence
- Extract failed step information from errors

**Without refactoring first**, we would:

- ❌ Make large, risky changes to complex methods
- ❌ Mix refactoring with feature implementation
- ❌ Increase bug risk and testing complexity
- ❌ Make code review harder

**With refactoring first**, we will:

- ✅ Smaller, focused commits
- ✅ Separate concerns clearly
- ✅ Reduce risk of Phase 5 implementation
- ✅ Improve code quality independently
- ✅ Make state management integration obvious and simple

## 📊 Current State Analysis

### ProvisionCommand Issues

**Structure Problems:**

- `execute()` method is 50+ lines doing multiple concerns
- `ssh_credentials` stored as field but should come from Environment
- Returns `IpAddr` instead of structured result
- No clear phases (setup → execution → validation)
- Step execution mixed with orchestration logic

**Testability Problems:**

- Hard to test individual phases
- Mock setup requires 5 dependencies
- Cannot test state transitions (not implemented yet)

**Future Integration Problems:**

- Adding Environment parameter means changing all step calls
- No clear injection points for state transitions
- No clear injection points for persistence
- Error handling doesn't extract step information

### ConfigureCommand Issues

**Structure Problems:**

- Very simple (good!), but no structure for future needs
- Returns `()` instead of structured result
- Direct step calls in execute method

**Future Integration Problems:**

- Adding Environment parameter requires signature changes
- No error context extraction
- No structure for state transitions

## 🔧 Refactoring Strategy

### Phase-Based Refactoring

We'll refactor in small, testable increments:

1. **Extract Execution Methods** - Break down `execute()` into logical phases
2. **Remove Redundant Fields** - Clean up `ssh_credentials` from ProvisionCommand
3. **Standardize Error Handling** - Add helper methods for error context extraction
4. **Prepare for Environment** - Create structure that makes Environment integration obvious

### Non-Goals

**What we're NOT doing:**

- ❌ Not adding Environment parameter yet (that's Phase 5)
- ❌ Not adding state transitions yet (that's Phase 5)
- ❌ Not adding repository yet (that's Phase 5)
- ❌ Not changing command contracts (return types stay same for now)

**What we ARE doing:**

- ✅ Making methods smaller and more focused
- ✅ Removing redundant dependencies
- ✅ Creating clear structure for future additions
- ✅ Improving testability
- ✅ Adding helper methods for future use

## 📋 Refactoring Tasks

### Task 1: Refactor ProvisionCommand Structure

**Purpose**: Break down the monolithic `execute()` method into logical phases.

**Changes**:

1. **Extract Template Rendering Phase**:

   ```rust
   async fn render_templates(&self) -> Result<(), ProvisionCommandError> {
       RenderOpenTofuTemplatesStep::new(Arc::clone(&self.tofu_template_renderer))
           .execute()
           .await?;
       Ok(())
   }
   ```

2. **Keep `create_instance()` as Infrastructure Phase** (already exists)

3. **Extract Instance Info Retrieval**:

   ```rust
   fn get_instance_info(&self) -> Result<InstanceInfo, ProvisionCommandError> {
       GetInstanceInfoStep::new(Arc::clone(&self.opentofu_client))
           .execute()
   }
   ```

4. **Extract Ansible Template Rendering Phase**:

   ```rust
   async fn render_ansible_templates(
       &self,
       instance_ip: IpAddr,
   ) -> Result<(), ProvisionCommandError> {
       let socket_addr = std::net::SocketAddr::new(instance_ip, 22);
       RenderAnsibleTemplatesStep::new(
           Arc::clone(&self.ansible_template_renderer),
           self.ssh_credentials.clone(),
           socket_addr,
       )
       .execute()
       .await
   }
   ```

5. **Extract Connectivity Validation Phase**:

   ```rust
   async fn validate_connectivity(
       &self,
       instance_ip: IpAddr,
   ) -> Result<(), ProvisionCommandError> {
       let ssh_connection = SshConnection::with_default_port(
           self.ssh_credentials.clone(),
           instance_ip,
       );

       WaitForSSHConnectivityStep::new(ssh_connection)
           .execute()
           .await?;

       WaitForCloudInitStep::new(Arc::clone(&self.ansible_client))
           .execute()?;

       Ok(())
   }
   ```

6. **Simplify `execute()` to orchestrate phases**:

   ```rust
   pub async fn execute(&self) -> Result<IpAddr, ProvisionCommandError> {
       info!("Starting complete infrastructure provisioning workflow");

       // Phase 1: Template rendering
       self.render_templates().await?;

       // Phase 2: Infrastructure creation
       self.create_instance()?;

       // Phase 3: Instance information retrieval
       let instance_info = self.get_instance_info()?;
       let instance_ip = instance_info.ip_address;

       // Phase 4: Ansible template rendering
       self.render_ansible_templates(instance_ip).await?;

       // Phase 5: Connectivity validation
       self.validate_connectivity(instance_ip).await?;

       info!(instance_ip = %instance_ip, "Provisioning completed successfully");
       Ok(instance_ip)
   }
   ```

**Benefits**:

- Clear phases for future state transition injection
- Each phase testable independently
- Easy to add persistence calls between phases
- Obvious where to add `Environment<Provisioning>` parameter later
- Reduced complexity in main `execute()` method

**Tests to Add**:

- Keep existing tests (they still pass)
- Optionally add tests for individual phases (if we make them `pub(crate)` for testing)

**Success Criteria**:

- ✅ `execute()` method < 30 lines
- ✅ 5 clear phases extracted
- ✅ All existing tests pass
- ✅ No behavior changes
- ✅ All linters pass

**Commit**: `refactor: extract logical phases in ProvisionCommand for better clarity`

---

### Task 2: Remove Redundant SshCredentials Field

**Purpose**: Remove `ssh_credentials` field from `ProvisionCommand` since it will come from `Environment` in Phase 5.

**Problem**: Currently `ssh_credentials` is:

- Stored as a field in the struct
- Passed to constructor
- Used in 2 places: ansible template rendering and SSH connectivity

**For Phase 5**, we'll have:

- `Environment<Created>` as input parameter
- SSH credentials accessed via `environment.ssh_credentials()`
- No need to store as field

**Changes**:

1. **Add `ssh_credentials` parameter to methods that need it**:

   ```rust
   async fn render_ansible_templates(
       &self,
       instance_ip: IpAddr,
       ssh_credentials: &SshCredentials,  // NEW parameter
   ) -> Result<(), ProvisionCommandError> {
       let socket_addr = std::net::SocketAddr::new(instance_ip, 22);
       RenderAnsibleTemplatesStep::new(
           Arc::clone(&self.ansible_template_renderer),
           ssh_credentials.clone(),
           socket_addr,
       )
       .execute()
       .await
   }

   async fn validate_connectivity(
       &self,
       instance_ip: IpAddr,
       ssh_credentials: &SshCredentials,  // NEW parameter
   ) -> Result<(), ProvisionCommandError> {
       let ssh_connection = SshConnection::with_default_port(
           ssh_credentials.clone(),
           instance_ip,
       );

       WaitForSSHConnectivityStep::new(ssh_connection)
           .execute()
           .await?;

       WaitForCloudInitStep::new(Arc::clone(&self.ansible_client))
           .execute()?;

       Ok(())
   }
   ```

2. **Remove from struct and constructor**:

   ```rust
   pub struct ProvisionCommand {
       tofu_template_renderer: Arc<TofuTemplateRenderer>,
       ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
       ansible_client: Arc<AnsibleClient>,
       opentofu_client: Arc<OpenTofuClient>,
       // REMOVED: ssh_credentials: SshCredentials,
   }

   pub fn new(
       tofu_template_renderer: Arc<TofuTemplateRenderer>,
       ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
       ansible_client: Arc<AnsibleClient>,
       opentofu_client: Arc<OpenTofuClient>,
       // REMOVED: ssh_credentials: SshCredentials,
   ) -> Self {
       Self {
           tofu_template_renderer,
           ansible_template_renderer,
           ansible_client,
           opentofu_client,
       }
   }
   ```

3. **Update `execute()` signature to accept ssh_credentials**:

   ```rust
   pub async fn execute(
       &self,
       ssh_credentials: &SshCredentials,  // NEW parameter
   ) -> Result<IpAddr, ProvisionCommandError> {
       info!("Starting complete infrastructure provisioning workflow");

       self.render_templates().await?;
       self.create_instance()?;

       let instance_info = self.get_instance_info()?;
       let instance_ip = instance_info.ip_address;

       self.render_ansible_templates(instance_ip, ssh_credentials).await?;
       self.validate_connectivity(instance_ip, ssh_credentials).await?;

       info!(instance_ip = %instance_ip, "Provisioning completed successfully");
       Ok(instance_ip)
   }
   ```

**Benefits**:

- Cleaner struct (4 dependencies instead of 5)
- Prepares for `Environment<S>` parameter (we'll get ssh_credentials from it)
- Makes data flow explicit (ssh_credentials passed when needed)
- Removes redundant storage

**Tests to Update**:

- Update E2E tests to pass ssh_credentials to `execute()`
- Update unit tests to not pass ssh_credentials to constructor
- All tests should still pass

**Success Criteria**:

- ✅ `ssh_credentials` removed from struct
- ✅ Constructor simplified
- ✅ `execute()` accepts ssh_credentials parameter
- ✅ All tests updated and passing
- ✅ No behavior changes

**Commit**: `refactor: pass SshCredentials as parameter instead of storing in ProvisionCommand`

---

### Task 3: Add Error Context Extraction Helpers

**Purpose**: Prepare for error state tracking by adding helper methods to extract failed step information.

**Why**: In Phase 5, when a command fails, we need to:

- Transition to error state (e.g., `ProvisionFailed`)
- Include the failed step name
- Persist this information

**Changes**:

1. **Add helper method to ProvisionCommand**:

   ```rust
   /// Extract the failed step name from an error for error state context
   ///
   /// This method will be used in Phase 5 to provide context when transitioning
   /// to error states like `ProvisionFailed { failed_step: String }`.
   fn extract_failed_step_name(&self, error: &ProvisionCommandError) -> String {
       match error {
           ProvisionCommandError::OpenTofuTemplateRendering(_) => {
               "render_opentofu_templates".to_string()
           }
           ProvisionCommandError::OpenTofu(tofu_error) => {
               self.extract_opentofu_failed_step(tofu_error)
           }
           ProvisionCommandError::AnsibleTemplateRendering(_) => {
               "render_ansible_templates".to_string()
           }
           ProvisionCommandError::SshConnectivity(_) => {
               "wait_ssh_connectivity".to_string()
           }
           ProvisionCommandError::Command(_) => {
               "wait_cloud_init".to_string()
           }
       }
   }

   /// Extract more specific step information from OpenTofu errors
   fn extract_opentofu_failed_step(&self, error: &OpenTofuError) -> String {
       // For now, return generic name. In future, could inspect error details
       // to determine if it was init, validate, plan, or apply that failed.
       "opentofu_operation".to_string()
   }
   ```

2. **Add similar helper to ConfigureCommand**:

   ```rust
   /// Extract the failed step name from an error for error state context
   ///
   /// This method will be used in Phase 5 to provide context when transitioning
   /// to error states like `ConfigureFailed { failed_step: String }`.
   fn extract_failed_step_name(&self, error: &ConfigureCommandError) -> String {
       match error {
           ConfigureCommandError::Command(cmd_error) => {
               // Try to infer which step failed from command error
               // For now, return generic name since both steps use CommandError
               "configuration_step".to_string()
           }
       }
   }
   ```

**Benefits**:

- Infrastructure ready for Phase 5 error tracking
- Helper methods are pure functions (easy to test)
- Doesn't change current behavior (methods not used yet)
- Makes Phase 5 implementation cleaner

**Tests to Add**:

- Test `extract_failed_step_name()` for each error variant
- Verify correct step names are returned

**Success Criteria**:

- ✅ Helper methods added to both commands
- ✅ Unit tests for helper methods pass
- ✅ Methods documented with Phase 5 purpose
- ✅ All existing tests still pass

**Commit**: `refactor: add error context extraction helpers for future state tracking`

---

### Task 4: Add Persistence Injection Points (Documentation)

**Purpose**: Document where persistence calls will go in Phase 5 without implementing them yet.

**Changes**:

1. **Add documentation comments in ProvisionCommand `execute()`**:

   ```rust
   pub async fn execute(
       &self,
       ssh_credentials: &SshCredentials,
   ) -> Result<IpAddr, ProvisionCommandError> {
       info!("Starting complete infrastructure provisioning workflow");

       // TODO [Phase 5]: Transition to Provisioning state
       // TODO [Phase 5]: Persist Provisioning state

       self.render_templates().await?;
       self.create_instance()?;

       let instance_info = self.get_instance_info()?;
       let instance_ip = instance_info.ip_address;

       self.render_ansible_templates(instance_ip, ssh_credentials).await?;
       self.validate_connectivity(instance_ip, ssh_credentials).await?;

       info!(instance_ip = %instance_ip, "Provisioning completed successfully");

       // TODO [Phase 5]: Transition to Provisioned state
       // TODO [Phase 5]: Persist Provisioned state

       Ok(instance_ip)
   }
   ```

2. **Add similar comments to ConfigureCommand**:

   ```rust
   pub fn execute(&self) -> Result<(), ConfigureCommandError> {
       info!("Starting complete infrastructure configuration workflow");

       // TODO [Phase 5]: Transition to Configuring state
       // TODO [Phase 5]: Persist Configuring state

       InstallDockerStep::new(Arc::clone(&self.ansible_client)).execute()?;
       InstallDockerComposeStep::new(Arc::clone(&self.ansible_client)).execute()?;

       info!("Infrastructure configuration completed successfully");

       // TODO [Phase 5]: Transition to Configured state
       // TODO [Phase 5]: Persist Configured state

       Ok(())
   }
   ```

**Benefits**:

- Makes Phase 5 implementation obvious
- Documents the design intent
- No code changes (just comments)
- Helps reviewers understand preparation

**Success Criteria**:

- ✅ TODO comments added at all injection points
- ✅ Comments reference Phase 5
- ✅ Clear indication of what will be added

**Commit**: `docs: add Phase 5 injection point markers in command handlers`

---

## 📊 Summary

### Refactoring Impact

| Metric                           | Before | After | Change                       |
| -------------------------------- | ------ | ----- | ---------------------------- |
| ProvisionCommand LOC             | ~180   | ~220  | +40 (extracted methods)      |
| ConfigureCommand LOC             | ~100   | ~110  | +10 (helper + comments)      |
| ProvisionCommand `execute()` LOC | ~50    | ~20   | -30 (clearer!)               |
| ProvisionCommand dependencies    | 5      | 4     | -1 (removed ssh_credentials) |
| Extracted methods (Provision)    | 1      | 5     | +4                           |
| Helper methods                   | 0      | 2     | +2                           |
| Test files updated               | 2      | 2     | 0 (same tests)               |

### Benefits for Phase 5

After this refactoring, Phase 5 implementation becomes:

1. **Add Environment parameter**:

   - Replace `ssh_credentials: &SshCredentials` with `environment: Environment<Created>`
   - Access ssh_credentials via `environment.ssh_credentials()`

2. **Add state transitions**:

   - Replace TODO comments with actual transition calls
   - Use extracted phase methods as transition boundaries

3. **Add persistence**:

   - Replace TODO comments with repository.save() calls
   - Use helper methods for error context

4. **Change return type**:
   - Replace `IpAddr` with `Environment<Provisioned>`
   - Replace `()` with `Environment<Configured>`

**Each change is now isolated and obvious!**

### Testing Strategy

1. **After each task**:

   - Run all existing tests
   - Verify no behavior changes
   - Run linters

2. **Before Phase 5**:
   - All 663 tests passing
   - No breaking changes to E2E tests
   - Clean linter output

### Commit Strategy

Each task is one commit:

1. `refactor: extract logical phases in ProvisionCommand for better clarity`
2. `refactor: pass SshCredentials as parameter instead of storing in ProvisionCommand`
3. `refactor: add error context extraction helpers for future state tracking`
4. `docs: add Phase 5 injection point markers in command handlers`

Total: **4 focused commits** before starting Phase 5 implementation.

## 🎯 Success Criteria

- ✅ All 4 tasks completed
- ✅ All existing tests passing (663 tests)
- ✅ No behavior changes to commands
- ✅ No breaking changes to E2E tests
- ✅ All linters passing
- ✅ Code more maintainable and testable
- ✅ Phase 5 implementation path is clear and obvious
- ✅ Each refactoring can be reviewed independently

## 📚 Related Documentation

- [Phase 5 Implementation Plan](../features/environment-state-management/implementation-plan/phase-5-command-integration.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Development Principles](../development-principles.md)
