# Linting Conventions and Tools

This document covers the linting tools, configurations, and conventions used in the Torrust Tracker Deploy project.

## 🔧 Linting Tools Overview

We use multiple linting tools to maintain code quality across different file types:

| Tool               | Purpose                       | File Types        | Configuration               |
| ------------------ | ----------------------------- | ----------------- | --------------------------- |
| `markdownlint-cli` | Markdown formatting and style | `*.md`            | `.markdownlint.json`        |
| `yamllint`         | YAML syntax and style         | `*.yml`, `*.yaml` | `.yamllint-ci.yml`          |
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

### Direct Script Execution

```bash
# Direct script calls (alternative approach)
./scripts/linting/markdown.sh
./scripts/linting/yaml.sh
./scripts/linting/clippy.sh
./scripts/linting/rustfmt.sh
./scripts/linting/shellcheck.sh
```

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
cargo run --bin linter markdown    # Only markdown (fastest)
cargo run --bin linter yaml        # Only YAML files
cargo run --bin linter clippy      # Only Rust analysis (slowest)
```

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
