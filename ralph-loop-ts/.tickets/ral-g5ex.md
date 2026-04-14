---
id: ral-g5ex
status: closed
deps: [ral-m4r6]
links: [configuration.md, lifecycle.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 2
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, checks, commands]
---
# Replace the mock checks runner with real command execution from ralph-checks.toml

Upgrade the function-shaped checks runner introduced by the mock vertical slice so it executes configured commands at the supported lifecycle hooks and captures structured results and logs.

## Acceptance Criteria

- after_iteration and before_final_success hooks are supported
- command cwd/env/timeout are honored
- stdout/stderr matching and exit-code rules are enforced
- per-command and aggregate results are returned in a persistable structured form
- checks execute in file order without short-circuiting after failures

## Implementation Notes

- `rlt-w903` already introduced the function-shaped checks seam and a trivial pass-through implementation; this ticket replaces that stub with real command execution semantics.
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
- Builds on the seam established by `rlt-w903`.
- Its result contract should remain usable by `ral-gu4t` and `ral-l3ri`.

## Out of Scope

- Deciding overall loop success
- Completion validation hook support beyond the checks-specific hooks

## Verification Notes

- Add tests for cwd/env propagation, timeout behavior, required stdout/stderr matching, and aggregate failure with continued execution.

## Suggested Implementation Checklist

1. Start from the normalized command/check result contracts already introduced in the vertical slice.
2. Introduce a real command-execution abstraction so the checks runner does not depend directly on process-spawning details.
3. Add red/green unit tests for command-result evaluation rules:
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

This ticket is done when the current stub checks runner has been replaced by deterministic command execution, all configured checks for a hook execute in order, aggregate results are computed correctly, and the implementation is covered by focused red/green tests without taking a dependency on runtime internals.

## Notes

**2026-04-14T22:33:44Z**

Implemented real command-backed checks runner under src/guards, removed before_final_success hook, updated controller/docs/tests, committed in 6902f2a and pushed to origin/main.
