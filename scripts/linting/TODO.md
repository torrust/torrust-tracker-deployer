# Linting Scripts TODO

## Current Technical Debt

### Bash Scripting Limitations

The current linting scripts are implemented in bash, which has several limitations:

- **Hard to unit test**: Bash scripts are difficult to test in isolation and require
  complex testing frameworks
- **Hard to write reusable components**: Code duplication and lack of proper modularity

### Code Duplication Issues

- **Logging functions duplicated**: The tracing-style logging functions (`log_info`,
  `log_success`, `log_error`, `log_warning`) are duplicated across all scripts:
  - `scripts/linting/markdown.sh`
  - `scripts/linting/yaml.sh`
  - `scripts/linting/clippy.sh`
  - `scripts/linting/shellcheck.sh`

These should be extracted into a shared utility script, but we're not doing it for now
due to bash's limitations.

### Output Format Limitations

The current human-readable output format has limitations for automated environments:

- **CI/CD Integration**: The current tracing-style output is great for human readability
  during development, but it's not optimal for parsing in CI/CD pipelines
- **Structured Data**: We might want to output results in JSON format for easier parsing
  and integration with other tools
- **Machine Consumption**: JSON output would enable better integration with:
  - GitHub Actions annotations
  - Code quality reporting tools
  - Custom dashboard integrations
  - Automated metrics collection

**Potential JSON output structure:**

```json
{
  "timestamp": "2025-09-02T16:51:02.167445Z",
  "summary": {
    "total_linters": 5,
    "passed": 3,
    "failed": 2,
    "status": "failed"
  },
  "results": [
    {
      "linter": "markdown",
      "status": "failed",
      "duration_ms": 123,
      "issues_count": 15,
      "files_checked": 8
    },
    {
      "linter": "rustfmt",
      "status": "passed",
      "duration_ms": 67,
      "issues_count": 0,
      "files_checked": 12
    }
  ]
}
```

### Integration as Rust Binary

Another consideration is integrating the linter as a binary within the main Rust package:

- **Unified tooling**: The linter could be `cargo run --bin linter` instead of separate scripts
- **Better testing**: Proper unit and integration tests for linting logic
- **Configuration**: Use Rust configuration libraries for better config management
- **Performance**: Native Rust performance for file discovery and orchestration
- **Consistency**: Same language as the main application

## Future Refactoring Plans

### Migration to Rust

If the scripts become much more complex, we will refactor them to use Rust instead of bash:

**Benefits of Rust migration:**

- **Testability**: Proper unit testing with `cargo test`
- **Modularity**: Shared libraries and reusable components
- **Type safety**: Compile-time error checking
- **Performance**: Better performance for complex operations
- **Maintainability**: Better code organization and documentation
- **Error handling**: Robust error handling with `Result<T, E>`

**Potential structure:**

```rust
src/
  linting/
    mod.rs
    utils/
      logging.rs      # Shared logging utilities
      installer.rs    # Tool installation logic
    linters/
      markdown.rs     # Markdown linting implementation
      yaml.rs         # YAML linting implementation
      shellcheck.rs   # ShellCheck linting implementation
      clippy.rs       # Clippy linting implementation
    main.rs           # CLI interface
```

## Decision Criteria

We will consider migrating to Rust when:

- Scripts exceed ~100 lines each
- Complex logic requires proper testing
- Installation logic becomes more sophisticated
- Integration with other Rust tools becomes necessary
- More linters are added (5+ different tools)
- **CI/CD integration needs improve**: JSON output becomes a requirement
- **Performance becomes critical**: File processing speed matters for large codebases
- **Advanced features needed**: Configuration management, plugin system, or custom reporting

## Migration Options

### Option 1: Enhanced Bash Scripts

- Add `--json` flag to existing scripts for structured output
- Keep current architecture but improve CI/CD integration
- Quick implementation but maintains bash limitations

### Option 2: Rust Binary Integration

- Implement as `cargo run --bin linter` in the main Rust workspace
- Full Rust benefits: testing, performance, type safety
- Better integration with existing Rust toolchain
- More development effort but long-term maintainability

## Current Status

✅ **Acceptable for now**: Current bash implementation works well for simple linting tasks
⚠️ **Monitor complexity**: Keep an eye on script growth and complexity
🔄 **Ready to refactor**: Migration plan is ready when needed
