# Console Application Output Patterns Research

> **📋 Research Document Status**  
> This document contains research findings on how popular console applications handle stdout/stderr separation and logging. These findings inform design decisions but do not represent current implementation choices.

## Overview

This research examines how popular console applications handle the challenge of separating **user-facing output** from **internal logging/diagnostics**. Console applications face a fundamental challenge: both user output and application logs typically compete for the same output channels (stdout and stderr), which can create confusion and poor user experience.

## The Core Problem

Console applications need to handle multiple types of output:

1. **User output**: Results, data, final outcomes meant for end users
2. **Progress information**: Status updates, progress indicators
3. **Diagnostic logs**: Debug info, internal operations, detailed execution traces
4. **Error messages**: User-facing errors vs. technical diagnostic errors
5. **Warnings and info**: Various levels of informational messages

The challenge is mapping these to the two available output channels (stdout and stderr) in a way that:

- Follows Unix conventions and user expectations
- Allows proper piping and redirection
- Provides clear separation for different use cases
- Supports various verbosity levels

## Traditional Unix Convention

The traditional Unix approach is simple and clear:

- **stdout**: Primary output, the "result" of the command
- **stderr**: Error messages and diagnostics

### Examples of Traditional Pattern

```bash
# ls - directory listing to stdout, errors to stderr
ls /some/directory > files.txt    # Results go to file
ls /nonexistent 2> errors.txt     # Errors go to file

# grep - matches to stdout, errors to stderr
cat file.txt | grep "pattern" | sort   # Clean pipeline
grep "pattern" /nonexistent 2>/dev/null # Suppress errors

# find - results to stdout, errors to stderr
find /etc -name "*.conf" 2>/dev/null | head -10
```

## Popular Application Patterns

### 1. Docker - Structured Logging with User Output

Docker uses a sophisticated approach that separates user output from logs:

**Normal Operations:**

```bash
# User output (container output) goes to stdout
docker run nginx
# Logs from container appear on stdout (container's stdout/stderr)

# Docker's own messages go to stderr
docker pull ubuntu  # Progress/status info to stderr
docker build .      # Build logs to stdout, Docker messages to stderr
```

**With verbosity:**

```bash
docker run --log-driver=json-file nginx   # Structured logging
docker logs container_name                # Retrieve logs separately
docker run -d nginx                       # Daemon mode, logs separate
```

**Key insights from Docker:**

- Container output (the actual result) goes to stdout
- Docker's operational messages go to stderr
- Supports structured logging with different drivers
- Daemon mode completely separates logging from user interaction

### 2. Cargo (Rust) - Clean User Experience

Cargo provides one of the cleanest console UX patterns:

**Default mode (clean):**

```bash
cargo build
   Compiling hello-world v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.75s

cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/hello-world`
Hello, world!    # <- This is the program output (stdout)
```

**Verbose mode (detailed):**

```bash
cargo build -v    # Much more detailed compilation info
cargo run -v      # Shows all the underlying commands being executed
```

**Key insights from Cargo:**

- Progress/status messages go to stderr (so they don't interfere with piping)
- Final program output goes to stdout
- Verbose mode shows all the underlying operations
- Clean separation allows: `cargo run | grep something` to work properly

### 3. Git - Mixed Approach with Context

Git uses a more complex pattern that varies by subcommand:

**Status and informational commands:**

```bash
git status        # All output to stdout (it's the result)
git log           # All output to stdout (it's the result)
git diff          # All output to stdout (it's the result)
```

**Operational commands:**

```bash
git clone https://...    # Progress to stderr, allows redirection
git push                 # Status messages to stderr
git pull                 # Progress/status to stderr
```

**Error handling:**

```bash
git checkout nonexistent-branch   # Error to stderr
git commit                        # Prompts/messages to stderr, success to stdout
```

**Key insights from Git:**

- Context matters - "query" commands put results on stdout
- "Action" commands put progress on stderr, results on stdout
- Interactive features (prompts, editor) use stderr

### 4. npm/yarn - Progress and Results Separation

Package managers show interesting patterns:

**npm:**

```bash
npm install    # Progress/warnings to stderr, allows clean piping
npm list       # Results to stdout
npm run test   # Test output to stdout, npm messages to stderr
```

**yarn:**

```bash
yarn install   # Progress indicators to stderr
yarn add pkg   # Success messages to stderr, allows piping
```

**Key insights:**

- Package manager operations put progress on stderr
- Actual command output (like test results) goes to stdout
- This allows piping package manager output: `npm test | grep "failed"`

### 5. curl - Data vs. Progress Separation

curl provides a clear example of data vs. metadata separation:

**Default:**

```bash
curl https://api.example.com/data
{"result": "data"}    # Response body to stdout

curl https://api.example.com/data -o file.json   # Body to file, progress to stderr
```

**Verbose:**

```bash
curl -v https://api.example.com/data
* Connected to api.example.com    # Debug info to stderr
* SSL connection established       # Debug info to stderr
{"result": "data"}                # Response body still to stdout
```

**Key insights:**

- The actual data (purpose of the command) goes to stdout
- All metadata, progress, debug info goes to stderr
- Verbose mode adds more stderr output but doesn't change stdout

### 6. rsync - Progress and Results

rsync shows how to handle long-running operations:

**Default:**

```bash
rsync -av source/ dest/
building file list ... done     # Progress to stderr
file1                           # File being transferred (stderr)
file2
sent 1,234 bytes  received 56 bytes  2,580.00 bytes/sec  # Summary to stderr
```

**Quiet mode:**

```bash
rsync -av --quiet source/ dest/    # Only errors to stderr, clean for scripts
```

**Key insights:**

- Progress information goes to stderr
- Allows scripting: `rsync --quiet ... && echo "success"`
- Summary and statistics are considered progress, not results

### 7. Terraform/OpenTofu - State and Logs

Infrastructure tools show complex logging patterns:

**Terraform:**

```bash
terraform apply
# Plan output to stdout (it's a result the user needs to see)
# Progress/status to stderr
# TF_LOG=DEBUG environment variable controls detailed logs
```

**Key insights:**

- Plan output is treated as "result" (stdout)
- Execution progress goes to stderr
- Debug logs are controlled via environment variables
- Supports structured logging for advanced use cases

## Pattern Analysis

### Common Strategies Identified

1. **Pure Unix Convention** (ls, cat, grep)

   - stdout: Command results/output
   - stderr: Errors only
   - Simple and predictable

2. **Progress/Results Separation** (curl, cargo, npm)

   - stdout: Final results/data
   - stderr: Progress, status, metadata
   - Excellent for piping and automation

3. **Context-Dependent** (git)

   - Query commands: Results to stdout
   - Action commands: Progress to stderr, results to stdout
   - More complex but intuitive per command

4. **Structured Logging** (docker, terraform)

   - Support for log drivers/files
   - Environment variable control
   - Separate logging infrastructure

5. **Verbosity-Controlled** (most modern tools)
   - Default: Minimal, user-focused output
   - -v: More operational details
   - -vv/-vvv: Debug/trace information

### Verbosity Patterns

Most modern CLI tools follow this pattern:

- **Default**: Essential user information only
- **-v/--verbose**: Operational details, progress info
- **-vv**: Debug information, internal operations
- **-vvv**: Trace-level, all internal details
- **-q/--quiet**: Suppress all non-essential output

## Recommendations for Console Applications

Based on this research, here are the identified best practices:

### 1. Follow the "Results vs. Progress" Pattern

- **stdout**: Final results, data that users want to pipe/redirect
- **stderr**: Progress updates, status messages, operational info

### 2. Implement Graduated Verbosity

```bash
myapp command              # Essential info only
myapp command -v           # + Progress/status
myapp command -vv          # + Debug info
myapp command -vvv         # + Trace/all details
myapp command -q           # Minimal output
```

### 3. Consider Structured Logging for Complex Apps

- Log files for persistent diagnostics
- JSON logs for machine parsing
- Separate debug logs controlled by environment variables

### 4. Make Errors Actionable

- Error messages should go to stderr
- Include context and suggested fixes
- Distinguish between user errors and system errors

### 5. Support Automation Use Cases

- Quiet modes for scripting
- Machine-readable output options (JSON, CSV)
- Exit codes that reflect operation success/failure

## Application to Torrust Deployer

For the Torrust Tracker Deployer application, these patterns suggest:

### Recommended Approach

1. **User Output** (stdout):

   - Deployment results
   - Configuration summaries
   - Final status reports

2. **Progress/Operational** (stderr):

   - Step progress ("Provisioning instance...")
   - Status updates ("Instance ready")
   - Non-critical warnings

3. **Debug Logs** (stderr + optional files):

   - Detailed operation logs
   - Ansible/Terraform output
   - System diagnostics

4. **Errors** (stderr):
   - User errors with actionable guidance
   - System errors with recovery instructions

### Example Implementation

```bash
# Default - clean user experience
torrust-tracker-deployer provision env1
✓ Instance provisioned successfully    # to stderr (progress)
Instance: env1-tracker (192.168.1.100) # to stdout (result)

# Verbose - show operational details
torrust-tracker-deployer provision env1 -v
→ Creating LXD instance...              # to stderr
→ Configuring network...               # to stderr
→ Installing packages...               # to stderr
✓ Instance provisioned successfully    # to stderr
Instance: env1-tracker (192.168.1.100) # to stdout

# Piping works cleanly
torrust-tracker-deployer provision env1 | jq .ip_address
torrust-tracker-deployer provision env1 -q > deployment.txt
```

This approach follows the successful patterns used by cargo, docker, and other modern CLI tools while maintaining Unix convention compatibility.

## References

- [UNIX Philosophy - Rule of Silence](http://www.catb.org/~esr/writings/taoup/html/ch01s06.html#id2877917)
- [GNU Coding Standards - Writing Robust Programs](https://www.gnu.org/prep/standards/html_node/Errors.html)
- [Cargo's output handling](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script)
- [Docker logging drivers](https://docs.docker.com/config/containers/logging/)
- [The Art of Command Line](https://github.com/jlevy/the-art-of-command-line)
