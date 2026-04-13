---
id: ral-g5ex
status: open
deps: [ral-m4r6]
links: [configuration.md, lifecycle.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 2
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, checks, commands]
---
# Implement checks runner from ralph-checks.toml

Run configured check commands at the supported lifecycle hooks and capture structured results and logs.

## Acceptance Criteria

- after_iteration and before_final_success hooks are supported
- command cwd/env/timeout are honored
- stdout/stderr matching and exit-code rules are enforced
- per-command and aggregate results are returned in a persistable structured form

## Implementation Notes

- Implement a command-runner layer shared across check hooks so behavior is identical for `after_iteration` and `before_final_success`.
- Normalize each command result into a structured shape such as:
  - `name`
  - `command`
  - `cwd`
  - `exitCode`
  - `stdout`
  - `stderr`
  - `timedOut`
  - `passed`
- Execute checks in file order and do not short-circuit after a failure.
- Compute both per-check status and aggregate hook status.
- Return structured results to the controller even when artifact persistence is implemented separately.
- Artifact persistence is owned by `ral-l3ri`; this ticket owns execution and normalized result production only.
- Keep command execution isolated behind an explicit runner interface so callers do not depend on child-process details.

## Architecture Constraints

- The checks runner should not depend on controller logic, runtime internals, or direct process-global state.
- If logging is needed, accept a logger dependency rather than importing a concrete logger directly.
- Prefer sharing a command-runner abstraction with completion validation instead of duplicating process-spawning logic.

## Relevant Spec

- `configuration.md`
- `lifecycle.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/checks/check-runner.ts`
- `src/checks/command-runner.ts`
- `src/checks/check-types.ts`

## Dependencies on Other Tickets

- Consumes normalized config from `ral-m4r6`.
- Its result contract should be usable by `ral-gu4t` and `ral-l3ri`.

## Out of Scope

- Deciding overall loop success
- Completion validation hook support beyond the checks-specific hooks

## Verification Notes

- Add tests for cwd/env propagation, timeout behavior, required stdout/stderr matching, and aggregate failure with continued execution.

## Suggested Implementation Checklist

1. Define the normalized command/check result contracts from `internal-contracts.md`.
2. Introduce a command-execution abstraction so the checks runner does not depend directly on process-spawning details.
3. Start with red/green unit tests for command-result evaluation rules:
   - required exit code
   - required stdout substring
   - required stderr substring
   - timeout handling
4. Implement the smallest shared command-runner behavior that passes those tests.
5. Add checks-runner tests for hook execution semantics:
   - `after_iteration`
   - `before_final_success`
   - execution in file order
   - no short-circuit after failure
   - aggregate pass/fail computation
6. Inject logger or diagnostics dependencies explicitly if needed; do not import concrete logging/process globals into the runner.
7. Keep persistence concerns separate: return structured results first, then let artifact-writing consume them.
8. Verify the checks runner remains runtime-agnostic and dependency-cruiser still passes.

## Definition of Done Heuristic

This ticket is done when checks can be executed through a normalized runner contract, all configured checks for a hook execute deterministically in order, aggregate results are computed correctly, and the implementation is covered by red/green tests without taking a dependency on runtime internals.

