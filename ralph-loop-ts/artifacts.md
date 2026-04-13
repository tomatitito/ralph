# Artifacts and State

## Base directory

Run artifacts live under:
- `~/.ralph-loop/`

This directory is shared across Ralph implementations.

Each run must record which implementation created it, for example:
- `implementation = "typescript"`
- `implementation = "rust"`

## Project/run directory layout

Canonical v1 layout:

```text
~/.ralph-loop/
  projects/
    <project-slug>-<project-hash>/
      runs/
        <run-id>/
          run.json
          iterations/
            001-metadata.json
            001-summary.md
            001-checks.json
            001-completion.json
            001-diagnostics.json
          logs/
            001-after-iteration-01.log
            001-after-iteration-02.log
            001-completion-01.log
            001-final-check-01.log
          exports/
      latest -> runs/<run-id>
```

## Project identity

Per-project storage should be grouped by:
- a readable slug derived from the project directory name
- a stable hash derived from the canonical project path

This avoids collisions for similarly named repositories.

## `run.json`

`run.json` is the canonical run-level metadata file.

### Required fields

```json
{
  "runId": "20260413-120000-abcd1234",
  "implementation": "typescript",
  "projectPath": "/absolute/path/to/project",
  "projectSlug": "ralph",
  "startedAt": "2026-04-13T12:00:00.000Z",
  "completedAt": null,
  "status": "running",
  "exitReason": null,
  "promptPreview": "first part of prompt",
  "provider": "anthropic",
  "model": "claude-sonnet-4-5",
  "thinking": "medium",
  "maxIterations": 10,
  "contextLimit": 180000,
  "checksConfigPath": "/abs/path/ralph-checks.toml",
  "completionConfigPath": "/abs/path/ralph-completion.toml",
  "iterations": []
}
```

### Required meanings
- `status`: `running | completed | failed | interrupted`
- `exitReason`: implementation-defined canonical reason string such as:
  - `loop_completed`
  - `max_iterations_exceeded`
  - `interrupted`
  - `fatal_error`

## Iteration metadata file

Each iteration writes `iterations/NNN-metadata.json`.

### Required fields

```json
{
  "iteration": 1,
  "startedAt": "2026-04-13T12:00:00.000Z",
  "endedAt": "2026-04-13T12:05:00.000Z",
  "endReason": "task_boundary",
  "taskComplete": true,
  "loopComplete": false,
  "contextLimitHit": false,
  "finalContextTokens": 42000,
  "peakContextTokens": 51000,
  "afterIterationChecksPassed": true,
  "completionValidated": null,
  "beforeFinalSuccessChecksPassed": null,
  "piSessionFile": "/path/to/pi/session.jsonl",
  "piSessionId": "optional-session-id",
  "summaryPath": "./001-summary.md",
  "checksPath": "./001-checks.json",
  "completionPath": "./001-completion.json",
  "diagnosticsPath": "./001-diagnostics.json"
}
```

### `endReason` values
Canonical v1 values should include:
- `task_boundary`
- `loop_complete_claim`
- `context_limit`
- `checks_failed`
- `incomplete`
- `interrupted`
- `error`

## Checks result file

Each iteration writes `iterations/NNN-checks.json`.

It should contain:
- hook name
- aggregate pass/fail
- per-command results in execution order
- stdout/stderr log file references
- timeout information
- execution durations

## Completion result file

Each iteration writes `iterations/NNN-completion.json`.

Rules:
- if no completion validation was attempted, write a file indicating `skipped`
- if validation ran, include aggregate pass/fail plus per-validator details

## Diagnostics file

Each iteration writes `iterations/NNN-diagnostics.json`.

This should capture:
- extension diagnostics
- controller diagnostics
- any non-fatal anomalies encountered during evaluation

## Transcript handling

Pi already manages session persistence. Ralph v1 should therefore store:
- Pi session file path if available
- Pi session id if available
- optional export paths if transcript export is later implemented

Ralph does not need to duplicate the full transcript in v1 to be useful.

## Summary file

Each iteration writes `iterations/NNN-summary.md`.

This summary is the handoff document for the next iteration.

### Required sections
- objective
- iteration outcome
- completed work
- changed files or affected areas
- checks and validation results
- outstanding work
- recommended next step
- notes for the next iteration

### Summary requirements
- concise enough to keep context small
- detailed enough to continue work accurately
- should prefer stable facts over conversational narration

## Run completion behavior

At run end, `run.json` must be updated with:
- final `status`
- final `exitReason`
- `completedAt`
- final aggregate iteration list or references

The `latest` symlink for the project should point at the newest run directory.
