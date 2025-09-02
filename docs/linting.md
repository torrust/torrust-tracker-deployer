# Linting

This project uses automated linting to maintain code quality and consistency across different file types.

## Available Linters

- **Markdown**: Uses `markdownlint-cli` with configuration in `.markdownlint.json`
- **YAML**: Uses `yamllint` with configuration in `.yamllint-ci.yml`
- **Shell Scripts**: Uses `ShellCheck` for bash/shell script analysis
- **Rust Code**: Uses `clippy` for Rust code analysis
- **Rust Formatting**: Uses `rustfmt` to check code formatting

## Usage

### Quick Commands (Recommended)

```bash
# Run all linters
./lint all

# Run specific linters
./lint md         # Markdown only
./lint yaml       # YAML only
./lint clippy     # Rust code analysis only
./lint rustfmt    # Rust formatting check only
./lint shellcheck # Shell scripts only

# Show help
./lint help
```

### Direct Script Execution

```bash
# Run all linters
./scripts/linting/lint.sh all

# Run specific linters
./scripts/linting/markdown.sh
./scripts/linting/yaml.sh
./scripts/linting/clippy.sh
./scripts/linting/rustfmt.sh
./scripts/linting/shellcheck.sh
```

## Installation

The scripts will automatically install the required tools if they're not already present:

- **markdownlint-cli**: Installed via npm
- **yamllint**: Installed via system package manager (apt, dnf, pacman) or pip3
- **ShellCheck**: Installed via system package manager (apt, dnf, pacman, brew)
- **Rust clippy & rustfmt**: Installed as part of the Rust toolchain

## CI/CD Integration

The same scripts are used in GitHub Actions, ensuring consistency between local development
and CI environments. The workflow runs on every push and pull request.

## Configuration

- **Markdown**: `.markdownlint.json` - Controls line length, heading styles, etc.
- **YAML**: `.yamllint-ci.yml` - Controls line length, indentation, etc.

## Benefits

✅ **Consistent formatting** across all team members  
✅ **Automatic tool installation** for easy setup  
✅ **Same scripts** used locally and in CI  
✅ **Clean, readable output** with emoji indicators  
✅ **Cross-platform support** for different package managers
