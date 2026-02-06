# Linter Reference Documentation

This document provides detailed information about each linter used in the Torrust Tracker Deployer project.

## Table of Contents

- [Markdown Linting](#markdown-linting)
- [YAML Linting](#yaml-linting)
- [TOML Linting](#toml-linting)
- [Spell Checking](#spell-checking)
- [Rust Linting](#rust-linting)
- [Shell Script Linting](#shell-script-linting)

## Markdown Linting

**Tool**: markdownlint  
**Configuration**: `.markdownlint.json`  
**Command**: `cargo run --bin linter markdown`

### Key Rules

- **MD013**: Line length limit (disabled for tables)
- **MD033**: Inline HTML (disabled)
- **MD041**: First line must be top-level heading
- **MD046**: Code block style (fenced preferred)

### Common Issues

| Issue                | Fix                                      |
| -------------------- | ---------------------------------------- |
| Lines too long       | Break into multiple lines (< 120 chars)  |
| Trailing spaces      | Remove spaces at end of lines            |
| Inconsistent heading | Use ATX style (`#` prefix)               |
| Bare URLs            | Wrap in angle brackets `<url>` or `[]()` |

### Configuration Example

```json
{
  "MD013": {
    "line_length": 120,
    "tables": false
  },
  "MD033": false,
  "MD041": true
}
```

## YAML Linting

**Tool**: yamllint  
**Configuration**: `.yamllint-ci.yml`  
**Command**: `cargo run --bin linter yaml`

### Key Rules

- **Indentation**: 2 spaces (consistent)
- **Line length**: 120 characters max
- **Trailing spaces**: Not allowed
- **Document start**: `---` marker required

### Common Issues

| Issue                  | Fix                       |
| ---------------------- | ------------------------- |
| Wrong indentation      | Use 2 spaces consistently |
| Line too long          | Break into multiple lines |
| Missing document start | Add `---` at file start   |
| Trailing spaces        | Remove spaces at line end |

### Configuration Example

```yaml
rules:
  line-length:
    max: 120
    level: warning
  indentation:
    spaces: 2
    indent-sequences: consistent
```

## TOML Linting

**Tool**: taplo  
**Configuration**: `.taplo.toml`  
**Command**: `cargo run --bin linter toml`

### Key Rules

- **Formatting**: Consistent spacing and alignment
- **Key ordering**: Alphabetical within sections
- **Inline tables**: Compact format
- **Array formatting**: Consistent brackets and commas

### Common Issues

| Issue                | Fix                              |
| -------------------- | -------------------------------- |
| Inconsistent spacing | Run formatter (`taplo fmt`)      |
| Unordered keys       | Manually reorder or use tool     |
| Array formatting     | Use consistent bracket placement |

### Configuration Example

```toml
[formatting]
align_entries = true
column_width = 120
indent_string = "    "
```

## Spell Checking

**Tool**: cspell  
**Configuration**: `cspell.json`  
**Dictionary**: `project-words.txt`  
**Command**: `cargo run --bin linter cspell`

### Key Features

- **Custom dictionary**: Project-specific terms in `project-words.txt`
- **Ignore patterns**: Paths, URLs, hex values
- **Case sensitivity**: Smart detection
- **Programming terms**: Rust, CLI, API terminology included

### Common Issues

| Issue                     | Fix                                   |
| ------------------------- | ------------------------------------- |
| Unknown project term      | Add to `project-words.txt`            |
| Camel case not recognized | Add variations to dictionary          |
| False positives           | Add to `ignoreWords` in `cspell.json` |

### Adding Words to Dictionary

Edit `project-words.txt`:

```text
ansible
clippy
deployer
rustfmt
tofu
```

Words should be lowercase, one per line, sorted alphabetically.

## Rust Linting

### Clippy

**Tool**: cargo-clippy  
**Command**: `cargo run --bin linter clippy`

#### Key Checks

- **Correctness**: Potential bugs and errors
- **Performance**: Suboptimal code patterns
- **Style**: Idiomatic Rust conventions
- **Complexity**: Overly complex code
- **Pedantic**: Extra style checks (opt-in)

#### Common Issues

| Issue                       | Fix                             |
| --------------------------- | ------------------------------- |
| Unused variables            | Remove or prefix with `_`       |
| Unnecessary clones          | Use references instead          |
| Complex boolean expressions | Simplify or extract to function |
| Missing error propagation   | Use `?` operator                |

#### Auto-Fix

```bash
cargo clippy --fix --allow-dirty --allow-staged
```

### Rustfmt

**Tool**: rustfmt  
**Configuration**: `rustfmt.toml` (default)  
**Command**: `cargo run --bin linter rustfmt`

#### Key Formatting Rules

- **Indentation**: 4 spaces
- **Line width**: 100 characters
- **Imports**: Grouped and sorted
- **Trailing commas**: Required in multi-line

#### Common Issues

| Issue                | Fix                             |
| -------------------- | ------------------------------- |
| Inconsistent spacing | Run `cargo fmt`                 |
| Long lines           | Let rustfmt break automatically |
| Import order         | Rustfmt sorts automatically     |

#### Auto-Format

```bash
cargo fmt
```

## Shell Script Linting

**Tool**: shellcheck  
**Command**: `cargo run --bin linter shellcheck`

### Key Checks

- **Quoting**: Proper variable quoting
- **Conditionals**: Correct test syntax
- **Portability**: POSIX compliance
- **Safety**: Potential command injection

### Common Issues

| Issue                      | Fix                                      |
| -------------------------- | ---------------------------------------- |
| Unquoted variables         | Use `"$variable"` instead of `$variable` |
| Use of `[` instead of `[[` | Use `[[` for bash, `[` for POSIX         |
| Missing error handling     | Add `set -e` or check exit codes         |
| Unnecessary `cat`          | Use `< file` instead of `cat file \|`    |

### ShellCheck Warning Codes

| Code   | Description                      |
| ------ | -------------------------------- |
| SC2086 | Double quote to prevent globbing |
| SC2046 | Quote parameter expansion        |
| SC2006 | Use `$()` instead of backticks   |
| SC2155 | Declare and assign separately    |

## Linting Framework Architecture

The unified linting framework is implemented in `packages/linting/`:

```text
packages/linting/
├── src/
│   ├── linters/          # Individual linter implementations
│   │   ├── markdown.rs
│   │   ├── yaml.rs
│   │   ├── toml.rs
│   │   ├── cspell.rs
│   │   ├── clippy.rs
│   │   ├── rustfmt.rs
│   │   └── shellcheck.rs
│   ├── runner.rs         # Execution logic
│   └── lib.rs
└── README.md
```

### Benefits

- **Unified interface**: Single binary for all linters
- **Parallel execution**: Fast linting with `--parallel` option
- **Consistent output**: Standardized error reporting
- **Easy extension**: Add new linters by implementing trait

## CI Integration

All linters run automatically in GitHub Actions CI pipeline:

```yaml
- name: Run linters
  run: cargo run --bin linter all
```

Local checks must pass before CI to avoid failures.

## Performance Tips

1. **Run individual linters** during development for fast feedback
2. **Use parallel mode** for comprehensive checks: `cargo run --bin linter all --parallel`
3. **Cache linter binaries** (CI does this automatically)
4. **Fix formatting early** to avoid compound issues

## Troubleshooting

### Linter Binary Not Found

**Issue**: `cargo run --bin linter` fails

**Solution**:

```bash
cargo build --bin linter
cargo run --bin linter --help
```

### Linter Hangs or Slow

**Issue**: Linting takes too long

**Solution**:

- Run individual linters instead of `all`
- Check for large files being processed
- Verify no infinite loops in shell scripts

### False Positives

**Issue**: Linter reports errors that shouldn't be errors

**Solution**:

- Check configuration files for overly strict rules
- Add exceptions to ignore lists
- Report issue if it's a bug in the linter tool

## References

- [Linting Guide](../../../docs/contributing/linting.md)
- [Linting Framework README](../../../packages/linting/README.md)
- [Pre-Commit Process](../../../docs/contributing/commit-process.md)
