# Meson Task Runner Setup

This project uses [Meson](https://mesonbuild.com/) as a task runner to wrap common development commands and make them easier to execute.

## Prerequisites

Make sure you have Meson installed:

```bash
# On Ubuntu/Debian
sudo apt install meson

# On other systems, check: https://mesonbuild.com/Getting-meson.html
```

## Setup

1. Initialize the Meson build directory (only needed once):

```bash
meson setup builddir
```

This creates a `builddir/` directory with the Meson configuration.

## Available Commands

### Clippy Linter

Run the comprehensive Clippy linter with all lint categories enabled:

```bash
meson compile -C builddir clippy
```

This is equivalent to running:

```bash
CARGO_INCREMENTAL=0 cargo clippy --no-deps --tests --benches --examples --workspace --all-targets --all-features -- -D clippy::correctness -D clippy::suspicious -D clippy::complexity -D clippy::perf -D clippy::style -D clippy::pedantic
```

## Why Use Meson?

- **Shorter commands**: Instead of typing long cargo commands, use simple meson targets
- **Consistent environment**: Commands run with the same environment variables every time
- **Easy to extend**: Add new targets to `meson.build` as needed
- **IDE integration**: Many IDEs can integrate with Meson build systems

## Adding More Commands

To add more commands, edit the `meson.build` file and add new `run_target()` definitions. After modifying the file, you may need to reconfigure:

```bash
meson setup builddir --reconfigure
```

## Directory Structure

```text
project-root/
├── meson.build         # Meson configuration with task definitions
├── builddir/           # Meson build directory (auto-generated)
└── ...                 # Your project files
```

The `builddir/` directory is automatically generated and can be safely deleted and recreated.
