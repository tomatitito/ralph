# ralph-loop

A concurrent Rust application that runs coding agents in a loop with real-time context monitoring.

Currently supported backends:
- Claude Code CLI
- OpenAI Codex CLI

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/tomatitito/ralph/main/install.sh | sh
```

This installs to `~/.local/bin`. To install system-wide:

```bash
curl -fsSL https://raw.githubusercontent.com/tomatitito/ralph/main/install.sh | sudo INSTALL_DIR=/usr/local/bin sh
```

## Usage

```bash
# With prompt file
ralph-loop -f prompt.txt

# With inline prompt
ralph-loop -p "Your prompt here"

# Use Codex instead of Claude
ralph-loop --agent-provider codex -p "Your prompt here"

# Limit iterations
ralph-loop -p "Complete the task" -m 5

# Custom completion promise
ralph-loop -p "Work on feature" -c "DONE"

# Override the agent executable or arguments
ralph-loop --agent-path /usr/local/bin/codex --agent-arg=exec --agent-arg=--json -p "Your prompt here"

# Upgrade to the latest released version
ralph-loop upgrade
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
| `--agent-provider <PROVIDER>` | Coding agent backend: `claude` or `codex` |
| `--agent-path <PATH>` | Path to the coding agent executable |
| `--agent-arg <ARG>` | Extra CLI arg to pass to the coding agent (repeatable) |
| `upgrade` | Replace the current `ralph-loop` binary with the latest GitHub release |

## Configuration

```toml
[agent]
provider = "codex"
path = "codex"
args = ["exec", "--json", "--dangerously-bypass-approvals-and-sandbox", "-"]
```

Claude remains the default backend, so existing Claude-based setups continue to work without changes.

## Building from Source

```bash
cd ralph-loop-rs
cargo build --release
```

Binary will be at `ralph-loop-rs/target/release/ralph-loop`.

## License

MIT
