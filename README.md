# ralph-loop

A concurrent Rust application that runs Claude Code in a loop with real-time context monitoring.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/tomatitito/ralph/main/install.sh | sh
```

This installs to `~/.local/bin`. Set `INSTALL_DIR` to customize:

```bash
curl -fsSL https://raw.githubusercontent.com/tomatitito/ralph/main/install.sh | INSTALL_DIR=/usr/local/bin sh
```

## Usage

```bash
# With prompt file
ralph-loop -f prompt.txt

# With inline prompt
ralph-loop -p "Your prompt here"

# Limit iterations
ralph-loop -p "Complete the task" -m 5

# Custom completion promise
ralph-loop -p "Work on feature" -c "DONE"
```

## Options

| Option | Description |
|--------|-------------|
| `-f, --prompt-file <FILE>` | Prompt file path |
| `-p, --prompt <TEXT>` | Inline prompt text |
| `-m, --max-iterations <N>` | Maximum iterations (omit for infinite) |
| `-c, --completion-promise <S>` | Promise text to detect (default: "TASK COMPLETE") |
| `-o, --output-dir <DIR>` | Output directory (default: .ralph-loop-output) |
| `--context-limit <N>` | Token limit before restart (default: 180000) |
| `--config <FILE>` | TOML configuration file |

## Building from Source

```bash
cd ralph-loop-rs
cargo build --release
```

Binary will be at `ralph-loop-rs/target/release/ralph-loop`.

## License

MIT
