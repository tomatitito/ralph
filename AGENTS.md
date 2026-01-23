# AGENTS.md

Instructions for building, testing, and running the ralph-loop Rust project.

## Prerequisites

- Rust toolchain (stable): Install via [rustup](https://rustup.rs/)
- Claude Code CLI (for integration testing)

## Project Location

The Rust project is located in the `ralph-loop-rs/` subdirectory. All commands below should be run from this directory.

## Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

The binary will be at `ralph-loop-rs/target/release/ralph-loop`.

## Test

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test loop_continues_until_promise_fulfilled
```

## Lint and Format

```bash
# Check formatting
cargo fmt --all --check

# Apply formatting
cargo fmt --all

# Run clippy lints
cargo clippy --all-targets
```

## Run

### Basic Usage

```bash
# With prompt file
./ralph-loop-rs/target/release/ralph-loop -f prompt.txt

# With inline prompt
./ralph-loop-rs/target/release/ralph-loop -p "Your prompt here"
```

### Common Options

| Option | Description |
|--------|-------------|
| `-f, --prompt-file <FILE>` | Prompt file path |
| `-p, --prompt <TEXT>` | Inline prompt text |
| `-m, --max-iterations <N>` | Maximum iterations (omit for infinite) |
| `-c, --completion-promise <S>` | Promise text to detect (default: "TASK COMPLETE") |
| `-o, --output-dir <DIR>` | Output directory (default: .ralph-loop-output) |
| `--context-limit <N>` | Token limit before restart (default: 180000) |
| `--config <FILE>` | TOML configuration file |

### Examples

```bash
# Run with max 5 iterations
./ralph-loop-rs/target/release/ralph-loop -p "Complete the task" -m 5

# Custom completion promise
./ralph-loop-rs/target/release/ralph-loop -p "Work on feature" -c "DONE"

# Infinite loop until promise found (Ctrl+C to stop)
./ralph-loop-rs/target/release/ralph-loop -f task.txt
```

## Documentation

- `specs/` - Detailed architecture documentation
- [TLA_SPEC.md](./TLA_SPEC.md) - TLA+ specification and verification instructions
