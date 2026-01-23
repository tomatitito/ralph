# CLI Interface

## Usage

```bash
ralph-loop [OPTIONS] <PROMPT_FILE | -p "PROMPT">
```

## Options

| Option | Description |
|--------|-------------|
| `-f, --prompt-file <FILE>` | Prompt file path |
| `-p, --prompt <TEXT>` | Prompt text |
| `-m, --max-iterations <N>` | Max iterations (optional, omit for infinite loop) |
| `-c, --completion-promise <S>` | Promise text (default: "TASK COMPLETE") |
| `-o, --output-dir <DIR>` | Output directory (default: .ralph-loop-output) |
| `--context-limit <N>` | Token limit (default: 180000) |
| `--config <FILE>` | Config file (TOML) |
