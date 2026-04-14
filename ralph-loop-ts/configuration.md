# Configuration

Version 1 uses three TOML files:
- loop config
- checks config
- completion config

All three use TOML.

## Configuration precedence

Values are resolved in this order, highest precedence first:
1. CLI flags
2. loop config file
3. built-in defaults

Checks and completion config files are referenced by the loop config and may also be overridden explicitly by CLI flags.

## 1. Loop config

### Purpose
The loop config controls:
- prompt defaults
- iteration limits
- Pi provider/model settings
- artifact storage behavior
- paths to the checks and completion configs

### Suggested file name
- `ralph.toml`

### Canonical sections
- `[prompt]`
- `[loop]`
- `[model]`
- `[artifacts]`
- `[paths]`

### Schema

```toml
[prompt]
text = "optional inline prompt"
file = "optional prompt file path"

[loop]
max_iterations = 10
context_limit = 180000
completion_promise = "TASK COMPLETE"

[model]
provider = "anthropic"
model = "claude-sonnet-4-5"
thinking = "medium"

[artifacts]
base_dir = "~/.ralph-loop"
project_slug = "optional-custom-project-slug"

[paths]
checks = "ralph-checks.toml"
completion = "ralph-completion.toml"
```

### Field definitions

#### `[prompt]`
- `text: string | omitted`
- `file: string | omitted`

Rules:
- at most one of `text` and `file` should be set in config
- CLI may override either
- after merging CLI + config, exactly one prompt source must exist

#### `[loop]`
- `max_iterations: integer | omitted`
  - omitted means unbounded loop
- `context_limit: integer`
  - default: `180000`
- `completion_promise: string`
  - compatibility setting retained for parity with Rust
  - default: `"TASK COMPLETE"`

#### `[model]`
- `provider: string`
  - examples: `anthropic`, `openai`, `google`
- `model: string`
  - Pi model id or pattern
- `thinking: string`
  - one of: `off`, `minimal`, `low`, `medium`, `high`, `xhigh`

#### `[artifacts]`
- `base_dir: string`
  - default: `"~/.ralph-loop"`
- `project_slug: string | omitted`
  - optional human-friendly override for the per-project directory name

#### `[paths]`
- `checks: string`
- `completion: string`

These paths are resolved relative to the loop config file if relative, otherwise as absolute paths.

### Validation rules

The loop config is invalid if:
- both `prompt.text` and `prompt.file` are set after merge
- neither prompt source exists after merge
- `max_iterations` is present but less than 1
- `context_limit` is less than 1
- `thinking` is not one of the supported values
- `checks` or `completion` path is missing

## 2. Checks config

### Purpose
The checks config defines commands run at lifecycle hook points.

Version 1 supports this hook:
- `after_iteration`

### Suggested file name
- `ralph-checks.toml`

### Schema

```toml
[[after_iteration]]
name = "test"
command = "bun test"

[[after_iteration]]
name = "lint"
command = "bun run lint"
timeout_seconds = 120
```

### Field definitions for each check
- `name: string` — human-readable name
- `command: string` — command executed through the shell
- `cwd: string | omitted` — working directory override
- `timeout_seconds: integer | omitted` — timeout in seconds
- `required_exit_code: integer | omitted` — default `0`
- `required_stdout: string | omitted` — substring that must appear in stdout
- `required_stderr: string | omitted` — substring that must appear in stderr
- `[...env]` / `env.<KEY> = "VALUE"` style env map

### Success rule for a check
A check succeeds only if all applicable conditions hold:
- actual exit code equals `required_exit_code` or default `0`
- `required_stdout` is present in stdout if configured
- `required_stderr` is present in stderr if configured
- command did not time out

### Execution order
- checks run in file order within each hook array
- all checks for a hook are executed, even if one fails
- results are recorded individually and as an aggregate pass/fail outcome

## 3. Completion config

### Purpose
The completion config defines validators that determine whether the overall objective is actually complete.

### Suggested file name
- `ralph-completion.toml`

### Supported hook
Version 1 supports one hook:
- `on_loop_complete_claim`

### Schema

```toml
[[on_loop_complete_claim]]
name = "all tickets closed"
command = "tk list --open"
required_stdout = "0 open"

[[on_loop_complete_claim]]
name = "domain-specific completion script"
command = "./scripts/check-complete.sh"
required_exit_code = 0
```

### Field definitions
Completion validators support the same fields and success rules as checks:
- `name`
- `command`
- `cwd`
- `timeout_seconds`
- `required_exit_code`
- `required_stdout`
- `required_stderr`
- `env`

### Execution rule
- validators run in file order
- all validators are executed
- overall completion validation succeeds only if all validators succeed

## Why separate files?

- avoids colliding with `prek.toml`
- keeps loop mechanics separate from operational checks
- keeps completion validation explicit and auditable
- makes it easy to reuse checks or completion definitions independently

## Example complete setup

```toml
# ralph.toml
[prompt]
file = "task.md"

[loop]
max_iterations = 10
context_limit = 180000
completion_promise = "TASK COMPLETE"

[model]
provider = "anthropic"
model = "claude-sonnet-4-5"
thinking = "medium"

[artifacts]
base_dir = "~/.ralph-loop"

[paths]
checks = "ralph-checks.toml"
completion = "ralph-completion.toml"
```

```toml
# ralph-checks.toml
[[after_iteration]]
name = "unit tests"
command = "bun test"

[[after_iteration]]
name = "lint"
command = "bun run lint"
```

```toml
# ralph-completion.toml
[[on_loop_complete_claim]]
name = "tickets closed"
command = "tk list --open"
required_stdout = "0 open"
```
