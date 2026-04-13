# CLI Specification

## Binary

The executable name should be:
- `ralph-ts`

## Compatibility goal

The CLI should be as similar as possible to `ralph-loop-rs` while reflecting TypeScript/Pi-specific behavior.

## Required flags

- `-f, --prompt-file <FILE>` — prompt file path
- `-p, --prompt <TEXT>` — inline prompt text
- `-m, --max-iterations <N>` — maximum iterations
- `-c, --completion-promise <TEXT>` — compatibility option for completion signaling
- `-o, --output-dir <DIR>` — optional override for run artifacts
- `--context-limit <N>` — token limit before forced restart
- `--config <FILE>` — loop config file

## TS/Pi-specific flags

Likely additions:
- `--checks-config <FILE>`
- `--completion-config <FILE>`
- `--provider <NAME>`
- `--model <ID-or-pattern>`
- `--thinking <LEVEL>`

## Input rules

Exactly one of the following should provide the initial objective:
- `--prompt-file`
- `--prompt`
- loop config file

CLI flags override config-file values.

## Exit semantics

Suggested exit codes:
- `0` — success
- `1` — failure
- `130` — interrupted by user

## Compatibility note

The Rust CLI currently models completion as a promise string. TS should preserve a compatibility flag for that idea, but internally it will likely use explicit Ralph markers and completion validation.
