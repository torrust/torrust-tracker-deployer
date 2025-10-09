# User Output vs Internal Logging: Architectural Decision

## 🎯 Decision

**Keep user output completely separate from internal logging**, even at high verbosity levels.

## 📋 Rationale

### Different Purposes

| Aspect       | User Output                       | Internal Logging                       |
| ------------ | --------------------------------- | -------------------------------------- |
| **Audience** | End users                         | Developers & ops teams                 |
| **Purpose**  | Progress, guidance, actionability | Debugging, traceability, observability |
| **Format**   | Human-friendly, polished          | Structured, machine-parseable          |
| **Lifetime** | Ephemeral (CLI session)           | Persistent (log files, traces)         |
| **Content**  | What users need to know           | What developers need to debug          |

### Benefits of Separation

1. **Independent Evolution**: Change one without affecting the other
2. **Audience Optimization**: Each optimized for its specific audience
3. **Principle Alignment**: Supports observability AND user-friendliness
4. **Flexibility**: Different output formats don't affect logging structure
5. **Maintainability**: Clear separation of concerns

## 🏗️ Implementation Pattern

### Basic Structure

```rust
use tracing::{info, debug};

pub struct UserOutput {
    verbosity: VerbosityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    Quiet,      // -q: Minimal output
    Normal,     // Default: Essential progress
    Verbose,    // -v: Detailed progress
    VeryVerbose, // -vv: Including decisions & retries
    Debug,      // -vvv: Maximum detail for troubleshooting
}

impl UserOutput {
    pub fn new(verbosity: VerbosityLevel) -> Self {
        Self { verbosity }
    }

    /// Show progress for a long-running operation
    pub fn progress(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            eprintln!("⏳ {}", message);
        }
    }

    /// Show detailed operation information
    pub fn detail(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Verbose {
            eprintln!("📋 {}", message);
        }
    }

    /// Show technical details for troubleshooting
    pub fn debug(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Debug {
            eprintln!("🔍 {}", message);
        }
    }

    /// Always show success messages
    pub fn success(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            eprintln!("✅ {}", message);
        }
    }

    /// Always show warnings
    pub fn warn(&self, message: &str) {
        if self.verbosity >= VerbosityLevel::Normal {
            eprintln!("⚠️  {}", message);
        }
    }
}
```

### Usage Example

```rust
pub async fn wait_for_ssh_connectivity(
    host: &AnsibleHost,
    timeout: Duration,
    user_output: &UserOutput,
) -> Result<(), ConnectivityError> {
    let max_attempts = 30;

    // User output: What's happening now (user-facing)
    user_output.progress(&format!("Waiting for instance at {} to be ready", host));

    // Internal logging: Technical details (developer-facing)
    info!(
        host = %host,
        timeout = ?timeout,
        max_attempts = max_attempts,
        "Starting SSH connectivity check"
    );

    for attempt in 1..=max_attempts {
        // User output: Progress update at verbose level
        user_output.detail(&format!(
            "SSH connection attempt {}/{} to {}",
            attempt, max_attempts, host
        ));

        // User output: Technical details at debug level
        user_output.debug(&format!(
            "Testing SSH: ssh -o ConnectTimeout=5 user@{}",
            host
        ));

        // Internal logging: Structured, always present
        debug!(
            attempt = attempt,
            max_attempts = max_attempts,
            host = %host,
            "Attempting SSH connection"
        );

        match test_ssh_connection(host).await {
            Ok(_) => {
                // User output: Success
                user_output.success(&format!("SSH connection established to {}", host));

                // Internal logging: Success with details
                info!(
                    host = %host,
                    attempts = attempt,
                    "SSH connectivity established"
                );

                return Ok(());
            }
            Err(e) => {
                // User output: Only show at debug level
                user_output.debug(&format!("Connection failed: {}", e));

                // Internal logging: Always log the error
                debug!(
                    error = %e,
                    attempt = attempt,
                    host = %host,
                    "SSH connection attempt failed"
                );
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    // User output: Failure with actionable guidance
    Err(ConnectivityError::SshTimeout {
        host: host.clone(),
        timeout,
        attempts: max_attempts,
    })
}
```

## 🎨 Verbosity Level Guidelines

### Quiet (`-q`)

- Only essential completion messages
- Errors and critical warnings
- Minimal distraction

**Example Output:**

```text
✅ Environment provisioned successfully
```

### Normal (default)

- High-level progress indicators
- Success/completion messages
- Important warnings
- Error messages

**Example Output:**

```text
⏳ Provisioning infrastructure...
⏳ Waiting for instance to be ready...
⏳ Configuring instance...
✅ Environment provisioned successfully
```

### Verbose (`-v`)

- Detailed progress for each major step
- What operations are being performed
- Resource names and identifiers

**Example Output:**

```text
⏳ Provisioning infrastructure...
📋 Creating LXD instance 'torrust-dev'
📋 Configuring cloud-init
📋 Starting instance
⏳ Waiting for instance to be ready...
📋 SSH connection attempt 1/30 to 10.140.190.14
📋 SSH connection attempt 2/30 to 10.140.190.14
✅ SSH connection established to 10.140.190.14
⏳ Configuring instance...
📋 Running Ansible playbook: site.yml
✅ Environment provisioned successfully
```

### Very Verbose (`-vv`)

- Decision points and retries
- Configuration values being used
- Sub-step details

**Example Output:**

```text
⏳ Provisioning infrastructure...
📋 Creating LXD instance 'torrust-dev'
📋 Instance configuration:
   - Image: ubuntu:22.04
   - CPU: 2 cores
   - Memory: 2048 MB
📋 Configuring cloud-init
📋 SSH key: /home/user/.ssh/testing_rsa.pub
📋 Starting instance
⏳ Waiting for instance to be ready...
📋 SSH connection attempt 1/30 to 10.140.190.14 (timeout: 5s)
📋 Connection refused, retrying in 5s...
📋 SSH connection attempt 2/30 to 10.140.190.14 (timeout: 5s)
✅ SSH connection established to 10.140.190.14
...
```

### Debug (`-vvv`)

- Technical details for troubleshooting
- Exact commands being executed
- File paths and configuration details
- Retry logic and timing information

**Example Output:**

```text
⏳ Provisioning infrastructure...
🔍 OpenTofu working directory: /path/to/build/e2e-full/tofu
🔍 Running: tofu init
🔍 Running: tofu apply -auto-approve
📋 Creating LXD instance 'torrust-dev'
🔍 Instance profile: default
🔍 Network: lxdbr0
📋 Instance configuration:
   - Image: ubuntu:22.04
   - CPU: 2 cores
   - Memory: 2048 MB
📋 Configuring cloud-init
🔍 Cloud-init template: /path/to/templates/cloud-init.yaml
📋 SSH key: /home/user/.ssh/testing_rsa.pub
🔍 SSH key fingerprint: SHA256:abc123...
📋 Starting instance
🔍 Executing: lxc start torrust-dev
⏳ Waiting for instance to be ready...
🔍 SSH timeout: 150s
🔍 Max attempts: 30
📋 SSH connection attempt 1/30 to 10.140.190.14 (timeout: 5s)
🔍 Testing SSH: ssh -o ConnectTimeout=5 user@10.140.190.14
📋 Connection refused, retrying in 5s...
📋 SSH connection attempt 2/30 to 10.140.190.14 (timeout: 5s)
🔍 Testing SSH: ssh -o ConnectTimeout=5 user@10.140.190.14
✅ SSH connection established to 10.140.190.14
...
```

## 🚫 What NOT To Do

### ❌ Don't Redirect Tracing Output to User

```rust
// DON'T DO THIS
if verbosity >= VerbosityLevel::Debug {
    // This exposes internal logging format to users
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_writer(std::io::stderr)
            .finish()
    )?;
}
```

**Why?** This breaks the separation and exposes technical log format to users.

### ❌ Don't Mix User Messages in Tracing Logs

```rust
// DON'T DO THIS
if verbosity >= VerbosityLevel::Verbose {
    info!("⏳ Waiting for instance to be ready...");
}
```

**Why?** Logs should remain consistent and not change based on user verbosity.

### ❌ Don't Duplicate Information

```rust
// DON'T DO THIS
tracing::info!("Creating instance: {}", name);
eprintln!("Creating instance: {}", name);
```

**Why?** Say different things! Users need progress, logs need context.

## ✅ Best Practices

### 1. User Output is About Progress and Context

```rust
// Users care about:
user_output.progress("Provisioning infrastructure...");
user_output.detail("Creating LXD instance 'torrust-dev'");
user_output.debug("Running: tofu apply -auto-approve");
```

### 2. Logging is About Traceability

```rust
// Developers care about:
info!(
    instance_name = %name,
    provider = "lxd",
    image = "ubuntu:22.04",
    "Creating VM instance"
);
```

### 3. They Can Say Different Things

```rust
// User sees progress in terms they understand
user_output.progress("Installing Tracker application...");

// Logs capture technical operation
info!(
    playbook = "deploy_tracker.yml",
    inventory = "torrust_servers",
    target = "torrust_vm",
    "Executing Ansible playbook"
);
```

## 🔄 Integration with Existing Logging

The current tracing infrastructure remains unchanged:

- **Structured logs** continue using `tracing` crate
- **Log levels** (TRACE, DEBUG, INFO, WARN, ERROR) control logging detail
- **Spans and events** provide observability and traceability
- **Log files and traces** capture everything for post-mortem analysis

User output is an **additional layer** that:

- Runs **alongside** logging (not instead of)
- Has its **own verbosity control** (independent of log levels)
- Provides **user-friendly** progress updates
- Never affects **internal logging** behavior

## 📚 Related Documentation

- [Development Principles](../../development-principles.md) - Observability and User Friendliness
- [Logging Guide](../../contributing/logging-guide.md) - Internal logging conventions
- [Console App Output Patterns](./console-app-output-patterns.md) - UX research and patterns
