# Linting Conventions and Tools

This document covers the linting tools, configurations, and conventions used in the Torrust Tracker Deployer project.

## 🔧 Linting Tools Overview

We use multiple linting tools to maintain code quality across different file types:

| Tool               | Purpose                       | File Types        | Configuration               |
| ------------------ | ----------------------------- | ----------------- | --------------------------- |
| `markdownlint-cli` | Markdown formatting and style | `*.md`            | `.markdownlint.json`        |
| `yamllint`         | YAML syntax and style         | `*.yml`, `*.yaml` | `.yamllint-ci.yml`          |
| `taplo`            | TOML formatting and linting   | `*.toml`          | `.taplo.toml`               |
| `cspell`           | Spell checking                | All text files    | `cspell.json`               |
| `shellcheck`       | Shell script analysis         | `*.sh`, `*.bash`  | Built-in rules              |
| `clippy`           | Rust code analysis            | `*.rs`            | `Cargo.toml` + command args |
| `rustfmt`          | Rust code formatting          | `*.rs`            | `rustfmt.toml` (default)    |

## 🚀 Quick Start

**Run all linters** (recommended):

```bash
cargo run --bin linter all
```

**Run individual linters**:

```bash
# Individual linters
cargo run --bin linter markdown
```

**YAML linting**:

```bash
cargo run --bin linter yaml
```

**TOML linting**:

```bash
cargo run --bin linter toml
```

**Spell checking**:

```bash
cargo run --bin linter cspell
```

**Rust code analysis**:

```bash
cargo run --bin linter clippy
```

**Rust formatting**:

```bash
cargo run --bin linter rustfmt
```

**Shell script linting**:

```bash
cargo run --bin linter shellcheck
```

### Linting Implementation

All linting is managed through a unified Rust binary (`src/bin/linter.rs`) that wraps the individual linting tools. This provides:

- **Consistent interface**: Single command structure across all linters
- **Better error handling**: Structured error messages and exit codes
- **Unified logging**: Consistent output formatting
- **Easy extensibility**: Add new linters by implementing the `Linter` trait

The linter binary is part of the `torrust-linting` package (`packages/linting/`), which provides a reusable linting framework.

### Alternative: Shell Script Wrapper

A convenience wrapper script is available:

```bash
# Wrapper that calls the Rust binary
./scripts/lint.sh
```

This script simply invokes `cargo run --bin linter all` and is provided for backwards compatibility.

## 📋 Tool-Specific Guidelines

### Markdown Linting (`markdownlint-cli`)

**Configuration**: `.markdownlint.json`

Key rules enabled:

- ✅ **MD031**: Fenced code blocks surrounded by blank lines
- ✅ **MD032**: Lists surrounded by blank lines
- ✅ **MD040**: Fenced code blocks have language specified
- ✅ **MD022**: Headings surrounded by blank lines
- ✅ **MD009**: No trailing spaces
- ❌ **MD013**: Line length (disabled for flexibility)
- ❌ **MD041**: First line in file should be top-level heading (disabled)
- ❌ **MD060**: Table column style (disabled - allows flexible table formatting and emoji usage)

**Common fixes**:

```bash
# Add language to code blocks
# Bad:
```

```text
code here
```

```bash
# Good:
```

```bash
code here
```

```text
# Add blank lines around headings and lists
```

### YAML Linting (`yamllint`)

**Configuration**: `.yamllint-ci.yml`

Key settings:

- **Line length**: 200 characters (extended for infrastructure code)
- **Comments**: Minimum 1 space from content
- **Document start**: Disabled (for cloud-init compatibility)
- **Truthy values**: Allows common values (`true`, `false`, `yes`, `no`, `on`, `off`)

**Common fixes**:

```yaml
# Ensure proper indentation (2 spaces)
services:
  web:
    image: nginx
    ports:
      - "80:80"

# Use consistent quotes
name: "my-service"
version: "1.0" # Consistent with project style
```

### TOML Linting (`taplo`)

**Configuration**: `.taplo.toml`

Key settings:

- **Formatting**: Preserves blank lines and doesn't reorder keys
- **Arrays**: Trailing commas enabled, consistent expansion
- **Alignment**: Comments aligned, entries not aligned for readability
- **Indentation**: Tables and entries maintain natural structure

**Common fixes**:

```toml
# Use consistent formatting
[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = "1.0"

# Arrays with trailing commas
features = [
    "derive",
    "serde",
]

# Proper spacing around values
name = "torrust-tracker"  # Good
name="torrust-tracker"    # Bad - needs spaces
```

**Auto-fix formatting**:

```bash
# Fix all TOML files automatically
taplo fmt **/*.toml
```

### Spell Checking (`cspell`)

**Configuration**: `cspell.json`

Key settings:

- **Custom dictionary**: `project-words.txt` for project-specific terms
- **Language**: English (US)
- **File types**: All text files (markdown, code, configs)

**Common workflow**:

```bash
# Add new words to project dictionary
echo "torrust" >> project-words.txt
echo "opentofu" >> project-words.txt

# Run spell check
cargo run --bin linter cspell
```

### Excluded Directories

**Important**: The following directories contain **generated or runtime data** and are excluded from all linting:

- `build/` - Generated build artifacts and rendered templates
- `data/` - Runtime application data and test outputs
- `envs/` - User environment configurations (JSON files)

These directories are configured to be ignored in:

- `.taplo.toml` - TOML linting exclusions
- `.markdownlint.json` - Markdown linting exclusions (via `ignores`)
- `.yamllint-ci.yml` - YAML linting exclusions (via `ignore`)
- `cspell.json` - Spell check exclusions (via `ignorePaths`)

**Why exclude these folders?**

1. **Generated content**: Linting generated files creates noise and false positives
2. **User data**: Environment configs are user-specific and may not follow project conventions
3. **Test artifacts**: Temporary test data shouldn't affect linting status
4. **Performance**: Excluding these folders significantly speeds up linting

If you add a new linting tool, ensure these directories are excluded from its scope.

### Shell Script Linting (`shellcheck`)

**Configuration**: Built-in ShellCheck rules

**Common fixes**:

```bash
# Quote variables to prevent word splitting
echo "$variable" # Good
echo $variable   # Bad

# Use [[ ]] instead of [ ] for conditionals
if [[ "$var" == "value" ]]; then # Good
if [ "$var" == "value" ]; then   # OK but prefer [[]]

# Check command existence
if command -v docker &> /dev/null; then # Good
if which docker; then                   # Less portable
```

### Rust Code Analysis (`clippy`)

**Configuration**: Command-line arguments in `scripts/linting/clippy.sh`

Enabled lint groups:

- **Correctness**: `-D clippy::correctness`
- **Suspicious**: `-D clippy::suspicious`
- **Complexity**: `-D clippy::complexity`
- **Performance**: `-D clippy::perf`
- **Style**: `-D clippy::style`
- **Pedantic**: `-D clippy::pedantic`

**Common fixes**:

```rust
// Use ? operator instead of unwrap
let value = some_function()?; // Good
let value = some_function().unwrap(); // Avoid in production code

// Prefer matches! macro for simple boolean checks
if matches!(status, Status::Ready) // Good
if status == Status::Ready // Also fine, but matches! is more explicit

// Use clippy suggestions for better performance
let items: Vec<_> = iterator.collect(); // Often suggested improvements
```

### Rust Formatting (`rustfmt`)

**Configuration**: Default `rustfmt` settings

**Automatic formatting**:

```bash
# Format code (modifies files)
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

## 📁 Configuration Files

### `.markdownlint.json`

```json
{
  "default": true,
  "MD013": false, // Line length disabled
  "MD031": true, // Fenced code blocks surrounded by blank lines
  "MD032": true, // Lists surrounded by blank lines
  "MD040": true, // Fenced code blocks have language
  "MD022": true, // Headings surrounded by blank lines
  "MD009": true, // No trailing spaces
  "MD007": {
    // Unordered list indentation
    "indent": 2
  },
  "MD026": false, // Trailing punctuation in headings
  "MD041": false, // First line in file should be top-level heading
  "MD034": false, // Bare URL used
  "MD024": false, // Multiple headings with same content
  "MD033": false // Inline HTML
}
```

### `.yamllint-ci.yml`

```yaml
extends: default

rules:
  line-length:
    max: 200 # More reasonable for infrastructure code
  comments:
    min-spaces-from-content: 1 # Allow single space before comments
  document-start: disable # Cloud-init files don't need --- start
  truthy:
    allowed-values: ["true", "false", "yes", "no", "on", "off"]

# Ignore cloud-init files for comment spacing
ignore: |
  **/cloud-init.yml
```

## 🔄 CI/CD Integration

### GitHub Actions Workflow

The same linting binary runs in CI/CD (`.github/workflows/linting.yml`):

```yaml
- name: Build Rust linter
  run: cargo build --release --bin linter

- name: Run all linters
  run: ./target/release/linter all
```

This ensures **consistency between local development and CI environments**.

### Pre-commit Integration

Integrate linting into your Git workflow:

```bash
#!/bin/bash
# .git/hooks/pre-commit
if ! cargo run --bin linter all; then
    echo "❌ Linting failed. Please fix issues before committing."
    exit 1
fi
```

## 📦 Tool Installation

The linting scripts automatically install required tools if missing:

### Automatic Installation

- **markdownlint-cli**: Installed via `npm install -g markdownlint-cli`
- **yamllint**: Installed via system package manager (`apt`, `dnf`, `pacman`) or `pip3`
- **shellcheck**: Installed via system package manager
- **clippy & rustfmt**: Installed as part of Rust toolchain

### Manual Installation

```bash
# Node.js tools
npm install -g markdownlint-cli

# Python tools
pip3 install yamllint

# System tools (Ubuntu/Debian)
sudo apt install shellcheck

# Rust tools
rustup component add clippy rustfmt
```

## 🎯 Best Practices

### Before Committing

1. **Always run linters**: `cargo run --bin linter all`
2. **Fix all issues**: Don't commit with linting errors
3. **Understand the rules**: Learn why rules exist, don't just fix blindly

### Code Organization

1. **Keep configs in root**: All linting configs should be in project root
2. **Document exceptions**: If you disable a rule, explain why
3. **Consistent style**: Follow existing patterns in the codebase

### Performance Tips

```bash
# Run specific linters for faster feedback during development
cargo run --bin linter markdown    # Only markdown (~1s)
cargo run --bin linter yaml        # Only YAML files (~0.2s)
cargo run --bin linter toml        # Only TOML files (~0.1s)
cargo run --bin linter cspell      # Spell check (~2.5s)
cargo run --bin linter clippy      # Only Rust analysis (~12s - slowest)

# Run non-Rust linters for quick checks
cargo run --bin linter markdown
cargo run --bin linter yaml
cargo run --bin linter toml
cargo run --bin linter cspell
# Skip clippy for faster iteration during active development
```

**Tip**: The linter binary runs tools sequentially with clean output. For fastest iteration during development, run only the linter relevant to the files you're editing.

## 🚨 Troubleshooting

### Common Issues

**Linter not found**:

```bash
# The scripts auto-install tools, but if it fails:
npm install -g markdownlint-cli  # For markdown
pip3 install yamllint           # For YAML
```

**Permission errors**:

```bash
# Make scripts executable
chmod +x scripts/linting/*.sh
```

**Rust toolchain issues**:

```bash
# Ensure clippy and rustfmt are installed
rustup component add clippy rustfmt
```

### Getting Help

1. **Check existing issues**: Look for similar problems in GitHub issues
2. **Run with verbose output**: Add `-v` or `--verbose` flags where available
3. **Manual tool execution**: Try running tools directly to isolate issues

## 📊 Linting Statistics

Track your linting improvements:

```bash
# Count linting issues over time
git log --grep="fix.*lint\|style:" --oneline | wc -l

# Check files that frequently need linting fixes
git log --name-only --grep="style\|lint" | sort | uniq -c | sort -nr
```
