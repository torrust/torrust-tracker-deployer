# Meson Build System

[Meson](https://mesonbuild.com/) is a cross-platform build system that can be used as a task runner to wrap common development commands and make them easier to execute.

## Overview

Meson is a modern build system designed to be:

- **Fast**: Optimized for speed and parallel execution
- **User-friendly**: Simple, clean syntax
- **Cross-platform**: Works on Linux, macOS, Windows
- **Flexible**: Can be used for building projects or as a task runner

## Installation

### Ubuntu/Debian

```bash
sudo apt install meson
```

### macOS

```bash
brew install meson
```

### Other Systems

Check the [official installation guide](https://mesonbuild.com/Getting-meson.html) for your platform.

### Verify Installation

```bash
meson --version
```

## Basic Usage

### Initialize a Build Directory

```bash
meson setup builddir
```

This creates a `builddir/` directory with the Meson configuration.

### Basic Commands

```bash
# Configure the build
meson setup builddir

# Reconfigure (after changes to meson.build)
meson setup builddir --reconfigure

# Compile targets
meson compile -C builddir

# Run tests
meson test -C builddir

# Clean build directory
rm -rf builddir
```

## Using Meson as a Task Runner

Meson can be used to define custom commands and tasks in your `meson.build` file:

```meson
# Example task definition
run_target('lint',
  command: ['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings']
)

run_target('format',
  command: ['cargo', 'fmt']
)

run_target('test',
  command: ['cargo', 'test']
)
```

### Running Custom Tasks

```bash
# Run a custom task
meson compile -C builddir taskname
```

## Basic Project Structure

```text
project-root/
├── meson.build         # Meson configuration file
├── builddir/           # Build directory (auto-generated)
└── src/                # Source files
```

## Common Use Cases

### As a Build System

- **C/C++ projects**: Native build support
- **Multi-language projects**: Can handle multiple languages
- **Cross-compilation**: Built-in support for different targets

### As a Task Runner

- **Consistent commands**: Standardize development workflows
- **Environment management**: Run commands with specific environment variables
- **IDE integration**: Many IDEs support Meson projects
- **Team collaboration**: Shared task definitions in version control

## Configuration Examples

### Basic meson.build

```meson
project('myproject', 'c')

# Define executable
executable('myapp', 'src/main.c')

# Custom task
run_target('clean-logs',
  command: ['rm', '-rf', 'logs/']
)
```

### Advanced Task with Environment

```meson
run_target('test-with-env',
  command: ['cargo', 'test'],
  env: {'RUST_LOG': 'debug'}
)
```

## Advantages of Using Meson

- **Shorter commands**: Replace long command lines with simple targets
- **Reproducible builds**: Consistent environment and parameters
- **IDE integration**: Good support in modern IDEs
- **Cross-platform**: Works the same on different operating systems
- **Fast**: Optimized for speed and parallel execution
- **Extensible**: Easy to add new tasks and modify existing ones

## Further Reading

- [Official Meson Documentation](https://mesonbuild.com/)
- [Meson Tutorial](https://mesonbuild.com/Tutorial.html)
- [Meson Reference Manual](https://mesonbuild.com/Reference-manual.html)
